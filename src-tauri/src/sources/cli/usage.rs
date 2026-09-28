use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::login::{CliError, executable};

pub const USAGE_ARGS: [&str; 4] = ["-p", "/usage", "--allowed-tools", ""];
pub const TIMEOUT: Duration = Duration::from_secs(20);
pub const MAX_OUTPUT_BYTES: u64 = 64 * 1024;

const POLL_EVERY: Duration = Duration::from_millis(100);
const READ_GRACE: Duration = Duration::from_secs(1);

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn read_usage(working_directory: &Path) -> Result<String, CliError> {
    let executable = executable().ok_or(CliError::NotFound)?;
    let mut child = usage_command(&executable, working_directory).spawn()?;
    let stdout = child.stdout.take().ok_or(CliError::NoOutput)?;
    let (sender, output) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let read = stdout.take(MAX_OUTPUT_BYTES).read_to_end(&mut bytes);
        sender.send(read.map(|_| bytes)).ok();
    });
    let deadline = Instant::now() + TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().ok();
            child.wait().ok();
            return Err(CliError::TimedOut);
        }
        thread::sleep(POLL_EVERY);
    };
    if !status.success() {
        return Err(CliError::Failed(status.code()));
    }
    let bytes = output
        .recv_timeout(READ_GRACE)
        .map_err(|_| CliError::NoOutput)??;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[must_use]
pub fn usage_command(executable: &Path, working_directory: &Path) -> Command {
    let mut command = Command::new(executable);
    command
        .args(USAGE_ARGS)
        .current_dir(working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    without_window(&mut command);
    command
}

#[cfg(windows)]
fn without_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn without_window(_command: &mut Command) {}

#[cfg(test)]
mod tests {
    use super::{USAGE_ARGS, usage_command};
    use std::ffi::OsStr;
    use std::path::Path;

    #[test]
    fn usage_runs_print_mode_without_tools_in_the_given_folder() {
        let command = usage_command(
            Path::new(r"C:\Tools\claude.exe"),
            Path::new(r"C:\Data\plimsoll\cli"),
        );
        assert_eq!(command.get_program(), OsStr::new(r"C:\Tools\claude.exe"));
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            vec!["-p", "/usage", "--allowed-tools", ""]
        );
        assert_eq!(
            command.get_current_dir(),
            Some(Path::new(r"C:\Data\plimsoll\cli"))
        );
    }

    #[test]
    fn usage_never_bypasses_permissions() {
        assert!(USAGE_ARGS.iter().all(|arg| !arg.contains("dangerously")));
    }
}
