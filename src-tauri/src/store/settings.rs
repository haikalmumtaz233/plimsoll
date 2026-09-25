use rusqlite::{OptionalExtension, params};

use super::{Database, DatabaseError};

impl Database {
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
