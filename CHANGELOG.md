# Changelog

All notable changes to KeepAwake are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [1.1.0] - 2026-10-30

### Added

- Custom time: pick any number of minutes (up to 24 hours), or "until" a time of day.
  A time that has already passed today means tomorrow. The tray menu has a
  "Custom…" item that opens the picker.
- A notification when the timer runs out and the computer may sleep again. It can be
  turned off in Settings.
- A keyboard shortcut to switch keep-awake on and off from anywhere: ⌥⌘K on macOS,
  Ctrl+Alt+K on Windows and Linux. It is off by default; switch it on in Settings.
  If another app already uses it, Settings says so.
- Updates: KeepAwake checks GitHub for a new version when it starts and when you open
  Settings. Settings shows "Update available" with an Install button. Nothing is
  installed until you press it, and the download must carry the project's signature.

### Fixed

- The Settings page was 8 pixels taller than the window, which cut off the bottom edge.

### Security

- The update plugin is 2.14.1, which uses `zip` 9.1.3 on Windows. Older `zip`
  versions had parser and extraction flaws (GHSA-m665-v7pm-7grj, GHSA-q7v9-x93c-xqv3,
  GHSA-8hxf-qr8p-62x4). `tokio` is 1.54.1 (GHSA-5wjr-9356-4wc8).

## [1.0.1] - 2026-10-21

### Fixed

- Switching between "keep the display on" and "keep only the system awake" while
  keep-awake is on no longer restarts the timer.

### Changed

- CI now runs the keep-awake request against the real operating system on macOS,
  Windows and Linux, and launches the app for a few seconds on each.

## [1.0.0] - 2026-10-20

### Added

- One-click keep awake from the window or the tray icon.
- Timer: 15 min, 30 min, 1 h, 2 h, 4 h or no limit. Keep-awake switches itself off when time is up.
- Two modes: keep the display on, or keep only the system awake.
- Tray menu with status and countdown, quick timer choices, open, settings and quit.
- Launch at login (starts hidden in the tray).
- Closing the window hides it to the tray (can be turned off).
- English and Vietnamese. English is the default.
- Light and dark themes that follow the system.
- Installers for macOS (universal `.dmg`), Windows (`.exe`, `.msi`) and Linux (`.AppImage`, `.deb`, `.rpm`).

[1.1.0]: https://github.com/kingleykin/keepawake/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/kingleykin/keepawake/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/kingleykin/keepawake/releases/tag/v1.0.0
