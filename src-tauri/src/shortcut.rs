//! Global capture hotkey.

use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::capture;

/// Default capture hotkey. macOS keyboards have no Print Screen key.
pub const DEFAULT: &str = if cfg!(target_os = "macos") {
    "CommandOrControl+Shift+9"
} else {
    "PrintScreen"
};

/// The global-shortcut plugin, wired to start a capture on key press.
pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                capture::trigger(app);
            }
        })
        .build()
}

/// Register the default hotkey. A failure (key taken by another app) is
/// logged, never fatal: the tray menu still works.
pub fn register(app: &AppHandle) {
    if let Err(e) = app.global_shortcut().register(DEFAULT) {
        eprintln!("[reshot] could not register hotkey {DEFAULT}: {e}");
    }
}
