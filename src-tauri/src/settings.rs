//! User preferences, stored as JSON in the app's config folder.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::awake::Mode;
use crate::i18n::Language;

/// Timer choices in minutes. `0` means no time limit.
pub const DURATIONS: [u32; 6] = [15, 30, 60, 120, 240, 0];
const DEFAULT_DURATION: u32 = 60;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub language: Language,
    pub close_to_tray: bool,
    pub duration_min: u32,
    pub mode: Mode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::En,
            close_to_tray: true,
            duration_min: DEFAULT_DURATION,
            mode: Mode::Display,
        }
    }
}

impl Settings {
    fn sanitized(mut self) -> Self {
        if !DURATIONS.contains(&self.duration_min) {
            self.duration_min = DEFAULT_DURATION;
        }
        self
    }
}

/// Reads settings. A missing or broken file falls back to defaults.
pub fn load(path: &Path) -> Settings {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Settings>(&text).ok())
        .unwrap_or_default()
        .sanitized()
}

pub fn save(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text)?;
    fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("keepawake-test-{}", std::process::id()));
        dir.join(name)
    }

    #[test]
    fn missing_file_gives_defaults() {
        assert_eq!(load(&temp_file("missing.json")), Settings::default());
    }

    #[test]
    fn round_trip() {
        let path = temp_file("round.json");
        let settings = Settings {
            language: Language::Vi,
            close_to_tray: false,
            duration_min: 0,
            mode: Mode::System,
        };
        save(&path, &settings).unwrap();
        assert_eq!(load(&path), settings);
    }

    #[test]
    fn bad_values_are_repaired() {
        let path = temp_file("bad.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"durationMin": 7, "language": "vi"}"#).unwrap();
        let loaded = load(&path);
        assert_eq!(loaded.duration_min, DEFAULT_DURATION);
        assert_eq!(loaded.language, Language::Vi);
        assert!(loaded.close_to_tray);

        fs::write(&path, "not json").unwrap();
        assert_eq!(load(&path), Settings::default());
    }
}
