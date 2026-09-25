use super::palette::Tone;
use crate::domain::clock::{Span, Timestamp};
use crate::domain::limit::{LimitKind, LimitSnapshot};
use crate::domain::severity::Severity;

const APP_NAME: &str = "Plimsoll";
const IDLE_LABEL: &str = "-";
const MAX_PERCENT_LABEL: f64 = 999.0;
const MILLIS_PER_MINUTE: i64 = 60_000;
const MINUTES_PER_HOUR: i64 = 60;
const MINUTES_PER_DAY: i64 = 1_440;

#[derive(Debug, Clone, PartialEq)]
pub enum TrayReading {
    Idle,
    Tokens(u64),
    Limits(Vec<LimitSnapshot>),
}

impl TrayReading {
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Idle => IDLE_LABEL.to_owned(),
            Self::Tokens(count) => compact_tokens(*count),
            Self::Limits(limits) => binding_limit(limits).map_or_else(
                || IDLE_LABEL.to_owned(),
                |limit| whole_percent(limit.utilization.percent()),
            ),
        }
    }

    #[must_use]
    pub fn tone(&self) -> Tone {
        match self {
            Self::Limits(limits) => binding_limit(limits).map_or(Tone::Neutral, |limit| {
                Tone::Severity(Severity::of(limit.utilization))
            }),
            Self::Idle | Self::Tokens(_) => Tone::Neutral,
        }
    }

    #[must_use]
    pub fn tooltip(&self, now: Timestamp) -> String {
        let detail = match self {
            Self::Tokens(count) => {
                format!("{} tokens in this 5-hour window", grouped_thousands(*count))
            }
            Self::Limits(limits) if !limits.is_empty() => limits
                .iter()
                .map(|limit| limit_line(limit, now))
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Idle | Self::Limits(_) => "No usage in the current 5-hour window".to_owned(),
        };
        format!("{APP_NAME}\n{detail}")
    }
}

fn binding_limit(limits: &[LimitSnapshot]) -> Option<&LimitSnapshot> {
    limits.iter().max_by(|first, second| {
        first
            .utilization
            .percent()
            .total_cmp(&second.utilization.percent())
    })
}

fn whole_percent(percent: f64) -> String {
    format!("{:.0}", percent.floor().min(MAX_PERCENT_LABEL))
}

fn compact_tokens(count: u64) -> String {
    match count {
        0..=999 => count.to_string(),
        1_000..=999_999 => format!("{}k", count / 1_000),
        1_000_000..=99_999_999 => format!("{}M", count / 1_000_000),
        _ => "99M".to_owned(),
    }
}

