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
/// Longest observation question (`E00_ASK`), in characters.
const MAX_QUESTION_CHARS: usize = 120;
/// Time to look at the robot after the question is shown, before the stop token is sent.
pub const LOOK_TIME: Duration = Duration::from_millis(1500);
/// Extra time after the planned end of a run before the watchdog sends the stop token itself.
pub const STOP_GRACE: Duration = Duration::from_millis(3000);
/// Invalid answers accepted before the question is given up as unanswered.
const MAX_ANSWER_ATTEMPTS: usize = 5;

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
    /// One yes/no question for the person watching, asked after the final wait (`E00_ASK`).
    pub ask: Option<String>,
    /// When the question is asked relative to the stop token (`E00_ASK_AFTER_STOP_MS`).
    pub ask_at: AskAt,
}

/// When [`observe`] asks its question, relative to the stop token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AskAt {
    /// While the robot is still moving; the stop token follows the look (the default).
    BeforeStop,
    /// This long after the stop token, to see whether the robot stopped (E13).
    AfterStop(Duration),
}

impl AskAt {
    /// Whether the person looks while the stop token is still to come, so that the stop is
    /// planned only after [`LOOK_TIME`] (see [`stop_deadline`]).
    #[must_use]
    pub fn looks_before_stop(self) -> bool {
        match self {
            Self::BeforeStop => true,
            Self::AfterStop(_) => false,
        }
    }
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
    /// `E00_ASK` that is too long or contains control characters.
    BadQuestion,
    /// `E00_ASK_AFTER_STOP_MS` without `E00_ASK`.
    AfterStopWithoutQuestion,
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
            Self::BadQuestion => write!(
                f,
                "E00_ASK: use at most {MAX_QUESTION_CHARS} characters and no control characters"
            ),
            Self::AfterStopWithoutQuestion => f.write_str(
                "E00_ASK_AFTER_STOP_MS needs E00_ASK (the question to ask after the stop)",
            ),
        }
    }
}

/// Usage text.
pub const USAGE: &str = "usage: e00-serial-probe <PORT> <TOKEN>...\n       e00-serial-probe --list\n\
env: E00_WAIT_MS (100..=10000, default 3000), E00_FINAL_WAIT_MS (100..=10000, default E00_WAIT_MS),\n     \
E00_SETTLE_MS (0..=10000, default 2000), E00_EOL (LF or CRLF, default LF), E00_LOG (log file),\n     \
E00_ASK (a yes/no question shown with a bell after the final wait; the robot keeps moving\n     \
1.5 s longer to be looked at, then the stop token is sent and the answer is typed),\n     \
E00_ASK_AFTER_STOP_MS (0..=10000; with E00_ASK, send the stop token first, ask this long after it,\n     \
and send the stop token again after the 1.5 s look)";

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

/// Invisible formatting characters that can reorder or hide text in a terminal or a pasted log
/// (bidirectional controls, zero-width characters, the byte order mark).
fn is_format_char(c: char) -> bool {
    matches!(c, '\u{061c}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}')
}

