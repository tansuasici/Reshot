//! Windows / Linux: the cross-platform Tauri window flags are enough.

use tauri::{AppHandle, WebviewWindow};

/// No extra tweaks needed outside macOS.
pub fn configure_overlay(_window: &WebviewWindow) {}

/// The overlay is shown as soon as it is painted; nothing to do here.
pub fn set_overlay_opaque(_window: &WebviewWindow, _opaque: bool) {}

/// Hiding the window returns focus to the previous app on its own.
pub fn return_focus(_app: &AppHandle) {}
