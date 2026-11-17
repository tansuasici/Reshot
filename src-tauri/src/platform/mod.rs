//! Per-OS differences: coordinate spaces and overlay window behaviour.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{configure_overlay, return_focus, set_overlay_opaque};

#[cfg(not(target_os = "macos"))]
mod other;
#[cfg(not(target_os = "macos"))]
pub use other::{configure_overlay, return_focus, set_overlay_opaque};

use tauri::{AppHandle, WebviewWindow};

use crate::capture::MonitorRect;
use crate::error::Result;

/// Cursor position in xcap's coordinate space.
///
/// tao reports macOS cursor positions as points × the *primary* monitor's
/// scale, while xcap uses plain points there; elsewhere both are physical.
pub fn cursor_point(app: &AppHandle) -> (i32, i32) {
    let Ok(pos) = app.cursor_position() else {
        return (0, 0);
    };
    let scale = if cfg!(target_os = "macos") {
        app.primary_monitor()
            .ok()
            .flatten()
            .map_or(1.0, |m| m.scale_factor())
    } else {
        1.0
    };
    (
        (pos.x / scale).round() as i32,
        (pos.y / scale).round() as i32,
    )
}

/// Size of `rect` (xcap coordinates) in logical (CSS) pixels.
pub fn logical_size(rect: MonitorRect, scale_factor: f32) -> (f64, f64) {
    if cfg!(target_os = "macos") {
        // xcap already reports points on macOS.
        (f64::from(rect.width), f64::from(rect.height))
    } else {
        let scale = f64::from(scale_factor).max(f64::EPSILON);
        (
            f64::from(rect.width) / scale,
            f64::from(rect.height) / scale,
        )
    }
}

/// Cover `rect` (xcap coordinates) with `window`.
///
/// Size first, then position: on macOS a resize keeps the window's
/// bottom-left corner fixed, so resizing after positioning would shift the
/// window up by the size difference (visible on the very first capture).
pub fn place_window(window: &WebviewWindow, rect: MonitorRect) -> Result<()> {
    if cfg!(target_os = "macos") {
        // xcap reports points on macOS — exactly tao's logical units.
        window.set_size(tauri::LogicalSize::new(rect.width, rect.height))?;
        window.set_position(tauri::LogicalPosition::new(rect.x, rect.y))?;
    } else {
        window.set_size(tauri::PhysicalSize::new(rect.width, rect.height))?;
        window.set_position(tauri::PhysicalPosition::new(rect.x, rect.y))?;
    }
    Ok(())
}
