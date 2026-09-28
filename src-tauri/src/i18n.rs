use crate::domain::clock::Span;
use crate::domain::limit::LimitKind;
use crate::domain::preferences::Language;
use crate::domain::severity::Severity;

const MILLIS_PER_MINUTE: i64 = 60_000;
const MINUTES_PER_HOUR: i64 = 60;
const MINUTES_PER_DAY: i64 = 1_440;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Text {
    language: Language,
}

impl Text {
    #[must_use]
    pub const fn new(language: Language) -> Self {
        Self { language }
    }

    #[must_use]
    pub const fn language(self) -> Language {
        self.language
    }

    #[must_use]
    pub const fn limit_label(self, kind: LimitKind) -> &'static str {
        match (self.language, kind) {
            (Language::English, LimitKind::FiveHour) => "5-hour",
            (Language::English, LimitKind::SevenDay) => "Weekly",
            (Language::Indonesian, LimitKind::FiveHour) => "5 jam",
            (Language::Indonesian, LimitKind::SevenDay) => "Mingguan",
        }
    }

    #[must_use]
    pub fn limit_title(self, kind: LimitKind, percent: &str) -> String {
        match (self.language, kind) {
            (Language::English, _) => format!("{} limit at {percent}%", self.limit_label(kind)),
            (Language::Indonesian, LimitKind::FiveHour) => format!("Limit 5 jam di {percent}%"),
            (Language::Indonesian, LimitKind::SevenDay) => {
                format!("Limit mingguan di {percent}%")
            }
        }
    }

    #[must_use]
    pub fn limit_line(self, kind: LimitKind, percent: &str, reset: Option<Span>) -> String {
        let head = format!("{}: {percent}%", self.limit_label(kind));
        match reset {
            Some(span) if span > Span::ZERO => format!("{head}, {}", self.resets_in(span)),
            Some(_) => match self.language {
                Language::English => format!("{head}, resetting now"),
                Language::Indonesian => format!("{head}, sedang reset"),
            },
            None => head,
        }
    }

    #[must_use]
    pub fn resets_in(self, span: Span) -> String {
        match self.language {
            Language::English => format!("resets in {}", self.countdown(span)),
            Language::Indonesian => format!("reset dalam {}", self.countdown(span)),
        }
    }

    #[must_use]
    pub fn resets_sentence(self, span: Span) -> String {
        match self.language {
            Language::English => format!("Resets in {}.", self.countdown(span)),
            Language::Indonesian => format!("Reset dalam {}.", self.countdown(span)),
        }
    }

    #[must_use]
    pub fn tokens_in_window(self, count: u64) -> String {
        match self.language {
            Language::English => format!("{} tokens in this 5-hour window", self.grouped(count)),
            Language::Indonesian => format!("{} token di jendela 5 jam ini", self.grouped(count)),
        }
    }

    #[must_use]
    pub const fn no_usage(self) -> &'static str {
        match self.language {
            Language::English => "No usage in the current 5-hour window",
            Language::Indonesian => "Belum ada pemakaian di jendela 5 jam ini",
        }
    }

    #[must_use]
    pub const fn outdated(self) -> &'static str {
        match self.language {
            Language::English => "Official reading is out of date",
            Language::Indonesian => "Data resmi belum diperbarui",
        }
    }

    #[must_use]
    pub fn past_level(self, severity: Severity) -> String {
        match self.language {
            Language::English => format!("Past your {} level.", self.level(severity)),
            Language::Indonesian => format!("Melewati level {}.", self.level(severity)),
        }
    }

    #[must_use]
    pub fn countdown(self, span: Span) -> String {
        let minutes = span.millis().max(0) / MILLIS_PER_MINUTE;
        let days = minutes / MINUTES_PER_DAY;
        let hours = minutes % MINUTES_PER_DAY / MINUTES_PER_HOUR;
        let rest = minutes % MINUTES_PER_HOUR;
        match (self.language, days, hours) {
            (Language::English, 0, 0) if rest == 0 => "under a minute".to_owned(),
            (Language::English, 0, 0) => format!("{rest}m"),
            (Language::English, 0, _) => format!("{hours}h {rest}m"),
            (Language::English, _, _) => format!("{days}d {hours}h"),
            (Language::Indonesian, 0, 0) if rest == 0 => "kurang dari semenit".to_owned(),
            (Language::Indonesian, 0, 0) => format!("{rest} menit"),
            (Language::Indonesian, 0, _) => format!("{hours} jam {rest} menit"),
            (Language::Indonesian, _, _) => format!("{days} hari {hours} jam"),
        }
    }

    #[must_use]
    pub fn grouped(self, count: u64) -> String {
        let separator = match self.language {
            Language::English => ',',
            Language::Indonesian => '.',
        };
        let digits = count.to_string();
        let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
        for (index, digit) in digits.chars().enumerate() {
            if index > 0 && (digits.len() - index).is_multiple_of(3) {
                grouped.push(separator);
            }
            grouped.push(digit);
        }
        grouped
    }

    const fn level(self, severity: Severity) -> &'static str {
        match (self.language, severity) {
            (_, Severity::Normal) => "normal",
            (Language::English, Severity::Elevated) => "warning",
            (Language::English, Severity::High) => "high",
            (Language::English, Severity::Critical) => "critical",
            (Language::Indonesian, Severity::Elevated) => "peringatan",
            (Language::Indonesian, Severity::High) => "tinggi",
            (Language::Indonesian, Severity::Critical) => "kritis",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Text;
    use crate::domain::clock::Span;
    use crate::domain::limit::LimitKind;
    use crate::domain::preferences::Language;
    use crate::domain::severity::Severity;

    const MINUTE: i64 = 60_000;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    const EN: Text = Text::new(Language::English);
    const ID: Text = Text::new(Language::Indonesian);

    #[test]
    fn countdowns_pick_the_two_largest_units() {
        assert_eq!(EN.countdown(Span::from_millis(59_999)), "under a minute");
        assert_eq!(EN.countdown(Span::from_millis(45 * MINUTE)), "45m");
        assert_eq!(EN.countdown(Span::hours(5)), "5h 0m");
        assert_eq!(
            EN.countdown(Span::from_millis(6 * DAY + 23 * HOUR)),
            "6d 23h"
        );
        assert_eq!(
            ID.countdown(Span::from_millis(59_999)),
            "kurang dari semenit"
        );
        assert_eq!(ID.countdown(Span::from_millis(45 * MINUTE)), "45 menit");
        assert_eq!(
            ID.countdown(Span::from_millis(2 * HOUR + 15 * MINUTE)),
            "2 jam 15 menit"
        );
        assert_eq!(
            ID.countdown(Span::from_millis(3 * DAY + 4 * HOUR)),
            "3 hari 4 jam"
        );
    }

    #[test]
    fn thousands_follow_the_language() {
        assert_eq!(EN.grouped(0), "0");
        assert_eq!(EN.grouped(999), "999");
        assert_eq!(EN.grouped(12_345_678), "12,345,678");
        assert_eq!(ID.grouped(241_532), "241.532");
    }

    #[test]
    fn limit_lines_include_the_reset() {
        assert_eq!(
            EN.limit_line(LimitKind::FiveHour, "42", Some(Span::hours(2))),
            "5-hour: 42%, resets in 2h 0m"
        );
        assert_eq!(
            ID.limit_line(LimitKind::SevenDay, "14", Some(Span::from_millis(-1))),
            "Mingguan: 14%, sedang reset"
        );
        assert_eq!(EN.limit_line(LimitKind::SevenDay, "3", None), "Weekly: 3%");
    }

    #[test]
    fn toast_text_reads_naturally_in_both_languages() {
        assert_eq!(
            EN.limit_title(LimitKind::FiveHour, "82"),
            "5-hour limit at 82%"
        );
        assert_eq!(
            EN.limit_title(LimitKind::SevenDay, "82"),
            "Weekly limit at 82%"
        );
        assert_eq!(
            ID.limit_title(LimitKind::FiveHour, "82"),
            "Limit 5 jam di 82%"
        );
        assert_eq!(
            ID.limit_title(LimitKind::SevenDay, "82"),
            "Limit mingguan di 82%"
        );
        assert_eq!(EN.past_level(Severity::High), "Past your high level.");
        assert_eq!(ID.past_level(Severity::Critical), "Melewati level kritis.");
        assert_eq!(ID.resets_in(Span::hours(1)), "reset dalam 1 jam 0 menit");
        assert_eq!(EN.resets_sentence(Span::hours(1)), "Resets in 1h 0m.");
        assert_eq!(
            ID.resets_sentence(Span::hours(1)),
            "Reset dalam 1 jam 0 menit."
        );
    }

    #[test]
    fn window_messages_are_translated() {
        assert_eq!(
            EN.tokens_in_window(241_532),
            "241,532 tokens in this 5-hour window"
        );
        assert_eq!(
            ID.tokens_in_window(241_532),
            "241.532 token di jendela 5 jam ini"
        );
        assert_eq!(ID.no_usage(), "Belum ada pemakaian di jendela 5 jam ini");
        assert_eq!(ID.language(), Language::Indonesian);
    }
}
