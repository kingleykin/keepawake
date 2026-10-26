//! KeepAwake: keep the computer from sleeping, from a small window or the tray.

mod awake;
mod i18n;
mod settings;
mod tray;

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Local, NaiveTime, TimeDelta, TimeZone};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::{Update, UpdaterExt};

use awake::{Awake, Mode};
use i18n::{Language, Text};
use settings::{Custom, Settings, DURATIONS};

/// Passed by the login item so a launch at login starts in the tray.
const HIDDEN_ARG: &str = "--hidden";
const EVENT_STATE: &str = "state";
const EVENT_SHOW_SETTINGS: &str = "show-settings";
const EVENT_SHOW_CUSTOM: &str = "show-custom";
/// Wait this long after launch before looking for an update.
const UPDATE_CHECK_DELAY: Duration = Duration::from_secs(5);

/// A running timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Countdown {
    ends_at_ms: u64,
    /// How long the timer was when it was set, for the progress ring.
    total_ms: u64,
}

#[derive(Default)]
struct Session {
    active: bool,
    /// `None` means no time limit.
    countdown: Option<Countdown>,
    error: Option<String>,
}

impl Session {
    /// A new session with a freshly set timer.
    fn started(result: Result<(), String>, countdown: Option<Countdown>) -> Self {
        Self::answered(result, countdown)
    }

    /// The same session with a new OS request, e.g. after a mode change.
    /// The countdown carries on instead of starting over.
    fn renewed(&self, result: Result<(), String>) -> Self {
        Self::answered(result, self.countdown)
    }

