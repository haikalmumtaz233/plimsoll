use rusqlite::{OptionalExtension, params};

use super::{Database, DatabaseError};

const OAUTH_OPT_IN: &str = "oauth.opt_in";
const ENABLED: &str = "true";
const DISABLED: &str = "false";

impl Database {
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

#[cfg(test)]
mod tests {
    use crate::store::Database;

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
