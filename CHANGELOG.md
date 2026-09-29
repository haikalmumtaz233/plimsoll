# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Plimsoll is released under the MIT License.

### Changed

- The tray icon shows the five-hour limit, and its color follows that limit. The weekly limit stays in the tooltip and the popup, and still sends its alerts.

## [0.7.0-rc.1] - 2026-09-28

### Added

- A local diagnostic log in `%LOCALAPPDATA%\com.haikalmumtaz.plimsoll\logs` (up to three files of 1 MB each). It records refresh failures with their HTTP status and retry hint, database and file errors, and fallback results, never tokens, headers, response bodies or conversation content.
- A refresh button beside the status badge. It waits a minute between requests and does not cut a retry pause short.
- The tray icon dims, and its tooltip says so, when the official reading is out of date or the last refresh failed.
- The popup header shows your Claude plan, such as Pro or Max 5x, while accurate mode is on.
- A "Log in to Claude Code" button appears when Claude Code is signed out or its sign-in has expired. It opens Claude Code's own login in a new window.
- Claude Code logs are also read from `%USERPROFILE%\.config\claude\projects` and from Claude Desktop's local Claude Code folders when they exist.
- An opt-in fallback in accurate mode: after two failed refreshes or an expired sign-in, Plimsoll asks Claude Code for its `/usage` panel in the background, at most every 10 minutes.
- The pace marker explains itself in a tooltip and to screen readers, and a link opens the usage page on claude.ai in your browser.

### Changed

- A saved refresh interval of 1 minute moves to Adaptive once, because polling every minute triggered rate limits.

## [0.6.0-rc.2] - 2026-09-28

### Added

- Adaptive refresh, now the default: official usage refreshes every 2 minutes after you open the popup, every 5 minutes while Claude Code is active, and up to every 30 minutes when idle. Opening the popup fetches a fresh reading when the last one is older than 2 minutes.
- The status badge shows how long ago the official reading arrived.
- A pace marker on each limit meter shows how much of the window has elapsed.

### Changed

- A failed refresh keeps the green official badge, with a small sync icon, while the last official reading is still shown. The badge turns yellow only when no official reading is left.
- Official readings stay on screen for 35 minutes instead of 15.
- Turning accurate mode off and on again waits at least a minute before the next request instead of asking straight away.

## [0.6.0-rc.1] - 2026-09-28

### Changed

- The popup shows less text: a small status badge replaces the status sentence, estimates read as "≈ 42% of limit", and reset countdowns sit beside each card title.
- Settings opens with accurate mode, drops the help paragraphs, shows the refresh interval only in accurate mode, and keeps the accurate mode warnings short.
- Manual percentages are dimmed, locked and marked with a lock icon while official data is shown, because they are not used then.
- The settings button in the header is an icon.

## [0.5.0-rc.1] - 2026-09-27

### Changed

- The popup is released 30 seconds after it closes, so Plimsoll uses about 6 MB of memory instead of about 180 MB while it sits in the tray. Opening the popup after that takes under a second.
- Pre-releases such as release candidates ship only the NSIS installers. The MSI is built for final releases.

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

[Unreleased]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.7.0-rc.1...HEAD
[0.7.0-rc.1]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.6.0-rc.2...v0.7.0-rc.1
[0.6.0-rc.2]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.6.0-rc.1...v0.6.0-rc.2
[0.6.0-rc.1]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.5.0-rc.1...v0.6.0-rc.1
[0.5.0-rc.1]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.4.0...v0.5.0-rc.1
[0.4.0]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/haikalmumtaz233/plimsoll/releases/tag/v0.1.0
