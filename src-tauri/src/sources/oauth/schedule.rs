use std::hash::{BuildHasher, RandomState};
use std::time::Duration;

pub const MIN_INTERVAL: Duration = Duration::from_secs(60);
pub const MAX_BACKOFF: Duration = Duration::from_secs(15 * 60);
pub const MAX_RETRY_AFTER: Duration = Duration::from_secs(60 * 60);

const PERMILLE: u16 = 1_000;
const JITTER_DIVISOR: u32 = 10_000;
const MAX_DOUBLINGS: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Jitter(u16);

impl Jitter {
    pub const NONE: Self = Self(0);
    pub const FULL: Self = Self(PERMILLE);

    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(u16::try_from(seed % (u64::from(PERMILLE) + 1)).unwrap_or(PERMILLE))
    }

    #[must_use]
    pub fn random() -> Self {
        Self::from_seed(RandomState::new().hash_one(0_u8))
    }

    fn apply(self, base: Duration) -> Duration {
        base + base * u32::from(self.0) / JITTER_DIVISOR
    }
}

#[derive(Debug, Clone)]
pub struct PollSchedule {
    base: Duration,
    failures: u32,
}

impl Default for PollSchedule {
    fn default() -> Self {
        Self::new(MIN_INTERVAL)
    }
}

impl PollSchedule {
    #[must_use]
    pub fn new(base: Duration) -> Self {
        Self {
            base: base.max(MIN_INTERVAL),
            failures: 0,
        }
    }

    pub fn set_base(&mut self, base: Duration) {
        self.base = base.max(MIN_INTERVAL);
    }

    #[must_use]
    pub const fn failures(&self) -> u32 {
        self.failures
    }

    pub fn after_success(&mut self, jitter: Jitter) -> Duration {
        self.failures = 0;
        jitter.apply(self.base)
    }

    pub fn after_failure(&mut self, retry_after: Option<Duration>, jitter: Jitter) -> Duration {
        self.failures = self.failures.saturating_add(1);
        let doublings = self.failures.min(MAX_DOUBLINGS);
        let backoff = (MIN_INTERVAL * (1 << doublings)).min(MAX_BACKOFF);
        let requested = retry_after.map_or(Duration::ZERO, |wait| wait.min(MAX_RETRY_AFTER));
        jitter.apply(backoff.max(requested))
    }
}

#[cfg(test)]
mod tests {
    use super::{Jitter, MAX_BACKOFF, MAX_RETRY_AFTER, MIN_INTERVAL, PollSchedule};
    use std::time::Duration;

    fn secs(duration: Duration) -> u64 {
        duration.as_secs()
    }

    #[test]
    fn a_longer_base_interval_stretches_success_but_not_the_failure_backoff() {
        let mut schedule = PollSchedule::new(Duration::from_secs(300));
        assert_eq!(secs(schedule.after_success(Jitter::NONE)), 300);
        assert_eq!(secs(schedule.after_failure(None, Jitter::NONE)), 120);
        assert_eq!(secs(schedule.after_failure(None, Jitter::NONE)), 240);
        let mut idle = PollSchedule::new(Duration::from_secs(30 * 60));
        assert_eq!(secs(idle.after_failure(None, Jitter::NONE)), 120);
        let delays: Vec<u64> = (0..5)
            .map(|_| secs(idle.after_failure(None, Jitter::NONE)))
            .collect();
        assert_eq!(delays, vec![240, 480, 900, 900, 900]);
    }

    #[test]
    fn the_base_interval_can_change_between_polls() {
        let mut schedule = PollSchedule::default();
        schedule.set_base(Duration::from_secs(300));
        assert_eq!(secs(schedule.after_success(Jitter::NONE)), 300);
        schedule.set_base(Duration::from_secs(5));
        assert_eq!(schedule.after_success(Jitter::NONE), MIN_INTERVAL);
    }

    #[test]
    fn base_intervals_never_drop_below_the_minimum() {
        let mut schedule = PollSchedule::new(Duration::from_secs(5));
        assert_eq!(schedule.after_success(Jitter::NONE), MIN_INTERVAL);
    }

    #[test]
    fn success_polls_at_the_minimum_interval() {
        let mut schedule = PollSchedule::default();
        assert_eq!(schedule.after_success(Jitter::NONE), MIN_INTERVAL);
        assert_eq!(secs(schedule.after_success(Jitter::FULL)), 66);
    }

    #[test]
    fn failures_back_off_exponentially_up_to_the_cap() {
        let mut schedule = PollSchedule::default();
        let delays: Vec<u64> = (0..6)
            .map(|_| secs(schedule.after_failure(None, Jitter::NONE)))
            .collect();
        assert_eq!(delays, vec![120, 240, 480, 900, 900, 900]);
        assert_eq!(schedule.failures(), 6);
        assert_eq!(schedule.after_failure(None, Jitter::NONE), MAX_BACKOFF);
    }

    #[test]
    fn success_resets_the_backoff() {
        let mut schedule = PollSchedule::default();
        schedule.after_failure(None, Jitter::NONE);
        schedule.after_failure(None, Jitter::NONE);
        assert_eq!(schedule.after_success(Jitter::NONE), MIN_INTERVAL);
        assert_eq!(schedule.failures(), 0);
        assert_eq!(secs(schedule.after_failure(None, Jitter::NONE)), 120);
    }

    #[test]
    fn honours_longer_retry_after_hints_within_a_bound() {
        let mut schedule = PollSchedule::default();
        assert_eq!(
            secs(schedule.after_failure(Some(Duration::from_secs(600)), Jitter::NONE)),
            600
        );
        assert_eq!(
            secs(schedule.after_failure(Some(Duration::from_secs(5)), Jitter::NONE)),
            240
        );
        assert_eq!(
            schedule.after_failure(Some(Duration::from_secs(86_400)), Jitter::NONE),
            MAX_RETRY_AFTER
        );
    }

    #[test]
    fn jitter_only_ever_lengthens_the_delay_by_up_to_ten_percent() {
        for seed in [0, 1, 500, 999, 1_000, 1_001, u64::MAX] {
            let delay = PollSchedule::default().after_success(Jitter::from_seed(seed));
            assert!(delay >= MIN_INTERVAL, "{seed}");
            assert!(delay <= Duration::from_secs(66), "{seed}");
        }
        assert_eq!(Jitter::from_seed(1_000), Jitter::FULL);
        assert_eq!(Jitter::from_seed(1_001), Jitter::NONE);
    }

    #[test]
    fn random_jitter_stays_in_range() {
        for _ in 0..32 {
            let delay = PollSchedule::default().after_success(Jitter::random());
            assert!((MIN_INTERVAL..=Duration::from_secs(66)).contains(&delay));
        }
    }
}
