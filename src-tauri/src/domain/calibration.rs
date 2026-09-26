use super::alerts::SAME_WINDOW_TOLERANCE;
use super::clock::Timestamp;
use super::limit::{LimitKind, LimitSnapshot, Utilization};
use super::period::{Window, five_hour_from_reset, weekly_from_reset};

pub const MIN_SAMPLES: usize = 3;
pub const MAX_SAMPLES: usize = 10;
pub const MIN_PERCENT: f64 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Peak {
    pub window: Window,
    pub observed_at: Timestamp,
    pub utilization: Utilization,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub utilization: Utilization,
    pub tokens: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Estimate {
    pub kind: LimitKind,
    pub utilization: Utilization,
    pub samples: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calibration {
    tokens_per_percent: f64,
    samples: usize,
}

impl Calibration {
    #[must_use]
    pub const fn tokens_per_percent(self) -> f64 {
        self.tokens_per_percent
    }

    #[must_use]
    pub const fn samples(self) -> usize {
        self.samples
    }

    #[must_use]
    pub fn estimate(self, tokens: u64) -> Option<Utilization> {
        Utilization::from_percent(as_f64(tokens) / self.tokens_per_percent)
    }
}

#[must_use]
pub fn window_of(kind: LimitKind, resets_at: Timestamp) -> Window {
    match kind {
        LimitKind::FiveHour => five_hour_from_reset(resets_at),
        LimitKind::SevenDay => weekly_from_reset(resets_at),
    }
}

#[must_use]
pub fn peaks(observations: &[(Timestamp, LimitSnapshot)], kind: LimitKind) -> Vec<Peak> {
    let mut candidates: Vec<(Timestamp, Timestamp, Utilization)> = observations
        .iter()
        .filter(|(_, snapshot)| snapshot.kind == kind)
        .filter_map(|(observed_at, snapshot)| {
            snapshot
                .resets_at
                .map(|resets_at| (resets_at, *observed_at, snapshot.utilization))
        })
        .collect();
    candidates.sort_by_key(|(resets_at, observed_at, _)| (*resets_at, *observed_at));

    let mut peaks: Vec<(Timestamp, Peak)> = Vec::new();
    for (resets_at, observed_at, utilization) in candidates {
        let peak = Peak {
            window: window_of(kind, resets_at),
            observed_at,
            utilization,
        };
        match peaks.last_mut() {
            Some((anchor, best)) if resets_at - *anchor <= SAME_WINDOW_TOLERANCE => {
                if utilization.percent() >= best.utilization.percent() {
                    *best = peak;
                }
            }
            _ => peaks.push((resets_at, peak)),
        }
    }
    peaks
        .into_iter()
        .rev()
        .map(|(_, peak)| peak)
        .filter(|peak| peak.utilization.percent() >= MIN_PERCENT)
        .take(MAX_SAMPLES)
        .collect()
}

#[must_use]
pub fn calibrate(samples: &[Sample]) -> Option<Calibration> {
    let mut ratios: Vec<f64> = samples
        .iter()
        .filter(|sample| sample.tokens > 0 && sample.utilization.percent() >= MIN_PERCENT)
        .map(|sample| as_f64(sample.tokens) / sample.utilization.percent())
        .collect();
    if ratios.len() < MIN_SAMPLES {
        return None;
    }
    ratios.sort_by(f64::total_cmp);
    let middle = ratios.len() / 2;
    let median = if ratios.len().is_multiple_of(2) {
        f64::midpoint(ratios[middle - 1], ratios[middle])
    } else {
        ratios[middle]
    };
    Some(Calibration {
        tokens_per_percent: median,
        samples: ratios.len(),
    })
}

#[allow(clippy::cast_precision_loss)]
fn as_f64(tokens: u64) -> f64 {
    tokens as f64
}

#[cfg(test)]
mod tests {
    use super::{MAX_SAMPLES, MIN_SAMPLES, Sample, calibrate, peaks, window_of};
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};

    const HOUR: i64 = 3_600_000;
    const RESET: Timestamp = Timestamp::from_unix_millis(1_790_317_800_000);

    fn utilization(percent: f64) -> Utilization {
        Utilization::from_percent(percent).expect("valid percent")
    }

    fn observation(
        kind: LimitKind,
        percent: f64,
        resets_at: Timestamp,
        minutes_before_reset: i64,
    ) -> (Timestamp, LimitSnapshot) {
        (
            resets_at - Span::from_millis(minutes_before_reset * 60_000),
            LimitSnapshot {
                kind,
                utilization: utilization(percent),
                resets_at: Some(resets_at),
            },
        )
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }

    fn sample(percent: f64, tokens: u64) -> Sample {
        Sample {
            utilization: utilization(percent),
            tokens,
        }
    }

    #[test]
    fn keeps_the_highest_reading_of_each_window_newest_first() {
        let earlier = RESET - Span::hours(5);
        let observations = [
            observation(LimitKind::FiveHour, 10.0, earlier, 120),
            observation(LimitKind::FiveHour, 30.0, earlier, 30),
            observation(LimitKind::FiveHour, 20.0, RESET, 200),
            observation(
                LimitKind::FiveHour,
                25.0,
                RESET + Span::from_millis(60_000),
                60,
            ),
            observation(LimitKind::SevenDay, 50.0, RESET, 10),
        ];
        let found = peaks(&observations, LimitKind::FiveHour);
        assert_eq!(found.len(), 2);
        assert_close(found[0].utilization.percent(), 25.0);
        assert_close(found[1].utilization.percent(), 30.0);
        assert_eq!(found[1].window, window_of(LimitKind::FiveHour, earlier));
    }

    #[test]
    fn ignores_low_readings_and_limits_the_sample_count() {
        let observations: Vec<(Timestamp, LimitSnapshot)> = (0..15)
            .map(|index| {
                observation(
                    LimitKind::FiveHour,
                    if index == 14 { 2.0 } else { 40.0 },
                    RESET + Span::from_millis(index * 6 * HOUR),
                    30,
                )
            })
            .collect();
        let found = peaks(&observations, LimitKind::FiveHour);
        assert_eq!(found.len(), MAX_SAMPLES);
        assert!(found.iter().all(|peak| peak.utilization.percent() >= 5.0));
    }

    #[test]
    fn needs_enough_samples_before_calibrating() {
        let samples = vec![sample(10.0, 1_000); MIN_SAMPLES - 1];
        assert_eq!(calibrate(&samples), None);
        assert_eq!(
            calibrate(&[sample(10.0, 0), sample(3.0, 50), sample(10.0, 900)]),
            None
        );
    }

    #[test]
    fn uses_the_median_ratio() {
        let calibration = calibrate(&[
            sample(10.0, 1_000),
            sample(20.0, 3_000),
            sample(10.0, 9_000),
        ])
        .expect("calibrated");
        assert_close(calibration.tokens_per_percent(), 150.0);
        assert_eq!(calibration.samples(), 3);
        let even = calibrate(&[
            sample(10.0, 1_000),
            sample(10.0, 2_000),
            sample(10.0, 3_000),
            sample(10.0, 4_000),
        ])
        .expect("calibrated");
        assert_close(even.tokens_per_percent(), 250.0);
    }

    #[test]
    fn estimates_percent_from_tokens() {
        let calibration = calibrate(&[
            sample(10.0, 1_000),
            sample(10.0, 1_000),
            sample(10.0, 1_000),
        ])
        .expect("calibrated");
        assert_eq!(
            calibration.estimate(4_200).map(Utilization::percent),
            Some(42.0)
        );
        assert_eq!(calibration.estimate(0).map(Utilization::percent), Some(0.0));
    }
}
