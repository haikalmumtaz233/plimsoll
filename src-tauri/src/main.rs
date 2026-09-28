#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    match plimsoll_lib::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            plimsoll_lib::diagnostics::error("app", &format!("plimsoll failed to start: {error}"));
            ExitCode::FAILURE
        }
    }
}
