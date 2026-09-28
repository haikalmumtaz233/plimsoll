use serde::{Deserialize, Serialize, Serializer};
use tauri::{AppHandle, Runtime};
use thiserror::Error;

use crate::app::runtime;
use crate::app::view::UsageView;
use crate::diagnostics;
use crate::domain::limit::{LimitKind, Utilization};
use crate::domain::preferences::{LanguageChoice, PollInterval, Preferences};
use crate::domain::severity::Thresholds;
use crate::tray;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("usage data is unavailable")]
    Unavailable,
    #[error("thresholds must rise strictly between 1 and 100 percent")]
    InvalidThresholds,
    #[error("poll interval must be one of the offered choices")]
    InvalidInterval,
    #[error("language must be system, en or id")]
    InvalidLanguage,
    #[error("manual readings need a known limit and a percent from 0 to 100")]
    InvalidManual,
    #[error("the popup could not be hidden")]
    Popup,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManualInput {
    pub kind: String,
    pub percent: Option<f64>,
}

impl ManualInput {
    pub fn validate(&self) -> Result<(LimitKind, Option<Utilization>), CommandError> {
        let kind = LimitKind::from_name(&self.kind).ok_or(CommandError::InvalidManual)?;
        let utilization = match self.percent {
            None => None,
            Some(percent) if percent <= 100.0 => {
                Some(Utilization::from_percent(percent).ok_or(CommandError::InvalidManual)?)
            }
            Some(_) => return Err(CommandError::InvalidManual),
        };
        Ok((kind, utilization))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreferencesInput {
    pub elevated: u8,
    pub high: u8,
    pub critical: u8,
    pub poll_minutes: u8,
    pub language: String,
}

impl PreferencesInput {
    pub fn validate(self) -> Result<Preferences, CommandError> {
        Ok(Preferences {
            thresholds: Thresholds::new(self.elevated, self.high, self.critical)
                .ok_or(CommandError::InvalidThresholds)?,
            poll_interval: PollInterval::from_minutes(self.poll_minutes)
                .ok_or(CommandError::InvalidInterval)?,
            language: LanguageChoice::from_code(&self.language)
                .ok_or(CommandError::InvalidLanguage)?,
        })
    }
}

impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)]
pub fn usage_summary<R: Runtime>(app: AppHandle<R>) -> Result<UsageView, CommandError> {
    runtime::note_popup_opened(&app);
    runtime::current_view(&app).ok_or(CommandError::Unavailable)
}

#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)]
pub fn refresh_now<R: Runtime>(app: AppHandle<R>) -> Result<UsageView, CommandError> {
    runtime::refresh_now(&app).ok_or(CommandError::Unavailable)
}

#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)]
pub fn set_accurate_mode<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
) -> Result<UsageView, CommandError> {
    runtime::set_accurate_mode(&app, enabled).ok_or(CommandError::Unavailable)
}

#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)]
pub fn set_preferences<R: Runtime>(
    app: AppHandle<R>,
    preferences: PreferencesInput,
) -> Result<UsageView, CommandError> {
    let preferences = preferences.validate()?;
    runtime::set_preferences(&app, preferences).ok_or(CommandError::Unavailable)
}

#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)]
pub fn set_manual_percent<R: Runtime>(
    app: AppHandle<R>,
    reading: ManualInput,
) -> Result<UsageView, CommandError> {
    let (kind, utilization) = reading.validate()?;
    runtime::set_manual_reading(&app, kind, utilization).ok_or(CommandError::Unavailable)
}

#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)]
pub fn set_autostart<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
) -> Result<UsageView, CommandError> {
    runtime::set_autostart(&app, enabled).ok_or(CommandError::Unavailable)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn hide_popup<R: Runtime>(app: AppHandle<R>) -> Result<(), CommandError> {
    tray::hide_popup(&app).map_err(|error| {
        diagnostics::error("popup", &format!("failed to hide the popup: {error}"));
        CommandError::Popup
    })
}

#[cfg(test)]
mod tests {
    use super::{CommandError, ManualInput, PreferencesInput};
    use crate::domain::preferences::PollInterval;
    use crate::domain::severity::Thresholds;
    use serde_json::json;

    fn input(elevated: u8, high: u8, critical: u8, poll_minutes: u8) -> PreferencesInput {
        PreferencesInput {
            elevated,
            high,
            critical,
            poll_minutes,
            language: "id".to_owned(),
        }
    }

    #[test]
    fn valid_input_becomes_preferences() {
        let preferences = input(40, 70, 90, 5).validate().expect("valid");
        assert_eq!(
            preferences.thresholds,
            Thresholds::new(40, 70, 90).expect("valid")
        );
        assert_eq!(
            preferences.poll_interval,
            PollInterval::from_minutes(5).expect("valid")
        );
    }

    #[test]
    fn invalid_input_is_rejected() {
        assert!(matches!(
            input(80, 50, 95, 1).validate(),
            Err(CommandError::InvalidThresholds)
        ));
        assert!(matches!(
            input(50, 80, 95, 3).validate(),
            Err(CommandError::InvalidInterval)
        ));
        let unknown = PreferencesInput {
            language: "fr".to_owned(),
            ..input(50, 80, 95, 1)
        };
        assert!(matches!(
            unknown.validate(),
            Err(CommandError::InvalidLanguage)
        ));
    }

    #[test]
    fn frontend_payloads_are_strict() {
        let valid = json!({ "elevated": 50, "high": 80, "critical": 95, "pollMinutes": 1, "language": "en" });
        assert!(serde_json::from_value::<PreferencesInput>(valid).is_ok());
        for invalid in [
            json!({ "elevated": 50, "high": 80, "critical": 95 }),
            json!({ "elevated": 50, "high": 80, "critical": 95, "pollMinutes": 1 }),
            json!({ "elevated": -1, "high": 80, "critical": 95, "pollMinutes": 1, "language": "en" }),
            json!({ "elevated": 50.5, "high": 80, "critical": 95, "pollMinutes": 1, "language": "en" }),
            json!({ "elevated": 50, "high": 80, "critical": 300, "pollMinutes": 1, "language": "en" }),
            json!({ "elevated": 50, "high": 80, "critical": 95, "pollMinutes": 1, "language": "en", "extra": true }),
        ] {
            assert!(
                serde_json::from_value::<PreferencesInput>(invalid.clone()).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn manual_input_is_validated() {
        let input = |kind: &str, percent: Option<f64>| ManualInput {
            kind: kind.to_owned(),
            percent,
        };
        assert!(input("five_hour", Some(42.5)).validate().is_ok());
        assert!(input("seven_day", Some(0.0)).validate().is_ok());
        assert!(input("seven_day", None).validate().is_ok());
        for invalid in [
            input("five_hour", Some(-1.0)),
            input("five_hour", Some(100.5)),
            input("five_hour", Some(f64::NAN)),
            input("monthly", Some(10.0)),
        ] {
            assert!(matches!(
                invalid.validate(),
                Err(CommandError::InvalidManual)
            ));
        }
        assert!(
            serde_json::from_value::<ManualInput>(
                json!({ "kind": "five_hour", "percent": 1, "extra": 1 })
            )
            .is_err()
        );
    }

    #[test]
    fn errors_reach_the_frontend_as_plain_messages() {
        assert_eq!(
            serde_json::to_string(&CommandError::Unavailable).ok(),
            Some("\"usage data is unavailable\"".to_owned())
        );
    }
}
