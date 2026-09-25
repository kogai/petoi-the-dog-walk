//! The testable core of `e00-serial-probe` (experimentals/E00-connectivity.md): parsing the
//! command line and environment, validating tokens, hiding the port name, and the [`Session`]
//! that sends tokens and guarantees the stop token.
//!
//! Stop guarantee: [`Session::stop`] writes the stop token to the port *before* logging anything,
//! logging can never panic ([`Log`]), and if the stop write fails, `Drop` tries once more.

use core::fmt;
use core::time::Duration;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// Serial speed of the `NyBoard` (docs/design/facts.md F-B4).
pub const BAUD_RATE: u32 = 115_200;
/// Stop token (docs/design/facts.md F-T1).
pub const STOP_TOKEN: &str = "d";
/// Longest token accepted on the command line.
const MAX_TOKEN_LEN: usize = 16;

/// Line ending appended to each token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineEnding {
    /// `\n` (default).
    Lf,
    /// `\r\n` (`E00_EOL=CRLF`).
    CrLf,
}

impl LineEnding {
    /// The bytes to append after a token.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
        }
    }
}

/// How long to wait at each stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing {
    /// After opening the port, before the first token (the board may reset on open).
    pub settle: Duration,
    /// After each token except the last.
    pub wait: Duration,
    /// After the last token, before the stop token is sent.
    pub final_wait: Duration,
}

/// A validated probe run.
#[derive(Debug, PartialEq, Eq)]
pub struct Probe {
    /// Serial port path. Never printed; see [`hide_port`].
    pub port: String,
    /// Tokens to send, in order.
    pub tokens: Vec<String>,
    /// Waits.
    pub timing: Timing,
    /// Line ending.
    pub eol: LineEnding,
    /// File to write the log to, in addition to the terminal (`E00_LOG`).
    pub log_path: Option<String>,
}

/// What the binary should do.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Print the available serial ports.
    List,
    /// Send tokens and record responses.
    Probe(Probe),
}

/// A problem with the command line or environment.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// Wrong arguments.
    Usage,
    /// A token that is empty, too long, or not ASCII alphanumeric.
    BadToken(String),
    /// An environment variable that is not a whole number.
    NotANumber {
        /// Variable name.
        name: &'static str,
        /// Its value.
        value: String,
    },
    /// An environment variable outside its allowed range.
    OutOfRange {
        /// Variable name.
        name: &'static str,
        /// Its value.
        value: u64,
        /// Smallest allowed value.
        min: u64,
        /// Largest allowed value.
        max: u64,
    },
    /// `E00_EOL` that is neither `LF` nor `CRLF`.
    BadLineEnding(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => f.write_str(USAGE),
            Self::BadToken(t) => write!(
                f,
                "invalid token {t:?}: use 1..={MAX_TOKEN_LEN} ASCII letters or digits"
            ),
            Self::NotANumber { name, value } => {
                write!(f, "{name}={value:?} is not a whole number of milliseconds")
            }
            Self::OutOfRange {
                name,
                value,
                min,
                max,
            } => {
                write!(f, "{name}={value} is outside {min}..={max}")
            }
            Self::BadLineEnding(v) => write!(f, "E00_EOL={v:?}: use LF or CRLF"),
        }
    }
}

/// Usage text.
pub const USAGE: &str = "usage: e00-serial-probe <PORT> <TOKEN>...\n       e00-serial-probe --list\n\
env: E00_WAIT_MS (100..=10000, default 3000), E00_FINAL_WAIT_MS (100..=10000, default E00_WAIT_MS),\n     \
E00_SETTLE_MS (0..=10000, default 2000), E00_EOL (LF or CRLF, default LF), E00_LOG (log file)";

fn millis(
    env: &impl Fn(&str) -> Option<String>,
    name: &'static str,
    default: u64,
    (min, max): (u64, u64),
) -> Result<u64, ConfigError> {
    let value = match env(name) {
        None => default,
        Some(raw) => raw
            .trim()
            .parse::<u64>()
            .map_err(|_| ConfigError::NotANumber {
                name,
                value: raw.clone(),
            })?,
    };
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(ConfigError::OutOfRange {
            name,
            value,
            min,
            max,
        })
    }
}

fn valid_token(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= MAX_TOKEN_LEN
        && token.bytes().all(|b| b.is_ascii_alphanumeric())
}

