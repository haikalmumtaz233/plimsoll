use rusqlite::Connection;

use super::DatabaseError;

const MIGRATIONS: [&str; 1] = [include_str!("../../migrations/0001_initial.sql")];

pub fn current_version(connection: &Connection) -> Result<i64, DatabaseError> {
    connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(DatabaseError::from)
}

pub fn apply(connection: &mut Connection) -> Result<(), DatabaseError> {
    let supported = i64::try_from(MIGRATIONS.len()).unwrap_or(i64::MAX);
    let current = current_version(connection)?;
    if current > supported {
        return Err(DatabaseError::SchemaTooNew {
            found: current,
            supported,
        });
    }
    let applied = usize::try_from(current).unwrap_or(0);
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(applied) {
        let version = i64::try_from(index + 1).unwrap_or(i64::MAX);
        let transaction = connection.transaction()?;
        transaction.execute_batch(sql)?;
        transaction.pragma_update(None, "user_version", version)?;
        transaction.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MIGRATIONS, apply, current_version};
    use crate::store::DatabaseError;
    use rusqlite::Connection;

    fn table_names(connection: &Connection) -> Vec<String> {
        let mut statement = connection
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .expect("prepare");
        statement
            .query_map([], |row| row.get(0))
            .expect("query")
            .collect::<Result<Vec<String>, _>>()
            .expect("collect")
    }

    #[test]
    fn fresh_database_gets_every_migration() {
        let mut connection = Connection::open_in_memory().expect("open");
        apply(&mut connection).expect("migrate");
        assert_eq!(
            current_version(&connection).expect("version"),
            i64::try_from(MIGRATIONS.len()).expect("fits")
        );
        assert_eq!(
            table_names(&connection),
            vec![
                "file_offsets",
                "limit_snapshots",
                "settings",
                "usage_events"
            ]
        );
    }

    #[test]
    fn applying_twice_is_a_no_op() {
        let mut connection = Connection::open_in_memory().expect("open");
        apply(&mut connection).expect("first migrate");
        apply(&mut connection).expect("second migrate");
        assert_eq!(current_version(&connection).expect("version"), 1);
    }

    #[test]
    fn newer_schema_is_rejected() {
        let mut connection = Connection::open_in_memory().expect("open");
        connection
            .pragma_update(None, "user_version", 99)
            .expect("set version");
        assert!(matches!(
            apply(&mut connection),
            Err(DatabaseError::SchemaTooNew {
                found: 99,
                supported: 1
            })
        ));
    }
}
