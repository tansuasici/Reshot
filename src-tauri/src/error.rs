//! Crate-wide error type. Serialised as a plain string for the frontend.

use std::fmt;

/// Everything that can go wrong in Reshot's backend.
#[derive(Debug)]
pub enum Error {
    /// Screen capture failed (permissions, no monitor, backend error).
    Capture(String),
    /// The OS has not granted screen recording access (macOS).
    PermissionDenied,
    /// A command needed the captured frame, but none is held.
    NoFrame,
    /// The system clipboard rejected the image.
    Clipboard(String),
    /// PNG/JPEG encoding failed.
    Encode(String),
    /// The frontend sent a malformed request.
    InvalidRequest(&'static str),
    /// File system error while saving.
    Io(std::io::Error),
    /// Error from the Tauri runtime (window, events, dialogs).
    Tauri(tauri::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Capture(e) => write!(f, "screen capture failed: {e}"),
            Self::PermissionDenied => write!(f, "screen recording permission not granted"),
            Self::NoFrame => write!(f, "no captured frame available"),
            Self::Clipboard(e) => write!(f, "clipboard error: {e}"),
            Self::Encode(e) => write!(f, "image encoding failed: {e}"),
            Self::InvalidRequest(e) => write!(f, "invalid request: {e}"),
            Self::Io(e) => write!(f, "file error: {e}"),
            Self::Tauri(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<tauri::Error> for Error {
    fn from(e: tauri::Error) -> Self {
        Self::Tauri(e)
    }
}

impl From<xcap::XCapError> for Error {
    fn from(e: xcap::XCapError) -> Self {
        Self::Capture(e.to_string())
    }
}

impl serde::Serialize for Error {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// Result alias used across the crate.
pub type Result<T> = std::result::Result<T, Error>;
