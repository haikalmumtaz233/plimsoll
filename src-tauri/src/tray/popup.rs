use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, Rect, Runtime, WebviewWindow};

use super::placement::{self, Area, Point, Size};
use crate::error::AppError;

const POPUP_LABEL: &str = "popup";
const TRAY_SCALE: f64 = 1.0;

fn window<R: Runtime>(app: &AppHandle<R>) -> Result<WebviewWindow<R>, AppError> {
    app.get_webview_window(POPUP_LABEL)
        .ok_or(AppError::MissingWindow(POPUP_LABEL))
}

pub fn show<R: Runtime>(app: &AppHandle<R>, anchor: Option<Rect>) -> Result<(), AppError> {
    let popup = window(app)?;
    if let Some(point) = placement_for(app, &popup, anchor) {
        popup.set_position(PhysicalPosition::new(point.x, point.y))?;
    }
    popup.show()?;
    popup.set_focus()?;
    Ok(())
}

pub fn toggle<R: Runtime>(app: &AppHandle<R>, anchor: Option<Rect>) -> Result<(), AppError> {
    let popup = window(app)?;
    if popup.is_visible()? {
        popup.hide()?;
        Ok(())
    } else {
        show(app, anchor)
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
