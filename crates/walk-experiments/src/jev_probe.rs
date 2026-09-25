//! Testable core of `e05-jev-ja` (experimentals/E05-jev-japanese.md): the requests, hiding the
//! API key in everything printed, and the loopback-only URL override.
//!
//! The request shape follows docs/design/facts.md F-J9 (read from a third-party repository, not
//! the official docs). The `"noul"` question type is a guess; the pre-checks exist to test it.

use serde_json::{Map, Value, json};
use ureq::http::Uri;

/// Endpoint (F-J9).
pub const API_URL: &str = "https://api.typesafe.ai/v1/systemone";
/// Model name (F-J9).
pub const MODEL: &str = "jev-latest";
/// Longest response text kept when the body is not JSON.
pub const MAX_TEXT: usize = 500;

/// Which texts are Japanese in a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    /// Personality, situation and instruction in Japanese; questions and option descriptions in
    /// English (in the product, those fixed texts are written by developers, in English).
    JaState,
    /// Everything in English.
    En,
    /// Everything in English again, as a baseline for how often Jev disagrees with itself.
    EnRepeat,
}

impl Condition {
    /// Code used in the output.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::JaState => "ja-state",
            Self::En => "en",
            Self::EnRepeat => "en-repeat",
        }
    }

    fn japanese_state(self) -> bool {
        match self {
            Self::JaState => true,
            Self::En | Self::EnRepeat => false,
        }
    }
}

struct Text {
    ja: &'static str,
    en: &'static str,
}

impl Text {
    fn get(&self, japanese: bool) -> &'static str {
        if japanese { self.ja } else { self.en }
    }
}

const PERSONALITIES: [(&str, Text); 3] = [
    (
        "shy",
        Text {
            ja: "とても臆病。知らないものが近づくと、すぐに後ずさりする。",
            en: "Very shy. When something unfamiliar approaches, it backs away immediately.",
        },
    ),
    (
        "friendly",
        Text {
            ja: "人懐っこい。人を見ると近寄って挨拶する。",
            en: "Friendly. When it sees a person, it walks up and greets them.",
        },
    ),
    (
        "lazy",
        Text {
            ja: "怠け者。できるだけ座っていたい。",
            en: "Lazy. Wants to stay sitting as much as possible.",
        },
    ),
];

const SITUATIONS: [Text; 3] = [
    Text {
        ja: "知らない人が近づいてきた。",
        en: "A stranger is approaching.",
    },
    Text {
        ja: "飼い主が名前を呼んだ。",
        en: "The owner called its name.",
    },
    Text {
        ja: "何も起きていない。",
        en: "Nothing is happening.",
    },
];

const INSTRUCTION: Text = Text {
    ja: "人が近づいたら座って。",
    en: "Sit down when a person approaches.",
};

/// Action names offered as Choice options.
pub const ACTIONS: [&str; 5] = ["walk", "back", "sit", "hello", "balance"];
const ACTION_TEXT: [&str; 5] = [
    "Walk forward",
    "Step backward",
    "Sit down",
    "Greet",
    "Stand still",
];

fn criteria() -> Value {
    Value::Object(
        ACTIONS
            .iter()
            .zip(ACTION_TEXT)
            .map(|(k, v)| ((*k).to_owned(), Value::from(v)))
            .collect(),
    )
}

fn noul_question() -> Value {
    json!({
        "type": "noul",
        "instructions": "The user's current instruction applies to the current situation.",
    })
}

fn choice_question(instructions: &str) -> Value {
    json!({ "type": "choice", "instructions": instructions, "criteria": criteria() })
}

/// All three questions (English; only the state changes with the condition).
fn questions() -> Value {
    json!({
        "instruction_applies": noul_question(),
        "next_action": choice_question("Which action should the robot take next?"),
        // D-11 candidate (a): a second Choice that ignores the instruction.
        "personality_action": choice_question(
            "Ignoring any instruction, which action fits the robot's personality best right now?"
        ),
    })
}

/// One request to send.
#[derive(Debug, PartialEq)]
pub struct Case {
    /// Fields that identify the case in the output line.
    pub label: Map<String, Value>,
    /// Request body.
    pub body: Value,
}

fn state(personality: &Text, situation: &Text, with_instruction: bool, ja: bool) -> Value {
    json!({
        "personality": personality.get(ja),
        "situation": situation.get(ja),
        "instruction": if with_instruction { Value::from(INSTRUCTION.get(ja)) } else { Value::Null },
    })
}

