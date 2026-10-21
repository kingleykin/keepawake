//! Owns the operating system's "keep awake" request.
//!
//! Every request is created and released on one dedicated thread. Windows ties
//! `SetThreadExecutionState` to the calling thread, so creating the request on
//! one thread and releasing it on another would leave the machine awake.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, Sender};
use std::thread;

use serde::{Deserialize, Serialize};

const REASON: &str = "KeepAwake is keeping this computer awake";
const APP_NAME: &str = "KeepAwake";
const APP_ID: &str = "io.github.kingleykin.keepawake";

/// What to keep awake.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Block idle sleep and keep the display on.
    #[default]
    Display,
    /// Block idle sleep only. The display may still turn off.
    System,
}

enum Command {
    Acquire(Mode, Sender<Result<(), String>>),
    Release(Sender<()>),
}

/// Handle to the worker thread that holds the OS request.
pub struct Awake {
    tx: Sender<Command>,
}

impl Awake {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel::<Command>();
        thread::Builder::new()
            .name("keepawake".into())
            .spawn(move || {
                let mut current: Option<platform::Inhibitor> = None;
                for command in rx {
                    match command {
                        Command::Acquire(mode, reply) => {
                            release(&mut current);
                            let result = catch_unwind(|| platform::Inhibitor::new(mode))
                                .unwrap_or_else(|_| Err("the system request panicked".into()))
                                .map(|inhibitor| current = Some(inhibitor));
                            let _ = reply.send(result);
                        }
                        Command::Release(reply) => {
                            release(&mut current);
                            let _ = reply.send(());
                        }
                    }
                }
                release(&mut current);
            })
            .expect("failed to start the keep-awake thread");
        Self { tx }
    }

    /// Ask the OS to keep the machine awake. Replaces any earlier request.
    pub fn acquire(&self, mode: Mode) -> Result<(), String> {
        let stopped = || "the keep-awake thread stopped".to_string();
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Command::Acquire(mode, tx))
            .map_err(|_| stopped())?;
        rx.recv().map_err(|_| stopped())?
    }

    /// Drop the current request, if any, and wait until the OS has it back.
    pub fn release(&self) {
        let (tx, rx) = mpsc::channel();
        if self.tx.send(Command::Release(tx)).is_ok() {
            let _ = rx.recv();
        }
    }
}

fn release(current: &mut Option<platform::Inhibitor>) {
    if let Some(inhibitor) = current.take() {
        // keepawake's Linux backend unwraps a D-Bus call while dropping.
        // A failed release must never take the whole app down.
        let _ = catch_unwind(AssertUnwindSafe(move || drop(inhibitor)));
    }
}

