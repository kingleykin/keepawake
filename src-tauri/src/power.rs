//! Whether the computer runs on battery, and how full the battery is.
//!
//! Only reads. Each platform reports what it knows, and anything it cannot
//! tell stays `None`, so a guard never switches off on a guess.

use std::sync::mpsc;
use std::time::Duration;

/// A reading that takes longer than this counts as failed.
const TIMEOUT: Duration = Duration::from_secs(3);

/// One reading of the battery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Power {
    /// `Some(true)` on battery, `Some(false)` plugged in.
    pub on_battery: Option<bool>,
    /// Charge from 0 to 100.
    pub percent: Option<u8>,
}

/// `Ok(None)` means this computer has no battery. `Err` means it could not tell.
/// Runs on a thread of its own and gives up after a few seconds, so a system
/// service that stops answering never holds up the caller.
pub fn read() -> Result<Option<Power>, String> {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("power".into())
        .spawn(move || {
            let _ = tx.send(platform::read());
        })
        .map_err(|e| e.to_string())?;
    rx.recv_timeout(TIMEOUT)
        .unwrap_or_else(|_| Err("reading the battery timed out".into()))
        .and_then(known)
}

/// A battery that reports nothing at all was not read.
fn known(reading: Option<Power>) -> Result<Option<Power>, String> {
    match reading {
        Some(Power {
            on_battery: None,
            percent: None,
        }) => Err("the battery reported nothing".into()),
        other => Ok(other),
    }
}

/// Rounds a reported charge to a whole percent, or `None` if it makes no sense.
fn whole_percent(value: f64) -> Option<u8> {
    (0.0..=100.0).contains(&value).then(|| value.round() as u8)
}

/// Windows' `SYSTEM_POWER_STATUS` fields. 255 means unknown in each of them.
#[cfg_attr(not(windows), allow(dead_code))]
fn from_windows(ac_line_status: u8, battery_flag: u8, battery_life_percent: u8) -> Option<Power> {
    const NO_BATTERY: u8 = 128;
    const UNKNOWN: u8 = 255;
    if battery_flag != UNKNOWN && battery_flag & NO_BATTERY != 0 {
        return None;
    }
    Some(Power {
        on_battery: match ac_line_status {
            0 => Some(true),
            1 => Some(false),
            _ => None,
        },
        percent: whole_percent(f64::from(battery_life_percent)),
    })
}

/// UPower's answers. Desktops run UPower too, so only a present battery or
/// UPS counts.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn from_upower(
    on_battery: Option<bool>,
    is_present: Option<bool>,
    kind: Option<u32>,
    percent: Option<f64>,
) -> Option<Power> {
    const BATTERY: u32 = 2;
    const UPS: u32 = 3;
    let has_battery = is_present == Some(true) && matches!(kind, Some(BATTERY | UPS));
    has_battery.then(|| Power {
        on_battery,
        percent: percent.and_then(whole_percent),
    })
}

#[cfg(target_os = "macos")]
mod platform {
    //! IOKit power sources, the same data the battery menu shows.

    use objc2_core_foundation::{
        CFArray, CFBoolean, CFDictionary, CFNumber, CFRetained, CFString, CFType,
    };
    use objc2_io_kit::{
        kIOPSBatteryPowerValue, kIOPSCurrentCapacityKey, kIOPSInternalBatteryType,
        kIOPSIsPresentKey, kIOPSMaxCapacityKey, kIOPSPowerSourceStateKey, kIOPSTypeKey,
        kIOPSUPSType, IOPSCopyPowerSourcesInfo, IOPSCopyPowerSourcesList,
        IOPSGetPowerSourceDescription,
    };

    use super::{whole_percent, Power};

    type Description = CFDictionary<CFString, CFType>;

    fn key(name: &std::ffi::CStr) -> CFRetained<CFString> {
        CFString::from_str(name.to_str().unwrap_or_default())
    }

    fn string(description: &Description, name: &std::ffi::CStr) -> Option<String> {
        let value = description.get(&key(name))?;
        Some(value.downcast_ref::<CFString>()?.to_string())
    }

    fn number(description: &Description, name: &std::ffi::CStr) -> Option<i64> {
        let value = description.get(&key(name))?;
        value.downcast_ref::<CFNumber>()?.as_i64()
    }

    fn boolean(description: &Description, name: &std::ffi::CStr) -> Option<bool> {
        let value = description.get(&key(name))?;
        Some(value.downcast_ref::<CFBoolean>()?.as_bool())
    }

    pub fn read() -> Result<Option<Power>, String> {
        let info = IOPSCopyPowerSourcesInfo().ok_or("no power source information")?;
        let list =
            unsafe { IOPSCopyPowerSourcesList(Some(&info)) }.ok_or("no list of power sources")?;
        // SAFETY: the list holds power source handles, which are CF objects.
        let list = unsafe { CFRetained::cast_unchecked::<CFArray<CFType>>(list) };
        let batteries = [kIOPSInternalBatteryType, kIOPSUPSType].map(|t| t.to_str().ok());
        // A source that could not be described may be the battery.
        let mut unread = false;
        for source in list.iter() {
            let Some(description) =
                (unsafe { IOPSGetPowerSourceDescription(Some(&info), Some(&source)) })
            else {
                unread = true;
                continue;
            };
            // SAFETY: Apple documents the description as a dictionary keyed by strings.
            let description = unsafe { CFRetained::cast_unchecked::<Description>(description) };
            let kind = string(&description, kIOPSTypeKey);
            if !batteries.contains(&kind.as_deref())
                || boolean(&description, kIOPSIsPresentKey) == Some(false)
            {
                continue;
            }
            let on_battery = string(&description, kIOPSPowerSourceStateKey)
                .map(|state| Some(state.as_str()) == kIOPSBatteryPowerValue.to_str().ok());
            let percent = match (
                number(&description, kIOPSCurrentCapacityKey),
                number(&description, kIOPSMaxCapacityKey),
            ) {
                (Some(current), Some(max)) if max > 0 => {
                    whole_percent(current as f64 * 100.0 / max as f64)
                }
                _ => None,
            };
            return Ok(Some(Power {
                on_battery,
                percent,
            }));
        }
        if unread {
            return Err("a power source could not be read".into());
        }
        Ok(None)
    }
}

