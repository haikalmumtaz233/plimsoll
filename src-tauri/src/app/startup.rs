use std::path::Path;

use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("start with windows failed: {0}")]
    Launch(#[from] auto_launch::Error),
    #[error("could not locate the running executable: {0}")]
    Executable(#[from] std::io::Error),
}

#[must_use]
pub fn quoted(path: &Path) -> String {
    format!("\"{}\"", path.display())
}

pub fn launcher(app_name: &str, executable: &Path) -> Result<AutoLaunch, StartupError> {
    Ok(AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&quoted(executable))
        .build()?)
}

pub fn current_launcher(app_name: &str) -> Result<AutoLaunch, StartupError> {
    launcher(app_name, &std::env::current_exe()?)
}

#[cfg(test)]
mod tests {
    use super::{launcher, quoted};
    use std::path::Path;

    #[test]
    fn paths_with_spaces_are_quoted() {
        let path = Path::new("C:\\Users\\Jane Doe\\AppData\\Local\\Plimsoll\\plimsoll.exe");
        assert_eq!(
            quoted(path),
            "\"C:\\Users\\Jane Doe\\AppData\\Local\\Plimsoll\\plimsoll.exe\""
        );
    }

    #[test]
    fn the_launcher_registers_the_quoted_path() {
        let path = Path::new("C:\\Program Files\\Plimsoll\\plimsoll.exe");
        let launcher = launcher("Plimsoll", path).expect("launcher");
        assert_eq!(launcher.get_app_name(), "Plimsoll");
        assert_eq!(
            launcher.get_app_path(),
            "\"C:\\Program Files\\Plimsoll\\plimsoll.exe\""
        );
    }
}
