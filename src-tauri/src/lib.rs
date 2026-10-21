//! KeepAwake: keep the computer from sleeping, from a small window or the tray.

mod awake;
mod i18n;
mod settings;
mod tray;

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use awake::{Awake, Mode};
use i18n::Language;
use settings::{Settings, DURATIONS};

/// Passed by the login item so a launch at login starts in the tray.
const HIDDEN_ARG: &str = "--hidden";
const EVENT_STATE: &str = "state";
const EVENT_SHOW_SETTINGS: &str = "show-settings";

#[derive(Default)]
struct Session {
    active: bool,
    ends_at_ms: Option<u64>,
    error: Option<String>,
}

impl Session {
    /// A new session: the timer, if any, starts counting from `now`.
    fn started(result: Result<(), String>, now: u64, duration_min: u32) -> Self {
        Self::answered(result, ends_at(now, duration_min))
    }

    /// The same session with a new OS request, e.g. after a mode change.
    /// The countdown carries on instead of starting over.
    fn renewed(&self, result: Result<(), String>) -> Self {
        Self::answered(result, self.ends_at_ms)
    }

    /// The session once the OS has answered a keep-awake request.
    fn answered(result: Result<(), String>, ends_at_ms: Option<u64>) -> Self {
        match result {
            Ok(()) => Session {
                active: true,
                ends_at_ms,
                error: None,
            },
            Err(error) => {
                eprintln!("keep awake failed: {error}");
                Session {
                    active: false,
                    ends_at_ms: None,
                    error: Some(error),
                }
            }
        }
    }
}

struct AppState {
    awake: Awake,
    /// Serialises start/stop so the OS request and `session` never disagree.
    op: Mutex<()>,
    session: Mutex<Session>,
    settings: Mutex<Settings>,
    settings_path: PathBuf,
    autostart: Mutex<bool>,
    tray: Mutex<Option<tray::Tray>>,
}

/// Everything the window and tray need to draw themselves.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub active: bool,
    pub ends_at_ms: Option<u64>,
    pub error: Option<String>,
    pub mode: Mode,
    pub duration_min: u32,
    pub language: Language,
    pub close_to_tray: bool,
    pub autostart: bool,
}

impl Snapshot {
    pub fn remaining_ms(&self) -> Option<u64> {
        self.ends_at_ms.map(|end| end.saturating_sub(now_ms()))
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Wall-clock time, so a timer still ends on schedule after the machine sleeps.
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// When a timer of `duration_min` started at `now` runs out. Zero means no limit.
fn ends_at(now: u64, duration_min: u32) -> Option<u64> {
    (duration_min > 0).then(|| now + u64::from(duration_min) * 60_000)
}

fn snapshot(app: &AppHandle) -> Snapshot {
    let state = app.state::<AppState>();
    let settings = lock(&state.settings).clone();
    let autostart = *lock(&state.autostart);
    let session = lock(&state.session);
    let snap = Snapshot {
        active: session.active,
        ends_at_ms: session.ends_at_ms,
        error: session.error.clone(),
        mode: settings.mode,
        duration_min: settings.duration_min,
        language: settings.language,
        close_to_tray: settings.close_to_tray,
        autostart,
    };
    drop(session);
    snap
}

/// Send the current state to the window and the tray.
fn publish(app: &AppHandle) -> Snapshot {
    let snap = snapshot(app);
    let _ = app.emit(EVENT_STATE, &snap);
    refresh_tray(app, snap.clone());
    snap
}

fn refresh_tray(app: &AppHandle, snap: Snapshot) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let state = handle.state::<AppState>();
        let mut tray = lock(&state.tray);
        if let Some(tray) = tray.as_mut() {
            tray.sync(&handle, &snap);
        }
    });
}

fn is_active(app: &AppHandle) -> bool {
    lock(&app.state::<AppState>().session).active
}

/// Ask the OS to keep the machine awake with the current mode and timer.
/// The session only reports "active" once the OS has accepted the request.
fn start(app: &AppHandle) {
    let state = app.state::<AppState>();
    let _op = lock(&state.op);
    let (mode, duration) = {
        let settings = lock(&state.settings);
        (settings.mode, settings.duration_min)
    };
    let result = state.awake.acquire(mode);
    *lock(&state.session) = Session::started(result, now_ms(), duration);
}

