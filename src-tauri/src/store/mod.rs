mod migrate;
mod settings;

use std::path::Path;

use rusqlite::Connection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("database schema version {found} is newer than supported version {supported}")]
    SchemaTooNew { found: i64, supported: i64 },
}

#[derive(Debug)]
pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DatabaseError> {
        Self::initialize(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, DatabaseError> {
        Self::initialize(Connection::open_in_memory()?)
    }

    pub fn schema_version(&self) -> Result<i64, DatabaseError> {
        migrate::current_version(&self.connection)
    }

    fn initialize(mut connection: Connection) -> Result<Self, DatabaseError> {
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;
        migrate::apply(&mut connection)?;
        Ok(Self { connection })
    }
}

#[cfg(test)]
mod tests {
    use super::Database;
    use std::fs;

    #[test]
    fn file_database_keeps_data_between_opens() {
        let path = std::env::temp_dir().join(format!("plimsoll-db-{}.sqlite", std::process::id()));
        if path.exists() {
            fs::remove_file(&path).expect("reset database file");
        }
        {
            let database = Database::open(&path).expect("open database");
            database
                .set_setting("theme", "dark")
                .expect("write setting");
        }
        {
            let database = Database::open(&path).expect("reopen database");
            assert_eq!(database.schema_version().expect("version"), 1);
            assert_eq!(
                database.setting("theme").expect("read setting").as_deref(),
                Some("dark")
            );
        }
        for suffix in ["", "-wal", "-shm"] {
            let file = path.with_file_name(format!(
                "{}{suffix}",
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
            ));
            fs::remove_file(file).ok();
        }
    }
}
