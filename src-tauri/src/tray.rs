//! Tray icon and its menu. Only touched from the main thread.

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Wry};

use crate::i18n::{self, Language, Text};
use crate::settings::DURATIONS;
use crate::Snapshot;

pub const MENU_TOGGLE: &str = "toggle";
pub const MENU_OPEN: &str = "open";
pub const MENU_SETTINGS: &str = "settings";
pub const MENU_QUIT: &str = "quit";
pub const MENU_DURATION_PREFIX: &str = "duration:";

// macOS draws monochrome "template" icons that follow the menu bar colour.
#[cfg(target_os = "macos")]
const ICON_ON: &[u8] = include_bytes!("../icons/tray/on-template.png");
#[cfg(target_os = "macos")]
const ICON_OFF: &[u8] = include_bytes!("../icons/tray/off-template.png");
#[cfg(not(target_os = "macos"))]
const ICON_ON: &[u8] = include_bytes!("../icons/tray/on.png");
#[cfg(not(target_os = "macos"))]
const ICON_OFF: &[u8] = include_bytes!("../icons/tray/off.png");
const IS_TEMPLATE: bool = cfg!(target_os = "macos");

struct Items {
    menu: Menu<Wry>,
    status: MenuItem<Wry>,
    toggle: CheckMenuItem<Wry>,
    durations: Vec<(u32, CheckMenuItem<Wry>)>,
}

pub struct Tray {
    icon: TrayIcon<Wry>,
    items: Items,
    language: Language,
    shown_active: Option<bool>,
}

fn icon(active: bool) -> tauri::Result<Image<'static>> {
    Image::from_bytes(if active { ICON_ON } else { ICON_OFF })
}

fn build_items(app: &AppHandle, lang: Language) -> tauri::Result<Items> {
    let status = MenuItem::with_id(app, "status", "KeepAwake", false, None::<&str>)?;
    let toggle = CheckMenuItem::with_id(
        app,
        MENU_TOGGLE,
        i18n::text(lang, Text::KeepAwake),
        true,
        false,
        None::<&str>,
    )?;
    let durations = DURATIONS
        .iter()
        .map(|&minutes| {
            CheckMenuItem::with_id(
                app,
                format!("{MENU_DURATION_PREFIX}{minutes}"),
                i18n::duration(lang, minutes),
                true,
                false,
                None::<&str>,
            )
            .map(|item| (minutes, item))
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let open = MenuItem::with_id(
        app,
        MENU_OPEN,
        i18n::text(lang, Text::Open),
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(
        app,
        MENU_SETTINGS,
        i18n::text(lang, Text::Settings),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(
        app,
        MENU_QUIT,
        i18n::text(lang, Text::Quit),
        true,
        None::<&str>,
    )?;

    let menu = Menu::with_items(
        app,
        &[&status, &PredefinedMenuItem::separator(app)?, &toggle],
    )?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    for (_, item) in &durations {
        menu.append(item)?;
    }
    menu.append_items(&[
        &PredefinedMenuItem::separator(app)?,
        &open,
        &settings,
        &PredefinedMenuItem::separator(app)?,
        &quit,
    ])?;

    Ok(Items {
        menu,
        status,
        toggle,
        durations,
    })
}

impl Tray {
    pub fn new(app: &AppHandle, snap: &Snapshot) -> tauri::Result<Self> {
        let items = build_items(app, snap.language)?;
        let icon = TrayIconBuilder::with_id("main")
            .icon(icon(false)?)
            .icon_as_template(IS_TEMPLATE)
            .tooltip("KeepAwake")
            .menu(&items.menu)
            // Left click toggles on macOS and Windows. Linux always opens the menu.
            .show_menu_on_left_click(false)
            .on_menu_event(|app, event| crate::on_menu(app, event.id().as_ref()))
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    crate::toggle_in_background(tray.app_handle());
                }
            })
            .build(app)?;

        let mut tray = Self {
            icon,
            items,
            language: snap.language,
            shown_active: None,
        };
        tray.sync(app, snap);
        Ok(tray)
    }

    /// Bring the icon and menu in line with the current state.
    pub fn sync(&mut self, app: &AppHandle, snap: &Snapshot) {
        if snap.language != self.language {
            match build_items(app, snap.language) {
                Ok(items) => {
                    let _ = self.icon.set_menu(Some(items.menu.clone()));
                    self.items = items;
                    self.language = snap.language;
                }
                Err(e) => eprintln!("could not rebuild the tray menu: {e}"),
            }
        }

        let status = i18n::status(
            snap.language,
            snap.active,
            snap.remaining_ms(),
            snap.error.is_some(),
        );
        let _ = self.items.status.set_text(&status);
        let _ = self.icon.set_tooltip(Some(&status));
        let _ = self.items.toggle.set_checked(snap.active);
        for (minutes, item) in &self.items.durations {
            let _ = item.set_checked(*minutes == snap.duration_min);
        }

        if self.shown_active != Some(snap.active) {
            if let Ok(image) = icon(snap.active) {
                let _ = self
                    .icon
                    .set_icon_with_as_template(Some(image), IS_TEMPLATE);
                self.shown_active = Some(snap.active);
            }
        }
    }
}
