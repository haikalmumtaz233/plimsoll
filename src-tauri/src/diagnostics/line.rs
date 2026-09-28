use crate::domain::clock::Timestamp;
use crate::sources::rfc3339;

pub const MAX_MESSAGE_CHARS: usize = 512;
pub const HOME_PLACEHOLDER: &str = "%USERPROFILE%";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Entry<'a> {
    pub at: Timestamp,
    pub level: Level,
    pub target: &'a str,
    pub message: &'a str,
}

#[must_use]
pub fn format_line(entry: &Entry<'_>, home: Option<&str>) -> String {
    let message: String = redact_home(entry.message, home)
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .take(MAX_MESSAGE_CHARS)
        .collect();
    format!(
        "{} {} {}: {message}\n",
        rfc3339::format(entry.at),
        entry.level.name(),
        entry.target
    )
}

fn redact_home(message: &str, home: Option<&str>) -> String {
    let Some(home) = home
        .map(|home| home.trim_end_matches(['\\', '/']))
        .filter(|home| !home.is_empty())
    else {
        return message.to_owned();
    };
    let haystack = message.to_ascii_lowercase();
    let needle = home.to_ascii_lowercase();
    let mut redacted = String::with_capacity(message.len());
    let mut kept_from = 0;
    for (start, _) in haystack.match_indices(&needle) {
        redacted.push_str(message.get(kept_from..start).unwrap_or_default());
        redacted.push_str(HOME_PLACEHOLDER);
        kept_from = start + needle.len();
    }
    redacted.push_str(message.get(kept_from..).unwrap_or_default());
    redacted
}

#[cfg(test)]
mod tests {
    use super::{Entry, Level, MAX_MESSAGE_CHARS, format_line};
    use crate::domain::clock::Timestamp;

    const AT: Timestamp = Timestamp::from_unix_millis(1_790_300_137_212);

    fn line(level: Level, message: &str, home: Option<&str>) -> String {
        format_line(
            &Entry {
                at: AT,
                level,
                target: "oauth",
                message,
            },
            home,
        )
    }

    #[test]
    fn lines_carry_time_level_target_and_message() {
        assert_eq!(
            line(Level::Warn, "http 429, retry after 300 s", None),
            "2026-09-25T01:35:37.212Z WARN oauth: http 429, retry after 300 s\n"
        );
        assert!(line(Level::Info, "ok", None).contains(" INFO oauth: ok"));
        assert!(line(Level::Error, "down", None).contains(" ERROR oauth: down"));
    }

    #[test]
    fn control_characters_cannot_forge_extra_lines() {
        assert_eq!(
            line(Level::Info, "first\nFAKE ERROR\r\tx", None),
            "2026-09-25T01:35:37.212Z INFO oauth: first FAKE ERROR  x\n"
        );
    }

    #[test]
    fn the_user_profile_is_redacted_case_insensitively() {
        let message = r"failed to watch C:\Users\Haikal\.claude\projects and c:\users\haikal\x";
        assert_eq!(
            line(Level::Warn, message, Some(r"C:\Users\Haikal\")),
            "2026-09-25T01:35:37.212Z WARN oauth: \
             failed to watch %USERPROFILE%\\.claude\\projects and %USERPROFILE%\\x\n"
        );
        assert!(line(Level::Warn, message, Some("")).contains(r"C:\Users\Haikal"));
    }

    #[test]
    fn redaction_keeps_non_ascii_text_intact() {
        assert!(
            line(Level::Info, r"é C:\Users\Zoë\a ü", Some(r"C:\Users\Zoë"))
                .ends_with("oauth: é %USERPROFILE%\\a ü\n")
        );
    }

    #[test]
    fn long_messages_are_truncated() {
        let message = "é".repeat(MAX_MESSAGE_CHARS * 2);
        let formatted = line(Level::Info, &message, None);
        let body = formatted
            .trim_end()
            .rsplit_once(": ")
            .map(|(_, body)| body)
            .unwrap_or_default();
        assert_eq!(body.chars().count(), MAX_MESSAGE_CHARS);
    }
}
