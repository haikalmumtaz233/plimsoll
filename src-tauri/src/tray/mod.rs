pub mod glyph;
mod menu;
pub mod palette;
mod popup;
pub mod reading;
pub mod render;

use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Runtime};

use crate::app::clock;
use crate::domain::clock::Timestamp;
use crate::error::AppError;
use menu::MenuAction;
use reading::TrayReading;

const TRAY_ID: &str = "plimsoll";
const DEFAULT_SCALE: f64 = 1.0;

pub fn install<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let idle = TrayReading::Idle;
    let open = menu_item(app, MenuAction::Open)?;
    let quit = menu_item(app, MenuAction::Quit)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon_image(app, &idle)?)
        .tooltip(idle.tooltip(clock::now()))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu_event(app, &event))
        .on_tray_icon_event(|tray, event| handle_tray_event(tray.app_handle(), &event))
        .build(app)?;
    Ok(())
}

pub fn show_reading<R: Runtime>(
    app: &AppHandle<R>,
    reading: &TrayReading,
    now: Timestamp,
) -> Result<(), AppError> {
    let tray = app.tray_by_id(TRAY_ID).ok_or(AppError::MissingTray)?;
    tray.set_icon(Some(icon_image(app, reading)?))?;
    tray.set_tooltip(Some(reading.tooltip(now)))?;
    Ok(())
}

fn icon_image<R: Runtime>(
    app: &AppHandle<R>,
    reading: &TrayReading,
) -> Result<Image<'static>, AppError> {
    let scale = app
        .primary_monitor()
        .ok()
        .flatten()
        .map_or(DEFAULT_SCALE, |monitor| monitor.scale_factor());
    let bitmap = render::render(
        &reading.label(),
        reading.tone().palette(),
        render::icon_size(scale),
    )
    .ok_or(AppError::IconRender)?;
    Ok(Image::new_owned(bitmap.rgba, bitmap.size, bitmap.size))
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
