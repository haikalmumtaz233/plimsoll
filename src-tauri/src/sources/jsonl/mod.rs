pub mod line;
pub mod reader;
pub mod scanner;
pub mod watch;

use std::ffi::OsString;
use std::path::PathBuf;

use super::claude_home_from;

#[must_use]
pub fn projects_root() -> Option<PathBuf> {
    projects_root_from(std::env::var_os)
}

#[must_use]
pub fn projects_root_from<F>(lookup: F) -> Option<PathBuf>
where
    F: Fn(&'static str) -> Option<OsString>,
{
    claude_home_from(lookup).map(|home| home.join("projects"))
}

#[cfg(test)]
mod tests {
    use super::projects_root_from;
    use crate::sources::test_env::lookup;
    use std::path::PathBuf;

    #[test]
    fn projects_live_under_the_claude_home() {
        assert_eq!(
            projects_root_from(lookup(&[("USERPROFILE", "C:\\Users\\dev")])),
            Some(
                PathBuf::from("C:\\Users\\dev")
                    .join(".claude")
                    .join("projects")
            )
        );
        assert_eq!(
            projects_root_from(lookup(&[("CLAUDE_CONFIG_DIR", "D:\\claude")])),
            Some(PathBuf::from("D:\\claude").join("projects"))
        );
        assert_eq!(projects_root_from(lookup(&[])), None);
    }
}
