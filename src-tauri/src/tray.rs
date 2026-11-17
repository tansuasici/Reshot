//! Tray / menu-bar icon and its menu.

use std::time::Duration;

use tauri::AppHandle;
use tauri::Wry;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;

use crate::error::Result;
use crate::{capture, shortcut};

/// Let the menu finish closing so it is not in the screenshot.
const MENU_CLOSE_DELAY: Duration = Duration::from_millis(200);

/// Build the tray icon. Must be called once from `setup`.
pub fn init(app: &AppHandle) -> Result<()> {
    let capture_item = MenuItem::with_id(
        app,
        "capture",
        "Take Screenshot",
        true,
        Some(shortcut::DEFAULT),
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit Reshot", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&capture_item, &PredefinedMenuItem::separator(app)?, &quit],
    )?;

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
            "quit" => app.exit(0),
            _ => {}
        });
    builder = with_icon(app, builder)?;
    builder.build(app)?;
    Ok(())
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
