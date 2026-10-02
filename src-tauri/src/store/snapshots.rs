use rusqlite::{OptionalExtension, params};

use super::{Database, DatabaseError};
use crate::domain::clock::Timestamp;
use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};

const INSERT_SNAPSHOT: &str = "INSERT INTO limit_snapshots (ts, window_kind, utilization, resets_at) \
    VALUES (?1, ?2, ?3, ?4)";

const SELECT_WINDOW_PEAKS: &str = "SELECT MAX(ts), MAX(utilization), resets_at FROM limit_snapshots \n    WHERE window_kind = ?1 AND ts >= ?2 AND resets_at IS NOT NULL GROUP BY resets_at ORDER BY resets_at";

const SELECT_LATEST: &str = "SELECT ts, utilization, resets_at FROM limit_snapshots \
    WHERE window_kind = ?1 ORDER BY ts DESC, id DESC LIMIT 1";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoredSnapshot {
    pub at: Timestamp,
    pub snapshot: LimitSnapshot,
}

impl Database {
    pub fn insert_snapshots(
        &mut self,
        at: Timestamp,
        snapshots: &[LimitSnapshot],
    ) -> Result<usize, DatabaseError> {
        let transaction = self.connection.transaction()?;
        let mut inserted = 0;
        {
            let mut statement = transaction.prepare(INSERT_SNAPSHOT)?;
            for snapshot in snapshots {
                inserted += statement.execute(params![
                    at.unix_millis(),
                    snapshot.kind.name(),
                    snapshot.utilization.percent(),
                    snapshot.resets_at.map(Timestamp::unix_millis),
                ])?;
            }
        }
        transaction.commit()?;
        Ok(inserted)
    }

    pub fn latest_snapshots(&self) -> Result<Vec<StoredSnapshot>, DatabaseError> {
        let mut statement = self.connection.prepare(SELECT_LATEST)?;
        let mut latest = Vec::new();
        for kind in LimitKind::ALL {
            let row = statement
                .query_row(params![kind.name()], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                    ))
                })
                .optional()?;
            if let Some((at, percent, resets_at)) = row
                && let Some(utilization) = Utilization::from_percent(percent)
            {
                latest.push(StoredSnapshot {
                    at: Timestamp::from_unix_millis(at),
                    snapshot: LimitSnapshot {
                        kind,
                        utilization,
                        resets_at: resets_at.map(Timestamp::from_unix_millis),
                    },
                });
            }
        }
        Ok(latest)
    }

    pub fn window_peaks(
        &self,
        kind: LimitKind,
        since: Timestamp,
    ) -> Result<Vec<(Timestamp, LimitSnapshot)>, DatabaseError> {
        let mut statement = self.connection.prepare(SELECT_WINDOW_PEAKS)?;
        let rows = statement.query_map(params![kind.name(), since.unix_millis()], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;
        let mut peaks = Vec::new();
        for row in rows {
            let (observed_at, percent, resets_at) = row?;
            if let Some(utilization) = Utilization::from_percent(percent) {
                peaks.push((
                    Timestamp::from_unix_millis(observed_at),
                    LimitSnapshot {
                        kind,
                        utilization,
                        resets_at: Some(Timestamp::from_unix_millis(resets_at)),
                    },
                ));
            }
        }
        Ok(peaks)
    }

    pub fn prune_snapshots_before(&self, cutoff: Timestamp) -> Result<usize, DatabaseError> {
        self.connection
            .execute(
                "DELETE FROM limit_snapshots WHERE ts < ?1",
                params![cutoff.unix_millis()],
            )
            .map_err(DatabaseError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::StoredSnapshot;
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};
    use crate::store::{Database, RETENTION};

    fn snapshot(kind: LimitKind, percent: f64, resets_at: Option<i64>) -> LimitSnapshot {
        LimitSnapshot {
            kind,
            utilization: Utilization::from_percent(percent).expect("valid percent"),
            resets_at: resets_at.map(Timestamp::from_unix_millis),
        }
    }

    fn at(millis: i64) -> Timestamp {
        Timestamp::from_unix_millis(millis)
    }

    #[test]
    fn window_peaks_keep_the_highest_reading_per_reset() {
        let mut database = Database::open_in_memory().expect("open");
        let first = Some(9_000);
        let second = Some(20_000);
        database
            .insert_snapshots(at(1_000), &[snapshot(LimitKind::FiveHour, 10.0, first)])
            .expect("insert");
        database
            .insert_snapshots(at(2_000), &[snapshot(LimitKind::FiveHour, 30.0, first)])
            .expect("insert");
        database
            .insert_snapshots(at(12_000), &[snapshot(LimitKind::FiveHour, 5.0, second)])
            .expect("insert");
        database
            .insert_snapshots(at(3_000), &[snapshot(LimitKind::SevenDay, 50.0, first)])
            .expect("insert");
        database
            .insert_snapshots(at(4_000), &[snapshot(LimitKind::FiveHour, 70.0, None)])
            .expect("insert");
        let peaks = database
            .window_peaks(LimitKind::FiveHour, at(0))
            .expect("query");
        assert_eq!(
            peaks,
            vec![
                (at(2_000), snapshot(LimitKind::FiveHour, 30.0, first)),
                (at(12_000), snapshot(LimitKind::FiveHour, 5.0, second)),
            ]
        );
        assert_eq!(
            database
                .window_peaks(LimitKind::FiveHour, at(2_001))
                .expect("query")
                .len(),
            1
        );
    }

    #[test]
    fn empty_store_has_no_latest_snapshots() {
        let database = Database::open_in_memory().expect("open");
        let snapshots = database.latest_snapshots().expect("query");
        assert!(snapshots.is_empty(), "{snapshots:?}");
    }

    #[test]
    fn keeps_the_latest_snapshot_per_kind() {
        let mut database = Database::open_in_memory().expect("open");
        database
            .insert_snapshots(
                at(1_000),
                &[
                    snapshot(LimitKind::FiveHour, 10.0, Some(9_000)),
                    snapshot(LimitKind::SevenDay, 20.0, None),
                ],
            )
            .expect("insert first");
        assert_eq!(
            database
                .insert_snapshots(
                    at(2_000),
                    &[snapshot(LimitKind::FiveHour, 12.5, Some(9_000))]
                )
                .expect("insert second"),
            1
        );
        assert_eq!(
            database.latest_snapshots().expect("query"),
            vec![
                StoredSnapshot {
                    at: at(2_000),
                    snapshot: snapshot(LimitKind::FiveHour, 12.5, Some(9_000)),
                },
                StoredSnapshot {
                    at: at(1_000),
                    snapshot: snapshot(LimitKind::SevenDay, 20.0, None),
                },
            ]
        );
    }

    #[test]
    fn expired_snapshots_are_pruned_with_events() {
        let mut database = Database::open_in_memory().expect("open");
        let now = at(10_000_000_000);
        database
            .insert_snapshots(
                now - RETENTION - Span::from_millis(1),
                &[snapshot(LimitKind::SevenDay, 1.0, None)],
            )
            .expect("insert old");
        database
            .insert_snapshots(now - RETENTION, &[snapshot(LimitKind::FiveHour, 2.0, None)])
            .expect("insert recent");
        assert_eq!(database.prune_expired(now).expect("prune"), 1);
        let latest = database.latest_snapshots().expect("query");
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].snapshot.kind, LimitKind::FiveHour);
    }
}
