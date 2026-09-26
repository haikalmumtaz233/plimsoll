pub mod clock;
pub mod engine;

use crate::error::AppError;
use crate::tray;

pub fn run() -> Result<(), AppError> {
    tauri::Builder::default()
        .setup(|app| {
            tray::install(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
