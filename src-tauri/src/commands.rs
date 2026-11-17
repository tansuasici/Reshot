//! IPC commands invoked by the overlay and Settings frontends.
//!
//! Pixel data travels as raw bytes in both directions (no JSON, no PNG on the
//! hot path): `get_frame` returns an `ipc::Response`, and `copy_image` /
//! `save_image` take a raw RGBA body with `x-width` / `x-height` headers.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_dialog::DialogExt;

use crate::capture::FrameStore;
use crate::error::{Error, Result};
use crate::settings::{Format, SettingsStore};
use crate::{overlay, shortcut};

/// Command-line flag the login item launches Reshot with.
pub const AUTOSTART_ARG: &str = "--autostart";

/// Lazily created clipboard handle. Kept alive for the app's lifetime because
/// on Linux (X11/Wayland) the clipboard contents die with their owner.
#[derive(Default)]
pub struct ClipboardStore(Mutex<Option<arboard::Clipboard>>);

/// An RGBA image received from the frontend.
#[derive(Debug, PartialEq, Eq)]
struct RgbaImage<'a> {
    width: u32,
    height: u32,
    pixels: &'a [u8],
}

fn header_u32(request: &Request<'_>, name: &'static str) -> Result<u32> {
    request
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .ok_or(Error::InvalidRequest(name))
}

fn parse_image<'a>(request: &'a Request<'_>) -> Result<RgbaImage<'a>> {
    let InvokeBody::Raw(body) = request.body() else {
        return Err(Error::InvalidRequest("expected raw RGBA body"));
    };
    let width = header_u32(request, "x-width")?;
    let height = header_u32(request, "x-height")?;
    validate(width, height, body)
}

fn validate(width: u32, height: u32, pixels: &[u8]) -> Result<RgbaImage<'_>> {
    let expected = u64::from(width) * u64::from(height) * 4;
    if width == 0 || height == 0 || pixels.len() as u64 != expected {
        return Err(Error::InvalidRequest(
            "body size does not match width x height x 4",
        ));
    }
    Ok(RgbaImage {
        width,
        height,
        pixels,
    })
}

fn encode(image: &RgbaImage<'_>, format: Format) -> Result<Vec<u8>> {
    use image::ImageEncoder;
    use image::codecs::{jpeg::JpegEncoder, png::PngEncoder};

    let mut out = Vec::new();
    let encoded = match format {
        Format::Png => PngEncoder::new(&mut out).write_image(
            image.pixels,
            image.width,
            image.height,
            image::ExtendedColorType::Rgba8,
        ),
        Format::Jpeg => {
            // JPEG has no alpha channel.
            let (pixels, _) = image.pixels.as_chunks::<4>();
            let rgb: Vec<u8> = pixels.iter().flat_map(|&[r, g, b, _]| [r, g, b]).collect();
            JpegEncoder::new_with_quality(&mut out, 92).write_image(
                &rgb,
                image.width,
                image.height,
                image::ExtendedColorType::Rgb8,
            )
        }
    };
    encoded.map_err(|e| Error::Encode(e.to_string()))?;
    Ok(out)
}

/// Hand the captured frame's raw RGBA pixels to the overlay (moved, not copied).
#[tauri::command]
pub fn get_frame(store: State<'_, FrameStore>) -> Result<Response> {
    Ok(Response::new(store.take()?))
}

/// Show the overlay once the frontend has painted the frame.
#[tauri::command]
pub fn show_overlay(app: AppHandle) -> Result<()> {
    overlay::show(&app)
}

/// Make the shown overlay visible once its new frame has been composited.
#[tauri::command]
pub fn reveal_overlay(app: AppHandle) -> Result<()> {
    overlay::reveal(&app)
}

/// The overlay is listening for captures. On the first call after launch,
/// take a screenshot: opening Reshot from Finder/Spotlight means "capture now".
/// Not when the login item started it, though.
#[tauri::command]
pub fn overlay_ready(app: AppHandle) {
    static LAUNCH_CAPTURE_DONE: AtomicBool = AtomicBool::new(false);
    if !LAUNCH_CAPTURE_DONE.swap(true, Ordering::SeqCst) && !launched_at_login(std::env::args()) {
        crate::capture::trigger(&app);
    }
}

