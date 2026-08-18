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
- **查看模式** — `Enter` 以 Quick Look 方式查看知识笔记（只读、无光标、无工具栏）；查看器内 `Enter` 原位进入编辑，`Ctrl+S` 保存，`Esc` 返回。每条内容分为**代码片段**（用于复制）与**知识笔记**（用于查看）两种类型。知识笔记支持富 Markdown 渲染：GFM 表格、KaTeX 数学公式、代码高亮。

## 截图

![Mnemo](docs/screenshots/main.png)

## 安装

### 预编译安装包

从 [GitHub Releases](https://github.com/xuziran666/Mnemo/releases) 下载：

- **Linux**：AppImage / deb / rpm（x86_64 & aarch64）
- **Windows**：exe / msi
- **macOS**：dmg / app

### Arch Linux（AUR）

即将上架：`mnemo-cm`（源码编译）、`mnemo-cm-bin`（预编译二进制）。

### 源码构建

需要 [Node.js](https://nodejs.org) 18+ 与 [Rust](https://rustup.rs)（stable）。

**Linux** 另需安装 [Tauri 系统依赖](https://tauri.app/start/prerequisites/)（如 `libwebkit2gtk-4.1-dev`、`librsvg2-dev` 等）。

```bash
npm install
npm run tauri build
```

## 快捷键

| 按键 | 功能 |
|---|---|
| `s` | 聚焦搜索框 |
| `↑` / `↓` | 在列表中移动选择 |
| `Enter` | 复制选中代码片段并关闭窗口，或打开知识笔记查看 |
| `c` | 复制选中代码片段并关闭窗口（仅代码片段） |
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
npm install          # 安装依赖
npm run tauri dev    # 热重载开发
npm run tauri build  # 构建发布包
```

## 数据存储

数据保存在系统 app data 目录下的单个 SQLite 数据库文件（`commands.db`）中：

| 系统 | 路径 |
|---|---|
| Linux | `~/.local/share/com.longanl.mnemo/commands.db` |
| macOS | `~/Library/Application Support/com.longanl.mnemo/commands.db` |
| Windows | `%APPDATA%\com.longanl.mnemo\commands.db` |

备份该文件即可迁移你的全部命令。

## 技术栈

- [Tauri 2](https://tauri.app) + [Rust](https://www.rust-lang.org)
- [React 19](https://react.dev) + [TypeScript](https://www.typescriptlang.org) + [Vite](https://vitejs.dev)
- [SQLite](https://www.sqlite.org)（通过 [rusqlite](https://github.com/rusqlite/rusqlite) 集成）

## 许可证

[MIT](LICENSE)
