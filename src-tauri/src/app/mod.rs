pub mod clock;
pub mod engine;
pub mod locale;
pub mod runtime;
pub mod view;

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
            commands::set_manual_percent
        ])
        .run(tauri::generate_context!())?;
    Ok(())
}
