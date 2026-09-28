pub mod file;
pub mod line;
pub mod rotation;

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, PoisonError};

use self::file::LogFile;
use self::line::{Entry, Level, format_line};
use crate::app::clock;

pub const DIRECTORY: &str = "logs";

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[derive(Debug)]
struct Logger {
    file: Mutex<LogFile>,
    home: Option<String>,
}

pub fn init(directory: PathBuf) {
    match LogFile::open(directory) {
        Ok(file) => {
            LOGGER
                .set(Logger {
                    file: Mutex::new(file),
                    home: std::env::var("USERPROFILE").ok(),
                })
                .ok();
        }
        Err(error) => mirror(&format!("diagnostic log is unavailable: {error}\n")),
    }
}

pub fn info(target: &str, message: &str) {
    write(Level::Info, target, message);
}

pub fn warn(target: &str, message: &str) {
    write(Level::Warn, target, message);
}

pub fn error(target: &str, message: &str) {
    write(Level::Error, target, message);
}

fn write(level: Level, target: &str, message: &str) {
    let logger = LOGGER.get();
    let home = logger
        .and_then(|logger| logger.home.clone())
        .or_else(|| std::env::var("USERPROFILE").ok());
    let entry = Entry {
        at: clock::now(),
        level,
        target,
        message,
    };
    let line = format_line(&entry, home.as_deref());
    mirror(&line);
    if let Some(logger) = logger {
        logger
            .file
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .append(&line)
            .ok();
    }
}

#[cfg(debug_assertions)]
fn mirror(line: &str) {
    eprint!("{line}");
}

#[cfg(not(debug_assertions))]
fn mirror(_line: &str) {}
