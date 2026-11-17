//! Global capture hotkey: registered from the settings, changeable at runtime.

use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::error::{Error, Result};
use crate::settings::{SettingsStore, parse_shortcut};
use crate::{capture, tray};

/// Default capture hotkey on every platform: ⌘⇧9 on macOS, Ctrl+Shift+9 elsewhere.
pub const DEFAULT: &str = "CommandOrControl+Shift+9";

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

/// Register the saved hotkey. A failure (key taken by another app) is
/// logged, never fatal: the tray menu still works.
pub fn register(app: &AppHandle) {
    if let Err(e) = resume(app) {
        eprintln!("[reshot] could not register hotkey: {e}");
    }
}

/// Make sure the saved hotkey is registered (after [`suspend`]).
pub fn resume(app: &AppHandle) -> Result<()> {
    let accelerator = app.state::<SettingsStore>().get().shortcut;
    let shortcut = parse_shortcut(&accelerator)?;
    let manager = app.global_shortcut();
    if !manager.is_registered(shortcut) {
        manager
            .register(shortcut)
            .map_err(|e| Error::Shortcut(format!("{accelerator}: {e}")))?;
    }
    Ok(())
}

/// Release the saved hotkey while the Settings window records a new one,
/// so pressing it there doesn't take a screenshot.
pub fn suspend(app: &AppHandle) {
    let accelerator = app.state::<SettingsStore>().get().shortcut;
    if let Ok(shortcut) = parse_shortcut(&accelerator) {
        let manager = app.global_shortcut();
        if manager.is_registered(shortcut) {
            let _ = manager.unregister(shortcut);
        }
    }
}

/// Switch to a new hotkey: register it first, then release the old one, so
/// a failure (combo taken by another app) leaves the old hotkey working.
pub fn change(app: &AppHandle, accelerator: &str) -> Result<()> {
    let new = parse_shortcut(accelerator)?;
    let store = app.state::<SettingsStore>();
    let old = parse_shortcut(&store.get().shortcut).ok();
    let manager = app.global_shortcut();
    if !manager.is_registered(new) {
        manager
            .register(new)
            .map_err(|e| Error::Shortcut(format!("{accelerator}: {e}")))?;
    }
    if let Some(old) = old.filter(|old| *old != new)
        && manager.is_registered(old)
    {
        let _ = manager.unregister(old);
    }
    store.update(|s| s.shortcut = accelerator.to_owned())?;
    tray::set_capture_accelerator(app, accelerator);
    Ok(())
}
