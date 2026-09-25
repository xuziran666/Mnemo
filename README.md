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
- **Pure Rust** — Native desktop UI; no web runtime, no Node.js. Two frontends share one core: [Slint](https://slint.dev) (active) and [egui](https://github.com/emilk/egui) (legacy).

## Screenshot

![Mnemo](docs/screenshots/main.png)

## Implementations

The workspace ships two frontends that share the same `mnemo-core` (SQLite persistence + business
logic) and the same data/settings files, so you can switch between them at any time:

| Crate | UI | Status | Run |
|---|---|---|---|
| `mnemo-slint` | [Slint](https://slint.dev) — frameless window with a self-drawn title bar, system tray, light/dark theme | **active** | `cargo run -p mnemo-slint` |
| `mnemo-egui` | [egui](https://github.com/emilk/egui) / eframe | legacy, still buildable | `cargo run -p mnemo-egui` |

### Differences you may notice

| Behavior | `mnemo-slint` | `mnemo-egui` |
|---|---|---|
| Window | Frameless, custom title bar, hides when it loses focus, comes back from the tray | Native window frame |
| `Enter` on a **note** | Copies its text (`v` / the **View** button opens the read-only viewer) | Opens the viewer |
| Theme | Dark by default, toggled in the toolbar and remembered | Follows the system theme on first run |

## Install

### Prebuilt binaries

Download from [GitHub Releases](https://github.com/xuziran666/Mnemo/releases):

- **Linux**: x86_64 / aarch64 executable
- **Windows**: x86_64 / aarch64 executable
- **macOS**: Apple Silicon executable

### Build from source

Requires [Rust](https://rustup.rs) (stable).

```bash
cargo build --release -p mnemo-slint   # Slint frontend (active)
cargo build --release -p mnemo-egui    # egui frontend (legacy)
```

The Slint binary is produced at `target/release/mnemo-slint` (`mnemo-slint.exe` on Windows);
the legacy egui binary at `target/release/mnemo`.

## Usage

| Key | Action |
|---|---|
| `s` | Focus search box |
| `↑` / `↓` | Navigate list |
| `Enter` | Copy selected snippet & close window, or open viewer for a note |
| `c` | Copy selected snippet & close window (snippets only) |
| `v` | View selected command |
| `d` | Delete selected command (with confirmation) |
| `Esc` | Close window (from list) |
| `r` | Edit selected command |
| `Ctrl+N` / `Cmd+N` | Add a new command |
| `+` | Add a new command (mouse) |

### In viewer

| Key | Action |
|---|---|
| `Enter` | Start editing (same window) |
| `Esc` | Back to search list |
| `Ctrl+S` / `Cmd+S` | Save edits & return to viewer |
| `Esc` (editing) | Discard changes & return to viewer |

## Development

```bash
cargo run -p mnemo-slint        # run the Slint frontend
cargo run -p mnemo-egui         # run the legacy egui frontend
cargo check -p mnemo-slint      # compile check only (fast)
cargo test --workspace
```

### Project layout

```
mnemo-core/   # SQLite persistence and business logic (framework-agnostic)
mnemo-slint/  # Slint desktop application (active)
mnemo-egui/   # egui/eframe desktop application (legacy)
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
| `window` | Last window size + maximized state (`mnemo-slint` only) |

## Tech Stack

- [Rust](https://www.rust-lang.org)
- [Slint](https://slint.dev) — declarative native GUI (`mnemo-slint`: custom title bar, tray, theming)
- [egui / eframe](https://github.com/emilk/egui) — native immediate-mode GUI (`mnemo-egui`, legacy)
- [SQLite](https://www.sqlite.org) (bundled via [rusqlite](https://github.com/rusqlite/rusqlite))
- [rfd](https://github.com/PolyMeilex/rfd) — native file dialogs
- [arboard](https://github.com/1Password/arboard) — system clipboard

## License

[MIT](LICENSE)
