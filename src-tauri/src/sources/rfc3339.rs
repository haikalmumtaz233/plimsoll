use crate::domain::clock::Timestamp;

const MILLIS_PER_SECOND: i64 = 1_000;
const MILLIS_PER_MINUTE: i64 = 60_000;
const MILLIS_PER_HOUR: i64 = 3_600_000;
const MILLIS_PER_DAY: i64 = 86_400_000;
const MINUTES_PER_DAY: i64 = 1_440;
const DAYS_BEFORE_UNIX_EPOCH: i64 = 719_468;
const DAYS_PER_ERA: i64 = 146_097;
const YEARS_PER_ERA: i64 = 400;

#[must_use]
pub fn parse(text: &str) -> Option<Timestamp> {
    let body = text
        .strip_suffix('Z')
        .or_else(|| text.strip_suffix("+00:00"))?;
    let (date, time) = body.split_once('T')?;
    let (year, month, day) = parse_date(date)?;
    let (clock, fraction) = match time.split_once('.') {
        Some((clock, fraction)) => (clock, Some(fraction)),
        None => (time, None),
    };
    let (hour, minute, second) = parse_clock(clock)?;
    let millis = match fraction {
        Some(fraction) => parse_millis(fraction)?,
        None => 0,
    };
    let total_minutes = days_from_civil(year, month, day) * MINUTES_PER_DAY + hour * 60 + minute;
    Some(Timestamp::from_unix_millis(
        total_minutes * MILLIS_PER_MINUTE + second * MILLIS_PER_SECOND + millis,
    ))
}

fn fixed_number(field: &str, width: usize) -> Option<i64> {
    if field.len() != width || !field.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    field.parse().ok()
}

fn parse_date(date: &str) -> Option<(i64, i64, i64)> {
    let mut parts = date.split('-');
    let year = fixed_number(parts.next()?, 4)?;
    let month = fixed_number(parts.next()?, 2)?;
    let day = fixed_number(parts.next()?, 2)?;
    let valid = parts.next().is_none()
        && (1..=12).contains(&month)
        && (1..=days_in_month(year, month)).contains(&day);
    valid.then_some((year, month, day))
}

fn parse_clock(clock: &str) -> Option<(i64, i64, i64)> {
    let mut parts = clock.split(':');
    let hour = fixed_number(parts.next()?, 2)?;
    let minute = fixed_number(parts.next()?, 2)?;
    let second = fixed_number(parts.next()?, 2)?;
    let valid = parts.next().is_none() && hour < 24 && minute < 60 && second < 60;
    valid.then_some((hour, minute, second))
}

fn parse_millis(fraction: &str) -> Option<i64> {
    if fraction.is_empty()
        || fraction.len() > 9
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let padded = format!("{fraction:0<3}");
    padded.get(..3)?.parse().ok()
}

const fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

const fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

const fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let shifted_year = if month <= 2 { year - 1 } else { year };
    let era = shifted_year.div_euclid(YEARS_PER_ERA);
    let year_of_era = shifted_year - era * YEARS_PER_ERA;
    let month_from_march = (month + 9) % 12;
    let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * DAYS_PER_ERA + day_of_era - DAYS_BEFORE_UNIX_EPOCH
}

#[must_use]
pub fn format(at: Timestamp) -> String {
    let millis = at.unix_millis();
    let days = millis.div_euclid(MILLIS_PER_DAY);
    let of_day = millis.rem_euclid(MILLIS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    let hour = of_day / MILLIS_PER_HOUR;
    let minute = of_day % MILLIS_PER_HOUR / MILLIS_PER_MINUTE;
    let second = of_day % MILLIS_PER_MINUTE / MILLIS_PER_SECOND;
    let fraction = of_day % MILLIS_PER_SECOND;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{fraction:03}Z")
}

const fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + DAYS_BEFORE_UNIX_EPOCH;
    let era = shifted.div_euclid(DAYS_PER_ERA);
    let day_of_era = shifted - era * DAYS_PER_ERA;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_era + era * YEARS_PER_ERA + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::{format, parse};
    use crate::domain::clock::Timestamp;

    fn millis(text: &str) -> Option<i64> {
        parse(text).map(Timestamp::unix_millis)
    }

    #[test]
    fn parses_claude_code_timestamps() {
        assert_eq!(millis("2026-09-25T01:35:37.212Z"), Some(1_790_300_137_212));
    }

    #[test]
    fn parses_the_unix_epoch_and_times_before_it() {
        assert_eq!(millis("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(millis("1969-12-31T23:59:59.999Z"), Some(-1));
    }

    #[test]
    fn pads_and_truncates_fractions_to_millis() {
        assert_eq!(millis("1970-01-01T00:00:00.2Z"), Some(200));
        assert_eq!(millis("1970-01-01T00:00:00.21Z"), Some(210));
        assert_eq!(millis("1970-01-01T00:00:00.123456789Z"), Some(123));
    }

    #[test]
    fn accepts_an_explicit_zero_offset() {
        assert_eq!(
            millis("2026-09-25T01:35:37.212+00:00"),
            millis("2026-09-25T01:35:37.212Z")
        );
    }

    #[test]
    fn handles_leap_days() {
        assert_eq!(millis("2024-02-29T00:00:00Z"), Some(1_709_164_800_000));
        assert_eq!(millis("2000-02-29T00:00:00Z"), Some(951_782_400_000));
        assert_eq!(millis("2023-02-29T00:00:00Z"), None);
        assert_eq!(millis("1900-02-29T00:00:00Z"), None);
    }

    #[test]
    fn formats_utc_timestamps_with_millis() {
        assert_eq!(
            format(Timestamp::from_unix_millis(1_790_300_137_212)),
            "2026-09-25T01:35:37.212Z"
        );
        assert_eq!(
            format(Timestamp::from_unix_millis(0)),
            "1970-01-01T00:00:00.000Z"
        );
        assert_eq!(
            format(Timestamp::from_unix_millis(-1)),
            "1969-12-31T23:59:59.999Z"
        );
        assert_eq!(
            format(Timestamp::from_unix_millis(1_709_164_800_000)),
            "2024-02-29T00:00:00.000Z"
        );
    }

    #[test]
    fn formatting_round_trips_through_parsing() {
        for millis in [0, 951_782_400_000, 1_790_300_137_212, 4_102_444_799_999] {
            let at = Timestamp::from_unix_millis(millis);
            assert_eq!(parse(&format(at)), Some(at), "{millis}");
        }
    }

    #[test]
    fn rejects_malformed_input() {
        for text in [
            "",
            "garbage",
            "2026-09-25",
            "2026-09-25T01:35:37",
            "2026-09-25T01:35:37+07:00",
            "2026-13-01T00:00:00Z",
            "2026-09-31T00:00:00Z",
            "2026-09-25T24:00:00Z",
            "2026-09-25T01:60:00Z",
            "2026-09-25T01:35:60Z",
            "2026-9-25T01:35:37Z",
            "+026-09-25T01:35:37Z",
            "2026-09-25T01:35:37.Z",
            "2026-09-25T01:35:37.1234567890Z",
            "2026-09-25T01:35:37.2a1Z",
            "2026-09-25T01:35:37:00Z",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }
}
