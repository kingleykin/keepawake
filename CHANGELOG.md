# Changelog

All notable changes to KeepAwake are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [1.0.1] - Unreleased

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

[1.0.1]: https://github.com/kingleykin/keepawake/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/kingleykin/keepawake/releases/tag/v1.0.0