fn valid_question(question: &str) -> bool {
    question.chars().count() <= MAX_QUESTION_CHARS
        && !question
            .chars()
            .any(|c| c.is_control() || is_format_char(c))
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
            let ask = match env("E00_ASK").map(|q| q.trim().to_owned()) {
                Some(q) if q.is_empty() => None,
                Some(q) if valid_question(&q) => Some(q),
                Some(_) => return Err(ConfigError::BadQuestion),
                None => None,
            };
            let ask_at = match env("E00_ASK_AFTER_STOP_MS") {
                None => AskAt::BeforeStop,
                Some(_) if ask.is_none() => return Err(ConfigError::AfterStopWithoutQuestion),
                Some(_) => AskAt::AfterStop(Duration::from_millis(millis(
                    &env,
                    "E00_ASK_AFTER_STOP_MS",
                    0,
                    (0, 10_000),
                )?)),
            };
            Ok(Command::Probe(Probe {
                port: port.clone(),
                tokens: tokens.to_vec(),
                timing,
                eol,
                log_path,
                ask,
                ask_at,
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
/// token: [`Session::stop`] once (again from `Drop` if the first attempt failed), and
/// [`Session::stop_again`] each time it is called.
pub struct Session<W: Write, L: Log> {
    port: W,
    log: L,
    eol: LineEnding,
    stop_requested: Arc<AtomicBool>,
    stop_sent: Arc<AtomicBool>,
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
            stop_sent: Arc::new(AtomicBool::new(false)),
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

    /// Sends the stop token once ([`write_stop`]). The port is written before anything is logged.
    ///
    /// # Errors
    /// The write error. The stop is then not marked as sent, so `Drop` tries again.
    pub fn stop(&mut self) -> io::Result<()> {
        if self.stop_sent.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.stop_again()
    }

    /// Sends the stop token again, even if it was already sent. Used after looking at a robot
    /// that should already have stopped, so that it is not left moving if the first one did not
    /// work. The port is written before anything is logged. Only the main thread sends this one;
    /// the watchdog stops watching once the first stop token is out.
    ///
    /// # Errors
    /// The write error.
    pub fn stop_again(&mut self) -> io::Result<()> {
        write_stop(&mut self.port, self.eol)?;
        self.stop_sent.store(true, Ordering::SeqCst);
        self.note(&format!("-> {STOP_TOKEN:?}"));
        Ok(())
    }

    /// Whether the stop token has been written.
    #[must_use]
    pub fn stop_sent(&self) -> bool {
        self.stop_sent.load(Ordering::SeqCst)
    }

    /// A shared view of [`Session::stop_sent`], for a watchdog on another thread.
    #[must_use]
    pub fn stop_sent_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop_sent)
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

/// Writes the stop token and a line ending to `port`, with nothing before the token. Writes
/// nothing else, logs nothing and never blocks on the terminal, so it can be called from a signal
/// handler or a watchdog while the main thread is stuck on output.
///
/// No line ending goes before the token (inference from the `OpenCat` source, commit 24563ca,
/// `src/io.h` `read_serial`; **unverified** on the board): the firmware takes the first byte
/// waiting as the token and everything else waiting, up to the last line ending, as its argument.
/// So `"\nd\n"` is read as the token `\n` with the argument `d`, not as a stop. This matches the
/// `\n\r\n` reply to the stop seen in E00. For the same reason, a stop that arrives while another
/// token is still waiting to be read joins that token; the main thread sends `d` again when it
/// stops.
///
/// # Errors
/// The write error.
pub fn write_stop<W: Write>(port: &mut W, eol: LineEnding) -> io::Result<()> {
    port.write_all(format!("{STOP_TOKEN}{}", eol.as_str()).as_bytes())
}

/// Time from the start of a run after which the stop token should already have been sent:
/// the settle time, every wait, the look time if a question is asked, and [`STOP_GRACE`].
/// A watchdog sends the stop token itself when this passes, so a main thread stuck on terminal
/// output (for example after Ctrl-S) cannot leave the robot moving.
#[must_use]
pub fn stop_deadline(timing: Timing, tokens: usize, ask: bool) -> Duration {
    let between = u32::try_from(tokens.saturating_sub(1)).unwrap_or(u32::MAX);
    let waits = if tokens == 0 {
        Duration::ZERO
    } else {
        timing
            .wait
            .saturating_mul(between)
            .saturating_add(timing.final_wait)
    };
    let look = if ask { LOOK_TIME } else { Duration::ZERO };
    timing
        .settle
        .saturating_add(waits)
        .saturating_add(look)
        .saturating_add(STOP_GRACE)
}

/// What the watchdog does at one look at the clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchdogStep {
    /// Not yet due: look again later.
    Keep,
    /// The stop token was sent: nothing left to watch.
    Done,
    /// Due and the stop token was not sent: force the stop ([`force_stop`]).
    Fire,
}

/// Decides the watchdog's step from the time since the start, the deadline
/// ([`stop_deadline`]) and whether the stop token was already sent.
#[must_use]
pub fn watchdog_step(elapsed: Duration, deadline: Duration, stop_sent: bool) -> WatchdogStep {
    if stop_sent {
        WatchdogStep::Done
    } else if elapsed >= deadline {
        WatchdogStep::Fire
    } else {
        WatchdogStep::Keep
    }
}

/// Result of [`force_stop`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForcedStop {
    /// No stop had been requested before.
    pub first: bool,
    /// The stop token was written.
    pub written: bool,
}

/// Stops from outside the main thread (signal handler, watchdog): first requests the stop, so
/// that the main thread sends no further token when it resumes, then writes the stop token to a
/// second handle on the port. A token the main thread is writing at the same moment can still
/// get through, and the firmware may then read this `d` as part of it ([`write_stop`]); the main
/// thread sends `d` again when it stops.
pub fn force_stop<W: Write>(requested: &AtomicBool, port: &mut W, eol: LineEnding) -> ForcedStop {
    let first = !requested.swap(true, Ordering::SeqCst);
    let written = write_stop(port, eol).is_ok();
    ForcedStop { first, written }
}

/// How [`retry_stop`] ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryOutcome {
    /// One of the attempts wrote the stop token.
    Written,
    /// The main thread sent the stop token meanwhile.
    AlreadySent,
    /// Every attempt failed.
    Failed,
}

/// Tries to write the stop token up to `attempts` times, with `pause` between attempts, until a
/// write succeeds or `already_sent` says the main thread has sent it. Used by the watchdog, whose
/// single failed write would otherwise leave the robot moving while the main thread is stuck.
pub fn retry_stop(
    attempts: u32,
    write: &mut dyn FnMut() -> bool,
    already_sent: &dyn Fn() -> bool,
    pause: &mut dyn FnMut(),
) -> RetryOutcome {
    for attempt in 1..=attempts {
        if write() {
            return RetryOutcome::Written;
        }
        if already_sent() {
            return RetryOutcome::AlreadySent;
        }
        if attempt < attempts {
            pause();
        }
    }
    RetryOutcome::Failed
}

/// What the watchdog did, shared with the main thread as a `u8` ([`WatchdogReport::as_u8`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchdogReport {
    /// It did not fire, or the main thread sent the stop token while it was retrying.
    Idle,
    /// It wrote the stop token.
    Wrote,
    /// Every attempt to write the stop token failed.
    Failed,
}

impl WatchdogReport {
    /// The value stored in the shared atomic.
    #[must_use]
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Wrote => 1,
            Self::Failed => 2,
        }
    }

    /// Reads the shared value back. `None` for a value no report produces.
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        [Self::Idle, Self::Wrote, Self::Failed]
            .into_iter()
            .find(|r| r.as_u8() == value)
    }

    /// The report for a finished [`retry_stop`].
    #[must_use]
    pub fn from_retry(outcome: RetryOutcome) -> Self {
        match outcome {
            RetryOutcome::Written => Self::Wrote,
            RetryOutcome::Failed => Self::Failed,
            RetryOutcome::AlreadySent => Self::Idle,
        }
    }
}