/// Parses the arguments (without the program name) and environment.
///
/// # Errors
/// Returns a [`ConfigError`] describing the first problem found.
pub fn parse(
    args: &[String],
    env: impl Fn(&str) -> Option<String>,
) -> Result<Command, ConfigError> {
    match args {
        [flag] if flag == "--list" => Ok(Command::List),
        [port, tokens @ ..] if !tokens.is_empty() && !port.starts_with('-') => {
            if let Some(bad) = tokens.iter().find(|t| !valid_token(t)) {
                return Err(ConfigError::BadToken(bad.clone()));
            }
            let wait = millis(&env, "E00_WAIT_MS", 3000, (100, 10_000))?;
            let settle = millis(&env, "E00_SETTLE_MS", 2000, (0, 10_000))?;
            let final_wait = millis(&env, "E00_FINAL_WAIT_MS", wait, (100, 10_000))?;
            let timing = Timing {
                settle: Duration::from_millis(settle),
                wait: Duration::from_millis(wait),
                final_wait: Duration::from_millis(final_wait),
            };
            let eol = match env("E00_EOL").as_deref().map(str::trim) {
                None | Some("LF") => LineEnding::Lf,
                Some("CRLF") => LineEnding::CrLf,
                Some(other) => return Err(ConfigError::BadLineEnding(other.to_owned())),
            };
            let log_path = env("E00_LOG")
                .map(|p| p.trim().to_owned())
                .filter(|p| !p.is_empty());
            Ok(Command::Probe(Probe {
                port: port.clone(),
                tokens: tokens.to_vec(),
                timing,
                eol,
                log_path,
            }))
        }
        _ => Err(ConfigError::Usage),
    }
}

/// Replaces the port path in a message with `<PORT>`, because output may be pasted into this
/// public repository (docs/rules/public-repo.md).
#[must_use]
pub fn hide_port(message: &str, port: &str) -> String {
    if port.is_empty() {
        return message.to_owned();
    }
    let hidden = message.replace(port, "<PORT>");
    match port.rsplit('/').next() {
        Some(base) if !base.is_empty() && base != port => hidden.replace(base, "<PORT>"),
        Some(_) | None => hidden,
    }
}

/// Printed when the stop token could not be sent.
pub const STOP_FAILED: &str =
    "!! STOP TOKEN NOT SENT. Bittle may still be moving: switch it off now.";

/// A sink for log lines. Implementations must never panic (for example on a closed pipe),
/// because logging happens on the stop path.
pub trait Log {
    /// Records one line.
    fn line(&mut self, text: &str);
}

/// Formats received bytes for the log: printable ASCII as is, everything else escaped.
#[must_use]
pub fn show_bytes(bytes: &[u8]) -> String {
    bytes.escape_ascii().to_string()
}

/// Why [`Session::send`] did not send a token.
#[derive(Debug)]
pub enum SendError {
    /// A stop was requested; no further tokens are sent.
    Stopping,
    /// Writing to the port failed.
    Io(io::Error),
}

/// An open connection. Sends tokens, refuses them once a stop is requested, and sends the stop
/// token exactly once (again from `Drop` if the first attempt failed).
pub struct Session<W: Write, L: Log> {
    port: W,
    log: L,
    eol: LineEnding,
    stop_requested: Arc<AtomicBool>,
    stop_sent: bool,
    start: Instant,
}

impl<W: Write, L: Log> Session<W, L> {
    /// Wraps an open port.
    pub fn new(port: W, log: L, eol: LineEnding, stop_requested: Arc<AtomicBool>) -> Self {
        Self {
            port,
            log,
            eol,
            stop_requested,
            stop_sent: false,
            start: Instant::now(),
        }
    }

    /// Milliseconds since the session started, for log lines.
    #[must_use]
    pub fn stamp(&self) -> String {
        format!("{:8.1} ms", self.start.elapsed().as_secs_f64() * 1000.0)
    }

    /// Writes a line to the log, prefixed with the time.
    pub fn note(&mut self, text: &str) {
        let line = format!("{}  {text}", self.stamp());
        self.log.line(&line);
    }

    /// Sends one token, unless a stop was requested.
    ///
    /// # Errors
    /// [`SendError::Stopping`] after a stop request; [`SendError::Io`] if the write fails.
    pub fn send(&mut self, token: &str) -> Result<(), SendError> {
        // Narrow race: a signal between this check and the write lets one token through;
        // the stop token follows it.
        if self.stop_requested.load(Ordering::SeqCst) {
            return Err(SendError::Stopping);
        }
        self.port
            .write_all(format!("{token}{}", self.eol.as_str()).as_bytes())
            .map_err(SendError::Io)?;
        self.note(&format!("-> {token:?}"));
        Ok(())
    }

