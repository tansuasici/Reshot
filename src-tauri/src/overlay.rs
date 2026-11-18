//! The single, pre-created overlay window: placed over the captured monitor,
//! shown once the frontend has painted the frame, hidden (never destroyed).

use tauri::{AppHandle, Manager, WebviewWindow};

use crate::capture::{FrameStore, MonitorRect};
use crate::error::{Error, Result};
use crate::platform;

/// Window label from `tauri.conf.json`.
pub const LABEL: &str = "overlay";

fn window(app: &AppHandle) -> Result<WebviewWindow> {
    app.get_webview_window(LABEL)
        .ok_or(Error::Tauri(tauri::Error::WindowNotFound))
}

/// Whether the overlay is currently on screen.
pub fn is_visible(app: &AppHandle) -> bool {
    window(app)
        .and_then(|w| Ok(w.is_visible()?))
        .unwrap_or(false)
}

/// One-time setup: platform window tweaks and "close means hide".
pub fn init(app: &AppHandle) -> Result<()> {
    let win = window(app)?;
    platform::configure_overlay(&win);
    let handle = app.clone();
    win.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            if let Err(e) = hide(&handle) {
                eprintln!("[reshot] {e}");
            }
        }
    });
    Ok(())
}

/// Move/resize the (hidden) overlay to cover `rect`.
pub fn place(app: &AppHandle, rect: MonitorRect) -> Result<()> {
    platform::place_window(&window(app)?, rect)
}

/// Show and focus the overlay, still invisible: the frontend calls
/// [`reveal`] once WebKit has composited the new frame.
pub fn show(app: &AppHandle) -> Result<()> {
    let win = window(app)?;
    platform::set_overlay_opaque(&win, false);
    win.show()?;
    win.set_focus()?;
    Ok(())
}

/// Make the (shown) overlay visible.
///
/// Focus is requested again here: on Linux, `show` is queued to the GTK
/// thread and tao drops a `set_focus` made before the window is mapped, so
/// the one in [`show`] is lost and the keyboard shortcuts would not work
/// until the user clicked. By the time the frame is painted it is mapped.
pub fn reveal(app: &AppHandle) -> Result<()> {
    let win = window(app)?;
    platform::set_overlay_opaque(&win, true);
    win.set_focus()?;
    Ok(())
}

/// Hide the overlay, drop any unfetched frame and hand focus back.
pub fn hide(app: &AppHandle) -> Result<()> {
    window(app)?.hide()?;
    app.state::<FrameStore>().clear();
    platform::return_focus(app);
    Ok(())
}