fn builder(mode: Mode) -> keepawake::Builder {
    let mut builder = keepawake::Builder::default();
    builder
        .display(mode == Mode::Display)
        .idle(true)
        // Blocking explicit sleep (lid close, Sleep menu) is out of scope for v1.
        .sleep(false)
        .reason(REASON)
        .app_name(APP_NAME)
        .app_reverse_domain(APP_ID);
    builder
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use super::{builder, Mode};

    /// macOS: IOKit power assertions. Windows: SetThreadExecutionState.
    pub struct Inhibitor(#[allow(dead_code)] keepawake::KeepAwake);

    impl Inhibitor {
        pub fn new(mode: Mode) -> Result<Self, String> {
            builder(mode).create().map(Self).map_err(|e| e.to_string())
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    //! Tries the XDG desktop portal first, because GNOME and KDE honour it on
    //! both Wayland and X11 for background apps. Falls back to the keepawake
    //! crate, which uses org.freedesktop.ScreenSaver and systemd-logind.

    use std::collections::HashMap;

    use zbus::blocking::Connection;
    use zbus::zvariant::{OwnedObjectPath, Value};

    use super::{builder, Mode, REASON};

    const PORTAL_SERVICE: &str = "org.freedesktop.portal.Desktop";
    const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
    const FLAG_SUSPEND: u32 = 4;
    const FLAG_IDLE: u32 = 8;

    pub enum Inhibitor {
        Portal {
            conn: Connection,
            handle: OwnedObjectPath,
        },
        Legacy(#[allow(dead_code)] keepawake::KeepAwake),
    }

    impl Inhibitor {
        pub fn new(mode: Mode) -> Result<Self, String> {
            match portal(mode) {
                Ok(inhibitor) => Ok(inhibitor),
                Err(portal_err) => builder(mode)
                    .create()
                    .map(Inhibitor::Legacy)
                    .map_err(|e| format!("{e} (desktop portal: {portal_err})")),
            }
        }
    }

    fn portal(mode: Mode) -> zbus::Result<Inhibitor> {
        let conn = Connection::session()?;
        let flags = match mode {
            Mode::Display => FLAG_IDLE | FLAG_SUSPEND,
            Mode::System => FLAG_SUSPEND,
        };
        let mut options: HashMap<&str, Value> = HashMap::new();
        options.insert("reason", Value::from(REASON));
        let reply = conn.call_method(
            Some(PORTAL_SERVICE),
            PORTAL_PATH,
            Some("org.freedesktop.portal.Inhibit"),
            "Inhibit",
            &("", flags, options),
        )?;
        let handle: OwnedObjectPath = reply.body().deserialize()?;
        Ok(Inhibitor::Portal { conn, handle })
    }

    impl Drop for Inhibitor {
        fn drop(&mut self) {
            if let Inhibitor::Portal { conn, handle } = self {
                let _ = conn.call_method(
                    Some(PORTAL_SERVICE),
                    handle.as_str(),
                    Some("org.freedesktop.portal.Request"),
                    "Close",
                    &(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks the real OS request on macOS. Run with `cargo test -- --ignored`.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "touches real power management"]
    fn macos_assertion_is_created_and_released() {
        let assertions = || {
            let out = std::process::Command::new("pmset")
                .args(["-g", "assertions"])
                .output()
                .expect("pmset");
            String::from_utf8_lossy(&out.stdout).into_owned()
        };
        let mine = |text: &str| text.lines().filter(|l| l.contains(REASON)).count();

        let awake = Awake::spawn();
        assert_eq!(mine(&assertions()), 0);

        awake.acquire(Mode::Display).unwrap();
        let held = assertions();
        assert!(held.contains("PreventUserIdleDisplaySleep") && mine(&held) == 2);

        awake.acquire(Mode::System).unwrap();
        let held = assertions();
        assert_eq!(
            mine(&held),
            1,
            "system mode holds only the idle-sleep assertion"
        );

        awake.release();
        assert_eq!(mine(&assertions()), 0);
    }

    /// Checks the real OS request on Windows with `powercfg /requests`, which
    /// needs an elevated shell. Run with `cargo test -- --ignored`.
    #[cfg(target_os = "windows")]
    #[test]
    #[ignore = "touches real power management"]
    fn windows_request_is_created_and_released() {
        // Which `powercfg /requests` sections list this test process.
        let held = || {
            let out = std::process::Command::new("powercfg")
                .arg("/requests")
                .output()
                .expect("powercfg");
            let text = String::from_utf8_lossy(&out.stdout).into_owned();
            assert!(out.status.success(), "powercfg /requests failed: {text}");
            let exe = std::env::current_exe().unwrap();
            let exe = exe.file_name().unwrap().to_string_lossy().to_lowercase();
            let mut section = String::new();
            let mut found = Vec::new();
            for line in text.lines() {
                if let Some(name) = line.strip_suffix(':').filter(|n| !n.starts_with(' ')) {
                    section = name.trim().to_string();
                } else if line.to_lowercase().contains(&exe) {
                    found.push(section.clone());
                }
            }
            found
        };

        let awake = Awake::spawn();
        assert_eq!(held(), Vec::<String>::new());

        awake.acquire(Mode::Display).unwrap();
        assert_eq!(held(), ["DISPLAY", "SYSTEM"]);

        awake.acquire(Mode::System).unwrap();
        assert_eq!(held(), ["SYSTEM"], "system mode lets the display sleep");

        awake.release();
        assert_eq!(held(), Vec::<String>::new());
    }

    /// Linux has no single API, so this only checks that every backend either
    /// works or fails cleanly, and that nothing is left behind. Run with
    /// `dbus-run-session -- cargo test -- --ignored`.
    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "touches real power management"]
    fn linux_request_never_panics_or_leaks() {
        let inhibitors = || {
            std::process::Command::new("systemd-inhibit")
                .arg("--list")
                .output()
                .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
                .unwrap_or_default()
        };

        let awake = Awake::spawn();
        for mode in [Mode::Display, Mode::System, Mode::Display] {
            match awake.acquire(mode) {
                Ok(()) => eprintln!("{mode:?}: held\n{}", inhibitors()),
                Err(error) => {
                    eprintln!("{mode:?}: refused: {error}");
                    assert!(
                        !error.contains("panicked") && !error.contains("thread stopped"),
                        "{error}"
                    );
                }
            }
            awake.release();
            assert!(
                !inhibitors().contains(REASON),
                "an inhibitor was left behind"
            );
        }
    }

    #[test]
    fn release_without_acquire_is_harmless() {
        let awake = Awake::spawn();
        awake.release();
        awake.release();
    }
}