/// Hand the OS a request for the current mode, keeping the running timer.
/// Does nothing when keep-awake is off.
fn renew(app: &AppHandle) {
    let state = app.state::<AppState>();
    let _op = lock(&state.op);
    if !lock(&state.session).active {
        return;
    }
    let mode = lock(&state.settings).mode;
    let result = state.awake.acquire(mode);
    let mut session = lock(&state.session);
    *session = session.renewed(result);
}

fn stop(app: &AppHandle) {
    let state = app.state::<AppState>();
    let _op = lock(&state.op);
    state.awake.release();
    *lock(&state.session) = Session::default();
}

fn update_settings(app: &AppHandle, change: impl FnOnce(&mut Settings)) {
    let state = app.state::<AppState>();
    let mut settings = lock(&state.settings);
    change(&mut settings);
    if let Err(e) = settings::save(&state.settings_path, &settings) {
        eprintln!("could not save settings: {e}");
    }
}

fn select_duration(app: &AppHandle, minutes: u32) -> Result<(), String> {
    if !DURATIONS.contains(&minutes) {
        return Err(format!("unsupported duration: {minutes}"));
    }
    update_settings(app, |s| s.duration_min = minutes);
    let state = app.state::<AppState>();
    let mut session = lock(&state.session);
    if session.active {
        session.ends_at_ms = ends_at(now_ms(), minutes);
    }
    Ok(())
}

fn show_window(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        let _ = app.show();
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    // A hidden tray app should not keep a Dock icon.
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
}

fn quit(app: &AppHandle) {
    stop(app);
    app.exit(0);
}

/// Run tray actions off the main thread so a slow OS call never freezes the menu bar.
fn in_background(app: &AppHandle, job: impl FnOnce(&AppHandle) + Send + 'static) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || job(&app));
}

pub(crate) fn toggle_in_background(app: &AppHandle) {
    in_background(app, |app| {
        if is_active(app) {
            stop(app);
        } else {
            start(app);
        }
        publish(app);
    });
}

pub(crate) fn on_menu(app: &AppHandle, id: &str) {
    match id {
        tray::MENU_TOGGLE => toggle_in_background(app),
        tray::MENU_OPEN => show_window(app),
        tray::MENU_SETTINGS => {
            show_window(app);
            let _ = app.emit(EVENT_SHOW_SETTINGS, ());
        }
        tray::MENU_QUIT => quit(app),
        other => {
            let Some(minutes) = other
                .strip_prefix(tray::MENU_DURATION_PREFIX)
                .and_then(|m| m.parse::<u32>().ok())
            else {
                return;
            };
            // Picking a time from the tray also switches keep-awake on.
            in_background(app, move |app| {
                if select_duration(app, minutes).is_ok() && !is_active(app) {
                    start(app);
                }
                publish(app);
            });
        }
    }
}

fn spawn_timer(app: AppHandle) {
    std::thread::Builder::new()
        .name("timer".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            let snap = snapshot(&app);
            match snap.ends_at_ms {
                Some(end) if snap.active && now_ms() >= end => {
                    stop(&app);
                    publish(&app);
                }
                // Keep the countdown in the tray menu ticking.
                Some(_) if snap.active => refresh_tray(&app, snap),
                _ => {}
            }
        })
        .expect("failed to start the timer thread");
}

// ---- commands called from the window ----

#[tauri::command]
async fn get_state(app: AppHandle) -> Snapshot {
    snapshot(&app)
}

#[tauri::command]
async fn set_active(app: AppHandle, active: bool) -> Snapshot {
    if active {
        start(&app);
    } else {
        stop(&app);
    }
    publish(&app)
}

#[tauri::command]
async fn set_duration(app: AppHandle, minutes: u32) -> Result<Snapshot, String> {
    select_duration(&app, minutes)?;
    Ok(publish(&app))
}

#[tauri::command]
async fn set_mode(app: AppHandle, mode: Mode) -> Snapshot {
    update_settings(&app, |s| s.mode = mode);
    renew(&app);
    publish(&app)
}