/// Whether the process was started by the open-at-login item.
fn launched_at_login<I: IntoIterator<Item = String>>(args: I) -> bool {
    args.into_iter().skip(1).any(|a| a == AUTOSTART_ARG)
}

/// Hide the overlay (Esc, after copy/save).
#[tauri::command]
pub fn close_overlay(app: AppHandle) -> Result<()> {
    overlay::hide(&app)
}

/// Put the edited selection on the clipboard.
#[tauri::command]
pub fn copy_image(request: Request<'_>, clipboard: State<'_, ClipboardStore>) -> Result<()> {
    let image = parse_image(&request)?;
    let mut slot = clipboard
        .0
        .lock()
        .map_err(|_| Error::Clipboard("clipboard lock poisoned".into()))?;
    let board = match slot.as_mut() {
        Some(board) => board,
        None => {
            slot.insert(arboard::Clipboard::new().map_err(|e| Error::Clipboard(e.to_string()))?)
        }
    };
    board
        .set_image(arboard::ImageData {
            width: image.width as usize,
            height: image.height as usize,
            bytes: Cow::Borrowed(image.pixels),
        })
        .map_err(|e| Error::Clipboard(e.to_string()))
}

/// Ask where to save, then write PNG or JPEG. Returns `false` if the user
/// cancelled — the overlay is shown again so they can keep editing.
#[tauri::command]
pub async fn save_image(app: AppHandle, request: Request<'_>) -> Result<bool> {
    let image = parse_image(&request)?;
    let settings = app.state::<SettingsStore>().get();
    let file_name = suggested_file_name(
        request
            .headers()
            .get("x-filename")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("Reshot"),
        settings.format,
    );

    // The overlay sits above everything, including the save panel.
    if let Some(win) = app.get_webview_window(overlay::LABEL) {
        win.hide()?;
    }

    // The default format's filter comes first, so the dialog preselects it.
    let mut dialog = app.dialog().file().set_file_name(file_name);
    dialog = match settings.format {
        Format::Png => dialog
            .add_filter("PNG", &["png"])
            .add_filter("JPEG", &["jpg", "jpeg"]),
        Format::Jpeg => dialog
            .add_filter("JPEG", &["jpg", "jpeg"])
            .add_filter("PNG", &["png"]),
    };
    if let Some(dir) = start_dir(&app, settings.existing_save_dir()) {
        dialog = dialog.set_directory(dir);
    }
    // Async command → runs on the async runtime, so blocking here is fine.
    let Some(target) = dialog.blocking_save_file() else {
        overlay::show(&app)?;
        overlay::reveal(&app)?;
        return Ok(false);
    };
    let path = target
        .into_path()
        .map_err(|_| Error::InvalidRequest("unsupported save location"))?;

    let bytes = encode(&image, Format::from_path(&path))?;
    std::fs::write(&path, bytes)?;
    if let Some(dir) = path.parent() {
        remember_save_dir(&app, dir);
    }
    Ok(true)
}

/// `name` with the extension of `format` (the overlay always sends `.png`).
fn suggested_file_name(name: &str, format: Format) -> String {
    Path::new(name)
        .with_extension(format.extension())
        .to_string_lossy()
        .into_owned()
}

/// Where the save dialog opens: the saved folder, else Pictures.
fn start_dir(app: &AppHandle, saved: Option<&Path>) -> Option<PathBuf> {
    saved
        .map(Path::to_path_buf)
        .or_else(|| app.path().picture_dir().ok())
}

/// Remember `dir` for the next save dialog. Best effort: a failure only
/// means the next dialog opens in the previous folder.
fn remember_save_dir(app: &AppHandle, dir: &Path) {
    let store = app.state::<SettingsStore>();
    if let Err(e) = store.update(|s| s.save_dir = Some(dir.to_path_buf())) {
        eprintln!("[reshot] could not remember save folder: {e}");
    }
}

/// What the Settings window shows.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    shortcut: String,
    save_dir: Option<String>,
    format: Format,
    autostart: bool,
}

