# Plimsoll

> Your Claude plan limits, in the Windows system tray.

[![Latest release](https://img.shields.io/github/v/release/haikalmumtaz233/plimsoll?style=flat-square&color=1f6feb)](https://github.com/haikalmumtaz233/plimsoll/releases/latest)
[![Windows 10 and 11](https://img.shields.io/badge/Windows-10%20%7C%2011-0078d4?style=flat-square)](#requirements)
[![License: MIT](https://img.shields.io/badge/license-MIT-6e5aff?style=flat-square)](LICENSE)

<p align="center">
  <img src="assets/popup.png" alt="The Plimsoll popup with the five-hour limit at 18 percent, the weekly limit at 98 percent, extra usage turned off, and a 24-hour token history chart." width="360" />
</p>

Plimsoll shows how much of your Claude Pro or Max plan you have used, when each limit resets, and warns you before you run out. It sits in the Windows system tray and reads the same numbers as the `/usage` command in Claude Code.

The name comes from the Plimsoll line, the mark on a ship's hull that shows how far it can safely be loaded.

> [!NOTE]
> Plimsoll is unofficial. It is not affiliated with or endorsed by Anthropic.

## Why

- Plan your work around the next reset instead of hitting the limit halfway through a task.
- See the weekly limit coming days ahead, not only the current session.
- Know whether extra usage credits will take over when a limit runs out.
- Keep everything on your own machine. There is no account to create, no telemetry, and no server in between.

## Install

Run this in PowerShell:

```powershell
irm https://github.com/haikalmumtaz233/plimsoll/releases/latest/download/install.ps1 | iex
```

The script picks the x64 or ARM64 installer from the latest release, checks it against the release `SHA256SUMS`, and installs Plimsoll for the current user without admin rights. Plimsoll then starts in the system tray.

To install a specific version, set `PLIMSOLL_VERSION` before running the same command:

```powershell
$env:PLIMSOLL_VERSION = 'v1.0.0'
irm https://github.com/haikalmumtaz233/plimsoll/releases/latest/download/install.ps1 | iex
```

You can also download the installer from [Releases](https://github.com/haikalmumtaz233/plimsoll/releases). The installers are not code signed yet, so Windows SmartScreen may warn about a file downloaded in a browser. Choose **More info**, then **Run anyway**.

### Update and uninstall

Plimsoll does not update itself. Run the install command again to move to the latest release. Your settings and history are kept.

To uninstall, open **Settings > Apps > Installed apps** and remove Plimsoll.

## First run

1. Click the Plimsoll icon in the tray to open the popup.
2. Out of the box, Plimsoll counts tokens from your Claude Code logs and shows when the current windows reset.
3. For the official percentages, open Settings and turn on **Accurate mode**. Plimsoll explains what it reads and sends before you confirm.

## Features

- Five-hour and weekly limits with reset countdowns and a pace marker that shows whether your usage is ahead of an even pace.
- Per-model weekly limits, such as Opus or Sonnet, on plans that have them.
- Extra usage credits: whether they are on, why they are off, and how much of the monthly limit is used.
- A tray icon that shows the five-hour percentage. Its color follows your alert levels, and it dims when the reading is out of date.
- Notifications at 50, 80, and 95 percent by default, with levels you can change.
- Token history for the last 24 hours or 7 days, broken down by model and by project.
- Adaptive refresh, from every 2 minutes while you use the popup to every 30 minutes when idle, plus a refresh button.
- Estimated percentages when official numbers are unavailable, based on your own past readings or a percentage you enter.
- English and Indonesian, following the Windows language unless you choose one.
- Light and dark themes that follow Windows, and support for contrast themes.
- Full keyboard and Narrator support, and text that scales with the Windows text size setting.
- An optional setting to start Plimsoll with Windows.

## How it works

Plimsoll combines two sources.

| Source                | Provides                                                             | Used                  |
| --------------------- | -------------------------------------------------------------------- | --------------------- |
| Claude Code logs      | Token counts, history, and the model and project breakdown           | Always, read only     |
| Claude usage endpoint | Official percentages, reset times, per-model limits, and extra usage | Only in accurate mode |

**Claude Code logs.** Plimsoll reads the usage fields from the JSONL logs in `%USERPROFILE%\.claude\projects` and never reads message content. Claude Code removes logs after about 30 days, so Plimsoll keeps its own 90-day history.

**Accurate mode.** Plimsoll reads the Claude Code sign-in token from `%USERPROFILE%\.claude\.credentials.json` right before each request, sends it to `api.anthropic.com`, and clears it from memory afterwards. This is the endpoint behind the `/usage` command. It is not a documented public API and may change without notice. Plimsoll never writes to the `.claude` folder and never refreshes the token itself. When the endpoint fails, an optional fallback asks Claude Code for its `/usage` panel instead, at most once every 10 minutes.

Claude Code logs only cover Claude Code. Chats on claude.ai, the desktop app, and mobile count toward your limits but do not appear in the token history. The official percentages in accurate mode include every surface.

## Privacy

- No telemetry, analytics, or crash reports.
- Conversation content is never read or stored.
- The sign-in token is never written to disk or to the log.
- Plimsoll connects to one host, `api.anthropic.com`, and only in accurate mode.
- Usage history is stored in `%LOCALAPPDATA%\com.haikalmumtaz.plimsoll` and pruned after 90 days.
- A small diagnostic log in the same folder records errors and HTTP status codes, never tokens or response bodies.

## Requirements

- Windows 10 version 1809 or later, or Windows 11, on x64 or ARM64
- Claude Code installed and signed in with a Claude Pro or Max plan
- Microsoft Edge WebView2 Runtime, which the installer adds when it is missing

## Build from source

You need Node.js 22.18 or later, pnpm 9, the stable Rust toolchain, and the Microsoft C++ Build Tools.

```powershell
pnpm install
pnpm tauri dev
```

Run the tests and build the installers:

```powershell
pnpm test
pnpm rust:test
pnpm tauri build
```

## Security

Please report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

## License

Released under the [MIT License](LICENSE).
