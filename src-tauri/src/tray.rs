//! Tray / menu-bar icon and its menu.

use std::time::Duration;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};

use crate::error::Result;
use crate::settings::SettingsStore;
use crate::{capture, settings_window};

/// Let the menu finish closing so it is not in the screenshot.
const MENU_CLOSE_DELAY: Duration = Duration::from_millis(200);

/// The "Take Screenshot" item, kept so its shortcut label can follow the setting.
struct CaptureItem(MenuItem<Wry>);

/// Build the tray icon. Must be called once from `setup`, after the
/// settings are managed.
pub fn init(app: &AppHandle) -> Result<()> {
    let accelerator = app.state::<SettingsStore>().get().shortcut;
    // The label is cosmetic: never fail startup over it.
    let capture_item = MenuItem::with_id(
        app,
        "capture",
        "Take Screenshot",
        true,
        Some(accelerator.as_str()),
    )
    .or_else(|_| MenuItem::with_id(app, "capture", "Take Screenshot", true, None::<&str>))?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Reshot", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &capture_item,
            &PredefinedMenuItem::separator(app)?,
            &settings,
            &quit,
        ],
    )?;
    app.manage(CaptureItem(capture_item));

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Reshot")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "capture" => {
                let app = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(MENU_CLOSE_DELAY);
                    capture::trigger(&app);
                });
            }
            "settings" => {
                // Building a window inside a menu event handler can deadlock
                // on Windows: do it from another thread.
                let app = app.clone();
                std::thread::spawn(move || {
                    if let Err(e) = settings_window::open(&app) {
                        eprintln!("[reshot] could not open settings: {e}");
                    }
                });
            }
            "quit" => app.exit(0),
            _ => {}
        });
    builder = with_icon(app, builder)?;
    builder.build(app)?;
    Ok(())
}

/// Show `accelerator` next to "Take Screenshot". Best effort.
pub fn set_capture_accelerator(app: &AppHandle, accelerator: &str) {
    if let Some(item) = app.try_state::<CaptureItem>()
        && let Err(e) = item.0.set_accelerator(Some(accelerator))
    {
        eprintln!("[reshot] could not update tray shortcut label: {e}");
    }
}

/// macOS: a monochrome template image, so the system tints it for light/dark
/// menu bars (Apple HIG). Elsewhere: the full-colour app icon.
#[cfg(target_os = "macos")]
fn with_icon(_app: &AppHandle, builder: TrayIconBuilder<Wry>) -> Result<TrayIconBuilder<Wry>> {
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray-template.png"))?;
    Ok(builder.icon(icon).icon_as_template(true))
}

#[cfg(not(target_os = "macos"))]
fn with_icon(app: &AppHandle, builder: TrayIconBuilder<Wry>) -> Result<TrayIconBuilder<Wry>> {
    Ok(match app.default_window_icon() {
        Some(icon) => builder.icon(icon.clone()),
        None => builder,
    })
}
