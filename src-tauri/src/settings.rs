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
/// Longest custom timer: one day.
pub const MAX_CUSTOM_MIN: u32 = 24 * 60;

/// A time the user typed in instead of picking a preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Custom {
    /// Run for this many minutes.
    Minutes { minutes: u32 },
    /// Run until the clock next shows this time, today or tomorrow.
    Until { hour: u32, minute: u32 },
}

impl Custom {
    pub fn is_valid(self) -> bool {
        match self {
            Custom::Minutes { minutes } => (1..=MAX_CUSTOM_MIN).contains(&minutes),
            Custom::Until { hour, minute } => hour < 24 && minute < 60,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub language: Language,
    pub close_to_tray: bool,
    pub duration_min: u32,
    /// The last custom time, kept so the picker can offer it again.
    pub custom: Option<Custom>,
    /// The timer uses `custom` instead of `duration_min`.
    pub use_custom: bool,
    pub mode: Mode,
    pub notify_when_done: bool,
    pub shortcut: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::En,
            close_to_tray: true,
            duration_min: DEFAULT_DURATION,
            custom: None,
            use_custom: false,
            mode: Mode::Display,
            notify_when_done: true,
            shortcut: true,
        }
    }
}

impl Settings {
    fn sanitized(mut self) -> Self {
        if !DURATIONS.contains(&self.duration_min) {
            self.duration_min = DEFAULT_DURATION;
        }
        if !self.custom.is_some_and(Custom::is_valid) {
            self.custom = None;
            self.use_custom = false;
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
            custom: Some(Custom::Until {
                hour: 18,
                minute: 30,
            }),
            use_custom: true,
            mode: Mode::System,
            notify_when_done: false,
            shortcut: false,
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
        assert!(
            loaded.notify_when_done,
            "1.0 settings files get the new defaults"
        );
        assert!(loaded.shortcut);

        for custom in [
            r#"{"kind": "minutes", "minutes": 0}"#,
            r#"{"kind": "minutes", "minutes": 1441}"#,
            r#"{"kind": "until", "hour": 24, "minute": 0}"#,
            r#"{"kind": "until", "hour": 9, "minute": 60}"#,
        ] {
            fs::write(
                &path,
                format!(r#"{{"useCustom": true, "custom": {custom}}}"#),
            )
            .unwrap();
            let loaded = load(&path);
            assert_eq!(loaded.custom, None, "{custom}");
            assert!(!loaded.use_custom, "{custom}");
        }

        fs::write(&path, "not json").unwrap();
        assert_eq!(load(&path), Settings::default());
    }
}
