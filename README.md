# Mnemo

[English](README.md) | [简体中文](README.zh-CN.md)

> A keyboard-first local command manager. Store your frequently used shell commands, search them instantly, and copy them to your clipboard in one keystroke.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/xuziran666/Mnemo)](https://github.com/xuziran666/Mnemo/releases)
![Platforms](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)

## Features

- **Local first** — All data stays on your machine in a SQLite database. No cloud, no accounts.
- **Full-text search** — Fuzzy search across title, command, note, and tags.
- **Keyboard-first** — `s` to search, `↑`/`↓` to navigate, `Enter` on a snippet copies it, `Enter` on a note opens the viewer, `r` to edit, `Ctrl+N`/`Cmd+N` to add.
- **Copy & close** — Copying a command puts it on your clipboard and closes the window instantly, so your terminal workflow is never interrupted.
- **Organized** — Each command can carry a title, note, and tags for easy management.
- **Viewer mode** — Press `Enter` to view a note like Quick Look (read-only, no caret, no toolbar); press `Enter` in the viewer to switch to editing in place, `Ctrl+S` to save, `Esc` to go back. Each entry is typed as **Snippet** (code, copied) or **Note** (knowledge, viewed). Notes render rich Markdown: GFM tables, KaTeX math, and syntax-highlighted code blocks.

## Screenshot

![Mnemo](docs/screenshots/main.png)

## Install

### Prebuilt binaries

Download from [GitHub Releases](https://github.com/xuziran666/Mnemo/releases):

- **Linux**: AppImage / deb / rpm (x86_64 & aarch64)
- **Windows**: exe / msi
- **macOS**: dmg / app

### Arch Linux (AUR)

Coming soon: `mnemo-cm` (build from source), `mnemo-cm-bin` (prebuilt binary).

### Build from source

Requires [Node.js](https://nodejs.org) 18+ and [Rust](https://rustup.rs) (stable).

**Linux** additionally needs the [Tauri system dependencies](https://tauri.app/start/prerequisites/) (`libwebkit2gtk-4.1-dev`, `librsvg2-dev`, etc.).

```bash
npm install
npm run tauri build
```

## Usage

| Key | Action |
|---|---|
| `s` | Focus search box |
| `↑` / `↓` | Navigate list |
| `Enter` | Copy selected snippet & close window, or open viewer for a note |
| `c` | Copy selected snippet & close window (snippets only) |
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
npm install          # install dependencies
npm run tauri dev    # run with hot reload
npm run tauri build  # produce release bundles
```

## Data Storage

Data is stored in a single SQLite database (`commands.db`) inside your system's app-data directory:

| OS | Location |
|---|---|
| Linux | `~/.local/share/com.longanl.mnemo/commands.db` |
| macOS | `~/Library/Application Support/com.longanl.mnemo/commands.db` |
| Windows | `%APPDATA%\com.longanl.mnemo\commands.db` |

Back up this file to keep your commands.

## Tech Stack

- [Tauri 2](https://tauri.app) + [Rust](https://www.rust-lang.org)
- [React 19](https://react.dev) + [TypeScript](https://www.typescriptlang.org) + [Vite](https://vitejs.dev)
- [SQLite](https://www.sqlite.org) (bundled via [rusqlite](https://github.com/rusqlite/rusqlite))

## License

[MIT](LICENSE)
