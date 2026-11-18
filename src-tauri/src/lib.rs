//! Reshot — a fast, cross-platform, Lightshot-style screenshot tool.

mod capture;
mod commands;
mod error;
mod overlay;
mod platform;
mod settings;
mod settings_window;
mod shortcut;
mod tray;
mod updates;

use tauri::{Manager, RunEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};

/// Build and run the application. Called from `main.rs`.
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Launching Reshot again = take a screenshot.
            capture::trigger(app);
        }))
        .plugin(shortcut::plugin())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(capture::FrameStore::default())
        .manage(commands::ClipboardStore::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_frame,
            commands::show_overlay,
            commands::reveal_overlay,
            commands::close_overlay,
            commands::copy_image,
            commands::save_image,
            commands::get_settings,
            commands::set_shortcut,
            commands::suspend_shortcut,
            commands::resume_shortcut,
            commands::choose_save_dir,
            commands::set_format,
            commands::set_autostart,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let handle = app.handle();
            app.manage(settings::SettingsStore::load(handle));
            overlay::init(handle)?;
            tray::init(handle)?;
            shortcut::register(handle);
            refresh_login_item(handle);
            updates::check_on_launch(handle);
            Ok(())
        })
        .build(tauri::generate_context!());

    match app {
        Ok(app) => app.run(|app, event| {
            // `app` is only needed for the macOS-only `Reopen` arm below.
            #[cfg(not(target_os = "macos"))]
            let _ = app;
            match event {
                // Tray app: stay alive with no visible window; only "Quit" exits.
                RunEvent::ExitRequested {
                    api, code: None, ..
                } => api.prevent_exit(),
                // Clicking Reshot in Finder/Dock/Spotlight while it runs = take a screenshot.
                #[cfg(target_os = "macos")]
                RunEvent::Reopen { .. } => capture::trigger(app),
                _ => {}
            }
        }),
        Err(e) => {
            eprintln!("[reshot] failed to start: {e}");
            std::process::exit(1);
        }
    }
}

/// Rewrite an existing open-at-login item so it points at this executable.
/// The item stores the binary's path, which can change between versions
/// (for example the macOS binary was renamed from `reshot` to `Reshot`).
fn refresh_login_item(app: &tauri::AppHandle) {
    let autolaunch = app.autolaunch();
    if autolaunch.is_enabled().unwrap_or(false)
        && let Err(e) = autolaunch.enable()
    {
        eprintln!("[reshot] failed to refresh the login item: {e}");
    }
}
