pub mod clock;
pub mod engine;
pub mod locale;
pub mod runtime;
pub mod startup;
pub mod view;

use tauri::RunEvent;

use crate::commands;
use crate::error::AppError;
use crate::tray;

pub fn run() -> Result<(), AppError> {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Err(error) = tray::open_popup(app) {
                eprintln!("failed to open the running instance: {error}");
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            tray::install(app.handle())?;
            runtime::start(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::usage_summary,
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
