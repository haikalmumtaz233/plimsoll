use std::iter::Peekable;
use std::str::Chars;

use crate::domain::clock::{Span, Timestamp};
use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};
use crate::sources::rfc3339;

const SESSION_LABEL: &str = "current session";
const WEEKLY_LABELS: [&str; 2] = ["current week (all models)", "current week"];
const RESETS: &str = "resets ";
const ZONE_START: &str = " (";
const USED: &str = " used";
const LEFT: &str = " left";
const AT_WORD: &str = "at";
const FULL_PERCENT: f64 = 100.0;
const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];
const MINUTES_PER_HOUR: i64 = 60;
const HOURS_PER_HALF_DAY: i64 = 12;
const ESCAPE: char = '\u{1b}';
const BELL: char = '\u{7}';
const STRING_TERMINATOR: char = '\\';

#[must_use]
pub fn parse(text: &str, now: Timestamp, offset: Span) -> Vec<LimitSnapshot> {
    let clean = strip_escapes(text).to_ascii_lowercase();
    let mut snapshots: Vec<LimitSnapshot> = Vec::new();
    for line in clean.lines() {
        if let Some(snapshot) = parse_line(line.trim(), now, offset)
            && snapshots.iter().all(|known| known.kind != snapshot.kind)
        {
            snapshots.push(snapshot);
        }
    }
    snapshots
}

fn parse_line(line: &str, now: Timestamp, offset: Span) -> Option<LimitSnapshot> {
    let (label, rest) = line.split_once(':')?;
    let kind = kind_of(label.trim())?;
    let utilization = used_percent(rest)?;
    let resets_at = rest
        .split_once(RESETS)
        .map(|(_, reset)| reset.split(ZONE_START).next().unwrap_or(reset))
        .and_then(|reset| parse_reset(reset, now, offset));
    Some(LimitSnapshot {
        kind,
        utilization,
        resets_at,
    })
}

fn kind_of(label: &str) -> Option<LimitKind> {
    if label == SESSION_LABEL {
        Some(LimitKind::FiveHour)
    } else if WEEKLY_LABELS.contains(&label) {
        Some(LimitKind::SevenDay)
    } else {
        None
    }
}

fn used_percent(rest: &str) -> Option<Utilization> {
    let (before, after) = rest.split_once('%')?;
    let digits_start = before
        .char_indices()
        .rev()
        .take_while(|(_, character)| character.is_ascii_digit() || *character == '.')
        .last()
        .map_or(before.len(), |(index, _)| index);
    let (prefix, number) = before.split_at(digits_start);
    if number.is_empty() || prefix.ends_with('-') {
        return None;
    }
    let value: f64 = number.parse().ok()?;
    if value > FULL_PERCENT {
        return None;
    }
    let used = if after.starts_with(USED) {
        value
    } else if after.starts_with(LEFT) {
        FULL_PERCENT - value
    } else {
        return None;
    };
    Utilization::from_percent(used)
}

fn parse_reset(text: &str, now: Timestamp, offset: Span) -> Option<Timestamp> {
    let cleaned = text.replace(',', " ");
    let mut tokens = cleaned
        .split_whitespace()
        .filter(|token| *token != AT_WORD)
        .peekable();
    let date = match tokens.peek().copied().and_then(month_of) {
        Some(month) => {
            tokens.next();
            let day = tokens.next()?.parse().ok()?;
            let year = tokens
                .next_if(|token| is_year(token))
                .and_then(|token| token.parse().ok());
            Some((year, month, day))
        }
        None => None,
    };
    let minute = match tokens.next() {
        Some(token) => Some(minute_of_day(token)?),
        None => None,
    };
    if tokens.next().is_some() {
        return None;
    }
    let local_now = now + offset;
    let local = match (date, minute) {
        (Some(date), minute) => next_date(date, minute.unwrap_or(0), local_now)?,
        (None, Some(minute)) => next_time(minute, local_now)?,
        (None, None) => return None,
    };
    Some(local - offset)
}

