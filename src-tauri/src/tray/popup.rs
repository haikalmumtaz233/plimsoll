use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

use crate::error::AppError;

const POPUP_LABEL: &str = "popup";

fn window<R: Runtime>(app: &AppHandle<R>) -> Result<WebviewWindow<R>, AppError> {
    app.get_webview_window(POPUP_LABEL)
        .ok_or(AppError::MissingWindow(POPUP_LABEL))
}

pub fn show<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let popup = window(app)?;
    popup.show()?;
    popup.set_focus()?;
    Ok(())
}

pub fn toggle<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let popup = window(app)?;
    if popup.is_visible()? {
        popup.hide()?;
        Ok(())
    } else {
        show(app)
    }
}
