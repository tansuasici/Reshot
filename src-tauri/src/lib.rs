//! Reshot — a fast, cross-platform, Lightshot-style screenshot tool.

mod capture;
mod commands;
mod error;
mod overlay;
mod platform;
mod shortcut;
mod tray;

use tauri::RunEvent;

/// Build and run the application. Called from `main.rs`.
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Launching Reshot again = take a screenshot.
            capture::trigger(app);
        }))
        .plugin(shortcut::plugin())
        .plugin(tauri_plugin_dialog::init())
        .manage(capture::FrameStore::default())
        .manage(commands::ClipboardStore::default())
        .invoke_handler(tauri::generate_handler![
            commands::overlay_ready,
            commands::get_frame,
            commands::show_overlay,
            commands::reveal_overlay,
            commands::close_overlay,
            commands::copy_image,
            commands::save_image,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let handle = app.handle();
            overlay::init(handle)?;
            tray::init(handle)?;
            shortcut::register(handle);
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
