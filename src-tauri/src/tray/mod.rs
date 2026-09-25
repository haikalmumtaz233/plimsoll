pub mod glyph;
mod menu;
pub mod palette;
mod popup;
pub mod render;

use tauri::menu::{Menu, MenuEvent, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Runtime};

use crate::error::AppError;
use menu::MenuAction;

const TRAY_ID: &str = "plimsoll";
const TOOLTIP: &str = "Plimsoll";

pub fn install<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or(AppError::MissingIcon)?;
    let open = menu_item(app, MenuAction::Open)?;
    let quit = menu_item(app, MenuAction::Quit)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip(TOOLTIP)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu_event(app, &event))
        .on_tray_icon_event(|tray, event| handle_tray_event(tray.app_handle(), &event))
        .build(app)?;
    Ok(())
}

fn menu_item<R: Runtime>(app: &AppHandle<R>, action: MenuAction) -> Result<MenuItem<R>, AppError> {
    Ok(MenuItem::with_id(
        app,
        action.id(),
        action.label(),
        true,
        None::<&str>,
    )?)
}

fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: &MenuEvent) {
    match MenuAction::from_id(event.id().as_ref()) {
        Some(MenuAction::Open) => report(popup::show(app)),
        Some(MenuAction::Quit) => app.exit(0),
        None => {}
    }
}

fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, event: &TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        report(popup::toggle(app));
    }
}

fn report(result: Result<(), AppError>) {
    if let Err(error) = result {
        eprintln!("tray action failed: {error}");
    }
}
