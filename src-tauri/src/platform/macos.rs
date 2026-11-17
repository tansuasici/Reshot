//! macOS: put the overlay above the menu bar, on every Space and over
//! full-screen apps; keep Reshot out of the Dock.

use objc2::MainThreadMarker;
use objc2_app_kit::{NSMainMenuWindowLevel, NSWindow, NSWindowCollectionBehavior};
use tauri::{AppHandle, WebviewWindow};

/// Run `f` with the window's `NSWindow`, on the main thread only.
fn with_ns_window(window: &WebviewWindow, f: impl FnOnce(&NSWindow)) {
    if MainThreadMarker::new().is_none() {
        eprintln!("[reshot] NSWindow access off the main thread skipped");
        return;
    }
    let ptr = match window.ns_window() {
        Ok(ptr) if !ptr.is_null() => ptr,
        _ => return,
    };
    // SAFETY: `ns_window()` returns the live `NSWindow*` owned by this Tauri
    // window (checked non-null above). The window outlives this call, and we
    // are on the main thread (checked above), as AppKit requires.
    let ns_window: &NSWindow = unsafe { &*ptr.cast::<NSWindow>() };
    f(ns_window);
}

/// Raise the overlay's window level and Spaces behaviour.
///
/// Called from `setup`, i.e. on the main thread.
pub fn configure_overlay(window: &WebviewWindow) {
    with_ns_window(window, |ns_window| {
        ns_window.setLevel(NSMainMenuWindowLevel + 1);
        ns_window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );
    });
}

/// Make the overlay fully transparent (`false`) or opaque (`true`).
///
/// WebKit does not paint a hidden window, so the overlay is shown invisible
/// first and made opaque once the new frame has been composited; otherwise
/// the stale/blank previous content flashes. Must run on the main thread
/// (sync Tauri commands do).
pub fn set_overlay_opaque(window: &WebviewWindow, opaque: bool) {
    with_ns_window(window, |ns_window| {
        ns_window.setAlphaValue(if opaque { 1.0 } else { 0.0 });
    });
}

/// Deactivate Reshot so the previously active app gets focus back.
pub fn return_focus(app: &AppHandle) {
    if let Err(e) = app.hide() {
        eprintln!("[reshot] could not return focus: {e}");
    }
}
