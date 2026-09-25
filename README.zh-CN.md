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
- **纯 Rust 实现** — 使用 [Slint](https://slint.dev) 原生桌面界面，无 Web 运行时、无 Node.js；持久化与业务逻辑集中在不依赖 GUI 的 `mnemo-core`。

## 截图

![Mnemo](docs/screenshots/main.png)

## 架构

| crate | 职责 |
|---|---|
| `mnemo-core` | SQLite 持久化与业务逻辑（不依赖 GUI 框架，带单元测试） |
| `mnemo-slint` | 桌面应用：[Slint](https://slint.dev) 界面、无边框窗口 + 自绘顶栏、系统托盘、日夜主题 |

> 早期的 [egui](https://github.com/emilk/egui)/eframe 前端（`mnemo-egui`）已移除。
> 它的最后状态保存在 git tag `legacy-egui`（`git show legacy-egui:mnemo-egui/<path>` 可取回），
> 该阶段的记录见 `docs/refactor/`。

### 行为说明

- 窗口无边框、顶栏自绘：拖顶栏移动窗口，双击顶栏最大化/还原，右侧三个按钮是最小化 / 最大化还原 / 关闭。
- **关闭或失去焦点只是隐藏窗口**（进程继续运行）；从托盘唤回（左键点图标，或托盘菜单）。
- `Enter` 复制选中命令；要只读查看笔记内容，用 `v` 或卡片上的「查看」按钮。
- 主题默认暗色，工具栏一键切换并记住；偏好保存在数据库同目录的 `settings.json`。

## 安装

### 预编译二进制

从 [GitHub Releases](https://github.com/xuziran666/Mnemo/releases) 下载：

- **Linux**：x86_64 / aarch64 可执行文件
- **Windows**：x86_64 / aarch64 可执行文件
- **macOS**：Apple Silicon 可执行文件

### 源码构建

只需 [Rust](https://rustup.rs)（stable）。

```bash
cargo build --release -p mnemo-slint
```

生成的二进制位于 `target/release/mnemo`（Windows 为 `mnemo.exe`）。

## 快捷键

| 按键 | 功能 |
|---|---|
| `s` | 聚焦搜索框 |
| `↑` / `↓` | 在列表中移动选择（列表会自动滚动跟随） |
| `Enter` | 复制选中命令并隐藏窗口 |
| `c` | 同 `Enter` |
| `v` | 只读查看选中命令 |
| `r` | 编辑选中命令 |
| `d` | 删除选中命令（需二次确认） |
| `Ctrl+N` / `Cmd+N` | 新建命令 |
| `Esc` | 先关闭当前查看层/弹层，没有弹层时隐藏窗口 |
| `+` | 新建命令（鼠标） |

### 弹层内

| 按键 | 功能 |
|---|---|
| `Enter`（搜索框内） | 把焦点交给列表，之后上表按键才作用于列表 |
| `Ctrl+S` / `Cmd+S`（编辑器） | 保存 |
| `Esc` | 关闭当前查看层 / 弹层（不保存） |

## 开发

```bash
cargo run -p mnemo-slint        # 运行应用
cargo check -p mnemo-slint      # 只做编译检查（快）
cargo test --workspace
```

### 项目结构

```
mnemo-core/   # SQLite 持久化与业务逻辑（不依赖 GUI 框架）
mnemo-slint/  # Slint 桌面应用（界面 + 窗口/托盘集成）
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
| `window` | 上次窗口尺寸与最大化状态 |

## 技术栈

- [Rust](https://www.rust-lang.org)
- [Slint](https://slint.dev) — 声明式原生 GUI（自绘顶栏、托盘、主题）
- [SQLite](https://www.sqlite.org)（通过 [rusqlite](https://github.com/rusqlite/rusqlite) 集成）
- [rfd](https://github.com/PolyMeilex/rfd) — 原生文件对话框
- [arboard](https://github.com/1Password/arboard) — 系统剪贴板

## 许可证

[MIT](LICENSE)
