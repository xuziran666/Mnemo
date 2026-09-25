# Phase 3：mnemo_core 独立化 + egui 完整 UI

## 背景与目标

Phase 2 已建立可运行的 egui 骨架，但未接入数据层。Phase 3 的目标是：

> 把 `mnemo_core` 抽为独立 crate，`mnemo-egui` 通过 path 依赖接入，实现与 Tauri 版功能对等的完整界面：列表、搜索、编辑、查看、导入导出、剪贴板、键盘快捷键与中英双语。

同时按用户确认，删除已被取代的孤儿文件 `src-tauri/src/db.rs`。

## 改动内容

### 1. `mnemo_core` 抽为独立 crate

- 新增 `mnemo-core/Cargo.toml`（rusqlite / serde / serde_json）
- `src-tauri/src/mnemo_core/{mod,models,db,service}.rs` 迁移为 `mnemo-core/src/{lib,models,db,service}.rs`
- `lib.rs` 增加 `pub use rusqlite;`，让上层持有 `Connection` 无需重复声明依赖
- `service.rs` 内部路径由 `crate::mnemo_core::models` 改为 `crate::models`
- 删除 `src-tauri/src/mnemo_core/` 与 `src-tauri/src/db.rs`

### 2. `src-tauri` 改为 path 依赖

- `Cargo.toml` 新增 `mnemo-core = { path = "../mnemo-core" }`
- `lib.rs` 移除 `mod mnemo_core;`，`commands.rs` 的 `crate::mnemo_core::*` 改为 `mnemo_core::*`
- 原有 6 个 `#[tauri::command]` 与 React 前端 API 完全不变

### 3. `mnemo-egui` 完整 UI

| 文件 | 作用 |
|---|---|
| `src/app.rs` | 应用状态机与全部视图（列表/编辑/查看/确认弹窗/Toast/快捷键/导入导出/剪贴板） |
| `src/i18n.rs` | 轻量 i18n：嵌套 JSON 展平为 `a.b.c` 映射，支持 `{{name}}` 插值 |
| `src/locales/en.json`、`zh.json` | 语言资源（从原前端复制，新增 `deleteFailed`） |
| `src/main.rs` | 入口（eframe 0.36 `App::ui`） |
| `Cargo.toml` | 新增 `serde_json`、`rfd 0.17`、`arboard 3`、`mnemo-core` |

### 功能对照（与 Tauri 版）

| 能力 | 实现 |
|---|---|
| 列表 + 搜索 | 搜索框 `changed()` 触发 `mnemo_core::list`，实时刷新 |
| 选中高亮 / 键盘导航 | `selected` + `ScrollArea` + `scroll_to_me` |
| `s` 聚焦搜索 | `Memory::request_focus` |
| `↑/↓`、`Enter`、`r/c/v/d` | `ctx.input` + `list_active` 状态，行为与原版一致 |
| `Ctrl/Cmd+N` 新建 | 全局快捷键 |
| `Esc` 关闭窗口 | `ViewportCommand::Close` |
| 复制并关闭 | `arboard` + `ViewportCommand::Close` |
| 编辑 / 保存 | `mnemo_core::create/update`，`Ctrl+S` 保存，`Esc` 取消 |
| 查看 | Note 以普通文本展示（按既定方案不做 Markdown） |
| 删除确认 | `egui::Window` 模态确认 |
| 导入 / 导出 | `rfd` 原生文件对话框 + `std::fs`；JSON 格式不变 |
| 中英切换 + 记忆 | `settings.json` 持久化语言选择 |
| Toast | `egui::Area` 底部居中，1.5s 自动消失 |

## 关键设计决策

1. **`mnemo-core` 独立 crate + path 依赖**
   避免在 `mnemo-egui` 中复制核心逻辑，也避免 `mnemo-egui` 链接 Tauri。后续 Phase 4 删除 Tauri 时，`mnemo-core` 不受影响。

2. **eframe 0.36 破坏性 API 变更**
   0.36 的 `eframe::App` 不再有 `update(&mut self, ctx, frame)`，改为必需的 `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)`；`CentralPanel::show` 也由接收 `&Context` 改为接收根 `&mut Ui`（`Window`/`Area` 仍接收 `&Context`）。已按新 API 实现，`ctx` 通过 `ui.ctx().clone()` 获取。

3. **Note 降级为纯文本**
   按用户既定方案，`kind == KIND_NOTE` 在查看器中以普通文本渲染，不引入 Markdown 渲染依赖。

4. **语言持久化**
   `settings.json` 存于应用数据目录；启动时优先读取，缺失则按系统 locale 判断，最后回退英文。

## 验证

- `mnemo-core`：`cargo check --offline` 通过，零 warning
- `src-tauri`：`cargo check --offline` 通过，零 warning（抽取未破坏 Tauri 版）
- `mnemo-egui`：`cargo check` 通过，零 warning

> 说明：为验证本次改动可编译，执行了上述 `cargo check`（`mnemo-egui` 因此下载了 eframe 依赖）。未执行 `cargo test` / `cargo run`。

## 遗留事项

1. **窗口高度自适应**：原 Tauri 版编辑时会动态增高窗口，egui 版暂未实现（可用 `ViewportCommand::InnerSize` 后续补上），不影响功能。
2. **Markdown 渲染**：按既定方案未实现（Note 为纯文本）。
3. **重复导入去重**：与原版一致，导入仅跳过空标题/内容，不做去重。
4. Phase 4 将删除 Tauri/React 相关文件，并把根目录整理为纯 Rust 工程。

## 相关文件

- `mnemo-core/Cargo.toml`、`mnemo-core/src/{lib,models,db,service}.rs`
- `src-tauri/Cargo.toml`、`src-tauri/src/{lib,commands}.rs`
- `mnemo-egui/Cargo.toml`、`mnemo-egui/src/{main,app,i18n,fonts,paths}.rs`
- `mnemo-egui/locales/{en,zh}.json`