fn next_date(
    (given_year, month, day): (Option<i64>, i64, i64),
    minute: i64,
    local_now: Timestamp,
) -> Option<Timestamp> {
    let (year, _, _) = rfc3339::civil_date(local_now);
    let candidate = rfc3339::from_civil(given_year.unwrap_or(year), month, day, minute)?;
    if given_year.is_none() && candidate < local_now - Span::days(1) {
        return rfc3339::from_civil(year + 1, month, day, minute);
    }
    Some(candidate)
}

fn next_time(minute: i64, local_now: Timestamp) -> Option<Timestamp> {
    let (year, month, day) = rfc3339::civil_date(local_now);
    let candidate = rfc3339::from_civil(year, month, day, minute)?;
    if candidate <= local_now {
        return Some(candidate + Span::days(1));
    }
    Some(candidate)
}

fn is_year(token: &str) -> bool {
    token.len() == 4 && token.bytes().all(|byte| byte.is_ascii_digit())
}

fn month_of(token: &str) -> Option<i64> {
    if token.len() < 3 || !token.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        return None;
    }
    let index = MONTHS.iter().position(|month| token.starts_with(month))?;
    i64::try_from(index + 1).ok()
}

fn minute_of_day(token: &str) -> Option<i64> {
    let (clock, afternoon) = if let Some(clock) = token.strip_suffix("am") {
        (clock, Some(false))
    } else if let Some(clock) = token.strip_suffix("pm") {
        (clock, Some(true))
    } else {
        (token, None)
    };
    let (hour, minute) = match clock.split_once(':') {
        Some((hour, minute)) if minute.len() == 2 => (number(hour)?, number(minute)?),
        None if afternoon.is_some() => (number(clock)?, 0),
        _ => return None,
    };
    let hour = match afternoon {
        Some(afternoon) if (1..=HOURS_PER_HALF_DAY).contains(&hour) => {
            hour % HOURS_PER_HALF_DAY + if afternoon { HOURS_PER_HALF_DAY } else { 0 }
        }
        None if hour < 2 * HOURS_PER_HALF_DAY => hour,
        _ => return None,
    };
    (minute < MINUTES_PER_HOUR).then_some(hour * MINUTES_PER_HOUR + minute)
}

fn number(text: &str) -> Option<i64> {
    let valid = (1..=2).contains(&text.len()) && text.bytes().all(|byte| byte.is_ascii_digit());
    if valid { text.parse().ok() } else { None }
}

fn strip_escapes(text: &str) -> String {
    let mut clean = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            ESCAPE => skip_sequence(&mut characters),
            '\r' => {}
            _ => clean.push(character),
        }
    }
    clean
}