    /// The session once the OS has answered a keep-awake request.
    fn answered(result: Result<(), String>, countdown: Option<Countdown>) -> Self {
        match result {
            Ok(()) => Session {
                active: true,
                countdown,
                error: None,
            },
            Err(error) => {
                eprintln!("keep awake failed: {error}");
                Session {
                    active: false,
                    countdown: None,
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
    /// Why the global shortcut could not be registered, e.g. another app holds it.
    shortcut_error: Mutex<Option<String>>,
    /// A newer release found by the last check. Only installed when the user asks.
    update: Mutex<Option<Update>>,
    tray: Mutex<Option<tray::Tray>>,
}

/// Everything the window and tray need to draw themselves.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub active: bool,
    pub ends_at_ms: Option<u64>,
    pub total_ms: Option<u64>,
    pub error: Option<String>,
    pub mode: Mode,
    pub duration_min: u32,
    pub custom: Option<Custom>,
    pub use_custom: bool,
    pub language: Language,
    pub close_to_tray: bool,
    pub autostart: bool,
    pub notify_when_done: bool,
    pub shortcut: bool,
    pub shortcut_error: Option<String>,
    /// Version of a newer release, if one was found.
    pub update: Option<String>,
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

/// The next moment after `now` when the clock shows `hour:minute`: today, or
/// tomorrow if that time has passed.
fn next_clock_time<Tz: TimeZone>(now: &DateTime<Tz>, hour: u32, minute: u32) -> u64 {
    let zone = now.timezone();
    let time = NaiveTime::from_hms_opt(hour, minute, 0).unwrap_or(NaiveTime::MIN);
    let today = now.date_naive();
    for date in today.iter_days().take(3) {
        let local = date.and_time(time);
        let at = zone
            .from_local_datetime(&local)
            .earliest()
            // The clock skips this time when daylight saving starts. Use the hour after.
            .or_else(|| {
                zone.from_local_datetime(&(local + TimeDelta::hours(1)))
                    .earliest()
            });
        if let Some(at) = at.filter(|at| at > now) {
            return at.timestamp_millis() as u64;
        }
    }
    // Not reachable with real time zones; a day from now is a safe answer.
    now.timestamp_millis() as u64 + 24 * 3_600_000
}

/// The timer the settings ask for, set at `now`.
fn countdown<Tz: TimeZone>(settings: &Settings, now: &DateTime<Tz>) -> Option<Countdown> {
    let now_ms = now.timestamp_millis() as u64;
    let ends_at_ms = match settings.custom {
        Some(Custom::Minutes { minutes }) if settings.use_custom => ends_at(now_ms, minutes)?,
        Some(Custom::Until { hour, minute }) if settings.use_custom => {
            next_clock_time(now, hour, minute)
        }
        _ => ends_at(now_ms, settings.duration_min)?,
    };
    Some(Countdown {
        ends_at_ms,
        total_ms: ends_at_ms.saturating_sub(now_ms),
    })
}

fn snapshot(app: &AppHandle) -> Snapshot {
    let state = app.state::<AppState>();
    let settings = lock(&state.settings).clone();
    let autostart = *lock(&state.autostart);
    let shortcut_error = lock(&state.shortcut_error).clone();
    let update = lock(&state.update).as_ref().map(|u| u.version.clone());
    let session = lock(&state.session);
    let snap = Snapshot {
        active: session.active,
        ends_at_ms: session.countdown.map(|c| c.ends_at_ms),
        total_ms: session.countdown.map(|c| c.total_ms),
        error: session.error.clone(),
        mode: settings.mode,
        duration_min: settings.duration_min,
        custom: settings.custom,
        use_custom: settings.use_custom,
        language: settings.language,
        close_to_tray: settings.close_to_tray,
        autostart,
        notify_when_done: settings.notify_when_done,
        shortcut: settings.shortcut,
        shortcut_error,
        update,
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
    let settings = lock(&state.settings).clone();
    let result = state.awake.acquire(settings.mode);
    *lock(&state.session) = Session::started(result, countdown(&settings, &Local::now()));
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

/// Change the timer. A running countdown starts over with the new time.
fn select_timer(app: &AppHandle, change: impl FnOnce(&mut Settings)) {
    let state = app.state::<AppState>();
    let _op = lock(&state.op);
    update_settings(app, change);
    let settings = lock(&state.settings).clone();
    let mut session = lock(&state.session);
    if session.active {
        session.countdown = countdown(&settings, &Local::now());
    }
}

fn select_duration(app: &AppHandle, minutes: u32) -> Result<(), String> {
    if !DURATIONS.contains(&minutes) {
        return Err(format!("unsupported duration: {minutes}"));
    }
    select_timer(app, |s| {
        s.duration_min = minutes;
        s.use_custom = false;
    });
    Ok(())
}

fn select_custom(app: &AppHandle, custom: Custom) -> Result<(), String> {
    if !custom.is_valid() {
        return Err(format!("unsupported custom time: {custom:?}"));
    }
    select_timer(app, |s| {
        s.custom = Some(custom);
        s.use_custom = true;
    });
    Ok(())
}

/// Tell the user that keep-awake switched itself off, if they want to know.
fn notify_time_up(app: &AppHandle) {
    let settings = lock(&app.state::<AppState>().settings).clone();
    if !settings.notify_when_done {
        return;
    }
    let shown = app
        .notification()
        .builder()
        .title("KeepAwake")
        .body(i18n::text(settings.language, Text::TimeUp))
        .show();
    if let Err(e) = shown {
        eprintln!("could not show the notification: {e}");
    }
}

/// Cmd+Option+K on macOS, Ctrl+Alt+K elsewhere.
fn toggle_shortcut() -> Shortcut {
    #[cfg(target_os = "macos")]
    let modifiers = Modifiers::SUPER | Modifiers::ALT;
    #[cfg(not(target_os = "macos"))]
    let modifiers = Modifiers::CONTROL | Modifiers::ALT;
    Shortcut::new(Some(modifiers), Code::KeyK)
}

/// Register or drop the global shortcut to match the setting. If another app
/// already holds the keys, the window shows why instead of failing.
/// Blocks on the main thread, so never call it from there or with a lock held.
fn apply_shortcut(app: &AppHandle) {
    let wanted = lock(&app.state::<AppState>().settings).shortcut;
    let shortcuts = app.global_shortcut();
    let shortcut = toggle_shortcut();
    let result = match (wanted, shortcuts.is_registered(shortcut)) {
        (true, false) => shortcuts.register(shortcut),
        (false, true) => shortcuts.unregister(shortcut),
        _ => Ok(()),
    };
    let error = result.err().map(|e| e.to_string());
    if let Some(e) = &error {
        eprintln!("could not set up the shortcut: {e}");
    }
    *lock(&app.state::<AppState>().shortcut_error) = error;
}

/// Look for a newer release. Never installs anything by itself.
async fn check_for_update(app: &AppHandle) -> Result<(), String> {
    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    *lock(&app.state::<AppState>().update) = update;
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
        tray::MENU_CUSTOM => {
            show_window(app);
            let _ = app.emit(EVENT_SHOW_CUSTOM, ());
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
                    notify_time_up(&app);
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
async fn set_custom(app: AppHandle, custom: Custom) -> Result<Snapshot, String> {
    select_custom(&app, custom)?;
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
async fn set_notify_when_done(app: AppHandle, enabled: bool) -> Snapshot {
    update_settings(&app, |s| s.notify_when_done = enabled);
    publish(&app)
}

#[tauri::command]
async fn set_shortcut(app: AppHandle, enabled: bool) -> Snapshot {
    update_settings(&app, |s| s.shortcut = enabled);
    apply_shortcut(&app);
    publish(&app)
}

#[tauri::command]
async fn check_update(app: AppHandle) -> Result<Snapshot, String> {
    check_for_update(&app).await?;
    Ok(publish(&app))
}

/// Download, verify and install the update found by the last check, then restart.
#[tauri::command]
async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = lock(&app.state::<AppState>().update)
        .clone()
        .ok_or("no update to install")?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;
    stop(&app);
    app.restart();
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
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        toggle_in_background(app);
                    }
                })
                .build(),
        )
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
                shortcut_error: Mutex::new(None),
                update: Mutex::new(None),
                tray: Mutex::new(None),
            });

            let handle = app.handle().clone();
            let tray = tray::Tray::new(&handle, &snapshot(&handle))?;
            *lock(&app.state::<AppState>().tray) = Some(tray);
            spawn_timer(handle.clone());
            in_background(&handle, |app| {
                apply_shortcut(app);
                publish(app);
            });
            let checker = handle.clone();
            tauri::async_runtime::spawn(async move {
                let _ =
                    tauri::async_runtime::spawn_blocking(|| std::thread::sleep(UPDATE_CHECK_DELAY))
                        .await;
                match check_for_update(&checker).await {
                    Ok(()) => {
                        publish(&checker);
                    }
                    Err(e) => eprintln!("could not check for updates: {e}"),
                }
            });

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
            set_custom,
            set_mode,
            set_language,
            set_close_to_tray,
            set_autostart,
            set_notify_when_done,
            set_shortcut,
            check_update,
            install_update,
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
    use chrono::FixedOffset;

    const NOW: u64 = 1_000_000;

    /// A fixed countdown of `minutes`, as a preset would set it at `NOW`.
    fn minutes(minutes: u32) -> Option<Countdown> {
        ends_at(NOW, minutes).map(|ends_at_ms| Countdown {
            ends_at_ms,
            total_ms: ends_at_ms - NOW,
        })
    }

    /// 2026-10-26 at `h:m:s` in UTC+7.
    fn at(h: u32, m: u32, s: u32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(7 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 26, h, m, s)
            .unwrap()
    }

    fn ms(time: DateTime<FixedOffset>) -> u64 {
        time.timestamp_millis() as u64
    }

    fn with_custom(custom: Custom) -> Settings {
        Settings {
            custom: Some(custom),
            use_custom: true,
            ..Settings::default()
        }
    }

    #[test]
    fn timer_ends_after_the_chosen_minutes() {
        assert_eq!(ends_at(NOW, 15), Some(NOW + 15 * 60_000));
        assert_eq!(ends_at(NOW, 0), None, "zero means no limit");
    }

    #[test]
    fn starting_begins_a_new_countdown() {
        let session = Session::started(Ok(()), minutes(60));
        assert!(session.active);
        assert_eq!(
            session.countdown,
            Some(Countdown {
                ends_at_ms: NOW + 60 * 60_000,
                total_ms: 60 * 60_000,
            })
        );
        assert_eq!(session.error, None);
    }

    #[test]
    fn renewing_keeps_the_countdown() {
        let session = Session::started(Ok(()), minutes(30));
        let renewed = session.renewed(Ok(()));
        assert!(renewed.active);
        assert_eq!(renewed.countdown, session.countdown);

        let unlimited = Session::started(Ok(()), minutes(0)).renewed(Ok(()));
        assert!(unlimited.active);
        assert_eq!(unlimited.countdown, None);
    }

    #[test]
    fn a_refused_request_switches_off() {
        let session = Session::started(Ok(()), minutes(30));
        let refused = session.renewed(Err("denied".into()));
        assert!(!refused.active);
        assert_eq!(refused.countdown, None);
        assert_eq!(refused.error.as_deref(), Some("denied"));

        let refused = Session::started(Err("denied".into()), minutes(30));
        assert!(!refused.active);
        assert_eq!(refused.countdown, None);
    }

    #[test]
    fn until_a_later_time_ends_today() {
        let now = at(10, 0, 0);
        assert_eq!(next_clock_time(&now, 18, 30), ms(at(18, 30, 0)));
        assert_eq!(next_clock_time(&at(18, 29, 30), 18, 30), ms(at(18, 30, 0)));
    }

    #[test]
    fn until_a_past_time_ends_tomorrow() {
        let tomorrow = ms(at(18, 30, 0)) + 24 * 3_600_000;
        assert_eq!(next_clock_time(&at(19, 0, 0), 18, 30), tomorrow);
        assert_eq!(
            next_clock_time(&at(18, 30, 0), 18, 30),
            tomorrow,
            "the time right now has passed"
        );
        assert_eq!(
            next_clock_time(&at(23, 59, 0), 0, 0),
            ms(at(23, 59, 0)) + 60_000,
            "midnight is tomorrow"
        );
    }

    #[test]
    fn until_handles_daylight_saving_changes() {
        use chrono_tz::Europe::Berlin;
        // 29 March 2026: Berlin clocks jump from 02:00 to 03:00, so 02:30 never happens.
        let night = Berlin.with_ymd_and_hms(2026, 3, 29, 1, 0, 0).unwrap();
        let end = next_clock_time(&night, 2, 30);
        let three_thirty = Berlin.with_ymd_and_hms(2026, 3, 29, 3, 30, 0).unwrap();
        assert_eq!(end, three_thirty.timestamp_millis() as u64);

        // 25 October 2026: 02:30 happens twice. The first one counts.
        let night = Berlin.with_ymd_and_hms(2026, 10, 25, 1, 0, 0).unwrap();
        let end = next_clock_time(&night, 2, 30);
        assert_eq!(end, night.timestamp_millis() as u64 + 90 * 60_000);
    }

    #[test]
    fn countdown_follows_the_chosen_timer() {
        let now = at(10, 0, 0);
        let preset = Settings {
            duration_min: 30,
            ..Settings::default()
        };
        assert_eq!(
            countdown(&preset, &now),
            Some(Countdown {
                ends_at_ms: ms(now) + 30 * 60_000,
                total_ms: 30 * 60_000,
            })
        );

        let no_limit = Settings {
            duration_min: 0,
            ..Settings::default()
        };
        assert_eq!(countdown(&no_limit, &now), None);

        let custom = with_custom(Custom::Minutes { minutes: 45 });
        assert_eq!(
            countdown(&custom, &now).map(|c| c.total_ms),
            Some(45 * 60_000)
        );

        let until = with_custom(Custom::Until {
            hour: 12,
            minute: 15,
        });
        assert_eq!(
            countdown(&until, &now),
            Some(Countdown {
                ends_at_ms: ms(at(12, 15, 0)),
                total_ms: (2 * 60 + 15) * 60_000,
            })
        );
    }

    #[test]
    fn a_saved_custom_time_is_ignored_while_a_preset_is_chosen() {
        let now = at(10, 0, 0);
        let settings = Settings {
            duration_min: 15,
            use_custom: false,
            ..with_custom(Custom::Until {
                hour: 18,
                minute: 0,
            })
        };
        assert_eq!(
            countdown(&settings, &now).map(|c| c.total_ms),
            Some(15 * 60_000)
        );
    }
}
