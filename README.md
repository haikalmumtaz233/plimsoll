# Plimsoll

A lightweight Windows tray app that shows how much of your Claude plan limit you have used, when it resets, and warns you before you run out.

Named after the Plimsoll line, the mark on a ship's hull that shows how far it can safely be loaded.

> Unofficial. Not affiliated with or endorsed by Anthropic.

## Features

- Five-hour session and weekly usage at a glance from the system tray
- Reset countdowns and configurable threshold notifications
- Usage history by model and project
- Local-first: no telemetry, no conversation content stored
- Keyboard, screen reader, and high contrast friendly

## Status

In active development. Not ready for general use yet.

## Requirements

- Windows 10 (1809) or later, x64 or ARM64
- Claude Code installed and signed in with a Pro or Max plan

## Install

Run this in PowerShell. It installs Plimsoll for the current user, without admin rights, and starts it in the system tray:

```powershell
irm https://github.com/haikalmumtaz233/plimsoll/releases/latest/download/install.ps1 | iex
```

The script picks the x64 or ARM64 installer from the latest release, checks it against the release `SHA256SUMS`, and runs it silently. Run the same command again to reinstall. To install a specific version, set `PLIMSOLL_VERSION` first:

```powershell
$env:PLIMSOLL_VERSION = 'v1.0.0'; irm https://github.com/haikalmumtaz233/plimsoll/releases/latest/download/install.ps1 | iex
```

You can also download the installer from [Releases](https://github.com/haikalmumtaz233/plimsoll/releases) yourself. Until the installers are code signed, Windows SmartScreen may warn about a download from the browser; choose **More info** and **Run anyway**. To uninstall, open **Settings > Apps > Installed apps** and remove Plimsoll.

## License

Released under the [MIT License](LICENSE).