#[cfg(windows)]
mod platform {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

    use super::{from_windows, Power};

    pub fn read() -> Result<Option<Power>, String> {
        let mut status = SYSTEM_POWER_STATUS::default();
        unsafe { GetSystemPowerStatus(&mut status) }.map_err(|e| e.to_string())?;
        Ok(from_windows(
            status.ACLineStatus,
            status.BatteryFlag,
            status.BatteryLifePercent,
        ))
    }
}

#[cfg(target_os = "linux")]
mod platform {
    //! UPower on the system bus. One connection is kept and made again after
    //! an error, and every call gives up after a short wait.

    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::Duration;

    use zbus::blocking::{connection, Connection};
    use zbus::zvariant::OwnedValue;

    use super::{from_upower, Power};

    const SERVICE: &str = "org.freedesktop.UPower";
    const PATH: &str = "/org/freedesktop/UPower";
    const DISPLAY_DEVICE: &str = "/org/freedesktop/UPower/devices/DisplayDevice";
    const TIMEOUT: Duration = Duration::from_secs(2);

    static CONNECTION: Mutex<Option<Connection>> = Mutex::new(None);

    type Properties = HashMap<String, OwnedValue>;

    fn properties(conn: &Connection, path: &str, interface: &str) -> zbus::Result<Properties> {
        let reply = conn.call_method(
            Some(SERVICE),
            path,
            Some("org.freedesktop.DBus.Properties"),
            "GetAll",
            &(interface,),
        )?;
        reply.body().deserialize()
    }

    fn get<T: TryFrom<OwnedValue>>(properties: &Properties, name: &str) -> Option<T> {
        T::try_from(properties.get(name)?.try_clone().ok()?).ok()
    }

    fn ask(conn: &Connection) -> zbus::Result<Option<Power>> {
        let upower = properties(conn, PATH, "org.freedesktop.UPower")?;
        let device = properties(conn, DISPLAY_DEVICE, "org.freedesktop.UPower.Device")?;
        Ok(from_upower(
            get(&upower, "OnBattery"),
            get(&device, "IsPresent"),
            get(&device, "Type"),
            get(&device, "Percentage"),
        ))
    }

    pub fn read() -> Result<Option<Power>, String> {
        let mut cached = CONNECTION.lock().unwrap_or_else(|p| p.into_inner());
        let conn = match cached.take() {
            Some(conn) => conn,
            None => connection::Builder::system()
                .and_then(|b| b.method_timeout(TIMEOUT).build())
                .map_err(|e| e.to_string())?,
        };
        let answer = ask(&conn).map_err(|e| e.to_string());
        // Keep the connection only while it works.
        if answer.is_ok() {
            *cached = Some(conn);
        }
        answer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prints what this computer reports. Run with `cargo test -- --ignored`.
    #[test]
    #[ignore = "reads the real battery"]
    fn reads_this_computer() {
        let reading = read();
        eprintln!("{reading:?}");
        assert!(reading.is_ok(), "{reading:?}");
    }

    #[test]
    fn windows_fields() {
        let power = |on_battery, percent| {
            Some(Power {
                on_battery,
                percent,
            })
        };
        assert_eq!(from_windows(0, 1, 87), power(Some(true), Some(87)));
        assert_eq!(from_windows(1, 8, 40), power(Some(false), Some(40)));
        assert_eq!(from_windows(1, 128, 255), None, "no battery");
        assert_eq!(
            from_windows(255, 1, 87),
            power(None, Some(87)),
            "charger state unknown"
        );
        assert_eq!(
            from_windows(0, 255, 255),
            power(Some(true), None),
            "battery status unknown is not the same as no battery"
        );
    }

    #[test]
    fn a_battery_that_says_nothing_was_not_read() {
        assert!(known(from_windows(255, 255, 255)).is_err());
        assert_eq!(known(None), Ok(None));
        let charger_only = Some(Power {
            on_battery: Some(false),
            percent: None,
        });
        assert_eq!(known(charger_only), Ok(charger_only));
    }

    #[test]
    fn upower_fields() {
        assert_eq!(
            from_upower(Some(true), Some(true), Some(2), Some(19.6)),
            Some(Power {
                on_battery: Some(true),
                percent: Some(20)
            })
        );
        assert_eq!(
            from_upower(Some(false), Some(true), Some(3), None),
            Some(Power {
                on_battery: Some(false),
                percent: None
            }),
            "a UPS counts"
        );
        assert_eq!(
            from_upower(Some(false), Some(false), Some(0), Some(0.0)),
            None,
            "a desktop has UPower but no battery"
        );
        assert_eq!(
            from_upower(Some(false), Some(false), Some(2), Some(0.0)),
            None,
            "a removed battery"
        );
        assert_eq!(
            from_upower(Some(true), Some(true), Some(2), Some(250.0)),
            Some(Power {
                on_battery: Some(true),
                percent: None
            }),
            "a nonsense charge is unknown"
        );
    }
}
