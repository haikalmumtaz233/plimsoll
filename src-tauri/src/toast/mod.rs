use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::domain::alerts::Alert;
use crate::domain::clock::Timestamp;
use crate::error::AppError;
use crate::i18n::Text;
use crate::tray::reading::whole_percent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub title: String,
    pub body: String,
}

#[must_use]
pub fn message(alert: &Alert, text: Text, now: Timestamp) -> Message {
    let title = text.limit_title(alert.kind, &whole_percent(alert.utilization.percent()));
    let level = text.past_level(alert.severity);
    let body = match alert.resets_at {
        Some(resets_at) if resets_at > now => {
            format!("{level} {}", text.resets_sentence(resets_at - now))
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

#[cfg(test)]
mod tests {
    use super::{Message, message};
    use crate::domain::alerts::Alert;
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::limit::{LimitKind, Utilization};
    use crate::domain::preferences::Language;
    use crate::domain::severity::Severity;
    use crate::i18n::Text;

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);
    const EN: Text = Text::new(Language::English);

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
                EN,
                NOW
            ),
            Message {
                title: "5-hour limit at 82%".to_owned(),
                body: "Past your high level. Resets in 2h 15m.".to_owned(),
            }
        );
    }

    #[test]
    fn speaks_indonesian_when_asked() {
        let indonesian = message(
            &alert(
                LimitKind::FiveHour,
                Severity::Critical,
                96.0,
                Some(Span::from_millis(45 * 60_000)),
            ),
            Text::new(Language::Indonesian),
            NOW,
        );
        assert_eq!(indonesian.title, "Limit 5 jam di 96%");
        assert_eq!(
            indonesian.body,
            "Melewati level kritis. Reset dalam 45 menit."
        );
    }

    #[test]
    fn leaves_out_unknown_or_past_resets() {
        assert_eq!(
            message(
                &alert(LimitKind::SevenDay, Severity::Elevated, 50.0, None),
                EN,
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
                EN,
                NOW
            )
            .body,
            "Past your critical level."
        );
    }
}
