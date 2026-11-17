//! Capture the monitor under the cursor and hold the frame until the overlay
//! has pulled it.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use xcap::Monitor;

use crate::error::{Error, Result};
use crate::{overlay, platform};

/// Monitor bounds in the coordinate space xcap reports: points on macOS,
/// physical pixels elsewhere. `platform` converts them for window placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorRect {
    /// Left edge.
    pub x: i32,
    /// Top edge.
    pub y: i32,
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
}

/// Frame metadata sent to the overlay with the `capture-ready` event.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameMeta {
    /// Width of the captured image in physical pixels.
    pub width: u32,
    /// Height of the captured image in physical pixels.
    pub height: u32,
    /// Width the overlay's viewport will have once placed, in CSS pixels.
    pub css_width: f64,
    /// Height the overlay's viewport will have once placed, in CSS pixels.
    pub css_height: f64,
}

/// The most recent capture, waiting for the overlay to fetch its pixels.
#[derive(Default)]
pub struct FrameStore(Mutex<Option<Vec<u8>>>);

impl FrameStore {
    fn put(&self, pixels: Vec<u8>) {
        if let Ok(mut slot) = self.0.lock() {
            *slot = Some(pixels);
        }
    }

    /// Move the pixels out (no copy); the overlay keeps them from here on.
    pub fn take(&self) -> Result<Vec<u8>> {
        self.0
            .lock()
            .map_err(|_| Error::Capture("frame store poisoned".into()))?
            .take()
            .ok_or(Error::NoFrame)
    }

    /// Drop any frame that was never fetched.
    pub fn clear(&self) {
        if let Ok(mut slot) = self.0.lock() {
            *slot = None;
        }
    }
}

/// Event the overlay listens to; payload is [`FrameMeta`].
pub const CAPTURE_READY: &str = "capture-ready";

/// Hotkey / tray entry point. Runs the capture off the main thread. A missing
/// screen recording permission is explained in a dialog; other failures go to
/// stderr (there is no UI to show them in yet).
pub fn trigger(app: &AppHandle) {
    if overlay::is_visible(app) {
        return; // never capture our own overlay
    }
    let app = app.clone();
    std::thread::spawn(move || match capture_under_cursor(&app) {
        Ok(()) => {}
        Err(Error::PermissionDenied) => on_permission_denied(),
        Err(e) => eprintln!("[reshot] {e}"),
    });
}

fn capture_under_cursor(app: &AppHandle) -> Result<()> {
    if !platform::ensure_screen_capture_access() {
        return Err(Error::PermissionDenied);
    }
    let started = Instant::now();
    let (cx, cy) = platform::cursor_point(app);
    let monitor = match Monitor::from_point(cx, cy) {
        Ok(m) => m,
        Err(_) => primary_monitor()?,
    };
    let rect = MonitorRect {
        x: monitor.x()?,
        y: monitor.y()?,
        width: monitor.width()?,
        height: monitor.height()?,
    };

    let image = monitor.capture_image()?;
    let (css_width, css_height) = platform::logical_size(rect, monitor.scale_factor()?);
    let meta = FrameMeta {
        width: image.width(),
        height: image.height(),
        css_width,
        css_height,
    };
    let captured = started.elapsed();

    app.state::<FrameStore>().put(image.into_raw());
    overlay::place(app, rect)?;
    app.emit_to(overlay::LABEL, CAPTURE_READY, meta)?;

    if cfg!(debug_assertions) {
        eprintln!(
            "[reshot] captured {}x{} at {rect:?} in {captured:?}",
            meta.width, meta.height
        );
    }
    Ok(())
}

/// Screen recording is not granted yet.
fn on_permission_denied() {
    // The first request in a process shows macOS's own permission prompt, so
    // stay quiet. If the user tries again while still denied, the system won't
    // prompt again: take them straight to the Screen Recording settings.
    static ASKED: AtomicBool = AtomicBool::new(false);
    if ASKED.swap(true, Ordering::SeqCst)
        && !platform::SCREEN_CAPTURE_SETTINGS_URL.is_empty()
        && let Err(e) = std::process::Command::new("open")
            .arg(platform::SCREEN_CAPTURE_SETTINGS_URL)
            .spawn()
    {
        eprintln!("[reshot] could not open System Settings: {e}");
    }
}

fn primary_monitor() -> Result<Monitor> {
    Monitor::all()?
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .ok_or_else(|| Error::Capture("no monitor found".into()))
}
