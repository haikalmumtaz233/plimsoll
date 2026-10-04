# Plimsoll

> Your Claude plan limits, in the Windows system tray.

[![Latest release](https://img.shields.io/github/v/release/haikalmumtaz233/plimsoll?style=flat-square&color=1f6feb)](https://github.com/haikalmumtaz233/plimsoll/releases/latest)
[![Windows 10 and 11](https://img.shields.io/badge/Windows-10%20%7C%2011-0078d4?style=flat-square)](#requirements)
[![License: MIT](https://img.shields.io/badge/license-MIT-6e5aff?style=flat-square)](LICENSE)

<p align="center">
  <img src="assets/popup.png" alt="The Plimsoll popup with the five-hour limit, the weekly limit, extra usage, and a 24-hour token history chart." width="360" />
</p>

## Install

Run this in PowerShell:

```powershell
irm https://github.com/haikalmumtaz233/plimsoll/releases/latest/download/install.ps1 | iex
```

It installs Plimsoll for the current user, without admin rights, and starts it in the tray. Run it again to update. You can also download the installer from [Releases](https://github.com/haikalmumtaz233/plimsoll/releases).

Turn on **Accurate mode** in Settings to see the official percentages.

## Features

- Five-hour and weekly limits with reset countdowns
- Per-model weekly limits and extra usage credits
- Tray icon with the five-hour percentage and alert colors
- Notifications at 50, 80, and 95 percent, adjustable
- Token history by model and project
- English and Indonesian, light and dark themes

## Privacy

Plimsoll reads usage fields from Claude Code logs and never reads conversation content. In accurate mode it sends your Claude Code sign-in token only to `api.anthropic.com`, the endpoint behind `/usage`. No telemetry, and all data stays on your machine.

## Requirements

- Windows 10 1809 or later, x64 or ARM64
- Claude Code signed in with a Pro or Max plan

## Build from source

```powershell
pnpm install
pnpm tauri dev
```

Unofficial. Not affiliated with or endorsed by Anthropic. Released under the [MIT License](LICENSE).