/// What the signal handler does after [`force_stop`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalAction {
    /// First request: let the main thread finish (it sends `d` again and reports).
    Continue,
    /// A repeated signal and `d` was written: end now.
    Exit,
    /// A repeated signal and `d` could not be written: wait for the main thread's attempt.
    WaitForMain,
}

/// Decides the signal handler's action.
#[must_use]
pub fn signal_action(first: bool, written: bool) -> SignalAction {
    match (first, written) {
        (true, _) => SignalAction::Continue,
        (false, true) => SignalAction::Exit,
        (false, false) => SignalAction::WaitForMain,
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

/// An answer to the observation question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// `y`
    Yes,
    /// `n`
    No,
    /// `?`: the person could not tell.
    Unknown,
}

impl Answer {
    /// The word written to the log.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::Unknown => "unknown",
        }
    }
}

/// Reads `y`/`yes`, `n`/`no` or `?` (case and surrounding spaces ignored).
#[must_use]
pub fn parse_answer(input: &str) -> Option<Answer> {
    match input.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => Some(Answer::Yes),
        "n" | "no" => Some(Answer::No),
        "?" => Some(Answer::Unknown),
        _ => None,
    }
}

/// What [`observe`] shows on the terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prompt<'a> {
    /// The question, at the moment to look (the terminal rings the bell).
    LookNow(&'a str),
    /// The request to type the answer.
    TypeAnswer,
}

/// One read from the keyboard while [`observe`] waits for the answer.
#[derive(Debug, PartialEq, Eq)]
pub enum Input {
    /// A line the person typed.
    Line(String),
    /// Input ended.
    End,
    /// A stop was requested (Ctrl-C) while waiting.
    Stopped,
}

/// How [`observe`] ended.
#[derive(Debug, PartialEq, Eq)]
pub enum Observed {
    /// The person answered.
    Answered(Answer),
    /// Input ended, or too many invalid answers.
    NoAnswer,
    /// A stop was requested while looking or while waiting for the answer; the stop token was
    /// sent and no answer was recorded.
    Interrupted,
    /// [`AskAt::AfterStop`]: the first stop token went out, but the second one could not be
    /// written, even on a retry. No answer was read.
    SecondStopFailed(io::ErrorKind),
}

/// Asks one question about what the robot is doing *now*, so that the person watching has to
/// remember nothing: logs and shows the question (`show` also rings the bell), gives
/// [`LOOK_TIME`] to look, and only then reads the answer, so the robot is not left moving while
/// the person types.
///
/// With [`AskAt::BeforeStop`] the stop token is sent after the look. With
/// [`AskAt::AfterStop`] it is sent first, the question comes the given time later, and the stop
/// token is sent again after the look.
///
/// # Errors
/// A stop token's write error; no answer is read then.
pub fn observe<W: Write, L: Log>(
    session: &mut Session<W, L>,
    question: &str,
    ask_at: AskAt,
    sleep: &mut dyn FnMut(Duration) -> bool,
    show: &mut dyn FnMut(Prompt<'_>),
    read_line: &mut dyn FnMut() -> Input,
) -> io::Result<Observed> {
    let looked = match ask_at {
        AskAt::BeforeStop => {
            session.note(&format!("?? {question}"));
            show(Prompt::LookNow(question));
            let looked = sleep(LOOK_TIME);
            session.stop()?;
            looked
        }
        AskAt::AfterStop(delay) => {
            session.stop()?;
            if !sleep(delay) {
                return Ok(Observed::Interrupted);
            }
            session.note(&format!("?? {question}"));
            show(Prompt::LookNow(question));
            let looked = sleep(LOOK_TIME);
            // The first stop already counts as sent, so neither the caller nor `Drop` would
            // write this one again: retry here, and report a failure as its own outcome.
            if let Err(e) = session.stop_again().or_else(|_| session.stop_again()) {
                session.note(&format!("second stop token not sent ({})", e.kind()));
                return Ok(Observed::SecondStopFailed(e.kind()));
            }
            looked
        }
    };
    if !looked {
        return Ok(Observed::Interrupted);
    }
    for _ in 0..MAX_ANSWER_ATTEMPTS {
        show(Prompt::TypeAnswer);
        match read_line() {
            Input::Line(line) => {
                if let Some(answer) = parse_answer(&line) {
                    session.note(&format!("answer: {}", answer.label()));
                    return Ok(Observed::Answered(answer));
                }
            }
            Input::End => break,
            Input::Stopped => {
                session.note("answer: interrupted");
                return Ok(Observed::Interrupted);
            }
        }
    }
    session.note("answer: none");
    Ok(Observed::NoAnswer)
}

/// How the program ends, from the most important problem down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    /// Everything was sent and the stop token went out the first time.
    Success,
    /// A stop was requested; the stop token was sent.
    Interrupted,
    /// A token could not be written; the stop token was sent.
    SendFailed,
    /// The first stop attempt failed and a retry was needed. The robot kept moving a little
    /// longer than planned, so the run is not a clean one.
    StopRetried,
    /// The stop token could not be sent at all.
    StopFailed,
    /// [`AskAt::AfterStop`]: the first stop token went out, the second could not be sent.
    SecondStopFailed,
}

