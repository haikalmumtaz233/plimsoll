use serde::Deserialize;

use crate::domain::record::UsageEvent;
use crate::domain::tokens::TokenCounts;
use crate::sources::rfc3339;

const USAGE_MARKER: &[u8] = b"\"usage\"";
const ASSISTANT_KIND: &str = "assistant";
const UNKNOWN_PROJECT: &str = "unknown";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DedupeKey {
    pub message_id: String,
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageRecord {
    pub key: DedupeKey,
    pub event: UsageEvent,
}

#[derive(Deserialize)]
struct RawLine {
    #[serde(rename = "type")]
    kind: Option<String>,
    timestamp: Option<String>,
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    cwd: Option<String>,
    message: Option<RawMessage>,
}

#[derive(Deserialize)]
struct RawMessage {
    id: Option<String>,
    model: Option<String>,
    usage: Option<RawUsage>,
}

#[derive(Deserialize)]
struct RawUsage {
    #[serde(rename = "input_tokens")]
    input: Option<u64>,
    #[serde(rename = "output_tokens")]
    output: Option<u64>,
    #[serde(rename = "cache_creation_input_tokens")]
    cache_creation: Option<u64>,
    #[serde(rename = "cache_read_input_tokens")]
    cache_read: Option<u64>,
}

impl RawUsage {
    fn into_counts(self) -> TokenCounts {
        TokenCounts {
            input: self.input.unwrap_or_default(),
            output: self.output.unwrap_or_default(),
            cache_creation: self.cache_creation.unwrap_or_default(),
            cache_read: self.cache_read.unwrap_or_default(),
        }
    }
}

#[must_use]
pub fn parse(line: &[u8]) -> Option<UsageRecord> {
    if !contains(line, USAGE_MARKER) {
        return None;
    }
    let raw: RawLine = serde_json::from_slice(line).ok()?;
    if raw.kind.as_deref() != Some(ASSISTANT_KIND) {
        return None;
    }
    let message = raw.message?;
    let model = message.model.filter(|model| !model.starts_with('<'))?;
    let usage = message.usage?;
    let message_id = message.id?;
    let at = rfc3339::parse(raw.timestamp.as_deref()?)?;
    Some(UsageRecord {
        key: DedupeKey {
            message_id,
            request_id: raw.request_id.unwrap_or_default(),
        },
        event: UsageEvent {
            at,
            model,
            project: project_name(raw.cwd.as_deref()),
            tokens: usage.into_counts(),
        },
    })
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn project_name(cwd: Option<&str>) -> String {
    cwd.and_then(|path| path.rsplit(['\\', '/']).find(|part| !part.is_empty()))
        .unwrap_or(UNKNOWN_PROJECT)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{DedupeKey, parse};
    use crate::domain::tokens::TokenCounts;
    use serde_json::{Value, json};

    fn assistant(overrides: &Value) -> Vec<u8> {
        let mut line = json!({
            "type": "assistant",
            "timestamp": "2026-09-25T01:35:37.212Z",
            "requestId": "req_1",
            "cwd": "C:\\Users\\dev\\plimsoll",
            "message": {
                "id": "msg_1",
                "model": "claude-opus-5",
                "role": "assistant",
                "content": [{ "type": "text", "text": "private conversation text" }],
                "usage": {
                    "input_tokens": 3,
                    "output_tokens": 40,
                    "cache_creation_input_tokens": 500,
                    "cache_read_input_tokens": 6000,
                    "service_tier": "standard"
                }
            }
        });
        merge(&mut line, overrides);
        serde_json::to_vec(&line).expect("serialize line")
    }

    fn merge(target: &mut Value, overrides: &Value) {
        if let (Value::Object(target), Value::Object(overrides)) = (target, overrides) {
            for (key, value) in overrides {
                if value.is_object()
                    && let Some(existing) = target.get_mut(key)
                {
                    merge(existing, value);
                } else {
                    target.insert(key.clone(), value.clone());
                }
            }
        }
    }

    #[test]
    fn parses_usage_metadata_only() {
        let record = parse(&assistant(&json!({}))).expect("record");
        assert_eq!(
            record.key,
            DedupeKey {
                message_id: "msg_1".to_owned(),
                request_id: "req_1".to_owned(),
            }
        );
        assert_eq!(record.event.at.unix_millis(), 1_790_300_137_212);
        assert_eq!(record.event.model, "claude-opus-5");
        assert_eq!(record.event.project, "plimsoll");
        assert_eq!(
            record.event.tokens,
            TokenCounts {
                input: 3,
                output: 40,
                cache_creation: 500,
                cache_read: 6_000,
            }
        );
        assert!(!format!("{record:?}").contains("private conversation text"));
    }

    #[test]
    fn tolerates_trailing_newlines() {
        let mut line = assistant(&json!({}));
        line.extend_from_slice(b"\r\n");
        assert!(parse(&line).is_some());
    }

    #[test]
    fn skips_lines_that_are_not_assistant_usage() {
        let user = serde_json::to_vec(&json!({
            "type": "user",
            "timestamp": "2026-09-25T01:35:37.212Z",
            "message": { "role": "user", "content": "hello" }
        }))
        .expect("serialize line");
        assert_eq!(parse(&user), None);
        assert_eq!(parse(&assistant(&json!({ "type": "user" }))), None);
        assert_eq!(
            parse(&assistant(
                &json!({ "message": { "model": "<synthetic>" } })
            )),
            None
        );
        assert_eq!(
            parse(&assistant(&json!({ "message": { "usage": null } }))),
            None
        );
        assert_eq!(
            parse(&assistant(&json!({ "message": { "id": null } }))),
            None
        );
        assert_eq!(
            parse(&assistant(&json!({ "timestamp": "yesterday" }))),
            None
        );
    }

    #[test]
    fn skips_malformed_json() {
        assert_eq!(parse(b"{\"type\":\"assistant\",\"usage\""), None);
        assert_eq!(parse(b""), None);
    }

    #[test]
    fn missing_optional_fields_fall_back_to_defaults() {
        let line = assistant(&json!({
            "requestId": null,
            "cwd": null,
            "message": { "usage": { "cache_read_input_tokens": null } }
        }));
        let record = parse(&line).expect("record");
        assert_eq!(record.key.request_id, "");
        assert_eq!(record.event.project, "unknown");
        assert_eq!(record.event.tokens.cache_read, 0);
    }

    #[test]
    fn project_is_the_last_path_segment() {
        let unix = assistant(&json!({ "cwd": "/home/dev/tracker/" }));
        assert_eq!(parse(&unix).expect("record").event.project, "tracker");
    }
}
