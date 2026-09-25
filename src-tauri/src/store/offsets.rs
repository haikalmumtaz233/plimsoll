use std::path::PathBuf;

use rusqlite::params;

use super::{Database, DatabaseError, from_sql_count, to_sql_count};

impl Database {
    pub fn load_offsets(&self) -> Result<Vec<(PathBuf, u64)>, DatabaseError> {
        let mut statement = self
            .connection
            .prepare("SELECT path, byte_offset FROM file_offsets ORDER BY path")?;
        let rows = statement.query_map([], |row| {
            let path: String = row.get(0)?;
            let offset: i64 = row.get(1)?;
            Ok((PathBuf::from(path), from_sql_count(offset)))
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(DatabaseError::from)
    }

    pub fn save_offsets(&mut self, offsets: &[(PathBuf, u64)]) -> Result<(), DatabaseError> {
        let transaction = self.connection.transaction()?;
        transaction.execute("DELETE FROM file_offsets", [])?;
        {
            let mut statement = transaction
                .prepare("INSERT INTO file_offsets (path, byte_offset) VALUES (?1, ?2)")?;
            for (path, offset) in offsets {
                statement.execute(params![
                    path.to_string_lossy().into_owned(),
                    to_sql_count(*offset)
                ])?;
            }
        }
        transaction.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::store::Database;
    use std::path::PathBuf;

    #[test]
    fn offsets_round_trip() {
        let mut database = Database::open_in_memory().expect("open");
        let offsets = vec![
            (PathBuf::from("C:\\p\\a.jsonl"), 10),
            (PathBuf::from("C:\\p\\b.jsonl"), 20),
        ];
        database.save_offsets(&offsets).expect("save");
        assert_eq!(database.load_offsets().expect("load"), offsets);
    }

    #[test]
    fn saving_replaces_previous_offsets() {
        let mut database = Database::open_in_memory().expect("open");
        database
            .save_offsets(&[(PathBuf::from("C:\\p\\gone.jsonl"), 5)])
            .expect("first save");
        let current = vec![(PathBuf::from("C:\\p\\kept.jsonl"), 7)];
        database.save_offsets(&current).expect("second save");
        assert_eq!(database.load_offsets().expect("load"), current);
    }

    #[test]
    fn empty_database_has_no_offsets() {
        let database = Database::open_in_memory().expect("open");
        assert!(database.load_offsets().expect("load").is_empty());
    }
}