/// Two small requests sent first: one with only a Choice, one with only a Noul. If either fails,
/// the request shape (F-J9) or the Noul type is wrong, and the paid main run is not started.
#[must_use]
pub fn prechecks() -> Vec<Case> {
    let (_, shy) = &PERSONALITIES[0];
    let base = state(shy, &SITUATIONS[0], true, false);
    [
        (
            "choice",
            json!({ "next_action": choice_question("Which action should the robot take next?") }),
        ),
        ("noul", json!({ "instruction_applies": noul_question() })),
    ]
    .into_iter()
    .map(|(kind, questions)| Case {
        label: Map::from_iter([("precheck".to_owned(), Value::from(kind))]),
        body: json!({ "model": MODEL, "state": base.clone(), "questions": questions }),
    })
    .collect()
}

/// The 54 main cases: 3 conditions × 3 personalities × 3 situations × with/without instruction.
#[must_use]
pub fn cases() -> Vec<Case> {
    [Condition::JaState, Condition::En, Condition::EnRepeat]
        .into_iter()
        .flat_map(|condition| {
            PERSONALITIES.iter().flat_map(move |(key, personality)| {
                SITUATIONS
                    .iter()
                    .enumerate()
                    .flat_map(move |(situation, text)| {
                        [false, true].into_iter().map(move |with_instruction| Case {
                            label: Map::from_iter([
                                ("condition".to_owned(), Value::from(condition.code())),
                                ("personality".to_owned(), Value::from(*key)),
                                ("situation".to_owned(), Value::from(situation)),
                                ("instruction".to_owned(), Value::from(with_instruction)),
                            ]),
                            body: json!({
                                "model": MODEL,
                                "state": state(
                                    personality, text, with_instruction, condition.japanese_state()
                                ),
                                "questions": questions(),
                            }),
                        })
                    })
            })
        })
        .collect()
}

/// Why an API key from the environment is unusable. The key itself is never included.
#[derive(Debug, PartialEq, Eq)]
pub enum KeyError {
    /// Not set or empty.
    Missing,
    /// Contains characters outside printable ASCII, or `"` / `\` (which JSON would escape,
    /// defeating [`hide_key`]).
    Malformed,
}

/// Trims and checks the key.
///
/// # Errors
/// Returns [`KeyError`] without echoing the key.
pub fn validate_key(raw: Option<&str>) -> Result<String, KeyError> {
    let key = raw
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .ok_or(KeyError::Missing)?;
    if key
        .bytes()
        .all(|b| b.is_ascii_graphic() && b != b'"' && b != b'\\')
    {
        Ok(key.to_owned())
    } else {
        Err(KeyError::Malformed)
    }
}

/// Replaces the key in a text with `<KEY>`, because output is pasted into this public repository.
#[must_use]
pub fn hide_key(text: &str, key: &str) -> String {
    if key.is_empty() {
        text.to_owned()
    } else {
        text.replace(key, "<KEY>")
    }
}

/// What came back for one request.
#[derive(Debug)]
pub enum Reply {
    /// No HTTP response (connection error, timeout before headers).
    Failed(String),
    /// An HTTP response; the body text, or why it could not be read.
    Http {
        /// HTTP status.
        status: u16,
        /// Body text, or the error from reading it.
        body: Result<String, String>,
    },
}

/// Renders one output line. The key is hidden in the final text, after JSON serialisation and
/// after truncation, so no path (JSON body, escaped text, truncated text, error) can leak it.
#[must_use]
pub fn render_line(
    label: &Map<String, Value>,
    reply: &Reply,
    latency_ms: u64,
    key: &str,
) -> String {
    let mut line = label.clone();
    line.insert("latency_ms".to_owned(), Value::from(latency_ms));
    match reply {
        Reply::Failed(e) => {
            line.insert("error".to_owned(), Value::from(hide_key(e, key)));
        }
        Reply::Http { status, body } => {
            line.insert("status".to_owned(), Value::from(*status));
            match body {
                Ok(text) => {
                    let value = serde_json::from_str::<Value>(text).unwrap_or_else(|_| {
                        let hidden = hide_key(text, key);
                        Value::from(hidden.chars().take(MAX_TEXT).collect::<String>())
                    });
                    line.insert("body".to_owned(), value);
                }
                Err(e) => {
                    line.insert("body_error".to_owned(), Value::from(hide_key(e, key)));
                }
            }
        }
    }
    hide_key(&Value::Object(line).to_string(), key)
}

