//! Strings shown by the tray menu. The window has its own copy in `src/i18n.js`.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Vi,
}

pub enum Text {
    KeepAwake,
    Open,
    Settings,
    Quit,
}

pub fn text(lang: Language, key: Text) -> &'static str {
    use Language::*;
    use Text::*;
    match (lang, key) {
        (En, KeepAwake) => "Keep awake",
        (Vi, KeepAwake) => "Giữ thức",
        (En, Open) => "Open KeepAwake",
        (Vi, Open) => "Mở cửa sổ",
        (En, Settings) => "Settings…",
        (Vi, Settings) => "Cài đặt…",
        (En, Quit) => "Quit",
        (Vi, Quit) => "Thoát",
    }
}

pub fn duration(lang: Language, minutes: u32) -> String {
    match (lang, minutes) {
        (Language::En, 0) => "Indefinitely".into(),
        (Language::Vi, 0) => "Vô thời hạn".into(),
        (Language::En, 60) => "1 hour".into(),
        (Language::En, m) if m % 60 == 0 => format!("{} hours", m / 60),
        (Language::En, m) => format!("{m} minutes"),
        (Language::Vi, m) if m % 60 == 0 => format!("{} giờ", m / 60),
        (Language::Vi, m) => format!("{m} phút"),
    }
}

/// Status line at the top of the tray menu, also used as the tooltip.
pub fn status(lang: Language, active: bool, remaining_ms: Option<u64>, failed: bool) -> String {
    let state = match (lang, active, remaining_ms) {
        (Language::En, true, Some(ms)) => format!("On · {} left", clock(ms)),
        (Language::Vi, true, Some(ms)) => format!("Đang bật · còn {}", clock(ms)),
        (Language::En, true, None) => "On · no time limit".into(),
        (Language::Vi, true, None) => "Đang bật · không giới hạn".into(),
        (Language::En, false, _) if failed => "Couldn't keep awake".into(),
        (Language::Vi, false, _) if failed => "Không giữ thức được".into(),
        (Language::En, false, _) => "Off".into(),
        (Language::Vi, false, _) => "Đang tắt".into(),
    };
    format!("KeepAwake — {state}")
}

/// `1:29:58` above an hour, `29:58` below.
pub fn clock(ms: u64) -> String {
    let total = ms.div_ceil(1000);
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_format() {
        assert_eq!(clock(0), "0:00");
        assert_eq!(clock(1), "0:01");
        assert_eq!(clock(59_000), "0:59");
        assert_eq!(clock(5_398_000), "1:29:58");
        assert_eq!(clock(3_600_000), "1:00:00");
    }

    #[test]
    fn duration_labels() {
        assert_eq!(duration(Language::En, 15), "15 minutes");
        assert_eq!(duration(Language::En, 60), "1 hour");
        assert_eq!(duration(Language::En, 240), "4 hours");
        assert_eq!(duration(Language::Vi, 120), "2 giờ");
        assert_eq!(duration(Language::Vi, 0), "Vô thời hạn");
    }

    #[test]
    fn status_line() {
        assert_eq!(
            status(Language::En, true, Some(90_000), false),
            "KeepAwake — On · 1:30 left"
        );
        assert_eq!(
            status(Language::Vi, false, None, false),
            "KeepAwake — Đang tắt"
        );
        assert_eq!(
            status(Language::En, false, None, true),
            "KeepAwake — Couldn't keep awake"
        );
    }
}