impl Exit {
    /// The process exit status.
    #[must_use]
    pub fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::SendFailed => 1,
            Self::StopRetried | Self::StopFailed | Self::SecondStopFailed => 3,
            Self::Interrupted => 130,
        }
    }
}

/// Decides how the program ends. `stop_ok` is whether the final stop attempt succeeded.
#[must_use]
pub fn exit_status(
    outcome: &io::Result<Outcome>,
    observed: Option<&io::Result<Observed>>,
    stop_ok: bool,
) -> Exit {
    if !stop_ok {
        return Exit::StopFailed;
    }
    match observed {
        Some(Err(_)) => return Exit::StopRetried,
        Some(Ok(Observed::SecondStopFailed(_))) => return Exit::SecondStopFailed,
        Some(Ok(Observed::Answered(_) | Observed::NoAnswer | Observed::Interrupted)) | None => {}
    }
    match outcome {
        Err(_) => Exit::SendFailed,
        Ok(Outcome::Interrupted) => Exit::Interrupted,
        Ok(Outcome::Completed) => match observed {
            Some(Ok(Observed::Interrupted)) => Exit::Interrupted,
            Some(
                Ok(Observed::Answered(_) | Observed::NoAnswer | Observed::SecondStopFailed(_))
                | Err(_),
            )
            | None => Exit::Success,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
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
        assert_eq!(port.text(), "ksit\nkhi\nd\n");
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
        assert_eq!(port.text(), "ksit\nd\n");
    }

    #[test]
    fn send_is_refused_after_stop_request() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, flag) = session(&port, &log);
        flag.store(true, Ordering::SeqCst);

        let result = s.send("kwkF");

        assert!(matches!(result, Err(SendError::Stopping)));
        drop(s);
        assert_eq!(port.text(), "d\n");
    }

    #[test]
    fn write_error_still_leads_to_stop() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        port.fail_next(1);

        let outcome = run(&mut s, &args(&["ksit"]), timing(), &mut |_| true);
        drop(s);

        assert!(outcome.is_err());
        assert_eq!(port.text(), "d\n");
    }

    #[test]
    fn failed_stop_is_retried_on_drop() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        port.fail_next(1);

        let first = s.stop();
        drop(s);

        assert!(first.is_err());
        assert_eq!(port.text(), "d\n");
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

        assert_eq!(port.text(), "d\n");
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
        assert_eq!(port.text(), "d\n");
    }
    // ---- observation question ----

    #[test]
    fn question_is_read_and_trimmed() {
        let p = probe(&["PORT", "kwkF"], &[("E00_ASK", "  まだ動いていますか？ ")]);

        assert_eq!(p.ask.as_deref(), Some("まだ動いていますか？"));
    }

    #[test]
    fn no_question_by_default() {
        let p = probe(&["PORT", "kwkF"], &[]);

        assert_eq!(p.ask, None);
    }

    #[test]
    fn empty_question_means_none() {
        let p = probe(&["PORT", "kwkF"], &[("E00_ASK", "   ")]);

        assert_eq!(p.ask, None);
    }

    #[test]
    fn question_with_control_characters_is_rejected() {
        let result = parse(
            &args(&["PORT", "kwkF"]),
            env_of(&[("E00_ASK", "moving?\x1b[2J")]),
        );

        assert_eq!(result, Err(ConfigError::BadQuestion));
    }

    #[test]
    fn question_length_limit_is_in_characters() {
        let ok = "あ".repeat(MAX_QUESTION_CHARS);
        let long = "あ".repeat(MAX_QUESTION_CHARS + 1);

        assert_eq!(
            probe(&["PORT", "kwkF"], &[("E00_ASK", &ok)]).ask.as_deref(),
            Some(ok.as_str())
        );
        assert_eq!(
            parse(&args(&["PORT", "kwkF"]), env_of(&[("E00_ASK", &long)])),
            Err(ConfigError::BadQuestion)
        );
    }

    #[test]
    fn answers_are_y_n_or_question_mark() {
        assert_eq!(parse_answer("y"), Some(Answer::Yes));
        assert_eq!(parse_answer(" YES \n"), Some(Answer::Yes));
        assert_eq!(parse_answer("n"), Some(Answer::No));
        assert_eq!(parse_answer("No"), Some(Answer::No));
        assert_eq!(parse_answer("?"), Some(Answer::Unknown));
        assert_eq!(parse_answer(""), None);
        assert_eq!(parse_answer("maybe"), None);
    }

    /// In scripted answers: Ctrl-C while waiting for the answer.
    const STOPPED: &str = "<stopped>";

    /// Runs `observe` with scripted answers and records the order of events.
    fn observe_with(
        answers: &[&str],
        look_ok: bool,
        ask_at: AskAt,
    ) -> (io::Result<Observed>, FakePort, Lines, Vec<String>) {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut queue: Vec<Input> = answers
            .iter()
            .rev()
            .map(|a| match *a {
                STOPPED => Input::Stopped,
                text => Input::Line(text.to_owned()),
            })
            .collect();
        let (e1, e2, e3) = (Rc::clone(&events), Rc::clone(&events), Rc::clone(&events));
        let (port_seen, port_at_sleep) = (port.clone(), port.clone());
        let result = observe(
            &mut s,
            "moving?",
            ask_at,
            &mut |d| {
                e1.borrow_mut().push(format!(
                    "sleep {} (port: {:?})",
                    d.as_millis(),
                    port_at_sleep.text()
                ));
                look_ok
            },
            &mut |prompt| e2.borrow_mut().push(format!("show {prompt:?}")),
            &mut || {
                e3.borrow_mut()
                    .push(format!("read (port: {:?})", port_seen.text()));
                queue.pop().unwrap_or(Input::End)
            },
        );
        drop(s);
        let events = events.borrow().clone();
        (result, port, log, events)
    }

    #[test]
    fn question_is_shown_then_robot_is_stopped_before_the_answer_is_read() {
        let (result, port, log, events) = observe_with(&["y"], true, AskAt::BeforeStop);

        assert_eq!(result.unwrap(), Observed::Answered(Answer::Yes));
        assert!(events.first().is_some_and(|e| e.starts_with("show ")));
        assert_eq!(
            events.get(1).map(String::as_str),
            Some("sleep 1500 (port: \"\")")
        );
        // The stop token was already on the port when the answer was read.
        assert!(events.iter().any(|e| e == "read (port: \"d\\n\")"));
        assert_eq!(port.text(), "d\n");
        let lines = log.0.borrow();
        assert!(lines.iter().any(|l| l.ends_with("?? moving?")));
        assert!(lines.iter().any(|l| l.ends_with("answer: yes")));
    }

    #[test]
    fn invalid_answers_are_asked_again() {
        let (result, _, log, _) = observe_with(&["maybe", "", "n"], true, AskAt::BeforeStop);

        assert_eq!(result.unwrap(), Observed::Answered(Answer::No));
        assert!(log.0.borrow().iter().any(|l| l.ends_with("answer: no")));
    }

    #[test]
    fn end_of_input_records_no_answer() {
        let (result, port, log, _) = observe_with(&[], true, AskAt::BeforeStop);

        assert_eq!(result.unwrap(), Observed::NoAnswer);
        assert_eq!(port.text(), "d\n");
        assert!(log.0.borrow().iter().any(|l| l.ends_with("answer: none")));
    }

    #[test]
    fn too_many_invalid_answers_record_no_answer() {
        let mut script = vec!["x"; MAX_ANSWER_ATTEMPTS];
        script.push("y");
        let (result, _, log, events) = observe_with(&script, true, AskAt::BeforeStop);

        assert_eq!(result.unwrap(), Observed::NoAnswer);
        let reads = events.iter().filter(|e| e.starts_with("read ")).count();
        assert_eq!(reads, MAX_ANSWER_ATTEMPTS);
        assert!(log.0.borrow().iter().any(|l| l.ends_with("answer: none")));
    }

    #[test]
    fn last_allowed_attempt_is_still_accepted() {
        let mut script = vec!["x"; MAX_ANSWER_ATTEMPTS - 1];
        script.push("y");
        let (result, _, _, _) = observe_with(&script, true, AskAt::BeforeStop);

        assert_eq!(result.unwrap(), Observed::Answered(Answer::Yes));
    }

    #[test]
    fn stop_while_waiting_for_the_answer_is_an_interruption() {
        let (result, port, log, _) = observe_with(&[STOPPED], true, AskAt::BeforeStop);

        assert_eq!(result.unwrap(), Observed::Interrupted);
        assert_eq!(port.text(), "d\n");
        let lines = log.0.borrow();
        assert!(lines.iter().any(|l| l.ends_with("answer: interrupted")));
        assert!(!lines.iter().any(|l| l.ends_with("answer: none")));
    }

    #[test]
    fn format_characters_in_the_question_are_rejected() {
        for bad in [
            "a\u{061c}b",
            "a\u{202e}b",
            "a\u{200b}b",
            "a\u{2066}b",
            "a\u{feff}b",
        ] {
            assert_eq!(
                parse(&args(&["PORT", "kwkF"]), env_of(&[("E00_ASK", bad)])),
                Err(ConfigError::BadQuestion),
                "{bad:?}"
            );
        }
    }

    // ---- exit status ----

    #[test]
    fn exit_codes_follow_the_most_important_problem() {
        let ok: io::Result<Outcome> = Ok(Outcome::Completed);
        let interrupted: io::Result<Outcome> = Ok(Outcome::Interrupted);
        let send_failed: io::Result<Outcome> = Err(io::Error::other("send"));
        let answered: io::Result<Observed> = Ok(Observed::Answered(Answer::Yes));
        let no_answer: io::Result<Observed> = Ok(Observed::NoAnswer);
        let observe_interrupted: io::Result<Observed> = Ok(Observed::Interrupted);
        let observe_stop_failed: io::Result<Observed> = Err(io::Error::other("stop"));

        assert_eq!(exit_status(&ok, None, true), Exit::Success);
        assert_eq!(exit_status(&ok, Some(&answered), true), Exit::Success);
        assert_eq!(exit_status(&ok, Some(&no_answer), true), Exit::Success);
        assert_eq!(exit_status(&interrupted, None, true), Exit::Interrupted);
        assert_eq!(
            exit_status(&ok, Some(&observe_interrupted), true),
            Exit::Interrupted
        );
        assert_eq!(exit_status(&send_failed, None, true), Exit::SendFailed);
        // The first stop attempt failed inside `observe`, even if the retry worked.
        assert_eq!(
            exit_status(&ok, Some(&observe_stop_failed), true),
            Exit::StopRetried
        );
        assert_eq!(exit_status(&ok, Some(&answered), false), Exit::StopFailed);
        assert_eq!(exit_status(&send_failed, None, false), Exit::StopFailed);

        assert_eq!(Exit::Success.code(), 0);
        assert_eq!(Exit::SendFailed.code(), 1);
        assert_eq!(Exit::StopFailed.code(), 3);
        assert_eq!(Exit::StopRetried.code(), 3);
        assert_eq!(Exit::Interrupted.code(), 130);
    }

    #[test]
    fn interrupt_while_looking_stops_and_asks_nothing() {
        let (result, port, _, events) = observe_with(&["y"], false, AskAt::BeforeStop);

        assert_eq!(result.unwrap(), Observed::Interrupted);
        assert_eq!(port.text(), "d\n");
        assert!(!events.iter().any(|e| e.starts_with("read ")));
    }

    #[test]
    fn failed_stop_is_returned_and_no_answer_is_read() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        port.fail_next(1);
        let mut read = false;

        let result = observe(
            &mut s,
            "moving?",
            AskAt::BeforeStop,
            &mut |_| true,
            &mut |_| {},
            &mut || {
                read = true;
                Input::End
            },
        );
        drop(s); // retries the stop

        assert!(result.is_err());
        assert!(!read);
        assert_eq!(port.text(), "d\n");
    }
    // ---- asking after the stop (E13) ----

    #[test]
    fn question_is_asked_before_the_stop_by_default() {
        let p = probe(&["PORT", "kwkF"], &[("E00_ASK", "moving?")]);

        assert_eq!(p.ask_at, AskAt::BeforeStop);
    }

    #[test]
    fn ask_after_stop_is_read() {
        let p = probe(
            &["PORT", "kwkF"],
            &[("E00_ASK", "moving?"), ("E00_ASK_AFTER_STOP_MS", "2000")],
        );

        assert_eq!(p.ask_at, AskAt::AfterStop(Duration::from_millis(2000)));
    }

    #[test]
    fn ask_after_stop_without_a_question_is_rejected() {
        let result = parse(
            &args(&["PORT", "kwkF"]),
            env_of(&[("E00_ASK_AFTER_STOP_MS", "1000")]),
        );

        assert_eq!(result, Err(ConfigError::AfterStopWithoutQuestion));
    }

    #[test]
    fn ask_after_stop_range_is_checked() {
        let result = parse(
            &args(&["PORT", "kwkF"]),
            env_of(&[("E00_ASK", "moving?"), ("E00_ASK_AFTER_STOP_MS", "10001")]),
        );

        assert_eq!(
            result,
            Err(ConfigError::OutOfRange {
                name: "E00_ASK_AFTER_STOP_MS",
                value: 10_001,
                min: 0,
                max: 10_000,
            })
        );
    }

    #[test]
    fn after_stop_the_stop_goes_out_first_and_again_after_looking() {
        let (result, port, log, events) =
            observe_with(&["n"], true, AskAt::AfterStop(Duration::from_millis(2000)));

        assert_eq!(result.unwrap(), Observed::Answered(Answer::No));
        // The stop token is on the port before the wait and before the question.
        assert_eq!(
            events.first().map(String::as_str),
            Some("sleep 2000 (port: \"d\\n\")")
        );
        assert!(events.get(1).is_some_and(|e| e.starts_with("show LookNow")));
        assert_eq!(
            events.get(2).map(String::as_str),
            Some("sleep 1500 (port: \"d\\n\")")
        );
        // A second stop goes out after looking, before the answer is read.
        assert!(events.iter().any(|e| e == "read (port: \"d\\nd\\n\")"));
        assert_eq!(port.text(), "d\nd\n");
        let lines = log.0.borrow();
        assert!(lines.iter().any(|l| l.ends_with("?? moving?")));
        assert!(lines.iter().any(|l| l.ends_with("answer: no")));
    }

    #[test]
    fn interrupt_during_the_wait_after_stop_asks_nothing() {
        let (result, port, _, events) =
            observe_with(&["y"], false, AskAt::AfterStop(Duration::from_millis(2000)));

        assert_eq!(result.unwrap(), Observed::Interrupted);
        assert_eq!(port.text(), "d\n");
        assert!(!events.iter().any(|e| e.starts_with("show ")));
        assert!(!events.iter().any(|e| e.starts_with("read ")));
    }

    #[test]
    fn failed_first_stop_after_stop_asks_nothing() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        port.fail_next(1);
        let mut shown = false;

        let result = observe(
            &mut s,
            "moving?",
            AskAt::AfterStop(Duration::from_millis(2000)),
            &mut |_| true,
            &mut |_| shown = true,
            &mut || Input::End,
        );
        drop(s); // retries the stop

        assert!(result.is_err());
        assert!(!shown);
        assert_eq!(port.text(), "d\n");
    }

    /// Runs `observe` after the stop with `sleep` answering from `sleeps` in order (then `true`).
    fn observe_after_stop(sleeps: &[bool]) -> (io::Result<Observed>, FakePort, bool) {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        let mut calls = 0;
        let mut read = false;
        let result = observe(
            &mut s,
            "moving?",
            AskAt::AfterStop(Duration::from_millis(2000)),
            &mut |_| {
                let ok = sleeps.get(calls).copied().unwrap_or(true);
                calls += 1;
                ok
            },
            &mut |_| {},
            &mut || {
                read = true;
                Input::Line("y".to_owned())
            },
        );
        drop(s);
        (result, port, read)
    }

    #[test]
    fn interrupt_while_looking_after_stop_sends_the_second_stop_and_reads_nothing() {
        let (result, port, read) = observe_after_stop(&[true, false]);

        assert_eq!(result.unwrap(), Observed::Interrupted);
        assert_eq!(port.text(), "d\nd\n");
        assert!(!read);
    }

    #[test]
    fn second_stop_that_fails_once_is_sent_by_the_retry() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        let mut calls = 0;
        let result = observe(
            &mut s,
            "moving?",
            AskAt::AfterStop(Duration::from_millis(2000)),
            &mut |_| {
                calls += 1;
                if calls == 2 {
                    port.fail_next(1); // the second stop's first attempt
                }
                true
            },
            &mut |_| {},
            &mut || Input::Line("n".to_owned()),
        );
        drop(s);

        assert_eq!(result.unwrap(), Observed::Answered(Answer::No));
        assert_eq!(port.text(), "d\nd\n");
    }

    #[test]
    fn second_stop_that_cannot_be_sent_is_reported_and_no_answer_is_read() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        let mut calls = 0;
        let mut read = false;
        let result = observe(
            &mut s,
            "moving?",
            AskAt::AfterStop(Duration::from_millis(2000)),
            &mut |_| {
                calls += 1;
                if calls == 2 {
                    port.fail_next(2); // the second stop and its retry
                }
                true
            },
            &mut |_| {},
            &mut || {
                read = true;
                Input::End
            },
        );
        drop(s); // the first stop went out, so Drop writes nothing more

        assert_eq!(
            result.unwrap(),
            Observed::SecondStopFailed(io::ErrorKind::TimedOut)
        );
        assert!(!read);
        assert_eq!(port.text(), "d\n");
    }

    #[test]
    fn failed_second_stop_has_its_own_exit() {
        let observed: io::Result<Observed> =
            Ok(Observed::SecondStopFailed(io::ErrorKind::TimedOut));

        let exit = exit_status(&Ok(Outcome::Completed), Some(&observed), true);

        assert_eq!(exit, Exit::SecondStopFailed);
        assert_eq!(exit.code(), 3);
    }

    #[test]
    fn deadline_has_no_look_time_when_asking_after_the_stop() {
        let t = timing();

        assert_eq!(
            stop_deadline(
                t,
                1,
                AskAt::AfterStop(Duration::from_millis(5000)).looks_before_stop()
            ),
            stop_deadline(t, 1, false)
        );
        assert!(AskAt::BeforeStop.looks_before_stop());
    }

    // ---- stop independent of the main thread ----

    #[test]
    fn stop_bytes_are_the_stop_token_and_a_line_ending_only() {
        // No line ending before `d`: the firmware would read `\n` as the token and `d` as its
        // argument (see `write_stop`).
        let port = FakePort::default();
        let mut w = port.clone();

        write_stop(&mut w, LineEnding::Lf).unwrap();
        write_stop(&mut w, LineEnding::CrLf).unwrap();

        assert_eq!(port.text(), "d\nd\r\n");
    }

    #[test]
    fn session_stop_writes_the_same_bytes_as_write_stop() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        let other = FakePort::default();

        s.stop().unwrap();
        write_stop(&mut other.clone(), LineEnding::Lf).unwrap();

        assert_eq!(port.text(), other.text());
    }

    #[test]
    fn deadline_covers_every_wait_plus_grace() {
        let timing = Timing {
            settle: Duration::from_millis(8000),
            wait: Duration::from_millis(3000),
            final_wait: Duration::from_millis(5000),
        };

        assert_eq!(
            stop_deadline(timing, 1, false),
            Duration::from_millis(8000 + 5000) + STOP_GRACE
        );
        assert_eq!(
            stop_deadline(timing, 3, false),
            Duration::from_millis(8000 + 3000 * 2 + 5000) + STOP_GRACE
        );
        assert_eq!(
            stop_deadline(timing, 3, true),
            Duration::from_millis(8000 + 3000 * 2 + 5000) + LOOK_TIME + STOP_GRACE
        );
    }

    #[test]
    fn deadline_with_no_tokens_is_settle_plus_grace() {
        let timing = Timing {
            settle: Duration::from_millis(2000),
            wait: Duration::from_millis(3000),
            final_wait: Duration::from_millis(3000),
        };

        assert_eq!(
            stop_deadline(timing, 0, false),
            Duration::from_millis(2000) + STOP_GRACE
        );
    }
    // ---- watchdog and signal decisions ----

    #[test]
    fn watchdog_keeps_waiting_before_the_deadline() {
        let d = Duration::from_millis(1000);

        assert_eq!(
            watchdog_step(Duration::from_millis(999), d, false),
            WatchdogStep::Keep
        );
    }

    #[test]
    fn watchdog_fires_at_the_deadline_if_the_stop_was_not_sent() {
        let d = Duration::from_millis(1000);

        assert_eq!(watchdog_step(d, d, false), WatchdogStep::Fire);
        assert_eq!(watchdog_step(d * 2, d, false), WatchdogStep::Fire);
    }

    #[test]
    fn watchdog_is_done_once_the_stop_was_sent() {
        let d = Duration::from_millis(1000);

        assert_eq!(watchdog_step(Duration::ZERO, d, true), WatchdogStep::Done);
        assert_eq!(watchdog_step(d * 2, d, true), WatchdogStep::Done);
    }

    /// A port that records whether the stop was already requested when it was written to.
    struct CheckingPort {
        requested: Arc<AtomicBool>,
        seen: Rc<RefCell<Vec<bool>>>,
        inner: FakePort,
    }

    impl Write for CheckingPort {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.seen
                .borrow_mut()
                .push(self.requested.load(Ordering::SeqCst));
            self.inner.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn force_stop_requests_the_stop_before_writing() {
        let requested = Arc::new(AtomicBool::new(false));
        let seen = Rc::new(RefCell::new(Vec::new()));
        let inner = FakePort::default();
        let mut port = CheckingPort {
            requested: Arc::clone(&requested),
            seen: Rc::clone(&seen),
            inner: inner.clone(),
        };

        let first = force_stop(&requested, &mut port, LineEnding::Lf);
        let second = force_stop(&requested, &mut port, LineEnding::Lf);

        assert_eq!(
            first,
            ForcedStop {
                first: true,
                written: true
            }
        );
        assert_eq!(
            second,
            ForcedStop {
                first: false,
                written: true
            }
        );
        assert!(seen.borrow().iter().all(|requested_then| *requested_then));
        assert_eq!(inner.text(), "d\nd\n");
    }

    #[test]
    fn force_stop_reports_a_failed_write_but_still_requests_the_stop() {
        let requested = Arc::new(AtomicBool::new(false));
        let port = FakePort::default();
        port.fail_next(1);

        let result = force_stop(&requested, &mut port.clone(), LineEnding::Lf);

        assert_eq!(
            result,
            ForcedStop {
                first: true,
                written: false
            }
        );
        assert!(requested.load(Ordering::SeqCst));
    }

    #[test]
    fn no_token_is_sent_after_a_forced_stop() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, flag) = session(&port, &log);
        let side = FakePort::default();
        let mut calls = 0;

        let outcome = run(
            &mut s,
            &args(&["ksit", "kwkF", "khi"]),
            timing(),
            &mut |_| {
                calls += 1;
                if calls == 2 {
                    // The watchdog (or the signal handler) fires while waiting after `ksit`.
                    let _ = force_stop(&flag, &mut side.clone(), LineEnding::Lf);
                }
                !flag.load(Ordering::SeqCst)
            },
        );
        drop(s);

        assert_eq!(outcome.unwrap(), Outcome::Interrupted);
        assert_eq!(port.text(), "ksit\nd\n");
        assert_eq!(side.text(), "d\n");
    }

    #[test]
    fn stop_sent_flag_is_shared_and_set_by_stop() {
        let (port, log) = (FakePort::default(), Lines::default());
        let (mut s, _) = session(&port, &log);
        let flag = s.stop_sent_flag();

        assert!(!flag.load(Ordering::SeqCst));
        s.stop().unwrap();
        assert!(flag.load(Ordering::SeqCst));
    }

    #[test]
    fn signal_actions() {
        assert_eq!(signal_action(true, true), SignalAction::Continue);
        assert_eq!(signal_action(true, false), SignalAction::Continue);
        assert_eq!(signal_action(false, true), SignalAction::Exit);
        assert_eq!(signal_action(false, false), SignalAction::WaitForMain);
    }
    #[test]
    fn retry_stops_at_the_first_successful_write() {
        let mut attempts = 0;

        let result = retry_stop(
            5,
            &mut || {
                attempts += 1;
                attempts == 3
            },
            &|| false,
            &mut || {},
        );

        assert_eq!(result, RetryOutcome::Written);
        assert_eq!(attempts, 3);
    }

    #[test]
    fn retry_gives_up_after_the_limit() {
        let mut attempts = 0;
        let mut pauses = 0;

        let result = retry_stop(
            4,
            &mut || {
                attempts += 1;
                false
            },
            &|| false,
            &mut || pauses += 1,
        );

        assert_eq!(result, RetryOutcome::Failed);
        assert_eq!(attempts, 4);
        assert_eq!(pauses, 3);
    }

    #[test]
    fn retry_ends_when_the_main_thread_has_sent_the_stop() {
        let sent = Cell::new(false);
        let mut attempts = 0;

        let result = retry_stop(
            5,
            &mut || {
                attempts += 1;
                sent.set(true); // the main thread got going and sent `d` meanwhile
                false
            },
            &|| sent.get(),
            &mut || {},
        );

        assert_eq!(result, RetryOutcome::AlreadySent);
        assert_eq!(attempts, 1);
    }

    #[test]
    fn watchdog_report_round_trips_through_u8() {
        for r in [
            WatchdogReport::Idle,
            WatchdogReport::Wrote,
            WatchdogReport::Failed,
        ] {
            assert_eq!(WatchdogReport::from_u8(r.as_u8()), Some(r));
        }
        assert_eq!(WatchdogReport::from_u8(7), None);
        assert_eq!(
            WatchdogReport::from_retry(RetryOutcome::Written),
            WatchdogReport::Wrote
        );
        assert_eq!(
            WatchdogReport::from_retry(RetryOutcome::Failed),
            WatchdogReport::Failed
        );
        assert_eq!(
            WatchdogReport::from_retry(RetryOutcome::AlreadySent),
            WatchdogReport::Idle
        );
    }
}
