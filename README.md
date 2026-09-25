# Mnemo

[English](README.md) | [简体中文](README.zh-CN.md)

> A keyboard-first local command manager. Store your frequently used shell commands, search them instantly, and copy them to your clipboard in one keystroke.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/xuziran666/Mnemo)](https://github.com/xuziran666/Mnemo/releases)
![Platforms](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)

## Features

- **Local first** — All data stays on your machine in a SQLite database. No cloud, no accounts.
- **Full-text search** — Fuzzy search across title, content, note, and tags.
- **Keyboard-first** — `s` to search, `↑`/`↓` to navigate, `Enter` on a snippet copies it, `Enter` on a note opens the viewer, `r` to edit, `Ctrl+N`/`Cmd+N` to add.
- **Copy & close** — Copying a command puts it on your clipboard and closes the window instantly, so your terminal workflow is never interrupted.
- **Organized** — Each command can carry a title, note, and tags for easy management.
- **Light & dark theme** — Toggle between light and dark mode; your choice is remembered.
- **Viewer mode** — Press `Enter` to view a note as plain text (read-only); press `Enter` in the viewer to switch to editing in place, `Ctrl+S` to save, `Esc` to go back. Each entry is typed as **Snippet** (code, copied) or **Note** (knowledge, viewed).
- **Pure Rust** — Native desktop UI built with [Slint](https://slint.dev); no web runtime, no Node.js. All persistence and business logic live in a framework-agnostic `mnemo-core`.

## Screenshot

![Mnemo](docs/screenshots/main.png)

## Architecture

| Crate | Role |
|---|---|
| `mnemo-core` | SQLite persistence and business logic (framework-agnostic, unit-tested) |
| `mnemo-slint` | Desktop application: [Slint](https://slint.dev) UI, frameless window with a self-drawn title bar, system tray, light/dark theme |

> An earlier [egui](https://github.com/emilk/egui)/eframe frontend (`mnemo-egui`) has been removed.
> Its last state is preserved in the git tag `legacy-egui`
> (`git show legacy-egui:mnemo-egui/<path>`), and the notes for that phase live in `docs/refactor/`.

### Notes on behavior

- The window is frameless with a custom title bar: drag it to move, double-click the title bar to
  maximize/restore, and use the buttons on its right edge for minimize / maximize / close.
- Closing or losing focus **hides** the window (the process keeps running); bring it back from the tray
  (left-click the icon, or the tray menu).
- `Enter` copies the selected command; use `v` (or the **View** button) to read a note without copying.
- The theme is dark by default, toggled in the toolbar and remembered. Preferences live in
  `settings.json` next to the database.

## Install

### Prebuilt binaries

Download from [GitHub Releases](https://github.com/xuziran666/Mnemo/releases):

- **Linux**: x86_64 / aarch64 executable
- **Windows**: x86_64 / aarch64 executable
- **macOS**: Apple Silicon executable

### Build from source

Requires [Rust](https://rustup.rs) (stable).

```bash
cargo build --release -p mnemo-slint
```

The binary is produced at `target/release/mnemo` (`mnemo.exe` on Windows).

## Usage

| Key | Action |
|---|---|
| `s` | Focus the search box |
| `↑` / `↓` | Move the selection (the list scrolls to follow) |
| `Enter` | Copy the selected command & hide the window |
| `c` | Same as `Enter` |
| `v` | Open the read-only viewer for the selected command |
| `r` | Edit the selected command |
| `d` | Delete the selected command (confirmation required) |
| `Ctrl+N` / `Cmd+N` | Add a new command |
| `Esc` | Close the open viewer/dialog, otherwise hide the window |
| `+` | Add a new command (mouse) |

### In a dialog

| Key | Action |
|---|---|
| `Enter` (in the search box) | Move focus to the list, so the keys above apply |
| `Ctrl+S` / `Cmd+S` (editor) | Save |
| `Esc` | Close the current viewer / dialog without saving |

## Development

```bash
cargo run -p mnemo-slint        # run the app
cargo check -p mnemo-slint      # compile check only (fast)
cargo test --workspace
```

### Project layout

```
mnemo-core/   # SQLite persistence and business logic (framework-agnostic)
mnemo-slint/  # Slint desktop application (UI + window/tray integration)
```

## Data Storage

Data is stored in a single SQLite database (`commands.db`) inside your system's app-data directory:

| OS | Location |
|---|---|
| Linux | `~/.local/share/com.longanl.mnemo/commands.db` |
| macOS | `~/Library/Application Support/com.longanl.mnemo/commands.db` |
| Windows | `%APPDATA%\com.longanl.mnemo\commands.db` |

Back up this file to keep your commands.

Preferences are stored next to it in `settings.json` (shared by both frontends):

| Key | Meaning |
|---|---|
| `lang` | UI language (`en` / `zh`) |
| `theme` | `dark` / `light` |
| `window` | Last window size + maximized state |

## Tech Stack

- [Rust](https://www.rust-lang.org)
- [Slint](https://slint.dev) — declarative native GUI (custom title bar, tray, theming)
- [SQLite](https://www.sqlite.org) (bundled via [rusqlite](https://github.com/rusqlite/rusqlite))
- [rfd](https://github.com/PolyMeilex/rfd) — native file dialogs
- [arboard](https://github.com/1Password/arboard) — system clipboard

## License

[MIT](LICENSE)
