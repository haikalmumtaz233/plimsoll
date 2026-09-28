pub mod browser;
pub mod clock;
pub mod engine;
pub mod locale;
pub mod runtime;
pub mod startup;
pub mod view;

use tauri::{AppHandle, Manager, RunEvent, Runtime};

use crate::commands;
use crate::diagnostics;
use crate::error::AppError;
use crate::tray;

pub fn run() -> Result<(), AppError> {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Err(error) = tray::open_popup(app) {
                diagnostics::error(
                    "app",
                    &format!("failed to open the running instance: {error}"),
                );
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            start_diagnostics(app.handle());
            tray::install(app.handle())?;
            runtime::start(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::usage_summary,
            commands::refresh_now,
            commands::open_login,
            commands::set_cli_fallback,
            commands::open_usage_page,
            commands::set_accurate_mode,
            commands::set_preferences,
            commands::set_manual_percent,
            commands::set_autostart,
            commands::hide_popup
        ])
        .build(tauri::generate_context!())?
        .run(|_, event| {
            if let RunEvent::ExitRequested {
                code: None, api, ..
            } = event
            {
                api.prevent_exit();
            }
        });
    Ok(())
}

fn start_diagnostics<R: Runtime>(app: &AppHandle<R>) {
    if let Ok(directory) = app.path().app_local_data_dir() {
        diagnostics::init(directory.join(diagnostics::DIRECTORY));
    }
    diagnostics::info(
        "app",
        &format!("plimsoll {} started", app.package_info().version),
    );
}
