use serde::{Deserialize, Serialize, Serializer};
use tauri::{AppHandle, Runtime};
use thiserror::Error;

use crate::app::runtime;
use crate::app::view::UsageView;
use crate::domain::preferences::{PollInterval, Preferences};
use crate::domain::severity::Thresholds;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("usage data is unavailable")]
    Unavailable,
    #[error("thresholds must rise strictly between 1 and 100 percent")]
    InvalidThresholds,
    #[error("poll interval must be one of the offered choices")]
    InvalidInterval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreferencesInput {
    pub elevated: u8,
    pub high: u8,
    pub critical: u8,
    pub poll_minutes: u8,
}

impl PreferencesInput {
    pub fn validate(self) -> Result<Preferences, CommandError> {
        Ok(Preferences {
            thresholds: Thresholds::new(self.elevated, self.high, self.critical)
                .ok_or(CommandError::InvalidThresholds)?,
            poll_interval: PollInterval::from_minutes(self.poll_minutes)
                .ok_or(CommandError::InvalidInterval)?,
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
    runtime::current_view(&app).ok_or(CommandError::Unavailable)
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

#[cfg(test)]
mod tests {
    use super::{CommandError, PreferencesInput};
    use crate::domain::preferences::PollInterval;
    use crate::domain::severity::Thresholds;
    use serde_json::json;

    fn input(elevated: u8, high: u8, critical: u8, poll_minutes: u8) -> PreferencesInput {
        PreferencesInput {
            elevated,
            high,
            critical,
            poll_minutes,
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
    }

    #[test]
    fn frontend_payloads_are_strict() {
        let valid = json!({ "elevated": 50, "high": 80, "critical": 95, "pollMinutes": 1 });
        assert!(serde_json::from_value::<PreferencesInput>(valid).is_ok());
        for invalid in [
            json!({ "elevated": 50, "high": 80, "critical": 95 }),
            json!({ "elevated": -1, "high": 80, "critical": 95, "pollMinutes": 1 }),
            json!({ "elevated": 50.5, "high": 80, "critical": 95, "pollMinutes": 1 }),
            json!({ "elevated": 50, "high": 80, "critical": 300, "pollMinutes": 1 }),
            json!({ "elevated": 50, "high": 80, "critical": 95, "pollMinutes": 1, "extra": true }),
        ] {
            assert!(
                serde_json::from_value::<PreferencesInput>(invalid.clone()).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn errors_reach_the_frontend_as_plain_messages() {
        assert_eq!(
            serde_json::to_string(&CommandError::Unavailable).ok(),
            Some("\"usage data is unavailable\"".to_owned())
        );
    }
}
