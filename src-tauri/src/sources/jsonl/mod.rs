pub mod line;
pub mod reader;
pub mod scanner;
pub mod watch;

use std::ffi::OsString;
use std::path::PathBuf;

#[must_use]
pub fn projects_root() -> Option<PathBuf> {
    projects_root_from(std::env::var_os)
}

#[must_use]
pub fn projects_root_from<F>(lookup: F) -> Option<PathBuf>
where
    F: Fn(&'static str) -> Option<OsString>,
{
    if let Some(config) = lookup("CLAUDE_CONFIG_DIR").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(config).join("projects"));
    }
    lookup("USERPROFILE")
        .filter(|value| !value.is_empty())
        .map(|home| PathBuf::from(home).join(".claude").join("projects"))
}

#[cfg(test)]
mod tests {
    use super::projects_root_from;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn lookup(
        pairs: &'static [(&'static str, &'static str)],
    ) -> impl Fn(&'static str) -> Option<OsString> {
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| OsString::from(value))
        }
    }

    #[test]
    fn defaults_to_the_user_profile() {
        assert_eq!(
            projects_root_from(lookup(&[("USERPROFILE", "C:\\Users\\dev")])),
            Some(
                PathBuf::from("C:\\Users\\dev")
                    .join(".claude")
                    .join("projects")
            )
        );
    }

    #[test]
    fn claude_config_dir_takes_precedence() {
        assert_eq!(
            projects_root_from(lookup(&[
                ("USERPROFILE", "C:\\Users\\dev"),
                ("CLAUDE_CONFIG_DIR", "D:\\claude"),
            ])),
            Some(PathBuf::from("D:\\claude").join("projects"))
        );
    }

    #[test]
    fn empty_or_missing_variables_yield_nothing() {
        assert_eq!(
            projects_root_from(lookup(&[("USERPROFILE", ""), ("CLAUDE_CONFIG_DIR", "")])),
            None
        );
        assert_eq!(projects_root_from(lookup(&[])), None);
    }
}
