use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const USAGE_PAGE: &str = "https://claude.ai/settings/usage";

const EXPLORER: &str = "explorer.exe";

pub fn open_usage_page() -> io::Result<()> {
    let system_root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "SystemRoot is not set"))?;
    browser_command(&system_root).spawn().map(drop)
}

#[must_use]
pub fn browser_command(system_root: &Path) -> Command {
    let mut command = Command::new(system_root.join(EXPLORER));
    command.arg(USAGE_PAGE);
    command
}

#[cfg(test)]
mod tests {
    use super::{USAGE_PAGE, browser_command};
    use std::ffi::OsStr;
    use std::path::Path;

    #[test]
    fn opens_only_the_fixed_claude_usage_page_through_windows() {
        let command = browser_command(Path::new(r"C:\Windows"));
        assert_eq!(
            command.get_program(),
            OsStr::new(r"C:\Windows\explorer.exe")
        );
        assert_eq!(command.get_args().collect::<Vec<_>>(), vec![USAGE_PAGE]);
        assert!(USAGE_PAGE.starts_with("https://claude.ai/"));
    }
}
