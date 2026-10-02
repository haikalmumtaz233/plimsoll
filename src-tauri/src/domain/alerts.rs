use super::clock::{Span, Timestamp};
use super::limit::{LimitKind, LimitSnapshot, Utilization};
use super::severity::{Severity, Thresholds};

pub const SAME_WINDOW_TOLERANCE: Span = Span::from_millis(10 * 60_000);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alert {
    pub kind: LimitKind,
    pub severity: Severity,
    pub utilization: Utilization,
    pub resets_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notified {
    pub kind: LimitKind,
    pub severity: Severity,
    pub resets_at: Option<Timestamp>,
}

impl Alert {
    #[must_use]
    pub const fn notified(&self) -> Notified {
        Notified {
            kind: self.kind,
            severity: self.severity,
            resets_at: self.resets_at,
        }
    }
}

#[must_use]
pub fn crossings(
    limits: &[LimitSnapshot],
    thresholds: Thresholds,
    notified: &[Notified],
) -> Vec<Alert> {
    limits
        .iter()
        .filter_map(|limit| {
            let severity = Severity::of(limit.utilization, thresholds);
            let already = notified
                .iter()
                .find(|entry| entry.kind == limit.kind)
                .filter(|entry| same_window(entry.resets_at, limit.resets_at))
                .map_or(Severity::Normal, |entry| entry.severity);
            (severity > already).then_some(Alert {
                kind: limit.kind,
                severity,
                utilization: limit.utilization,
                resets_at: limit.resets_at,
            })
        })
        .collect()
}

fn same_window(previous: Option<Timestamp>, current: Option<Timestamp>) -> bool {
    match (previous, current) {
        (Some(previous), Some(current)) => {
            let gap = if previous > current {
                previous - current
            } else {
                current - previous
            };
            gap <= SAME_WINDOW_TOLERANCE
        }
        (None, None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{Notified, SAME_WINDOW_TOLERANCE, crossings};
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};
    use crate::domain::severity::{Severity, Thresholds};

    const RESET: Timestamp = Timestamp::from_unix_millis(1_790_317_800_000);

    fn limit(kind: LimitKind, percent: f64, resets_at: Option<Timestamp>) -> LimitSnapshot {
        LimitSnapshot {
            kind,
            utilization: Utilization::from_percent(percent).expect("valid percent"),
            resets_at,
        }
    }

    fn notified(kind: LimitKind, severity: Severity, resets_at: Option<Timestamp>) -> Notified {
        Notified {
            kind,
            severity,
            resets_at,
        }
    }

    fn severities(limits: &[LimitSnapshot], seen: &[Notified]) -> Vec<(LimitKind, Severity)> {
        crossings(limits, Thresholds::DEFAULT, seen)
            .iter()
            .map(|alert| (alert.kind, alert.severity))
            .collect()
    }

    #[test]
    fn normal_usage_never_alerts() {
        let crossed = severities(&[limit(LimitKind::FiveHour, 49.9, Some(RESET))], &[]);
        assert!(crossed.is_empty(), "{crossed:?}");
    }

    #[test]
    fn the_first_crossing_in_a_window_alerts_once_at_the_highest_level() {
        let limits = [limit(LimitKind::FiveHour, 83.0, Some(RESET))];
        assert_eq!(
            severities(&limits, &[]),
            vec![(LimitKind::FiveHour, Severity::High)]
        );
        let seen = [notified(LimitKind::FiveHour, Severity::High, Some(RESET))];
        let crossed = severities(&limits, &seen);
        assert!(crossed.is_empty(), "{crossed:?}");
    }

    #[test]
    fn rising_to_a_higher_level_alerts_again() {
        let seen = [notified(
            LimitKind::FiveHour,
            Severity::Elevated,
            Some(RESET),
        )];
        assert_eq!(
            severities(&[limit(LimitKind::FiveHour, 96.0, Some(RESET))], &seen),
            vec![(LimitKind::FiveHour, Severity::Critical)]
        );
    }

    #[test]
    fn small_shifts_in_the_reset_time_count_as_the_same_window() {
        let seen = [notified(
            LimitKind::FiveHour,
            Severity::High,
            Some(RESET + SAME_WINDOW_TOLERANCE),
        )];
        let crossed = severities(&[limit(LimitKind::FiveHour, 85.0, Some(RESET))], &seen);
        assert!(crossed.is_empty(), "{crossed:?}");
    }

    #[test]
    fn a_new_window_starts_alerting_from_scratch() {
        let seen = [notified(
            LimitKind::FiveHour,
            Severity::Critical,
            Some(RESET - Span::hours(5)),
        )];
        assert_eq!(
            severities(&[limit(LimitKind::FiveHour, 55.0, Some(RESET))], &seen),
            vec![(LimitKind::FiveHour, Severity::Elevated)]
        );
    }

    #[test]
    fn each_limit_is_tracked_separately() {
        let seen = [notified(
            LimitKind::FiveHour,
            Severity::Critical,
            Some(RESET),
        )];
        let limits = [
            limit(LimitKind::FiveHour, 97.0, Some(RESET)),
            limit(LimitKind::SevenDay, 81.0, None),
        ];
        assert_eq!(
            severities(&limits, &seen),
            vec![(LimitKind::SevenDay, Severity::High)]
        );
    }

    #[test]
    fn alerts_remember_what_was_notified() {
        let alert = crossings(
            &[limit(LimitKind::SevenDay, 60.0, None)],
            Thresholds::DEFAULT,
            &[],
        )[0];
        assert_eq!(
            alert.notified(),
            notified(LimitKind::SevenDay, Severity::Elevated, None)
        );
    }
}
