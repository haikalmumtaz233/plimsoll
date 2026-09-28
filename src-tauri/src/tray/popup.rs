use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use tauri::webview::PageLoadEvent;
use tauri::{
    AppHandle, Manager, Monitor, PhysicalPosition, Rect, Runtime, WebviewWindow,
    WebviewWindowBuilder,
};

use super::placement::{self, Area, Point, Size};
use crate::diagnostics;
use crate::error::AppError;

const POPUP_LABEL: &str = "popup";
const TRAY_SCALE: f64 = 1.0;
const RELEASE_DELAY: Duration = Duration::from_secs(30);

#[derive(Debug, Default)]
pub struct PopupLifetime {
    generation: AtomicU64,
}

impl PopupLifetime {
    pub fn renew(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    #[must_use]
    pub fn is_current(&self, generation: u64) -> bool {
        self.generation.load(Ordering::SeqCst) == generation
    }
}

fn create<R: Runtime>(app: &AppHandle<R>) -> Result<WebviewWindow<R>, AppError> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == POPUP_LABEL)
        .ok_or(AppError::MissingWindow(POPUP_LABEL))?;
    Ok(WebviewWindowBuilder::from_config(app, config)?
        .on_page_load(|popup, payload| {
            if matches!(payload.event(), PageLoadEvent::Finished)
                && let Err(error) = reveal(&popup)
            {
                diagnostics::error("popup", &format!("failed to show the popup: {error}"));
            }
        })
        .build()?)
}

pub fn show<R: Runtime>(app: &AppHandle<R>, anchor: Option<Rect>) -> Result<(), AppError> {
    renew(app);
    match app.get_webview_window(POPUP_LABEL) {
        Some(popup) => {
            place(app, &popup, anchor)?;
            reveal(&popup)?;
            Ok(())
        }
        None => place(app, &create(app)?, anchor),
    }
}

fn place<R: Runtime>(
    app: &AppHandle<R>,
    popup: &WebviewWindow<R>,
    anchor: Option<Rect>,
) -> Result<(), AppError> {
    if let Some(point) = placement_for(app, popup, anchor) {
        popup.set_position(PhysicalPosition::new(point.x, point.y))?;
    }
    Ok(())
}

fn reveal<R: Runtime>(popup: &WebviewWindow<R>) -> Result<(), AppError> {
    popup.show()?;
    popup.set_focus()?;
    Ok(())
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let Some(popup) = app.get_webview_window(POPUP_LABEL) else {
        return Ok(());
    };
    popup.hide()?;
    schedule_release(app);
    Ok(())
}

pub fn toggle<R: Runtime>(app: &AppHandle<R>, anchor: Option<Rect>) -> Result<(), AppError> {
    let visible = match app.get_webview_window(POPUP_LABEL) {
        Some(popup) => popup.is_visible()?,
        None => false,
    };
    if visible {
        hide(app)
    } else {
        show(app, anchor)
    }
}

fn renew<R: Runtime>(app: &AppHandle<R>) -> Option<u64> {
    app.try_state::<PopupLifetime>()
        .map(|lifetime| lifetime.renew())
}

fn schedule_release<R: Runtime>(app: &AppHandle<R>) {
    let Some(generation) = renew(app) else {
        return;
    };
    let app = app.clone();
    thread::spawn(move || {
        thread::sleep(RELEASE_DELAY);
        let main = app.clone();
        if let Err(error) = app.run_on_main_thread(move || release(&main, generation)) {
            diagnostics::error("popup", &format!("failed to release the popup: {error}"));
        }
    });
}

fn release<R: Runtime>(app: &AppHandle<R>, generation: u64) {
    let current = app
        .try_state::<PopupLifetime>()
        .is_some_and(|lifetime| lifetime.is_current(generation));
    if !current {
        return;
    }
    if let Some(popup) = app.get_webview_window(POPUP_LABEL)
        && let Err(error) = popup.destroy()
    {
        diagnostics::error("popup", &format!("failed to release the popup: {error}"));
    }
}

fn placement_for<R: Runtime>(
    app: &AppHandle<R>,
    popup: &WebviewWindow<R>,
    anchor: Option<Rect>,
) -> Option<Point> {
    let outer = popup.outer_size().ok()?;
    let size = Size {
        width: i32::try_from(outer.width).ok()?,
        height: i32::try_from(outer.height).ok()?,
    };
    let icon = anchor.and_then(icon_area);
    let monitor = icon
        .and_then(|icon| {
            let center_x = f64::from(icon.x) + f64::from(icon.width) / 2.0;
            let center_y = f64::from(icon.y) + f64::from(icon.height) / 2.0;
            app.monitor_from_point(center_x, center_y).ok().flatten()
        })
        .or_else(|| app.primary_monitor().ok().flatten())?;
    let work = work_area(&monitor)?;
    Some(icon.map_or_else(
        || placement::corner(size, work),
        |icon| placement::beside_icon(icon, size, work),
    ))
}

fn icon_area(rect: Rect) -> Option<Area> {
    let position = rect.position.to_physical::<i32>(TRAY_SCALE);
    let size = rect.size.to_physical::<u32>(TRAY_SCALE);
    Some(Area {
        x: position.x,
        y: position.y,
        width: i32::try_from(size.width).ok()?,
        height: i32::try_from(size.height).ok()?,
    })
}

fn work_area(monitor: &Monitor) -> Option<Area> {
    let rect = monitor.work_area();
    Some(Area {
        x: rect.position.x,
        y: rect.position.y,
        width: i32::try_from(rect.size.width).ok()?,
        height: i32::try_from(rect.size.height).ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::PopupLifetime;

    #[test]
    fn showing_again_cancels_a_pending_release() {
        let lifetime = PopupLifetime::default();
        let hidden = lifetime.renew();
        assert!(lifetime.is_current(hidden));
        let shown = lifetime.renew();
        assert!(!lifetime.is_current(hidden));
        assert!(lifetime.is_current(shown));
    }
}
