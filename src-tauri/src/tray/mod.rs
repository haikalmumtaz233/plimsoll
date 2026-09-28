pub mod glyph;
mod menu;
pub mod palette;
pub mod placement;
mod popup;
pub mod reading;
pub mod render;

use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

use crate::app::{clock, locale};
use crate::diagnostics;
use crate::domain::clock::Timestamp;
use crate::domain::preferences::{Language, LanguageChoice};
use crate::domain::severity::Thresholds;
use crate::error::AppError;
use crate::i18n::Text;
use menu::MenuAction;
use reading::TrayReading;

const TRAY_ID: &str = "plimsoll";
const DEFAULT_SCALE: f64 = 1.0;

pub fn open_popup<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    popup::show(app, None)
}

pub fn hide_popup<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    popup::hide(app)
}

pub fn install<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let idle = TrayReading::Idle;
    let language = locale::resolve(LanguageChoice::System);
    let menu = build_menu(app, language)?;
    app.manage(popup::PopupLifetime::default());

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon_image(app, &idle, Thresholds::DEFAULT)?)
        .tooltip(idle.tooltip(Text::new(language), clock::now()))
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
    thresholds: Thresholds,
    text: Text,
    now: Timestamp,
) -> Result<(), AppError> {
    let tray = app.tray_by_id(TRAY_ID).ok_or(AppError::MissingTray)?;
    tray.set_icon(Some(icon_image(app, reading, thresholds)?))?;
    tray.set_tooltip(Some(reading.tooltip(text, now)))?;
    Ok(())
}

fn icon_image<R: Runtime>(
    app: &AppHandle<R>,
    reading: &TrayReading,
    thresholds: Thresholds,
) -> Result<Image<'static>, AppError> {
    let scale = app
        .primary_monitor()
        .ok()
        .flatten()
        .map_or(DEFAULT_SCALE, |monitor| monitor.scale_factor());
    let bitmap = render::render(
        &reading.label(),
        reading.tone(thresholds).palette(),
        render::icon_size(scale),
    )
    .ok_or(AppError::IconRender)?;
    Ok(Image::new_owned(bitmap.rgba, bitmap.size, bitmap.size))
}

pub fn set_language<R: Runtime>(app: &AppHandle<R>, language: Language) -> Result<(), AppError> {
    let tray = app.tray_by_id(TRAY_ID).ok_or(AppError::MissingTray)?;
    tray.set_menu(Some(build_menu(app, language)?))?;
    Ok(())
}

fn build_menu<R: Runtime>(app: &AppHandle<R>, language: Language) -> Result<Menu<R>, AppError> {
    let open = menu_item(app, MenuAction::Open, language)?;
    let quit = menu_item(app, MenuAction::Quit, language)?;
    Ok(Menu::with_items(app, &[&open, &quit])?)
}

fn menu_item<R: Runtime>(
    app: &AppHandle<R>,
    action: MenuAction,
    language: Language,
) -> Result<MenuItem<R>, AppError> {
    Ok(MenuItem::with_id(
        app,
        action.id(),
        action.label(language),
        true,
        None::<&str>,
    )?)
}

fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: &MenuEvent) {
    match MenuAction::from_id(event.id().as_ref()) {
        Some(MenuAction::Open) => report(popup::show(app, None)),
        Some(MenuAction::Quit) => app.exit(0),
        None => {}
    }
}

fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, event: &TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        rect,
        ..
    } = event
    {
        report(popup::toggle(app, Some(*rect)));
    }
}

fn report(result: Result<(), AppError>) {
    if let Err(error) = result {
        diagnostics::error("tray", &format!("tray action failed: {error}"));
    }
}
