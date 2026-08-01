# Mnemo

> A keyboard-first local command manager. Store your frequently used shell commands, search them instantly, and copy them to your clipboard in one keystroke.
>
> 一个键盘优先的本地命令管理器。收藏常用 shell 命令，秒搜秒复制，一键回到终端。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/xuziran666/Mnemo)](https://github.com/xuziran666/Mnemo/releases)
![Platforms](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)

## Features / 功能

- **Local first** — All data stays on your machine in a SQLite database. No cloud, no accounts.
  **本地优先** — 所有数据保存在本地 SQLite 数据库中，无云端、无账号。
- **Full-text search** — Fuzzy search across title, command, note, and tags.
  **全文搜索** — 对标题、命令、备注、标签进行模糊搜索。
- **Keyboard-first** — `s` to search, `↑`/`↓` to navigate, `Enter` to copy and close, `r` to edit, `Ctrl+N`/`Cmd+N` to add.
  **键盘优先** — `s` 搜索、`↑`/`↓` 选择、`Enter` 复制并关闭、`r` 编辑、`Ctrl+N`/`Cmd+N` 新建。
- **Copy & close** — Copying a command puts it on your clipboard and closes the window instantly, so your terminal workflow is never interrupted.
  **复制即关闭** — 复制命令后窗口自动关闭，命令已上剪贴板，终端工作流不被打断。
- **Organized** — Each command can carry a title, note, and tags for easy management.
  **结构化管理** — 每条命令可附带标题、备注和标签，方便整理。

## Screenshot / 截图

![Mnemo](docs/screenshots/main.png)

## Install / 安装

### Prebuilt binaries / 预编译安装包

Download from [GitHub Releases](https://github.com/xuziran666/Mnemo/releases):

- **Linux**: AppImage / deb / rpm (x86_64 & aarch64)
- **Windows**: exe / msi
- **macOS**: dmg / app

### Arch Linux (AUR)

Coming soon: `mnemo-cm` (build from source), `mnemo-cm-bin` (prebuilt binary).

即将上架：`mnemo-cm`（源码编译）、`mnemo-cm-bin`（预编译二进制）。

### Build from source / 源码构建

Requires [Node.js](https://nodejs.org) 18+ and [Rust](https://rustup.rs) (stable).

需要 [Node.js](https://nodejs.org) 18+ 与 [Rust](https://rustup.rs)（stable）。

**Linux** additionally needs the [Tauri system dependencies](https://tauri.app/start/prerequisites/) (`libwebkit2gtk-4.1-dev`, `librsvg2-dev`, etc.).

**Linux** 另需安装 [Tauri 系统依赖](https://tauri.app/start/prerequisites/)（如 `libwebkit2gtk-4.1-dev`、`librsvg2-dev` 等）。

```bash
npm install
npm run tauri build
```

## Usage / 快捷键

| Key / 按键 | Action / 功能 |
|---|---|
| `s` | Focus search box / 聚焦搜索框 |
| `↑` / `↓` | Navigate list / 在列表中移动选择 |
| `Enter` | Copy selected command & close window / 复制选中命令并关闭窗口 |
| `r` | Edit selected command / 编辑选中命令 |
| `Ctrl+N` / `Cmd+N` | Add a new command / 新建命令 |
| `+` | Add a new command (mouse) / 新建命令（鼠标） |

## Development / 开发

```bash
npm install          # install dependencies / 安装依赖
npm run tauri dev    # run with hot reload / 热重载开发
npm run tauri build  # produce release bundles / 构建发布包
```

## Data Storage / 数据存储

Data is stored in a single SQLite database (`commands.db`) inside your system's app-data directory:

数据保存在系统 app data 目录下的单个 SQLite 数据库文件（`commands.db`）中：

| OS / 系统 | Location / 路径 |
|---|---|
| Linux | `~/.local/share/com.longanl.mnemo/commands.db` |
| macOS | `~/Library/Application Support/com.longanl.mnemo/commands.db` |
| Windows | `%APPDATA%\com.longanl.mnemo\commands.db` |

Back up this file to keep your commands. / 备份该文件即可迁移你的全部命令。

## Tech Stack / 技术栈

- [Tauri 2](https://tauri.app) + [Rust](https://www.rust-lang.org)
- [React 19](https://react.dev) + [TypeScript](https://www.typescriptlang.org) + [Vite](https://vitejs.dev)
- [SQLite](https://www.sqlite.org) (bundled via [rusqlite](https://github.com/rusqlite/rusqlite))

## License / 许可证

[MIT](LICENSE)
