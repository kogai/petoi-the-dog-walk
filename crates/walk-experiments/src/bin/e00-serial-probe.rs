//! E00: send tokens to Bittle over a serial port and log every response with a timestamp.
//! Procedure: experimentals/E00-connectivity.md. Run by a human; never by an agent.
//!
//! Safety (the logic lives in `walk_experiments::serial_probe::Session` and is unit-tested):
//! the stop token `d` is written to the port before anything is logged, on normal end, on errors,
//! on Ctrl-C / SIGTERM / SIGHUP and while unwinding from a panic; logging never panics; after a
//! stop request no further token is sent (except in a microsecond race, followed by `d`). If `d`
//! cannot be sent, the program says so and exits with status 3.
//!
//! With `E00_ASK`, after the final wait the program rings the bell and shows one yes/no question
//! about what the robot is doing at that moment, sends `d`, and then reads the answer from the
//! keyboard into the log (`walk_experiments::serial_probe::observe`).

use std::cell::Cell;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use walk_experiments::serial_probe::{
    self, BAUD_RATE, Command, Exit, Input, Log, Observed, Outcome, Probe, Prompt, STOP_FAILED,
    Session, hide_port, show_bytes,
};

/// Read and write timeout on the port, so that a vanished device cannot hang a write.
const IO_TIMEOUT: Duration = Duration::from_millis(1000);
/// Granularity of interruptible sleeps.
const TICK: Duration = Duration::from_millis(50);
/// Time given to the robot to answer the stop token before the program ends.
const AFTER_STOP: Duration = Duration::from_millis(500);

/// Stop state shared with the signal handler.
const STOP_PENDING: u8 = 0;
const STOP_DONE: u8 = 1;
const STOP_FAILED_STATE: u8 = 2;

/// Writes log lines to stdout and, if given, a file. Never panics: write errors are ignored,
/// because a closed pipe must not break the stop path.
struct TerminalLog {
    file: Option<File>,
}

impl Log for TerminalLog {
    fn line(&mut self, text: &str) {
        let _ = writeln!(io::stdout().lock(), "{text}");
        if let Some(file) = self.file.as_mut() {
            let _ = writeln!(file, "{text}");
        }
    }
}

/// Prints a warning without panicking.
fn warn(text: &str) {
    let _ = writeln!(io::stderr().lock(), "{text}");
}

/// Replaces the default panic message (which contains source paths) with the message only.
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown".to_owned());
        warn(&format!("panic: {message}"));
    }));
}

/// Sleeps for `total`, returning early (with `false`) if a stop was requested.
fn sleep_unless_stopped(total: Duration, stop: &AtomicBool) -> bool {
    let deadline = Instant::now() + total;
    loop {
        if stop.load(Ordering::SeqCst) {
            return false;
        }
        let now = Instant::now();
        if now >= deadline {
            return true;
        }
        thread::sleep(TICK.min(deadline - now));
    }
}

fn spawn_reader(mut reader: Box<dyn serialport::SerialPort>, start: Instant, log: TerminalLog) {
    let mut log = log;
    thread::spawn(move || {
        let mut buf = [0_u8; 256];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {}
                Ok(n) => {
                    let ms = start.elapsed().as_secs_f64() * 1000.0;
                    let bytes = show_bytes(buf.get(..n).unwrap_or_default());
                    log.line(&format!("{ms:8.1} ms  <- \"{bytes}\""));
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {}
                Err(e) => {
                    log.line(&format!("read error: {}", e.kind()));
                    return;
                }
            }
        }
    });
}

