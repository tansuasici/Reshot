//! IPC commands invoked by the overlay frontend.
//!
//! Pixel data travels as raw bytes in both directions (no JSON, no PNG on the
//! hot path): `get_frame` returns an `ipc::Response`, and `copy_image` /
//! `save_image` take a raw RGBA body with `x-width` / `x-height` headers.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::capture::FrameStore;
use crate::error::{Error, Result};
use crate::overlay;

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

/// Output file format, chosen from the file extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Png,
    Jpeg,
}

impl Format {
    fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("jpg" | "jpeg") => Self::Jpeg,
            _ => Self::Png,
        }
    }
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
#[tauri::command]
pub fn overlay_ready(app: AppHandle) {
    static LAUNCH_CAPTURE_DONE: AtomicBool = AtomicBool::new(false);
    if !LAUNCH_CAPTURE_DONE.swap(true, Ordering::SeqCst) {
        crate::capture::trigger(&app);
    }
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
    let file_name = request
        .headers()
        .get("x-filename")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Reshot.png")
        .to_owned();

    // The overlay sits above everything, including the save panel.
    if let Some(win) = app.get_webview_window(overlay::LABEL) {
        win.hide()?;
    }

    let mut dialog = app
        .dialog()
        .file()
        .set_file_name(file_name)
        .add_filter("PNG", &["png"])
        .add_filter("JPEG", &["jpg", "jpeg"]);
    if let Some(dir) = last_save_dir(&app).or_else(|| app.path().picture_dir().ok()) {
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

/// File in the app config dir holding the folder of the last save.
fn last_save_dir_file(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("last-save-dir"))
}

/// The folder the user last saved into, if it still exists.
fn last_save_dir(app: &AppHandle) -> Option<PathBuf> {
    let saved = std::fs::read_to_string(last_save_dir_file(app)?).ok()?;
    let dir = PathBuf::from(saved.trim_end_matches('\n'));
    dir.is_dir().then_some(dir)
}

/// Remember `dir` for the next save dialog. Best effort: a failure only
/// means the next dialog opens in the default folder.
fn remember_save_dir(app: &AppHandle, dir: &Path) {
    let Some(file) = last_save_dir_file(app) else {
        return;
    };
    let written = file
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(&file, dir.to_string_lossy().as_bytes()));
    if let Err(e) = written {
        eprintln!("[reshot] could not remember save folder: {e}");
    }
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
    fn format_from_extension() {
        assert_eq!(Format::from_path(Path::new("a.PNG")), Format::Png);
        assert_eq!(Format::from_path(Path::new("a.jpg")), Format::Jpeg);
        assert_eq!(Format::from_path(Path::new("a.JPEG")), Format::Jpeg);
        assert_eq!(Format::from_path(Path::new("noext")), Format::Png);
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