fn skip_sequence(characters: &mut Peekable<Chars<'_>>) {
    match characters.next() {
        Some('[') => {
            for character in characters.by_ref() {
                if ('@'..='~').contains(&character) {
                    break;
                }
            }
        }
        Some(']') => {
            while let Some(character) = characters.next() {
                let terminated = character == BELL
                    || (character == ESCAPE && characters.next_if_eq(&STRING_TERMINATOR).is_some());
                if terminated {
                    break;
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::limit::{LimitKind, LimitSnapshot};
    use crate::sources::rfc3339;

    const JAKARTA: Span = Span::hours(7);

    fn at(text: &str) -> Timestamp {
        rfc3339::parse(text).expect("valid timestamp")
    }

    fn summary(snapshots: &[LimitSnapshot]) -> Vec<(LimitKind, f64, Option<Timestamp>)> {
        snapshots
            .iter()
            .map(|snapshot| {
                (
                    snapshot.kind,
                    snapshot.utilization.percent(),
                    snapshot.resets_at,
                )
            })
            .collect()
    }

    const PANEL: &str = "You are currently using your subscription to power your Claude Code usage\n\n\
        Current session: 31% used · resets Sep 29, 1:59am (Asia/Jakarta)\n\
        Current week (all models): 12.5% used · resets Oct 5, 6:59am (Asia/Jakarta)\n\
        Current week (Opus): 3% used · resets Oct 5, 6:59am (Asia/Jakarta)\n\n\
        What's contributing to your limits usage?\n\
        Last 24h · 40 requests · 2 sessions\n  80% of your usage was at >150k context\n";

    #[test]
    fn reads_session_and_weekly_limits_from_the_print_panel() {
        let now = at("2026-09-28T15:00:00Z");
        assert_eq!(
            summary(&parse(PANEL, now, JAKARTA)),
            vec![
                (LimitKind::FiveHour, 31.0, Some(at("2026-09-28T18:59:00Z"))),
                (LimitKind::SevenDay, 12.5, Some(at("2026-10-04T23:59:00Z"))),
            ]
        );
    }

    #[test]
    fn terminal_escapes_and_carriage_returns_are_ignored() {
        let noisy = "\u{1b}[1mCurrent session\u{1b}[22m: 31% used · resets 4pm (UTC)\r\n";
        let now = at("2026-09-28T10:00:00Z");
        assert_eq!(
            summary(&parse(noisy, now, Span::ZERO)),
            vec![(LimitKind::FiveHour, 31.0, Some(at("2026-09-28T16:00:00Z")))]
        );
    }

    #[test]
    fn a_time_without_a_date_is_the_next_occurrence() {
        let now = at("2026-09-28T15:00:00Z");
        let passed = "Current session: 5% used · resets 9:30pm (Asia/Jakarta)";
        assert_eq!(
            parse(passed, now, JAKARTA)[0].resets_at,
            Some(at("2026-09-29T14:30:00Z"))
        );
        let later = "Current session: 5% used · resets 11pm (Asia/Jakarta)";
        assert_eq!(
            parse(later, now, JAKARTA)[0].resets_at,
            Some(at("2026-09-28T16:00:00Z"))
        );
        let clock = "Current session: 5% used · resets 23:15 (Asia/Jakarta)";
        assert_eq!(
            parse(clock, now, JAKARTA)[0].resets_at,
            Some(at("2026-09-28T16:15:00Z"))
        );
    }

    #[test]
    fn dates_roll_into_the_next_year_when_needed() {
        let now = at("2026-12-30T12:00:00Z");
        let text = "Current week (all models): 90% used · resets Jan 2 at 7am (UTC)";
        assert_eq!(
            parse(text, now, Span::ZERO)[0].resets_at,
            Some(at("2027-01-02T07:00:00Z"))
        );
        let midnight = "Current week: 90% used · resets 12am (UTC)";
        assert_eq!(
            parse(midnight, now, Span::ZERO)[0].resets_at,
            Some(at("2026-12-31T00:00:00Z"))
        );
    }

    #[test]
    fn remaining_percentages_are_turned_into_used() {
        let text = "Current session: 74% left · resets 4pm (UTC)";
        assert!(
            (parse(text, at("2026-09-28T10:00:00Z"), Span::ZERO)[0]
                .utilization
                .percent()
                - 26.0)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn unreadable_resets_keep_the_percentage() {
        let text = "Current session: 40% used · resets whenever (Mars/Base)";
        assert_eq!(
            summary(&parse(text, at("2026-09-28T10:00:00Z"), Span::ZERO)),
            vec![(LimitKind::FiveHour, 40.0, None)]
        );
    }

    #[test]
    fn unrelated_or_malformed_text_yields_nothing() {
        let now = at("2026-09-28T10:00:00Z");
        for text in [
            "",
            "Unknown command: /usage",
            "Current session: lots used",
            "Current session: -5% used",
            "Current session: 101% used",
            "Current session: \u{e9}% used",
            "Current week (Sonnet): 10% used",
            "Please run /login · API Error: 401",
        ] {
            assert!(parse(text, now, Span::ZERO).is_empty(), "{text:?}");
        }
    }

    #[test]
    fn multibyte_text_before_the_number_is_handled() {
        let text = "Current session: \u{b7}\u{e9}7% used";
        let snapshots = parse(text, at("2026-09-28T10:00:00Z"), Span::ZERO);
        assert!((snapshots[0].utilization.percent() - 7.0).abs() < 1e-9);
    }

    #[test]
    fn each_limit_is_reported_once() {
        let text = "Current session: 10% used\nCurrent session: 20% used\n";
        assert_eq!(parse(text, at("2026-09-28T10:00:00Z"), Span::ZERO).len(), 1);
    }
}
