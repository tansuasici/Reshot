//! The Settings window: created on demand from the tray, destroyed on close.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::Result;

/// Window label (also used by `capabilities/settings.json`).
pub const LABEL: &str = "settings";

/// Open the Settings window, or bring it to the front if it is already open.
///
/// Call from a spawned thread, not from an event handler (window creation
/// there can deadlock on Windows).
pub fn open(app: &AppHandle) -> Result<()> {
    // Reshot runs as a menu-bar accessory app; unhide it so its window can
    // come to the front.
    #[cfg(target_os = "macos")]
    app.show()?;

    let window = match app.get_webview_window(LABEL) {
        Some(window) => window,
        None => WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("settings.html".into()))
            .title("Reshot Settings")
            .inner_size(440.0, 300.0)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .center()
            .visible(false)
            .build()?,
    };
    window.unminimize()?;
    window.show()?;
    window.set_focus()?;
    Ok(())
}

/// Whether the Settings window is on screen.
pub fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_window(LABEL)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}