#[tauri::command]
async fn set_language(app: AppHandle, language: Language) -> Snapshot {
    update_settings(&app, |s| s.language = language);
    publish(&app)
}

#[tauri::command]
async fn set_close_to_tray(app: AppHandle, enabled: bool) -> Snapshot {
    update_settings(&app, |s| s.close_to_tray = enabled);
    publish(&app)
}

#[tauri::command]
async fn set_autostart(app: AppHandle, enabled: bool) -> Result<Snapshot, String> {
    let launcher = app.autolaunch();
    if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    }
    .map_err(|e| e.to_string())?;
    *lock(&app.state::<AppState>().autostart) = launcher.is_enabled().unwrap_or(enabled);
    Ok(publish(&app))
}

#[tauri::command]
fn hide_to_tray(app: AppHandle) {
    hide_window(&app);
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    quit(&app);
}

pub fn run() {
    tauri::Builder::default()
        // Must be registered first: a second launch just shows the running window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_window(app)
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![HIDDEN_ARG]),
        ))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let settings_path = app.path().app_config_dir()?.join("settings.json");
            let settings = settings::load(&settings_path);
            let autostart = app.autolaunch().is_enabled().unwrap_or(false);
            app.manage(AppState {
                awake: Awake::spawn(),
                op: Mutex::new(()),
                session: Mutex::new(Session::default()),
                settings: Mutex::new(settings),
                settings_path,
                autostart: Mutex::new(autostart),
                tray: Mutex::new(None),
            });

            let handle = app.handle().clone();
            let tray = tray::Tray::new(&handle, &snapshot(&handle))?;
            *lock(&app.state::<AppState>().tray) = Some(tray);
            spawn_timer(handle.clone());

            if std::env::args().any(|arg| arg == HIDDEN_ARG) {
                hide_window(&handle);
            } else {
                show_window(&handle);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let close_to_tray = lock(&window.state::<AppState>().settings).close_to_tray;
                if close_to_tray {
                    api.prevent_close();
                    hide_window(window.app_handle());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            set_active,
            set_duration,
            set_mode,
            set_language,
            set_close_to_tray,
            set_autostart,
            hide_to_tray,
            quit_app,
        ])
        .build(tauri::generate_context!())
        .expect("error while building KeepAwake")
        .run(|app, event| match event {
            // Hand the power request back before the process goes away.
            RunEvent::Exit => app.state::<AppState>().awake.release(),
            #[cfg(target_os = "macos")]
            RunEvent::Reopen {
                has_visible_windows: false,
                ..
            } => show_window(app),
            _ => {}
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_000_000;

    #[test]
    fn timer_ends_after_the_chosen_minutes() {
        assert_eq!(ends_at(NOW, 15), Some(NOW + 15 * 60_000));
        assert_eq!(ends_at(NOW, 0), None, "zero means no limit");
    }

    #[test]
    fn starting_begins_a_new_countdown() {
        let session = Session::started(Ok(()), NOW, 60);
        assert!(session.active);
        assert_eq!(session.ends_at_ms, Some(NOW + 60 * 60_000));
        assert_eq!(session.error, None);
    }

    #[test]
    fn renewing_keeps_the_countdown() {
        let session = Session::started(Ok(()), NOW, 30);
        let renewed = session.renewed(Ok(()));
        assert!(renewed.active);
        assert_eq!(renewed.ends_at_ms, session.ends_at_ms);

        let unlimited = Session::started(Ok(()), NOW, 0).renewed(Ok(()));
        assert!(unlimited.active);
        assert_eq!(unlimited.ends_at_ms, None);
    }

    #[test]
    fn a_refused_request_switches_off() {
        let session = Session::started(Ok(()), NOW, 30);
        let refused = session.renewed(Err("denied".into()));
        assert!(!refused.active);
        assert_eq!(refused.ends_at_ms, None);
        assert_eq!(refused.error.as_deref(), Some("denied"));

        let refused = Session::started(Err("denied".into()), NOW, 30);
        assert!(!refused.active);
        assert_eq!(refused.ends_at_ms, None);
    }
}
