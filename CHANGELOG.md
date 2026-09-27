# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- The popup is released 30 seconds after it closes, so Plimsoll uses about 6 MB of memory instead of about 180 MB while it sits in the tray. Opening the popup after that takes under a second.

### Fixed

- Usage no longer becomes unavailable when a Claude Code log reports an implausibly large token count.
- Log lines without a model name are ignored instead of showing an empty model in the breakdown.

## [0.4.0] - 2026-09-27

### Added

- Option in Settings to start Plimsoll when Windows starts. It is off by default.
- Installers for each release: per-user NSIS setup for x64 and ARM64, and an MSI for x64. Uninstalling removes the start with Windows entry. They are not code signed yet, so SmartScreen may warn on first run.

### Changed

- Launching Plimsoll again opens the popup of the running instance instead of starting a second copy.

## [0.3.0] - 2026-09-27

### Added

- A confirmation that explains how accurate mode uses the Claude Code sign-in, and its risks, before the mode is turned on.
- Experimental percent estimates when official data is not available, calibrated from past official readings and local token counts once at least three limit windows were seen.
- Manual percentage in Settings for each limit, copied from /usage, used as a fallback until the limit could have reset and projected forward with calibrated usage.

## [0.2.0] - 2026-09-26

### Added

- Token history chart in the popup for the last 24 hours or the last 7 days, with a data table for screen readers and keyboard users.
- Breakdown of token usage by model and by project for the same range as the history chart.
- Settings panel with configurable alert levels for the tray and meters, and the refresh interval for accurate mode.
- Windows notifications when an official limit passes an alert level, once per level and limit window, also announced to screen readers in the popup.
- English and Indonesian for the popup, tray tooltip, tray menu and notifications, following the Windows display language unless a language is chosen in Settings.

### Changed

- The popup opens next to the tray icon instead of the center of the screen.
- The accurate mode switch moved into Settings.

### Fixed

- Selected options, the Save button and limit meters stay readable in Windows contrast themes.
- The popup reflows without horizontal overflow at 200 percent text size, and small controls meet the 24 px minimum target size.

## [0.1.0] - 2026-09-26

### Added

- System tray icon with Open and Quit actions and a popup window.
- Popup with the 5-hour and weekly limits, reset countdowns and a status line. Without accurate mode it shows token estimates from local Claude Code logs.
- Opt-in accurate mode that reads the Claude Code sign-in on this PC to fetch official percentages, falling back to local estimates when it is unavailable.
- Tray icon drawn as a sharp badge at 100 to 300 percent display scaling: the most used limit in percent, colored at 50, 80 and 95 percent, or tokens used when accurate mode is off. The tooltip lists each limit with its reset countdown.

[Unreleased]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/haikalmumtaz233/plimsoll/releases/tag/v0.1.0
