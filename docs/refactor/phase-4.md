# Phase 4：切换为纯 Rust 工程

> 历史记录：本文描述的是已下线的 egui 版前端（`mnemo-egui`）。该 crate 已从仓库移除，
> 文中的代码路径可能不再存在；需要对照时用 `git show legacy-egui:mnemo-egui/<path>` 取回。
> 当前实现是 `mnemo-slint/`（Slint UI），`mnemo-core/` 保持本文所述的分层不变。

## 背景与目标

Phase 1–3 已完成核心抽离与 egui 完整 UI，但仓库仍保留 Tauri/React 工程。Phase 4 的目标是：

> 删除 Tauri/React 相关文件与依赖，把仓库整理为纯 Rust（Cargo workspace）工程。

## 删除内容

经用户确认，删除以下前端与 Tauri 文件（均通过 `git rm`，可从历史恢复）：

- 前端：`src/`、`index.html`、`vite.config.ts`、`tsconfig.json`、`tsconfig.node.json`、`public/`
- Tauri：`src-tauri/`（`Cargo.toml`、`build.rs`、`tauri.conf.json`、`capabilities/`、`icons/`、`src/`）
- 包管理：`package.json`、`pnpm-lock.yaml`、`pnpm-workspace.yaml`
- 未跟踪产物：`node_modules/`、`dist/`、各 crate 的 `target/`、`src-tauri/target/`

## 保留内容

- `src-tauri/icons/` 中的应用图标全部迁移至 `mnemo-egui/assets/icons/`，避免丢失品牌资源。
- `README` 中原有截图 `docs/screenshots/` 保留。

## 新增内容

- 根 `Cargo.toml`：虚拟 workspace，成员 `mnemo-core`、`mnemo-egui`
- 根 `Cargo.lock`：workspace 统一锁定（删除成员级 `mnemo-egui/Cargo.lock`）
- `mnemo-egui/assets/icons/`：迁移后的图标

## 修改内容

| 文件 | 改动 |
|---|---|
| `mnemo-egui/src/main.rs` | 用 `eframe::icon_data::from_png_bytes` 加载 `assets/icons/icon.png` 并设置窗口图标 |
| `.gitignore` | 移除前端相关忽略，改为 `/target` 与 Rust/编辑器忽略规则 |
| `mnemo-core/.gitignore` | 去掉 `Cargo.lock`（workspace 统一在根锁定） |
| `.vscode/tasks.json` | 由 `pnpm run tauri dev` 改为 `cargo run -p mnemo-egui` / `cargo build --release -p mnemo-egui` |
| `.vscode/extensions.json` | 仅推荐 `rust-lang.rust-analyzer` |
| `.vscode/settings.json` | 移除 TypeScript/Vite 文件嵌套规则 |
| `.github/workflows/ci.yml` | 改为 `cargo fmt` / `clippy` / `test` / `build`（workspace） |
| `.github/workflows/release.yml` | 移除 tauri-action，改为 `cargo build --release -p mnemo-egui` + `softprops/action-gh-release` 上传可执行文件 |
| `README.md` / `README.zh-CN.md` | 更新为纯 Rust 技术栈、构建方式与项目结构；Note 描述改为纯文本 |

## 关键设计决策

1. **Cargo workspace**
   根目录统一管理两个 crate，`cargo build`/`test` 直接作用于整个 workspace，`target/` 与 `Cargo.lock` 集中在根目录，符合纯 Rust 工程惯例。

2. **窗口图标无需新增依赖**
   eframe 提供 `icon_data::from_png_bytes`（内部用 image crate 解码），可直接把现有 PNG 作为窗口图标，避免为图标额外引入并声明依赖。

3. **发布产物为原生可执行文件**
   当前 Release 上传各平台 `target/release/mnemo` 二进制。相比 Tauri 的安装包（msi/dmg/AppImage），这是更简单的形式；如需安装包/AppImage 可后续引入 `cargo-bundle`、`cargo-wix` 等工具。

## 验证

- 根目录 `cargo check --workspace` 通过，零 warning（仅提示 rusqlite 存在更新版本，未升级）。

> 按项目规则未运行 `cargo test` / `cargo run`。

## 当前工程结构

```
mnemo/
├── Cargo.toml            # workspace
├── Cargo.lock
├── mnemo-core/           # SQLite 持久化与业务逻辑（不依赖 GUI）
│   └── src/{lib,models,db,service}.rs
├── mnemo-egui/           # egui/eframe 桌面应用
│   ├── src/{main,app,i18n,fonts,paths}.rs
│   ├── locales/{en,zh}.json
│   └── assets/{fonts,icons}/
├── docs/
└── README.md / README.zh-CN.md
```

## 遗留事项

1. **窗口高度自适应**：编辑时动态调整窗口高度尚未实现（可用 `ViewportCommand::InnerSize`）。
2. **Markdown 渲染**：按既定方案未实现，Note 为纯文本。
3. **发布打包**：当前仅输出可执行文件，未生成安装包；如需可引入打包工具。
4. CI 的 Linux 系统依赖列表与 runner 标签建议在首次实际运行时校验。

## 相关文件

- `Cargo.toml`、`Cargo.lock`
- `mnemo-egui/src/main.rs`、`mnemo-egui/assets/icons/`
- `.gitignore`、`.vscode/*`、`.github/workflows/*`
- `README.md`、`README.zh-CN.md`
