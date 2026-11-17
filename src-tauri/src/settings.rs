//! User settings, persisted as `settings.json` in the app config dir.
//!
//! A missing or unreadable file is never an error: Reshot falls back to the
//! defaults, so a corrupt file can't stop the app from starting.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::Shortcut;

use crate::error::{Error, Result};
use crate::shortcut;

/// File name inside the app config dir.
const FILE_NAME: &str = "settings.json";
/// v0.1 stored only the last save folder, in this plain-text file.
const LEGACY_SAVE_DIR_FILE: &str = "last-save-dir";

/// Image format used for saving.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// Lossless PNG (default).
    #[default]
    Png,
    /// JPEG, quality 92.
    Jpeg,
}

impl Format {
    /// Pick the format from a file extension (`.jpg` / `.jpeg` → JPEG, else PNG).
    pub fn from_path(path: &Path) -> Self {
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

    /// Preferred file extension.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }
}

/// Everything the user can change in the Settings window. Open-at-login is
/// not stored here: the OS login item is the source of truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// Global capture hotkey, as a Tauri accelerator string.
    pub shortcut: String,
    /// Folder the save dialog opens in. Updated on every save, so it is
    /// also "the folder you used last".
    pub save_dir: Option<PathBuf>,
    /// Format the save dialog suggests.
    pub format: Format,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: shortcut::DEFAULT.to_owned(),
            save_dir: None,
            format: Format::Png,
        }
    }
}

impl Settings {
    /// Parse `settings.json`. Unknown fields are ignored, missing ones take
    /// their default, and anything unreadable gives the defaults.
    fn from_json(json: &str) -> Self {
        let mut settings: Self = serde_json::from_str(json).unwrap_or_default();
        if parse_shortcut(&settings.shortcut).is_err() {
            settings.shortcut = shortcut::DEFAULT.to_owned();
        }
        settings
    }

    /// Defaults, carrying over the v0.1 `last-save-dir` file if there was one.
    fn migrated(legacy_save_dir: Option<&str>) -> Self {
        let save_dir = legacy_save_dir
            .map(|s| s.trim_end_matches(['\r', '\n']))
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        Self {
            save_dir,
            ..Self::default()
        }
    }

    /// The save folder, if it still exists.
    pub fn existing_save_dir(&self) -> Option<&Path> {
        self.save_dir.as_deref().filter(|d| d.is_dir())
    }
}

/// Parse an accelerator string such as `CommandOrControl+Shift+9`.
pub fn parse_shortcut(accelerator: &str) -> Result<Shortcut> {
    accelerator
        .parse()
        .map_err(|_| Error::InvalidRequest("unknown key combination"))
}

/// The settings in memory, plus where they are saved.
pub struct SettingsStore {
    file: Option<PathBuf>,
    current: Mutex<Settings>,
}

impl SettingsStore {
    /// Read the settings file (migrating v0.1's file on first run).
    pub fn load(app: &AppHandle) -> Self {
        let dir = app.path().app_config_dir().ok();
        let file = dir.as_ref().map(|d| d.join(FILE_NAME));
        let settings = match file.as_ref().map(std::fs::read_to_string) {
            Some(Ok(json)) => Settings::from_json(&json),
            _ => {
                let legacy = dir.as_ref().map(|d| d.join(LEGACY_SAVE_DIR_FILE));
                let legacy_dir = legacy
                    .as_ref()
                    .and_then(|f| std::fs::read_to_string(f).ok());
                let settings = Settings::migrated(legacy_dir.as_deref());
                if legacy_dir.is_some()
                    && write(file.as_deref(), &settings).is_ok()
                    && let Some(legacy) = legacy
                {
                    let _ = std::fs::remove_file(legacy);
                }
                settings
            }
        };
        Self {
            file,
            current: Mutex::new(settings),
        }
    }

    /// A copy of the current settings (never hold the lock across an await).
    pub fn get(&self) -> Settings {
        self.current.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Change the settings and write them to disk.
    pub fn update(&self, change: impl FnOnce(&mut Settings)) -> Result<()> {
        let snapshot = {
            let mut current = self
                .current
                .lock()
                .map_err(|_| Error::InvalidRequest("settings lock poisoned"))?;
            change(&mut current);
            current.clone()
        };
        write(self.file.as_deref(), &snapshot)
    }
}

fn write(file: Option<&Path>, settings: &Settings) -> Result<()> {
    let Some(file) = file else {
        return Err(Error::InvalidRequest("no config folder"));
    };
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|_| Error::InvalidRequest("could not serialise settings"))?;
    std::fs::write(file, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let s = Settings::default();
        assert_eq!(s.shortcut, "CommandOrControl+Shift+9");
        assert_eq!(s.save_dir, None);
        assert_eq!(s.format, Format::Png);
    }

    #[test]
    fn json_round_trip() {
        let s = Settings {
            shortcut: "Alt+Shift+KeyS".into(),
            save_dir: Some(PathBuf::from("/tmp/shots")),
            format: Format::Jpeg,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"format\":\"jpeg\""));
        assert!(json.contains("\"saveDir\""));
        assert_eq!(Settings::from_json(&json), s);
    }

    #[test]
    fn partial_unknown_and_corrupt_json_fall_back_to_defaults() {
        let partial = Settings::from_json(r#"{"format":"jpeg","future":1}"#);
        assert_eq!(partial.format, Format::Jpeg);
        assert_eq!(partial.shortcut, shortcut::DEFAULT);
        assert_eq!(Settings::from_json("{not json"), Settings::default());
        assert_eq!(Settings::from_json(""), Settings::default());
    }

    #[test]
    fn invalid_shortcut_falls_back_to_default() {
        let s = Settings::from_json(r#"{"shortcut":"Shift+Banana"}"#);
        assert_eq!(s.shortcut, shortcut::DEFAULT);
    }

    #[test]
    fn migrates_legacy_save_dir() {
        let s = Settings::migrated(Some("/Users/me/Shots\n"));
        assert_eq!(s.save_dir, Some(PathBuf::from("/Users/me/Shots")));
        assert_eq!(Settings::migrated(Some("\n")).save_dir, None);
        assert_eq!(Settings::migrated(None), Settings::default());
    }

    #[test]
    fn format_from_extension() {
        assert_eq!(Format::from_path(Path::new("a.PNG")), Format::Png);
        assert_eq!(Format::from_path(Path::new("a.jpg")), Format::Jpeg);
        assert_eq!(Format::from_path(Path::new("a.JPEG")), Format::Jpeg);
        assert_eq!(Format::from_path(Path::new("noext")), Format::Png);
    }

    /// Strings the Settings window's recorder emits (`KeyboardEvent.code`
    /// plus modifier names) must parse.
    #[test]
    fn recorder_output_parses() {
        for s in [
            "CommandOrControl+Shift+9",
            "Command+Shift+Digit9",
            "Control+Alt+KeyS",
            "Super+Shift+Digit4",
            "Control+Shift+F5",
            "Alt+Minus",
            "Control+Space",
            "Shift+ArrowUp",
            "PrintScreen",
            "F12",
        ] {
            assert!(parse_shortcut(s).is_ok(), "{s} should parse");
        }
        assert!(parse_shortcut("Shift+Banana").is_err());
        assert!(parse_shortcut("").is_err());
    }

    #[test]
    fn equal_shortcuts_compare_equal() {
        // Same combo, different spelling: no re-registration needed.
        assert_eq!(
            parse_shortcut("Super+Shift+Digit9").unwrap(),
            parse_shortcut("Command+Shift+9").unwrap()
        );
    }
}