/// Reads keyboard lines on a separate thread, each with the time it arrived, so that waiting for
/// an answer can still notice Ctrl-C and lines typed before the question can be ignored. The
/// thread is left behind when the program ends.
fn spawn_stdin_lines() -> Receiver<(Instant, String)> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for line in io::stdin().lines() {
            match line {
                Ok(text) => {
                    if tx.send((Instant::now(), text)).is_err() {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    });
    rx
}

/// Next keyboard line typed after the question was shown (`asked_at`); earlier lines, such as
/// keys pressed by accident while the robot was moving, are dropped.
fn next_line(
    lines: &Receiver<(Instant, String)>,
    asked_at: Option<Instant>,
    stop: &AtomicBool,
) -> Input {
    loop {
        if stop.load(Ordering::SeqCst) {
            return Input::Stopped;
        }
        match lines.recv_timeout(TICK) {
            Ok((at, text)) => {
                if asked_at.is_some_and(|asked| at >= asked) {
                    return Input::Line(text);
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Input::End,
        }
    }
}

/// Shows a prompt on stderr, apart from the robot's responses on stdout and not in the log file.
/// The question rings the bell.
fn show_prompt(prompt: Prompt<'_>) {
    let mut out = io::stderr().lock();
    let _ = match prompt {
        Prompt::LookNow(question) => writeln!(out, "\x07>>> LOOK NOW: {question}"),
        Prompt::TypeAnswer => writeln!(
            out,
            ">>> answer y (yes), n (no) or ? (could not tell), then press Enter:"
        ),
    };
}

/// Creates the log file, and its folder if needed (`experimentals/raw/` is not in the repository).
fn open_log(path: Option<&str>) -> io::Result<Option<File>> {
    path.map(|p| {
        let path = std::path::Path::new(p);
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)?;
        }
        File::create(path)
    })
    .transpose()
}

fn probe(probe: &Probe) -> ExitCode {
    let stop = Arc::new(AtomicBool::new(false));
    let state = Arc::new(AtomicU8::new(STOP_PENDING));
    let (handler_stop, handler_state) = (Arc::clone(&stop), Arc::clone(&state));
    let handler = ctrlc::set_handler(move || {
        if !handler_stop.swap(true, Ordering::SeqCst) {
            return; // first signal: the main thread stops and sends `d`
        }
        // Second signal: wait for the stop attempt to finish before deciding.
        let deadline = Instant::now() + IO_TIMEOUT * 3;
        while handler_state.load(Ordering::SeqCst) == STOP_PENDING && Instant::now() < deadline {
            thread::sleep(TICK);
        }
        if handler_state.load(Ordering::SeqCst) != STOP_DONE {
            warn(STOP_FAILED);
        }
        std::process::exit(130);
    });
    if let Err(e) = handler {
        warn(&format!(
            "cannot install the Ctrl-C handler, refusing to run: {e}"
        ));
        return ExitCode::from(1);
    }

    let (file, reader_file) = match open_log(probe.log_path.as_deref()) {
        Ok(file) => {
            let copy = file.as_ref().and_then(|f| f.try_clone().ok());
            (file, copy)
        }
        Err(e) => {
            warn(&format!("cannot open the log file: {}", e.kind()));
            return ExitCode::from(2);
        }
    };

    let port = match serialport::new(&probe.port, BAUD_RATE)
        .timeout(IO_TIMEOUT)
        .open()
    {
        Ok(port) => port,
        Err(e) => {
            // Nothing was sent: the port never opened.
            warn(&format!(
                "cannot open <PORT>: {}",
                hide_port(&e.to_string(), &probe.port)
            ));
            return ExitCode::from(1);
        }
    };
    let start = Instant::now();
    match port.try_clone() {
        Ok(reader) => spawn_reader(reader, start, TerminalLog { file: reader_file }),
        Err(e) => warn(&format!(
            "cannot read responses: {}",
            hide_port(&e.to_string(), &probe.port)
        )),
    }

    // Started before the run so that keys pressed during it are known (and dropped) later.
    let lines = probe.ask.as_ref().map(|_| spawn_stdin_lines());
    let mut session = Session::new(port, TerminalLog { file }, probe.eol, Arc::clone(&stop));
    let outcome = serial_probe::run(&mut session, &probe.tokens, probe.timing, &mut |d| {
        sleep_unless_stopped(d, &stop)
    });
    let observed = match (&outcome, probe.ask.as_deref(), lines.as_ref()) {
        (Ok(Outcome::Completed), Some(question), Some(lines)) => {
            let asked_at = Cell::new(None);
            Some(serial_probe::observe(
                &mut session,
                question,
                &mut |d| sleep_unless_stopped(d, &stop),
                &mut |prompt| {
                    if matches!(prompt, Prompt::LookNow(_)) {
                        asked_at.set(Some(Instant::now()));
                    }
                    show_prompt(prompt);
                },
                &mut || next_line(lines, asked_at.get(), &stop),
            ))
        }
        _ => None,
    };
    // Already sent by `observe` when it asked; then this is a no-op that reports success.
    let stopped = session.stop();
    state.store(
        if stopped.is_ok() {
            STOP_DONE
        } else {
            STOP_FAILED_STATE
        },
        Ordering::SeqCst,
    );
    thread::sleep(AFTER_STOP);
    drop(session); // retries the stop once more if it failed

    let exit = serial_probe::exit_status(&outcome, observed.as_ref(), stopped.is_ok());
    report_exit(exit, &stopped, observed.as_ref(), &outcome);
    ExitCode::from(exit.code())
}

/// Tells the person why the program ends this way.
fn report_exit(
    exit: Exit,
    stopped: &io::Result<()>,
    observed: Option<&io::Result<Observed>>,
    outcome: &io::Result<Outcome>,
) {
    match exit {
        Exit::Success => {}
        Exit::Interrupted => warn("interrupted; stop token sent"),
        Exit::SendFailed => {
            if let Err(e) = outcome {
                warn(&format!("send failed ({}); stop token sent", e.kind()));
            }
        }
        Exit::StopRetried => {
            if let Some(Err(e)) = observed {
                warn(&format!(
                    "the first stop attempt failed ({}); the retry sent the stop token",
                    e.kind()
                ));
            }
        }
        Exit::StopFailed => match stopped {
            Err(e) => warn(&format!("{STOP_FAILED} ({})", e.kind())),
            Ok(()) => warn(STOP_FAILED),
        },
    }
}

fn main() -> ExitCode {
    install_panic_hook();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match serial_probe::parse(&args, |name| std::env::var(name).ok()) {
        Err(e) => {
            warn(&e.to_string());
            ExitCode::from(2)
        }
        Ok(Command::List) => match serialport::available_ports() {
            Ok(ports) => {
                warn("(port names can include your device or computer name: do not paste them)");
                let mut out = io::stdout().lock();
                for p in &ports {
                    let _ = writeln!(out, "{}", p.port_name);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                warn(&format!("cannot list ports: {e}"));
                ExitCode::from(1)
            }
        },
        Ok(Command::Probe(p)) => probe(&p),
    }
}