fn grouped_thousands(count: u64) -> String {
    let digits = count.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

fn limit_line(limit: &LimitSnapshot, now: Timestamp) -> String {
    let name = match limit.kind {
        LimitKind::FiveHour => "5-hour",
        LimitKind::SevenDay => "Weekly",
    };
    let percent = whole_percent(limit.utilization.percent());
    match limit.resets_at {
        Some(resets_at) if resets_at > now => {
            format!(
                "{name}: {percent}%, resets in {}",
                countdown(resets_at - now)
            )
        }
        Some(_) => format!("{name}: {percent}%, resetting now"),
        None => format!("{name}: {percent}%"),
    }
}

fn countdown(span: Span) -> String {
    let minutes = span.millis().max(0) / MILLIS_PER_MINUTE;
    let days = minutes / MINUTES_PER_DAY;
    let hours = minutes % MINUTES_PER_DAY / MINUTES_PER_HOUR;
    let rest = minutes % MINUTES_PER_HOUR;
    match (days, hours) {
        (0, 0) if rest == 0 => "under a minute".to_owned(),
        (0, 0) => format!("{rest}m"),
        (0, _) => format!("{hours}h {rest}m"),
        _ => format!("{days}d {hours}h"),
    }
}

#[cfg(test)]
mod tests {
    use super::{TrayReading, compact_tokens, countdown, grouped_thousands};
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};
    use crate::domain::severity::Severity;
    use crate::tray::glyph::glyph;
    use crate::tray::palette::Tone;

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);
    const MINUTE: i64 = 60_000;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;

    fn limit(kind: LimitKind, percent: f64, resets_in: Option<Span>) -> LimitSnapshot {
        LimitSnapshot {
            kind,
            utilization: Utilization::from_percent(percent).expect("valid percent"),
            resets_at: resets_in.map(|span| NOW + span),
        }
    }

    #[test]
    fn idle_shows_a_dash_on_a_neutral_badge() {
        let reading = TrayReading::Idle;
        assert_eq!(reading.label(), "-");
        assert_eq!(reading.tone(), Tone::Neutral);
        assert_eq!(
            reading.tooltip(NOW),
            "Plimsoll\nNo usage in the current 5-hour window"
        );
        assert_eq!(TrayReading::Limits(Vec::new()).label(), "-");
    }

    #[test]
    fn tokens_are_compact_on_the_icon_and_exact_in_the_tooltip() {
        let reading = TrayReading::Tokens(241_532);
        assert_eq!(reading.label(), "241k");
        assert_eq!(reading.tone(), Tone::Neutral);
        assert_eq!(
            reading.tooltip(NOW),
            "Plimsoll\n241,532 tokens in this 5-hour window"
        );
    }

    #[test]
    fn compact_tokens_never_exceed_four_glyphs() {
        let cases = [
            (0, "0"),
            (999, "999"),
            (1_000, "1k"),
            (999_999, "999k"),
            (1_000_000, "1M"),
            (99_999_999, "99M"),
            (u64::MAX, "99M"),
        ];
        for (count, expected) in cases {
            assert_eq!(compact_tokens(count), expected);
        }
    }

    #[test]
    fn groups_thousands_with_commas() {
        assert_eq!(grouped_thousands(0), "0");
        assert_eq!(grouped_thousands(999), "999");
        assert_eq!(grouped_thousands(1_000), "1,000");
        assert_eq!(grouped_thousands(12_345_678), "12,345,678");
    }

    #[test]
    fn the_most_used_limit_drives_the_icon() {
        let reading = TrayReading::Limits(vec![
            limit(LimitKind::FiveHour, 12.9, None),
            limit(LimitKind::SevenDay, 96.2, None),
        ]);
        assert_eq!(reading.label(), "96");
        assert_eq!(reading.tone(), Tone::Severity(Severity::Critical));
    }

    #[test]
    fn percentages_round_down_so_full_means_full() {
        let almost = TrayReading::Limits(vec![limit(LimitKind::FiveHour, 99.9, None)]);
        assert_eq!(almost.label(), "99");
        let over = TrayReading::Limits(vec![limit(LimitKind::FiveHour, 1_234.0, None)]);
        assert_eq!(over.label(), "999");
    }

    #[test]
    fn tooltip_lists_every_limit_with_its_reset() {
        let reading = TrayReading::Limits(vec![
            limit(
                LimitKind::FiveHour,
                42.0,
                Some(Span::from_millis(2 * HOUR + 15 * MINUTE)),
            ),
            limit(
                LimitKind::SevenDay,
                14.0,
                Some(Span::from_millis(3 * DAY + 4 * HOUR)),
            ),
        ]);
        assert_eq!(
            reading.tooltip(NOW),
            "Plimsoll\n5-hour: 42%, resets in 2h 15m\nWeekly: 14%, resets in 3d 4h"
        );
    }

    #[test]
    fn tooltip_handles_missing_or_past_resets() {
        let reading = TrayReading::Limits(vec![
            limit(LimitKind::FiveHour, 100.0, Some(Span::from_millis(-1))),
            limit(LimitKind::SevenDay, 3.0, None),
        ]);
        assert_eq!(
            reading.tooltip(NOW),
            "Plimsoll\n5-hour: 100%, resetting now\nWeekly: 3%"
        );
    }

    #[test]
    fn countdowns_pick_the_two_largest_units() {
        assert_eq!(countdown(Span::from_millis(59_999)), "under a minute");
        assert_eq!(countdown(Span::from_millis(45 * MINUTE)), "45m");
        assert_eq!(countdown(Span::hours(5)), "5h 0m");
        assert_eq!(countdown(Span::from_millis(6 * DAY + 23 * HOUR)), "6d 23h");
    }

    #[test]
    fn every_label_can_be_drawn() {
        let readings = [
            TrayReading::Idle,
            TrayReading::Tokens(u64::MAX),
            TrayReading::Tokens(123_456),
            TrayReading::Limits(vec![limit(LimitKind::FiveHour, 100.0, None)]),
        ];
        for reading in readings {
            assert!(
                reading
                    .label()
                    .chars()
                    .all(|character| glyph(character).is_some())
            );
        }
    }
}
