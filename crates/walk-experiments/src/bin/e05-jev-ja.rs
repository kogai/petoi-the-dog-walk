//! E05: does a Japanese personality text work with Jev? Sends 2 pre-check requests and then 54
//! requests to the paid `TypeSafe` API, printing one JSON line per request.
//! Procedure: experimentals/E05-jev-japanese.md. Run by a human with their own API key; never by
//! an agent.
//!
//! The pre-checks (one Choice-only and one Noul-only request) must succeed before the main run
//! starts. In the main run each request is independent: a timeout or network error becomes an
//! error line and the run goes on; 401/403 stops the run. The key is read from
//! `TYPESAFE_API_KEY`, never printed, and hidden if a response echoes it
//! (`walk_experiments::jev_probe::render_line`). Output never panics.

use std::io::{self, Write};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use serde_json::Value;
use walk_experiments::jev_probe::{
    self, Case, KeyError, Reply, is_fatal_status, precheck_ok, render_line,
};

const TIMEOUT: Duration = Duration::from_secs(10);

/// Prints to stdout without panicking (output is redirected to a file).
fn out(text: &str) {
    let _ = writeln!(io::stdout().lock(), "{text}");
}

/// Prints to stderr without panicking.
fn warn(text: &str) {
    let _ = writeln!(io::stderr().lock(), "{text}");
}

fn agent(test_override: bool) -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .http_status_as_error(false);
    // A local test server must not be reached through a proxy (the key would pass through it).
    let config = if test_override {
        config.proxy(None)
    } else {
        config
    };
    config.build().into()
}

/// Sends one request. Latency covers reading the whole body.
fn ask(agent: &ureq::Agent, url: &str, key: &str, body: &Value) -> (Reply, u64) {
    let started = Instant::now();
    let reply = match agent
        .post(url)
        .header("Authorization", &format!("Bearer {key}"))
        .send_json(body)
    {
        Err(e) => Reply::Failed(e.to_string()),
        Ok(mut res) => {
            let status = res.status().as_u16();
            let body = res.body_mut().read_to_string().map_err(|e| e.to_string());
            Reply::Http { status, body }
        }
    };
    let latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    (reply, latency_ms)
}

fn run_case(agent: &ureq::Agent, url: &str, key: &str, case: &Case) -> Reply {
    let (reply, latency_ms) = ask(agent, url, key, &case.body);
    out(&render_line(&case.label, &reply, latency_ms, key));
    reply
}

fn main() -> ExitCode {
    let key = match jev_probe::validate_key(std::env::var("TYPESAFE_API_KEY").ok().as_deref()) {
        Ok(key) => key,
        Err(KeyError::Missing) => {
            warn("TYPESAFE_API_KEY is not set");
            return ExitCode::from(2);
        }
        Err(KeyError::Malformed) => {
            warn(
                "TYPESAFE_API_KEY contains spaces, quotes, backslashes or unusual characters (not shown)",
            );
            return ExitCode::from(2);
        }
    };
    let (url, test_override) =
        match jev_probe::api_url(std::env::var("E05_API_URL").ok().as_deref()) {
            Ok(pair) => pair,
            Err(rejected) => {
                warn(&format!(
                    "E05_API_URL must be a loopback http URL (test only): {rejected}"
                ));
                return ExitCode::from(2);
            }
        };
    let agent = agent(test_override);

    for (case, question) in jev_probe::prechecks()
        .iter()
        .zip(["next_action", "instruction_applies"])
    {
        let reply = run_case(&agent, &url, &key, case);
        if !precheck_ok(&reply, question) {
            warn(&format!(
                "stopped: the pre-check for {question} failed; the main run was not started (see the line above)"
            ));
            return ExitCode::from(1);
        }
    }
    warn("pre-checks passed");

    let cases = jev_probe::cases();
    let total = cases.len();
    for (i, case) in cases.iter().enumerate() {
        let reply = run_case(&agent, &url, &key, case);
        warn(&format!("{}/{total}", i + 1));
        if let Reply::Http { status, .. } = reply
            && is_fatal_status(status)
        {
            warn(&format!(
                "stopped: the API rejected the key (HTTP {status})"
            ));
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}