    /// Sends the stop token once. A line ending goes first, so that a partly written token
    /// cannot join with it. The port is written before anything is logged.
    ///
    /// # Errors
    /// The write error. The stop is then not marked as sent, so `Drop` tries again.
    pub fn stop(&mut self) -> io::Result<()> {
        if self.stop_sent {
            return Ok(());
        }
        let eol = self.eol.as_str();
        self.port
            .write_all(format!("{eol}{STOP_TOKEN}{eol}").as_bytes())?;
        self.stop_sent = true;
        self.note(&format!("-> {STOP_TOKEN:?}"));
        Ok(())
    }

    /// Whether the stop token has been written.
    #[must_use]
    pub fn stop_sent(&self) -> bool {
        self.stop_sent
    }
}

impl<W: Write, L: Log> Drop for Session<W, L> {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            let line = format!("{STOP_FAILED} ({})", e.kind());
            self.log.line(&line);
        }
    }
}

/// How [`run`] ended.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every token was sent and every wait finished.
    Completed,
    /// A stop was requested.
    Interrupted,
}

/// Sends the tokens with the probe's waits. `sleep` waits for the given time and returns `false`
/// if a stop was requested meanwhile. Does not send the stop token; the caller does.
///
/// # Errors
/// The first write error.
pub fn run<W: Write, L: Log>(
    session: &mut Session<W, L>,
    tokens: &[String],
    timing: Timing,
    sleep: &mut dyn FnMut(Duration) -> bool,
) -> io::Result<Outcome> {
    if !sleep(timing.settle) {
        return Ok(Outcome::Interrupted);
    }
    let last = tokens.len().saturating_sub(1);
    for (i, token) in tokens.iter().enumerate() {
        match session.send(token) {
            Ok(()) => {}
            Err(SendError::Stopping) => return Ok(Outcome::Interrupted),
            Err(SendError::Io(e)) => return Err(e),
        }
        let wait = if i == last {
            timing.final_wait
        } else {
            timing.wait
        };
        if !sleep(wait) {
            return Ok(Outcome::Interrupted);
        }
    }
    Ok(Outcome::Completed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        move |k| map.get(k).cloned()
    }

    fn probe(list: &[&str], env: &[(&str, &str)]) -> Probe {
        match parse(&args(list), env_of(env)) {
            Ok(Command::Probe(p)) => p,
            other => panic!("expected probe, got {other:?}"),
        }
    }

    // ---- parsing ----

    #[test]
    fn defaults_apply_when_env_is_empty() {
        let p = probe(&["PORT", "ksit", "kbalance"], &[]);

        assert_eq!(p.tokens, vec!["ksit", "kbalance"]);
        assert_eq!(p.timing.settle, Duration::from_millis(2000));
        assert_eq!(p.timing.wait, Duration::from_millis(3000));
        assert_eq!(p.timing.final_wait, Duration::from_millis(3000));
        assert_eq!(p.eol, LineEnding::Lf);
        assert_eq!(p.log_path, None);
    }

    #[test]
    fn final_wait_defaults_to_wait() {
        let p = probe(&["PORT", "ksit"], &[("E00_WAIT_MS", "700")]);

        assert_eq!(p.timing.final_wait, Duration::from_millis(700));
    }

    #[test]
    fn final_wait_can_differ_from_wait() {
        let p = probe(
            &["PORT", "ksit"],
            &[("E00_WAIT_MS", "200"), ("E00_FINAL_WAIT_MS", "5000")],
        );

        assert_eq!(p.timing.final_wait, Duration::from_millis(5000));
    }

    #[test]
    fn log_path_is_read_and_trimmed() {
        let p = probe(&["PORT", "ksit"], &[("E00_LOG", " out.txt ")]);

        assert_eq!(p.log_path.as_deref(), Some("out.txt"));
    }

    #[test]
    fn list_flag_is_recognised() {
        assert_eq!(parse(&args(&["--list"]), env_of(&[])), Ok(Command::List));
    }

    #[test]
    fn port_without_tokens_is_usage_error() {
        assert_eq!(
            parse(&args(&["PORT"]), env_of(&[])),
            Err(ConfigError::Usage)
        );
    }

    #[test]
    fn no_arguments_is_usage_error() {
        assert_eq!(parse(&args(&[]), env_of(&[])), Err(ConfigError::Usage));
    }

    #[test]
    fn list_with_extra_argument_is_usage_error() {
        assert_eq!(
            parse(&args(&["--list", "x"]), env_of(&[])),
            Err(ConfigError::Usage)
        );
    }

    #[test]
    fn port_starting_with_dash_is_usage_error() {
        assert_eq!(
            parse(&args(&["-p", "ksit"]), env_of(&[])),
            Err(ConfigError::Usage)
        );
    }

    #[test]
    fn non_numeric_wait_is_rejected() {
        let err = parse(&args(&["PORT", "ksit"]), env_of(&[("E00_WAIT_MS", "abc")]));

        assert!(matches!(
            err,
            Err(ConfigError::NotANumber {
                name: "E00_WAIT_MS",
                ..
            })
        ));
    }

    #[test]
    fn number_with_spaces_is_accepted() {
        let p = probe(&["PORT", "ksit"], &[("E00_WAIT_MS", " 500 ")]);

        assert_eq!(p.timing.wait, Duration::from_millis(500));
    }

    #[test]
    fn wait_bounds_are_inclusive() {
        for (value, ok) in [
            ("99", false),
            ("100", true),
            ("10000", true),
            ("10001", false),
        ] {
            let result = parse(&args(&["PORT", "ksit"]), env_of(&[("E00_WAIT_MS", value)]));

            assert_eq!(result.is_ok(), ok, "E00_WAIT_MS={value}");
        }
    }

    #[test]
    fn settle_may_be_zero() {
        let p = probe(&["PORT", "ksit"], &[("E00_SETTLE_MS", "0")]);

        assert_eq!(p.timing.settle, Duration::ZERO);
    }

    #[test]
    fn final_wait_out_of_range_is_rejected() {
        let err = parse(
            &args(&["PORT", "ksit"]),
            env_of(&[("E00_FINAL_WAIT_MS", "50")]),
        );

        assert!(matches!(
            err,
            Err(ConfigError::OutOfRange {
                name: "E00_FINAL_WAIT_MS",
                ..
            })
        ));
    }

    #[test]
    fn token_length_limit_is_sixteen() {
        let ok = "a".repeat(16);
        let too_long = "a".repeat(17);

        assert!(parse(&args(&["PORT", &ok]), env_of(&[])).is_ok());
        assert_eq!(
            parse(&args(&["PORT", &too_long]), env_of(&[])),
            Err(ConfigError::BadToken(too_long.clone()))
        );
    }

    #[test]
    fn empty_token_is_rejected() {
        assert_eq!(
            parse(&args(&["PORT", ""]), env_of(&[])),
            Err(ConfigError::BadToken(String::new()))
        );
    }

    #[test]
    fn token_with_symbols_is_rejected() {
        let err = parse(&args(&["PORT", "ksit;rm"]), env_of(&[]));

        assert_eq!(err, Err(ConfigError::BadToken("ksit;rm".to_owned())));
    }

    #[test]
    fn non_ascii_token_is_rejected() {
        let err = parse(&args(&["PORT", "kすわる"]), env_of(&[]));

        assert!(matches!(err, Err(ConfigError::BadToken(_))));
    }

    #[test]
    fn unknown_line_ending_is_rejected() {
        let err = parse(&args(&["PORT", "ksit"]), env_of(&[("E00_EOL", "CR")]));

        assert_eq!(err, Err(ConfigError::BadLineEnding("CR".to_owned())));
    }

    #[test]
    fn crlf_is_accepted() {
        let p = probe(&["PORT", "ksit"], &[("E00_EOL", "CRLF")]);

        assert_eq!(p.eol.as_str(), "\r\n");
    }

    #[test]
    fn error_messages_name_the_variable() {
        let err = ConfigError::OutOfRange {
            name: "E00_WAIT_MS",
            value: 1,
            min: 100,
            max: 10_000,
        };

        assert_eq!(err.to_string(), "E00_WAIT_MS=1 is outside 100..=10000");
    }

    // ---- hiding and formatting ----

    #[test]
    fn port_path_is_hidden_in_messages() {
        assert_eq!(
            hide_port("cannot open SERIAL-X: busy", "SERIAL-X"),
            "cannot open <PORT>: busy"
        );
    }

    #[test]
    fn port_base_name_is_hidden_too() {
        assert_eq!(
            hide_port("device DOG-1 gone", "/devices/DOG-1"),
            "device <PORT> gone"
        );
    }

    #[test]
    fn empty_port_hides_nothing() {
        assert_eq!(hide_port("text", ""), "text");
    }

    #[test]
    fn received_bytes_are_escaped() {
        assert_eq!(show_bytes(b"ok\r\n\xff"), "ok\\r\\n\\xff");
    }

    // ---- session ----

    #[derive(Clone, Default)]
    struct Lines(Rc<RefCell<Vec<String>>>);

    impl Log for Lines {
        fn line(&mut self, text: &str) {
            self.0.borrow_mut().push(text.to_owned());
        }
    }

    /// A port that records writes and can be told to fail.
    #[derive(Clone, Default)]
    struct FakePort {
        written: Rc<RefCell<Vec<u8>>>,
        failures_left: Rc<RefCell<u32>>,
    }

    impl FakePort {
        fn text(&self) -> String {
            String::from_utf8(self.written.borrow().clone()).unwrap()
        }

        fn fail_next(&self, n: u32) {
            *self.failures_left.borrow_mut() = n;
        }
    }

    impl Write for FakePort {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut left = self.failures_left.borrow_mut();
            if *left > 0 {
                *left -= 1;
                return Err(io::Error::new(io::ErrorKind::TimedOut, "fake timeout"));
            }
            self.written.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn session(port: &FakePort, log: &Lines) -> (Session<FakePort, Lines>, Arc<AtomicBool>) {
        let flag = Arc::new(AtomicBool::new(false));
        (
            Session::new(port.clone(), log.clone(), LineEnding::Lf, Arc::clone(&flag)),
            flag,
        )
    }

    fn timing() -> Timing {
        Timing {
            settle: Duration::ZERO,
            wait: Duration::ZERO,
            final_wait: Duration::ZERO,
        }
    }

    #[test]
    fn completed_run_sends_tokens_then_stop_once() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);

        let outcome = run(&mut s, &args(&["ksit", "khi"]), timing(), &mut |_| true);
        s.stop().unwrap();
        drop(s);

        assert_eq!(outcome.unwrap(), Outcome::Completed);
        assert_eq!(port.text(), "ksit\nkhi\n\nd\n");
    }

    #[test]
    fn stop_request_during_wait_sends_nothing_but_stop() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, flag) = session(&port, &log);
        let mut calls = 0;

        let outcome = run(
            &mut s,
            &args(&["ksit", "kwkF", "khi"]),
            timing(),
            &mut |_| {
                calls += 1;
                if calls == 2 {
                    flag.store(true, Ordering::SeqCst); // Ctrl-C while waiting after the first token
                    return false;
                }
                true
            },
        );
        drop(s);

        assert_eq!(outcome.unwrap(), Outcome::Interrupted);
        assert_eq!(port.text(), "ksit\n\nd\n");
    }

    #[test]
    fn send_is_refused_after_stop_request() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, flag) = session(&port, &log);
        flag.store(true, Ordering::SeqCst);

        let result = s.send("kwkF");

        assert!(matches!(result, Err(SendError::Stopping)));
        drop(s);
        assert_eq!(port.text(), "\nd\n");
    }

    #[test]
    fn write_error_still_leads_to_stop() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        port.fail_next(1);

        let outcome = run(&mut s, &args(&["ksit"]), timing(), &mut |_| true);
        drop(s);

        assert!(outcome.is_err());
        assert_eq!(port.text(), "\nd\n");
    }

    #[test]
    fn failed_stop_is_retried_on_drop() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        port.fail_next(1);

        let first = s.stop();
        drop(s);

        assert!(first.is_err());
        assert_eq!(port.text(), "\nd\n");
    }

    #[test]
    fn stop_failure_on_drop_is_reported() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (s, _) = session(&port, &log);
        port.fail_next(1);

        drop(s);

        assert!(log.0.borrow().iter().any(|l| l.contains(STOP_FAILED)));
    }

    #[test]
    fn stop_is_sent_only_once() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);

        s.stop().unwrap();
        s.stop().unwrap();
        drop(s);

        assert_eq!(port.text(), "\nd\n");
    }

    #[test]
    fn port_is_written_before_the_log_on_stop() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);

        s.stop().unwrap();

        assert!(s.stop_sent());
        assert!(
            log.0
                .borrow()
                .last()
                .is_some_and(|l| l.ends_with("-> \"d\""))
        );
    }

    #[test]
    fn interrupted_during_settle_sends_no_token() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);

        let outcome = run(&mut s, &args(&["kwkF"]), timing(), &mut |_| false);
        drop(s);

        assert_eq!(outcome.unwrap(), Outcome::Interrupted);
        assert_eq!(port.text(), "\nd\n");
    }
}
