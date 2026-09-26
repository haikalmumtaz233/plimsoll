use rusqlite::{OptionalExtension, params};

use super::{Database, DatabaseError};
use crate::domain::preferences::{PollInterval, Preferences};
use crate::domain::severity::Thresholds;

const OAUTH_OPT_IN: &str = "oauth.opt_in";
const ALERT_THRESHOLDS: &str = "alerts.thresholds";
const POLL_MINUTES: &str = "oauth.poll_minutes";
const LIST_SEPARATOR: char = ',';
const ENABLED: &str = "true";
const DISABLED: &str = "false";

impl Database {
    pub fn preferences(&self) -> Result<Preferences, DatabaseError> {
        Ok(Preferences {
            thresholds: self
                .setting(ALERT_THRESHOLDS)?
                .as_deref()
                .and_then(parse_thresholds)
                .unwrap_or_default(),
            poll_interval: self
                .setting(POLL_MINUTES)?
                .and_then(|value| value.parse().ok())
                .and_then(PollInterval::from_minutes)
                .unwrap_or_default(),
        })
    }

    pub fn set_preferences(&self, preferences: Preferences) -> Result<(), DatabaseError> {
        let thresholds = preferences.thresholds;
        self.set_setting(
            ALERT_THRESHOLDS,
            &format!(
                "{}{LIST_SEPARATOR}{}{LIST_SEPARATOR}{}",
                thresholds.elevated(),
                thresholds.high(),
                thresholds.critical()
            ),
        )?;
        self.set_setting(
            POLL_MINUTES,
            &preferences.poll_interval.minutes().to_string(),
        )
    }

    pub fn oauth_opted_in(&self) -> Result<bool, DatabaseError> {
        Ok(self.setting(OAUTH_OPT_IN)?.as_deref() == Some(ENABLED))
    }

    pub fn set_oauth_opted_in(&self, opted_in: bool) -> Result<(), DatabaseError> {
        self.set_setting(OAUTH_OPT_IN, if opted_in { ENABLED } else { DISABLED })
    }

    pub fn setting(&self, name: &str) -> Result<Option<String>, DatabaseError> {
        self.connection
            .query_row(
                "SELECT value FROM settings WHERE name = ?1",
                params![name],
                |row| row.get(0),
            )
            .optional()
            .map_err(DatabaseError::from)
    }

    pub fn set_setting(&self, name: &str, value: &str) -> Result<(), DatabaseError> {
        self.connection.execute(
            "INSERT INTO settings (name, value) VALUES (?1, ?2) \
             ON CONFLICT(name) DO UPDATE SET value = excluded.value",
            params![name, value],
        )?;
        Ok(())
    }
}

fn parse_thresholds(value: &str) -> Option<Thresholds> {
    let mut parts = value.split(LIST_SEPARATOR).map(str::parse::<u8>);
    let elevated = parts.next()?.ok()?;
    let high = parts.next()?.ok()?;
    let critical = parts.next()?.ok()?;
    if parts.next().is_some() {
        return None;
    }
    Thresholds::new(elevated, high, critical)
}

#[cfg(test)]
mod tests {
    use crate::domain::preferences::{PollInterval, Preferences};
    use crate::domain::severity::Thresholds;
    use crate::store::Database;

    #[test]
    fn preferences_default_until_saved() {
        let database = Database::open_in_memory().expect("open");
        assert_eq!(
            database.preferences().expect("read"),
            Preferences::default()
        );
    }

    #[test]
    fn preferences_round_trip() {
        let database = Database::open_in_memory().expect("open");
        let preferences = Preferences {
            thresholds: Thresholds::new(40, 70, 90).expect("valid thresholds"),
            poll_interval: PollInterval::from_minutes(5).expect("valid interval"),
        };
        database.set_preferences(preferences).expect("write");
        assert_eq!(database.preferences().expect("read"), preferences);
    }

    #[test]
    fn corrupt_preferences_fall_back_to_defaults() {
        let database = Database::open_in_memory().expect("open");
        for value in ["", "50,80", "50,80,95,99", "90,80,95", "a,b,c", "50,80,300"] {
            database
                .set_setting("alerts.thresholds", value)
                .expect("write");
            assert_eq!(
                database.preferences().expect("read").thresholds,
                Thresholds::DEFAULT,
                "{value}"
            );
        }
        database
            .set_setting("oauth.poll_minutes", "7")
            .expect("write");
        assert_eq!(
            database.preferences().expect("read").poll_interval,
            PollInterval::DEFAULT
        );
    }

    #[test]
    fn missing_setting_is_none() {
        let database = Database::open_in_memory().expect("open");
        assert_eq!(database.setting("theme").expect("read"), None);
    }

    #[test]
    fn oauth_is_opt_in_and_can_be_turned_off_again() {
        let database = Database::open_in_memory().expect("open");
        assert!(!database.oauth_opted_in().expect("default"));
        database.set_oauth_opted_in(true).expect("opt in");
        assert!(database.oauth_opted_in().expect("opted in"));
        database.set_oauth_opted_in(false).expect("opt out");
        assert!(!database.oauth_opted_in().expect("opted out"));
    }

    #[test]
    fn unexpected_opt_in_values_count_as_opted_out() {
        let database = Database::open_in_memory().expect("open");
        database.set_setting("oauth.opt_in", "yes").expect("write");
        assert!(!database.oauth_opted_in().expect("read"));
    }

    #[test]
    fn settings_are_upserted() {
        let database = Database::open_in_memory().expect("open");
        database.set_setting("theme", "light").expect("write");
        database.set_setting("theme", "dark").expect("overwrite");
        assert_eq!(
            database.setting("theme").expect("read").as_deref(),
            Some("dark")
        );
    }
}