/// HTTP statuses after which the run stops, because every later call would fail the same way.
#[must_use]
pub fn is_fatal_status(status: u16) -> bool {
    matches!(status, 401 | 403)
}

/// Whether a pre-check reply shows the request shape works: status 200 and a JSON body with
/// `answers.<question>`.
#[must_use]
pub fn precheck_ok(reply: &Reply, question: &str) -> bool {
    match reply {
        Reply::Http {
            status: 200,
            body: Ok(text),
        } => serde_json::from_str::<Value>(text)
            .ok()
            .and_then(|v| v.get("answers").and_then(|a| a.get(question)).cloned())
            .is_some(),
        Reply::Http { .. } | Reply::Failed(_) => false,
    }
}

/// Where to send requests: [`API_URL`], or `E05_API_URL` for testing this tool against a local
/// server. The override must be `http://` with host `127.0.0.1`, `localhost` or `[::1]` and no
/// user info, so that a typo cannot send the key elsewhere. The flag is `true` for an override.
///
/// # Errors
/// Returns the rejected value.
pub fn api_url(override_url: Option<&str>) -> Result<(String, bool), String> {
    let Some(raw) = override_url.map(str::trim).filter(|u| !u.is_empty()) else {
        return Ok((API_URL.to_owned(), false));
    };
    let uri: Uri = raw.parse().map_err(|_| raw.to_owned())?;
    let authority = uri
        .authority()
        .map(ureq::http::uri::Authority::as_str)
        .unwrap_or_default();
    let loopback = matches!(uri.host(), Some("127.0.0.1" | "localhost" | "[::1]"));
    if uri.scheme_str() == Some("http") && loopback && !authority.contains('@') {
        Ok((raw.to_owned(), true))
    } else {
        Err(raw.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    const KEY: &str = "SECRETKEY1234567890";

    fn label() -> Map<String, Value> {
        Map::from_iter([("condition".to_owned(), Value::from("en"))])
    }

    #[test]
    fn there_are_54_distinct_main_cases() {
        let all = cases();

        let keys: std::collections::BTreeSet<String> = all
            .iter()
            .map(|c| Value::Object(c.label.clone()).to_string())
            .collect();
        assert_eq!(all.len(), 54);
        assert_eq!(keys.len(), 54);
    }

    #[test]
    fn ja_state_changes_only_the_state() {
        let all = cases();
        let ja = all
            .iter()
            .find(|c| c.label["condition"] == "ja-state")
            .unwrap();
        let en = all.iter().find(|c| c.label["condition"] == "en").unwrap();

        assert_eq!(ja.body["questions"], en.body["questions"]);
        assert_ne!(ja.body["state"], en.body["state"]);
        assert!(
            ja.body["state"]["personality"]
                .as_str()
                .unwrap()
                .contains("臆病")
        );
    }

    #[test]
    fn repeat_condition_sends_the_same_bodies_as_en() {
        let all = cases();
        let bodies = |code: &str| -> Vec<Value> {
            all.iter()
                .filter(|c| c.label["condition"] == code)
                .map(|c| c.body.clone())
                .collect()
        };

        assert_eq!(bodies("en"), bodies("en-repeat"));
    }

    #[test]
    fn instruction_is_null_when_absent() {
        let case = cases()
            .into_iter()
            .find(|c| c.label["instruction"] == false)
            .unwrap();

        assert_eq!(case.body["state"]["instruction"], Value::Null);
    }

    #[test]
    fn main_request_has_model_and_three_questions() {
        let body = &cases()[0].body;

        assert_eq!(body["model"], MODEL);
        assert_eq!(body["questions"]["instruction_applies"]["type"], "noul");
        assert_eq!(body["questions"]["next_action"]["type"], "choice");
        assert_eq!(
            body["questions"]["personality_action"]["criteria"]["sit"],
            "Sit down"
        );
    }

    #[test]
    fn prechecks_ask_one_question_each() {
        let pre = prechecks();

        assert_eq!(pre.len(), 2);
        assert!(
            pre.iter()
                .all(|c| c.body["questions"].as_object().unwrap().len() == 1)
        );
    }

    #[test]
    fn key_is_trimmed_and_validated() {
        assert_eq!(validate_key(Some("  abc-123 \n")), Ok("abc-123".to_owned()));
        assert_eq!(validate_key(None), Err(KeyError::Missing));
        assert_eq!(validate_key(Some("   ")), Err(KeyError::Missing));
        assert_eq!(validate_key(Some("ab c")), Err(KeyError::Malformed));
        assert_eq!(validate_key(Some("abç")), Err(KeyError::Malformed));
        assert_eq!(validate_key(Some("ab\"c")), Err(KeyError::Malformed));
        assert_eq!(validate_key(Some("ab\\c")), Err(KeyError::Malformed));
    }

    #[test]
    fn key_in_json_body_is_hidden() {
        let reply = Reply::Http {
            status: 401,
            body: Ok(format!("{{\"error\":\"bad key {KEY}\"}}")),
        };

        let line = render_line(&label(), &reply, 5, KEY);

        assert!(!line.contains(KEY), "{line}");
        assert!(line.contains("<KEY>"));
    }

    #[test]
    fn unicode_escaped_key_in_json_body_is_hidden() {
        let escaped = KEY.chars().fold(String::new(), |mut acc, c| {
            let _ = write!(acc, "\\u{:04x}", u32::from(c));
            acc
        });
        let reply = Reply::Http {
            status: 401,
            body: Ok(format!("{{\"error\":\"{escaped}\"}}")),
        };

        let line = render_line(&label(), &reply, 5, KEY);

        assert!(!line.contains(KEY), "{line}");
    }

    #[test]
    fn key_across_the_truncation_boundary_is_hidden() {
        let text = format!("{}{KEY}", "x".repeat(MAX_TEXT - 5));
        let reply = Reply::Http {
            status: 200,
            body: Ok(text),
        };

        let line = render_line(&label(), &reply, 5, KEY);

        assert!(!line.contains(&KEY[..5]), "{line}");
    }

    #[test]
    fn key_in_errors_is_hidden() {
        let failed = Reply::Failed(format!("connect failed Bearer {KEY}"));
        let unreadable = Reply::Http {
            status: 200,
            body: Err(format!("read {KEY}")),
        };

        assert!(!render_line(&label(), &failed, 1, KEY).contains(KEY));
        assert!(!render_line(&label(), &unreadable, 1, KEY).contains(KEY));
    }

    #[test]
    fn unreadable_body_is_distinguished_from_empty_body() {
        let reply = Reply::Http {
            status: 200,
            body: Err("too large".to_owned()),
        };

        let line = render_line(&label(), &reply, 1, KEY);

        assert!(line.contains("body_error"));
        assert!(!line.contains("\"body\""));
    }

    #[test]
    fn precheck_needs_status_200_and_the_answer() {
        let ok = Reply::Http {
            status: 200,
            body: Ok("{\"answers\":{\"q\":{}}}".to_owned()),
        };
        let missing = Reply::Http {
            status: 200,
            body: Ok("{\"answers\":{}}".to_owned()),
        };
        let bad = Reply::Http {
            status: 400,
            body: Ok("{\"answers\":{\"q\":{}}}".to_owned()),
        };

        assert!(precheck_ok(&ok, "q"));
        assert!(!precheck_ok(&missing, "q"));
        assert!(!precheck_ok(&bad, "q"));
        assert!(!precheck_ok(&Reply::Failed("x".to_owned()), "q"));
    }

    #[test]
    fn default_url_is_the_api() {
        assert_eq!(api_url(None), Ok((API_URL.to_owned(), false)));
    }

    #[test]
    fn loopback_overrides_are_accepted() {
        for url in [
            "http://127.0.0.1:8080/x",
            "http://localhost:9/",
            "http://[::1]:9/",
        ] {
            assert!(api_url(Some(url)).is_ok(), "{url}");
        }
    }

    #[test]
    fn non_loopback_overrides_are_rejected() {
        for url in [
            "https://127.0.0.1:8080/",
            "http://example.com/",
            "http://127.0.0.1.evil.test:80/",
            "http://localhost:9@api.example.com/", // public-ok: userinfo attack test, not an address
            "http://127.0.0.1:1@api.example.com/", // public-ok: userinfo attack test, not an address
        ] {
            assert!(api_url(Some(url)).is_err(), "{url}");
        }
    }

    #[test]
    fn auth_failures_are_fatal() {
        assert!(is_fatal_status(401));
        assert!(is_fatal_status(403));
        assert!(!is_fatal_status(200));
        assert!(!is_fatal_status(500));
    }
}
