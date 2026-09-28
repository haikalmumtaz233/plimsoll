use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use thiserror::Error;

use super::locate;

pub const LOGIN_ARGS: [&str; 3] = ["auth", "login", "--claudeai"];

#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("claude code was not found on this pc")]
    NotFound,
    #[error("failed to run claude code: {0}")]
    Spawn(#[from] io::Error),
    #[error("claude code did not answer within the time limit")]
    TimedOut,
    #[error("claude code exited with code {0:?}")]
    Failed(Option<i32>),
    #[error("claude code printed no output")]
    NoOutput,
}

#[must_use]
pub fn executable() -> Option<PathBuf> {
    let path = std::env::var_os("PATH");
    let home = std::env::var_os("USERPROFILE").map(PathBuf::from);
    locate::locate(path.as_deref(), home.as_deref(), Path::is_file)
}

pub fn open_login() -> Result<Child, CliError> {
    let executable = executable().ok_or(CliError::NotFound)?;
    Ok(login_command(&executable).spawn()?)
}

#[must_use]
pub fn login_command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    command.args(LOGIN_ARGS);
    in_new_console(&mut command);
    command
}

#[cfg(windows)]
fn in_new_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(CREATE_NEW_CONSOLE);
}

#[cfg(not(windows))]
fn in_new_console(_command: &mut Command) {}

#[cfg(test)]
mod tests {
    use super::{LOGIN_ARGS, login_command};
    use std::ffi::OsStr;
    use std::path::Path;

    #[test]
    fn login_runs_the_subscription_flow_with_fixed_arguments() {
        let command = login_command(Path::new(r"C:\Tools\claude.exe"));
        assert_eq!(command.get_program(), OsStr::new(r"C:\Tools\claude.exe"));
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            vec!["auth", "login", "--claudeai"]
        );
    }

    #[test]
    fn login_never_bypasses_permissions() {
        assert!(LOGIN_ARGS.iter().all(|arg| !arg.contains("dangerously")));
    }
}
