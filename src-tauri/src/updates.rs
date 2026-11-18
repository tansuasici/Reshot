//! Self-update from GitHub releases via `tauri-plugin-updater`.
//!
//! Everything runs in Rust: a quiet check shortly after launch, and a manual
//! check from the tray. An update is only installed after the user agrees.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::error::Result;

/// Wait this long after launch before the automatic check, so it never
/// competes with startup.
const LAUNCH_DELAY: Duration = Duration::from_secs(10);

/// Release notes longer than this are cut in the prompt.
const MAX_NOTES_CHARS: usize = 400;

/// Set while a check or an install is running, so a second one is not started.
static BUSY: AtomicBool = AtomicBool::new(false);

/// What started a check. Decides how much the user is told.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// The quiet check after launch: only speaks up when there is an update.
    Auto,
    /// "Check for Updates…" in the tray: always reports the result.
    Manual,
}

/// Check once, [`LAUNCH_DELAY`] after launch.
pub fn check_on_launch(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio_sleep(LAUNCH_DELAY).await;
        check(app, Trigger::Auto).await;
    });
}

/// Check now, on behalf of the user.
pub fn check_now(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(check(app, Trigger::Manual));
}

async fn tokio_sleep(d: Duration) {
    // The Tauri async runtime is tokio, but tokio is not a direct dependency:
    // sleep on a blocking-pool thread instead of pulling it in.
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(d)).await;
}

async fn check(app: AppHandle, trigger: Trigger) {
    if BUSY.swap(true, Ordering::SeqCst) {
        if trigger == Trigger::Manual {
            inform(&app, "Reshot is already checking for updates.");
        }
        return;
    }
    let found = find(&app).await;
    match found {
        Ok(Some(update)) => offer(app, update),
        Ok(None) => {
            BUSY.store(false, Ordering::SeqCst);
            if let Some(text) = up_to_date_message(trigger) {
                inform(&app, text);
            }
        }
        Err(e) => {
            BUSY.store(false, Ordering::SeqCst);
            eprintln!("[reshot] update check failed: {e}");
            if trigger == Trigger::Manual {
                warn(&app, &format!("Could not check for updates.\n\n{e}"));
            }
        }
    }
}

async fn find(app: &AppHandle) -> Result<Option<Update>> {
    Ok(app.updater()?.check().await?)
}

/// Ask the user; install and restart on yes. Clears [`BUSY`] when done.
fn offer(app: AppHandle, update: Update) {
    let text = prompt_text(
        &update.current_version,
        &update.version,
        update.body.as_deref(),
    );
    let handle = app.clone();
    app.dialog()
        .message(text)
        .title("Update available")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Install and Restart".into(),
            "Later".into(),
        ))
        .show(move |yes| {
            if !yes {
                BUSY.store(false, Ordering::SeqCst);
                return;
            }
            tauri::async_runtime::spawn(async move {
                // Windows: the installer closes and restarts Reshot itself.
                match update.download_and_install(|_, _| {}, || {}).await {
                    Ok(()) => handle.restart(),
                    Err(e) => {
                        BUSY.store(false, Ordering::SeqCst);
                        eprintln!("[reshot] update failed: {e}");
                        warn(
                            &handle,
                            &format!("The update could not be installed.\n\n{e}"),
                        );
                    }
                }
            });
        });
}

fn inform(app: &AppHandle, text: &str) {
    app.dialog()
        .message(text)
        .title("Reshot")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::Ok)
        .show(|_| {});
}

fn warn(app: &AppHandle, text: &str) {
    app.dialog()
        .message(text)
        .title("Reshot")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::Ok)
        .show(|_| {});
}

/// The message shown when no update is found, if any.
fn up_to_date_message(trigger: Trigger) -> Option<&'static str> {
    match trigger {
        Trigger::Auto => None,
        Trigger::Manual => Some("You're up to date."),
    }
}

/// The question asked when an update is found.
fn prompt_text(current: &str, new: &str, notes: Option<&str>) -> String {
    let mut text = format!("Reshot {new} is available (you have {current}).");
    if let Some(notes) = notes.map(str::trim).filter(|n| !n.is_empty()) {
        text.push_str("\n\n");
        if notes.chars().count() > MAX_NOTES_CHARS {
            text.extend(notes.chars().take(MAX_NOTES_CHARS));
            text.push('…');
        } else {
            text.push_str(notes);
        }
    }
    text.push_str("\n\nInstall it and restart Reshot now?");
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_text_names_versions() {
        let text = prompt_text("0.2.1", "0.3.0", None);
        assert!(text.starts_with("Reshot 0.3.0 is available (you have 0.2.1)."));
        assert!(text.ends_with("restart Reshot now?"));
    }

    #[test]
    fn prompt_text_includes_notes() {
        let text = prompt_text("0.2.1", "0.3.0", Some("  Auto-update.\n"));
        assert!(text.contains("\n\nAuto-update.\n\n"));
        // Blank notes add nothing.
        assert_eq!(
            prompt_text("1", "2", Some("  ")),
            prompt_text("1", "2", None)
        );
    }

    #[test]
    fn prompt_text_cuts_long_notes() {
        let notes = "ü".repeat(MAX_NOTES_CHARS + 50);
        let text = prompt_text("1", "2", Some(&notes));
        let shown = "ü".repeat(MAX_NOTES_CHARS) + "…";
        assert!(text.contains(&shown));
        assert!(!text.contains(&"ü".repeat(MAX_NOTES_CHARS + 1)));
    }

    #[test]
    fn manual_check_reports_up_to_date() {
        assert_eq!(up_to_date_message(Trigger::Auto), None);
        assert_eq!(
            up_to_date_message(Trigger::Manual),
            Some("You're up to date.")
        );
    }
}