/// Current settings, with open-at-login read from the OS.
#[tauri::command]
pub fn get_settings(app: AppHandle) -> SettingsView {
    let settings = app.state::<SettingsStore>().get();
    let save_dir =
        start_dir(&app, settings.existing_save_dir()).map(|d| d.to_string_lossy().into_owned());
    SettingsView {
        shortcut: settings.shortcut,
        save_dir,
        format: settings.format,
        autostart: app.autolaunch().is_enabled().unwrap_or(false),
    }
}

/// Use `accelerator` as the capture hotkey from now on.
#[tauri::command]
pub async fn set_shortcut(app: AppHandle, accelerator: String) -> Result<()> {
    shortcut::change(&app, &accelerator)
}

/// Release the hotkey while the Settings window records a new one.
#[tauri::command]
pub async fn suspend_shortcut(app: AppHandle) {
    shortcut::suspend(&app);
}

/// Re-register the saved hotkey (recording cancelled).
#[tauri::command]
pub async fn resume_shortcut(app: AppHandle) -> Result<()> {
    shortcut::resume(&app)
}

/// Pick the save folder. Returns the new folder, or `None` if cancelled.
#[tauri::command]
pub async fn choose_save_dir(app: AppHandle) -> Result<Option<String>> {
    let current = app.state::<SettingsStore>().get();
    let mut dialog = app.dialog().file();
    if let Some(dir) = start_dir(&app, current.existing_save_dir()) {
        dialog = dialog.set_directory(dir);
    }
    // Async command → runs on the async runtime, so blocking here is fine.
    let Some(picked) = dialog.blocking_pick_folder() else {
        return Ok(None);
    };
    let dir = picked
        .into_path()
        .map_err(|_| Error::InvalidRequest("unsupported folder"))?;
    app.state::<SettingsStore>()
        .update(|s| s.save_dir = Some(dir.clone()))?;
    Ok(Some(dir.to_string_lossy().into_owned()))
}

/// Set the format the save dialog suggests.
#[tauri::command]
pub fn set_format(app: AppHandle, format: Format) -> Result<()> {
    app.state::<SettingsStore>().update(|s| s.format = format)
}

/// Turn open-at-login on or off. Returns the state the OS reports afterwards.
#[tauri::command]
pub async fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool> {
    let autolaunch = app.autolaunch();
    let changed = if enabled {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };
    changed.map_err(|e| Error::Autostart(e.to_string()))?;
    autolaunch
        .is_enabled()
        .map_err(|e| Error::Autostart(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(width: u32, height: u32) -> Vec<u8> {
        [10u8, 20, 30, 255].repeat((width * height) as usize)
    }

    #[test]
    fn validate_accepts_matching_size() {
        let px = solid(3, 2);
        let img = validate(3, 2, &px).unwrap();
        assert_eq!((img.width, img.height), (3, 2));
    }

    #[test]
    fn validate_rejects_mismatch_and_zero() {
        let px = solid(3, 2);
        assert!(validate(3, 3, &px).is_err());
        assert!(validate(0, 2, &[]).is_err());
    }

    #[test]
    fn suggested_name_follows_format() {
        let name = "Reshot 2026-11-17 at 10.00.00.png";
        assert_eq!(
            suggested_file_name(name, Format::Jpeg),
            "Reshot 2026-11-17 at 10.00.00.jpg"
        );
        assert_eq!(suggested_file_name(name, Format::Png), name);
        assert_eq!(suggested_file_name("Reshot", Format::Png), "Reshot.png");
    }

    #[test]
    fn detects_login_launch() {
        let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(launched_at_login(args(&["reshot", "--autostart"])));
        assert!(!launched_at_login(args(&["reshot"])));
        // The program path itself is never the flag.
        assert!(!launched_at_login(args(&["--autostart"])));
    }

    #[test]
    fn encodes_png_and_jpeg_with_correct_dimensions() {
        let px = solid(4, 3);
        let img = validate(4, 3, &px).unwrap();
        for format in [Format::Png, Format::Jpeg] {
            let bytes = encode(&img, format).unwrap();
            let decoded = image::load_from_memory(&bytes).unwrap();
            assert_eq!((decoded.width(), decoded.height()), (4, 3));
        }
    }
}
