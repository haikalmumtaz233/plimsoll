use rusqlite::params;

use super::{Database, DatabaseError, RETENTION, from_sql_count, to_sql_count};
use crate::domain::clock::Timestamp;
use crate::domain::period::Window;
use crate::domain::record::{KeyedEvent, UsageEvent};
use crate::domain::tokens::TokenCounts;

const JSONL_SOURCE: &str = "jsonl";

const INSERT_EVENT: &str = "INSERT OR IGNORE INTO usage_events \
    (message_id, request_id, ts, source, model, project, input, output, cache_create, cache_read) \
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";

const SELECT_EVENTS: &str = "SELECT ts, model, project, input, output, cache_create, cache_read \
    FROM usage_events WHERE ts >= ?1 AND ts < ?2 ORDER BY ts, id";

impl Database {
    pub fn insert_events(&mut self, events: &[KeyedEvent]) -> Result<usize, DatabaseError> {
        let transaction = self.connection.transaction()?;
        let mut inserted = 0;
        {
            let mut statement = transaction.prepare(INSERT_EVENT)?;
            for keyed in events {
                let tokens = keyed.event.tokens;
                inserted += statement.execute(params![
                    keyed.key.message_id,
                    keyed.key.request_id,
                    keyed.event.at.unix_millis(),
                    JSONL_SOURCE,
                    keyed.event.model,
                    keyed.event.project,
                    to_sql_count(tokens.input),
                    to_sql_count(tokens.output),
                    to_sql_count(tokens.cache_creation),
                    to_sql_count(tokens.cache_read),
                ])?;
            }
        }
        transaction.commit()?;
        Ok(inserted)
    }

    pub fn events_in(&self, window: Window) -> Result<Vec<UsageEvent>, DatabaseError> {
        let mut statement = self.connection.prepare(SELECT_EVENTS)?;
        let rows = statement.query_map(
            params![window.start().unix_millis(), window.end().unix_millis()],
            |row| {
                Ok(UsageEvent {
                    at: Timestamp::from_unix_millis(row.get(0)?),
                    model: row.get(1)?,
                    project: row.get(2)?,
                    tokens: TokenCounts {
                        input: from_sql_count(row.get(3)?),
                        output: from_sql_count(row.get(4)?),
                        cache_creation: from_sql_count(row.get(5)?),
                        cache_read: from_sql_count(row.get(6)?),
                    },
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(DatabaseError::from)
    }

    pub fn prune_events_before(&self, cutoff: Timestamp) -> Result<usize, DatabaseError> {
        self.connection
            .execute(
                "DELETE FROM usage_events WHERE ts < ?1",
                params![cutoff.unix_millis()],
            )
            .map_err(DatabaseError::from)
    }

    pub fn prune_expired(&self, now: Timestamp) -> Result<usize, DatabaseError> {
        self.prune_events_before(now - RETENTION)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::period::Window;
    use crate::domain::record::{EventKey, KeyedEvent, UsageEvent};
    use crate::domain::tokens::TokenCounts;
    use crate::store::{Database, RETENTION};

    fn keyed(message_id: &str, millis: i64, output: u64) -> KeyedEvent {
        KeyedEvent {
            key: EventKey {
                message_id: message_id.to_owned(),
                request_id: format!("req_{message_id}"),
            },
            event: UsageEvent {
                at: Timestamp::from_unix_millis(millis),
                model: "claude-opus-5".to_owned(),
                project: "plimsoll".to_owned(),
                tokens: TokenCounts {
                    input: 1,
                    output,
                    cache_creation: 10,
                    cache_read: 100,
                },
            },
        }
    }

    fn everything() -> Window {
        Window::starting_at(Timestamp::from_unix_millis(0), Span::days(36_500))
    }

    #[test]
    fn duplicate_keys_are_ignored() {
        let mut database = Database::open_in_memory().expect("open");
        let inserted = database
            .insert_events(&[
                keyed("a", 1_000, 5),
                keyed("a", 1_000, 5),
                keyed("b", 2_000, 7),
            ])
            .expect("insert");
        assert_eq!(inserted, 2);
        assert_eq!(
            database
                .insert_events(&[keyed("b", 2_000, 7)])
                .expect("insert again"),
            0
        );
        assert_eq!(database.events_in(everything()).expect("query").len(), 2);
    }

    #[test]
    fn events_round_trip_within_a_half_open_window() {
        let mut database = Database::open_in_memory().expect("open");
        database
            .insert_events(&[
                keyed("early", 999, 1),
                keyed("start", 1_000, 2),
                keyed("end", 2_000, 3),
            ])
            .expect("insert");
        let window =
            Window::starting_at(Timestamp::from_unix_millis(1_000), Span::from_millis(1_000));
        let events = database.events_in(window).expect("query");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], keyed("start", 1_000, 2).event);
    }

    #[test]
    fn expired_events_are_pruned() {
        let mut database = Database::open_in_memory().expect("open");
        let now = Timestamp::from_unix_millis(10_000_000_000);
        let old = (now - RETENTION - Span::from_millis(1)).unix_millis();
        let recent = (now - RETENTION).unix_millis();
        database
            .insert_events(&[keyed("old", old, 1), keyed("recent", recent, 2)])
            .expect("insert");
        assert_eq!(database.prune_expired(now).expect("prune"), 1);
        let remaining = database.events_in(everything()).expect("query");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].tokens.output, 2);
    }
}
