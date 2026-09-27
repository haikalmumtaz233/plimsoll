use std::path::{Path, PathBuf};

use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use thiserror::Error;

#[cfg(windows)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("start with windows failed: {0}")]
    Launch(#[from] auto_launch::Error),
    #[error("could not locate the running executable: {0}")]
    Executable(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct Startup {
    launcher: AutoLaunch,
    app_name: String,
    executable: PathBuf,
}

impl Startup {
    pub fn current(app_name: &str) -> Result<Self, StartupError> {
        let executable = std::env::current_exe()?;
        Ok(Self {
            launcher: launcher(app_name, &executable)?,
            app_name: app_name.to_owned(),
            executable,
        })
    }

    pub fn enable(&self) -> Result<(), StartupError> {
        Ok(self.launcher.enable()?)
    }

    pub fn disable(&self) -> Result<(), StartupError> {
        Ok(self.launcher.disable()?)
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.launcher.is_enabled().unwrap_or(false)
            && registered_command(&self.app_name)
                .is_some_and(|command| points_to(&command, &self.executable))
    }
}

#[must_use]
pub fn quoted(path: &Path) -> String {
    format!("\"{}\"", path.display())
}

#[must_use]
pub fn points_to(command: &str, executable: &Path) -> bool {
    command.trim() == quoted(executable)
}

pub fn launcher(app_name: &str, executable: &Path) -> Result<AutoLaunch, StartupError> {
    Ok(AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&quoted(executable))
        .build()?)
}

#[cfg(windows)]
fn registered_command(app_name: &str) -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(RUN_KEY)
        .ok()?
        .get_value(app_name)
        .ok()
}

#[cfg(not(windows))]
fn registered_command(_app_name: &str) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::{launcher, points_to, quoted};
    use std::path::Path;

    const INSTALLED: &str = "C:\\Users\\Jane Doe\\AppData\\Local\\Plimsoll\\plimsoll.exe";

    #[test]
    fn paths_with_spaces_are_quoted() {
        assert_eq!(
            quoted(Path::new(INSTALLED)),
            "\"C:\\Users\\Jane Doe\\AppData\\Local\\Plimsoll\\plimsoll.exe\""
        );
    }

    #[test]
    fn the_launcher_registers_the_quoted_path() {
        let launcher = launcher("Plimsoll", Path::new(INSTALLED)).expect("launcher");
        assert_eq!(launcher.get_app_name(), "Plimsoll");
        assert_eq!(launcher.get_app_path(), quoted(Path::new(INSTALLED)));
    }

    #[test]
    fn only_entries_for_this_executable_count_as_enabled() {
        let executable = Path::new(INSTALLED);
        assert!(points_to(&format!("{} ", quoted(executable)), executable));
        assert!(!points_to(INSTALLED, executable));
        assert!(!points_to(
            "\"D:\\Self Project\\plimsoll\\target\\debug\\plimsoll.exe\" ",
            executable
        ));
    }
}
