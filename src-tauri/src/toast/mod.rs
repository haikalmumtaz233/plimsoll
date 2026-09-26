use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::domain::alerts::Alert;
use crate::domain::clock::Timestamp;
use crate::domain::severity::Severity;
use crate::error::AppError;
use crate::tray::reading::{countdown, limit_name, whole_percent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub title: String,
    pub body: String,
}

#[must_use]
pub fn message(alert: &Alert, now: Timestamp) -> Message {
    let title = format!(
        "{} limit at {}%",
        limit_name(alert.kind),
        whole_percent(alert.utilization.percent())
    );
    let level = format!("Past your {} level.", level_name(alert.severity));
    let body = match alert.resets_at {
        Some(resets_at) if resets_at > now => {
            format!("{level} Resets in {}.", countdown(resets_at - now))
        }
        _ => level,
    };
    Message { title, body }
}

pub fn show<R: Runtime>(app: &AppHandle<R>, message: &Message) -> Result<(), AppError> {
    app.notification()
        .builder()
        .title(&message.title)
        .body(&message.body)
        .show()?;
    Ok(())
}

const fn level_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Normal => "normal",
        Severity::Elevated => "warning",
        Severity::High => "high",
        Severity::Critical => "critical",
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, message};
    use crate::domain::alerts::Alert;
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::limit::{LimitKind, Utilization};
    use crate::domain::severity::Severity;

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);

    fn alert(kind: LimitKind, severity: Severity, percent: f64, resets_in: Option<Span>) -> Alert {
        Alert {
            kind,
            severity,
            utilization: Utilization::from_percent(percent).expect("valid percent"),
            resets_at: resets_in.map(|span| NOW + span),
        }
    }

    #[test]
    fn names_the_limit_level_and_reset() {
        assert_eq!(
            message(
                &alert(
                    LimitKind::FiveHour,
                    Severity::High,
                    82.6,
                    Some(Span::from_millis(2 * 3_600_000 + 15 * 60_000))
                ),
                NOW
            ),
            Message {
                title: "5-hour limit at 82%".to_owned(),
                body: "Past your high level. Resets in 2h 15m.".to_owned(),
            }
        );
    }

    #[test]
    fn leaves_out_unknown_or_past_resets() {
        assert_eq!(
            message(
                &alert(LimitKind::SevenDay, Severity::Elevated, 50.0, None),
                NOW
            )
            .body,
            "Past your warning level."
        );
        assert_eq!(
            message(
                &alert(
                    LimitKind::SevenDay,
                    Severity::Critical,
                    99.0,
                    Some(Span::from_millis(-1))
                ),
                NOW
            )
            .body,
            "Past your critical level."
        );
    }
}
