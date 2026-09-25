use serde::Deserialize;
use serde_json::Value;

use crate::domain::clock::Timestamp;
use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};
use crate::sources::rfc3339;

#[derive(Debug, Deserialize)]
struct Body {
    #[serde(default)]
    limits: Option<Value>,
    #[serde(default)]
    five_hour: Option<Value>,
    #[serde(default)]
    seven_day: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct LimitEntry {
    kind: String,
    percent: f64,
    #[serde(default)]
    resets_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyWindow {
    utilization: f64,
    #[serde(default)]
    resets_at: Option<String>,
}

#[must_use]
pub fn parse(body: &[u8]) -> Option<Vec<LimitSnapshot>> {
    let body: Body = serde_json::from_slice(body).ok()?;
    let listed = body
        .limits
        .as_ref()
        .map(listed_snapshots)
        .unwrap_or_default();
    let snapshots: Vec<LimitSnapshot> = LimitKind::ALL
        .into_iter()
        .filter_map(|kind| {
            listed
                .iter()
                .find(|snapshot| snapshot.kind == kind)
                .copied()
                .or_else(|| legacy_snapshot(kind, legacy_window(kind, &body)?))
        })
        .collect();
    (!snapshots.is_empty()).then_some(snapshots)
}

fn listed_snapshots(value: &Value) -> Vec<LimitSnapshot> {
    let Value::Array(items) = value else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| LimitEntry::deserialize(item).ok())
        .filter_map(|entry| {
            Some(LimitSnapshot {
                kind: listed_kind(&entry.kind)?,
                utilization: Utilization::from_percent(entry.percent)?,
                resets_at: reset_time(entry.resets_at.as_deref()),
            })
        })
        .collect()
}

fn listed_kind(name: &str) -> Option<LimitKind> {
    match name {
        "session" => Some(LimitKind::FiveHour),
        "weekly_all" => Some(LimitKind::SevenDay),
        _ => None,
    }
}

fn legacy_window(kind: LimitKind, body: &Body) -> Option<&Value> {
    match kind {
        LimitKind::FiveHour => body.five_hour.as_ref(),
        LimitKind::SevenDay => body.seven_day.as_ref(),
    }
}

fn legacy_snapshot(kind: LimitKind, value: &Value) -> Option<LimitSnapshot> {
    let window = LegacyWindow::deserialize(value).ok()?;
    Some(LimitSnapshot {
        kind,
        utilization: Utilization::from_percent(window.utilization)?,
        resets_at: reset_time(window.resets_at.as_deref()),
    })
}

fn reset_time(text: Option<&str>) -> Option<Timestamp> {
    text.and_then(rfc3339::parse)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::domain::clock::Timestamp;
    use crate::domain::limit::{LimitKind, LimitSnapshot};
    use serde_json::{Value, json};

    const FIVE_HOUR_RESET: i64 = 1_790_317_800_000;
    const WEEKLY_RESET: i64 = 1_790_553_600_000;

    fn parsed(body: &Value) -> Option<Vec<(LimitKind, f64, Option<i64>)>> {
        parse(body.to_string().as_bytes()).map(|snapshots| {
            snapshots
                .iter()
                .map(
                    |LimitSnapshot {
                         kind,
                         utilization,
                         resets_at,
                     }| {
                        (
                            *kind,
                            utilization.percent(),
                            resets_at.map(Timestamp::unix_millis),
                        )
                    },
                )
                .collect()
        })
    }

    fn full_response() -> Value {
        json!({
            "five_hour": { "utilization": 12.6, "resets_at": "2026-09-25T06:30:00.000000+00:00" },
            "seven_day": { "utilization": 14.2, "resets_at": "2026-09-28T00:00:00+00:00" },
            "seven_day_opus": null,
            "limits": [
                { "kind": "session", "group": "all", "percent": 13, "severity": "normal",
                  "resets_at": "2026-09-25T06:30:00Z", "scope": "user", "is_active": true },
                { "kind": "weekly_all", "group": "all", "percent": 14, "severity": "normal",
                  "resets_at": "2026-09-28T00:00:00Z", "scope": "user", "is_active": true }
            ],
            "seven_day_breakdown": { "rows": [{ "key": "claude_code", "percent": 75 }] },
            "tangelo": { "anything": [1, 2, 3] }
        })
    }

    #[test]
    fn prefers_the_generic_limits_list() {
        assert_eq!(
            parsed(&full_response()),
            Some(vec![
                (LimitKind::FiveHour, 13.0, Some(FIVE_HOUR_RESET)),
                (LimitKind::SevenDay, 14.0, Some(WEEKLY_RESET)),
            ])
        );
    }

    #[test]
    fn falls_back_to_legacy_windows_without_limits() {
        let mut body = full_response();
        body["limits"] = Value::Null;
        assert_eq!(
            parsed(&body),
            Some(vec![
                (LimitKind::FiveHour, 12.6, Some(FIVE_HOUR_RESET)),
                (LimitKind::SevenDay, 14.2, Some(WEEKLY_RESET)),
            ])
        );
    }

    #[test]
    fn fills_kinds_missing_from_limits_with_legacy_windows() {
        let body = json!({
            "seven_day": { "utilization": 40.0, "resets_at": null },
            "limits": [{ "kind": "session", "percent": 90 }]
        });
        assert_eq!(
            parsed(&body),
            Some(vec![
                (LimitKind::FiveHour, 90.0, None),
                (LimitKind::SevenDay, 40.0, None),
            ])
        );
    }

    #[test]
    fn skips_malformed_or_unknown_limit_entries() {
        let body = json!({
            "limits": [
                "not an object",
                { "kind": "weekly_opus", "percent": 5 },
                { "kind": "session", "percent": "high" },
                { "kind": "session", "percent": -3 },
                { "kind": "weekly_all", "percent": 22, "resets_at": "next monday" }
            ]
        });
        assert_eq!(parsed(&body), Some(vec![(LimitKind::SevenDay, 22.0, None)]));
    }

    #[test]
    fn tolerates_unexpected_shapes_for_known_keys() {
        let body = json!({
            "limits": { "session": 10 },
            "five_hour": "busy",
            "seven_day": { "utilization": 3 }
        });
        assert_eq!(parsed(&body), Some(vec![(LimitKind::SevenDay, 3.0, None)]));
    }

    #[test]
    fn unrecognized_bodies_yield_nothing() {
        for body in [
            json!({}),
            json!({ "limits": [] }),
            json!({ "five_hour": null, "seven_day": null }),
            json!([1, 2, 3]),
            json!("usage"),
        ] {
            assert_eq!(parsed(&body), None, "{body}");
        }
        assert_eq!(parse(b"not json"), None);
        assert_eq!(parse(b""), None);
    }
}
