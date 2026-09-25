# Mnemo

[English](README.md) | [简体中文](README.zh-CN.md)

> 一个键盘优先的本地命令管理器。收藏常用 shell 命令，秒搜秒复制，一键回到终端。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/xuziran666/Mnemo)](https://github.com/xuziran666/Mnemo/releases)
![Platforms](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)

## 功能

- **本地优先** — 所有数据保存在本地 SQLite 数据库中，无云端、无账号。
- **全文搜索** — 对标题、命令、备注、标签进行模糊搜索。
- **键盘优先** — `s` 搜索、`↑`/`↓` 选择、`Enter` 直接复制代码片段（或打开知识笔记查看）、`r` 编辑、`Ctrl+N`/`Cmd+N` 新建。
- **复制即关闭** — 复制命令后窗口自动关闭，命令已上剪贴板，终端工作流不被打断。
- **结构化管理** — 每条命令可附带标题、备注和标签，方便整理。
- **日夜主题** — 一键切换日间/夜间模式，选择会被记住。
- **查看模式** — `Enter` 以只读方式查看知识笔记（纯文本）；查看器内 `Enter` 原位进入编辑，`Ctrl+S` 保存，`Esc` 返回。每条内容分为**代码片段**（用于复制）与**知识笔记**（用于查看）两种类型。
- **纯 Rust 实现** — 原生桌面界面，无 Web 运行时、无 Node.js。两套前端共用同一份核心：[Slint](https://slint.dev)（当前主用）与 [egui](https://github.com/emilk/egui)（旧版）。

## 截图

![Mnemo](docs/screenshots/main.png)

## 实现版本

仓库内有两套前端，共用同一个 `mnemo-core`（SQLite 持久化与业务逻辑）以及同一份数据/设置文件，
可以随时切换：

| crate | 界面 | 状态 | 运行 |
|---|---|---|---|
| `mnemo-slint` | [Slint](https://slint.dev) — 无边框窗口 + 自绘顶栏、系统托盘、日夜主题 | **当前主用** | `cargo run -p mnemo-slint` |
| `mnemo-egui` | [egui](https://github.com/emilk/egui) / eframe | 旧版，仍可构建 | `cargo run -p mnemo-egui` |

### 你可能注意到的差异

| 行为 | `mnemo-slint` | `mnemo-egui` |
|---|---|---|
| 窗口 | 无边框 + 自绘顶栏，失焦自动隐藏，从托盘唤回 | 系统原生边框 |
| 对**笔记**按 `Enter` | 复制其正文（用 `v` 或「查看」按钮打开只读查看层） | 打开查看器 |
| 主题 | 默认暗色，工具栏一键切换并记住 | 首次运行跟随系统主题 |

## 安装

### 预编译二进制

从 [GitHub Releases](https://github.com/xuziran666/Mnemo/releases) 下载：

- **Linux**：x86_64 / aarch64 可执行文件
- **Windows**：x86_64 / aarch64 可执行文件
- **macOS**：Apple Silicon 可执行文件

### 源码构建

只需 [Rust](https://rustup.rs)（stable）。

```bash
cargo build --release -p mnemo-slint   # Slint 前端（当前主用）
cargo build --release -p mnemo-egui    # egui 前端（旧版）
```

Slint 版二进制位于 `target/release/mnemo-slint`（Windows 为 `mnemo-slint.exe`）；
旧版 egui 版位于 `target/release/mnemo`。

## 快捷键

| 按键 | 功能 |
|---|---|
| `s` | 聚焦搜索框 |
| `↑` / `↓` | 在列表中移动选择 |
| `Enter` | 复制选中代码片段并关闭窗口，或打开知识笔记查看 |
| `c` | 复制选中代码片段并关闭窗口（仅代码片段） |
| `v` | 查看选中命令 |
| `d` | 删除选中命令（需二次确认） |
| `Esc` | 关闭窗口（列表中） |
| `r` | 编辑选中命令 |
| `Ctrl+N` / `Cmd+N` | 新建命令 |
| `+` | 新建命令（鼠标） |

### 查看器内

| 按键 | 功能 |
|---|---|
| `Enter` | 进入编辑（同一窗口） |
| `Esc` | 返回搜索列表 |
| `Ctrl+S` / `Cmd+S` | 保存并返回查看 |
| `Esc`（编辑中） | 取消修改并返回查看 |

## 开发

```bash
cargo run -p mnemo-slint        # 运行 Slint 前端
cargo run -p mnemo-egui         # 运行旧版 egui 前端
cargo check -p mnemo-slint      # 只做编译检查（快）
cargo test --workspace
```

### 项目结构

```
mnemo-core/   # SQLite 持久化与业务逻辑（不依赖 GUI 框架）
mnemo-slint/  # Slint 桌面应用（当前主用）
mnemo-egui/   # egui/eframe 桌面应用（旧版）
```

## 数据存储

数据保存在系统 app data 目录下的单个 SQLite 数据库文件（`commands.db`）中：

| 系统 | 路径 |
|---|---|
| Linux | `~/.local/share/com.longanl.mnemo/commands.db` |
| macOS | `~/Library/Application Support/com.longanl.mnemo/commands.db` |
| Windows | `%APPDATA%\com.longanl.mnemo\commands.db` |

备份该文件即可迁移你的全部命令。

偏好设置保存在同目录的 `settings.json`（两套前端共用）：

| 字段 | 含义 |
|---|---|
| `lang` | 界面语言（`en` / `zh`） |
| `theme` | `dark` / `light` |
| `window` | 上次窗口尺寸与最大化状态（仅 `mnemo-slint`） |

## 技术栈

- [Rust](https://www.rust-lang.org)
- [Slint](https://slint.dev) — 声明式原生 GUI（`mnemo-slint`：自绘顶栏、托盘、主题）
- [egui / eframe](https://github.com/emilk/egui) — 原生立即模式 GUI（`mnemo-egui`，旧版）
- [SQLite](https://www.sqlite.org)（通过 [rusqlite](https://github.com/rusqlite/rusqlite) 集成）
- [rfd](https://github.com/PolyMeilex/rfd) — 原生文件对话框
- [arboard](https://github.com/1Password/arboard) — 系统剪贴板

## 许可证

[MIT](LICENSE)
