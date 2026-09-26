# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Token history chart in the popup for the last 24 hours or the last 7 days, with a data table for screen readers and keyboard users.
- Breakdown of token usage by model and by project for the same range as the history chart.

### Changed

- The popup opens next to the tray icon instead of the center of the screen.

## [0.1.0] - 2026-09-26

### Added

- System tray icon with Open and Quit actions and a popup window.
- Popup with the 5-hour and weekly limits, reset countdowns and a status line. Without accurate mode it shows token estimates from local Claude Code logs.
- Opt-in accurate mode that reads the Claude Code sign-in on this PC to fetch official percentages, falling back to local estimates when it is unavailable.
- Tray icon drawn as a sharp badge at 100 to 300 percent display scaling: the most used limit in percent, colored at 50, 80 and 95 percent, or tokens used when accurate mode is off. The tooltip lists each limit with its reset countdown.

[Unreleased]: https://github.com/haikalmumtaz233/plimsoll/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/haikalmumtaz233/plimsoll/releases/tag/v0.1.0
