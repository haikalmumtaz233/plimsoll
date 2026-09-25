mod events;
mod migrate;
mod offsets;
mod settings;
mod snapshots;

use std::path::Path;

use rusqlite::Connection;
use thiserror::Error;

use crate::domain::clock::{Span, Timestamp};

pub use snapshots::StoredSnapshot;

pub const RETENTION: Span = Span::days(90);

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

    pub fn prune_expired(&self, now: Timestamp) -> Result<usize, DatabaseError> {
        let cutoff = now - RETENTION;
        Ok(self.prune_events_before(cutoff)? + self.prune_snapshots_before(cutoff)?)
    }

    fn initialize(mut connection: Connection) -> Result<Self, DatabaseError> {
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;
        migrate::apply(&mut connection)?;
        Ok(Self { connection })
    }
}

fn to_sql_count(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn from_sql_count(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{Database, from_sql_count, to_sql_count};
    use std::fs;

    #[test]
    fn counts_round_trip_and_clamp() {
        assert_eq!(from_sql_count(to_sql_count(42)), 42);
        assert_eq!(to_sql_count(u64::MAX), i64::MAX);
        assert_eq!(from_sql_count(-5), 0);
    }

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
