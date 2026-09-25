pub mod jsonl;
pub mod rfc3339;

use std::ffi::OsString;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("failed to read usage files: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to watch usage files: {0}")]
    Watch(#[from] notify::Error),
}

#[must_use]
pub fn claude_home_from<F>(lookup: F) -> Option<PathBuf>
where
    F: Fn(&'static str) -> Option<OsString>,
{
    if let Some(config) = lookup("CLAUDE_CONFIG_DIR").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(config));
    }
    lookup("USERPROFILE")
        .filter(|value| !value.is_empty())
        .map(|home| PathBuf::from(home).join(".claude"))
}

#[cfg(test)]
pub(crate) mod test_env {
    use std::ffi::OsString;

    pub fn lookup(
        pairs: &'static [(&'static str, &'static str)],
    ) -> impl Fn(&'static str) -> Option<OsString> {
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::claude_home_from;
    use super::test_env::lookup;
    use std::path::PathBuf;

    #[test]
    fn defaults_to_the_user_profile() {
        assert_eq!(
            claude_home_from(lookup(&[("USERPROFILE", "C:\\Users\\dev")])),
            Some(PathBuf::from("C:\\Users\\dev").join(".claude"))
        );
    }

    #[test]
    fn claude_config_dir_takes_precedence() {
        assert_eq!(
            claude_home_from(lookup(&[
                ("USERPROFILE", "C:\\Users\\dev"),
                ("CLAUDE_CONFIG_DIR", "D:\\claude"),
            ])),
            Some(PathBuf::from("D:\\claude"))
        );
    }

    #[test]
    fn empty_or_missing_variables_yield_nothing() {
        assert_eq!(
            claude_home_from(lookup(&[("USERPROFILE", ""), ("CLAUDE_CONFIG_DIR", "")])),
            None
        );
        assert_eq!(claude_home_from(lookup(&[])), None);
    }
}
