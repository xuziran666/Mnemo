# Phase 2：新增 egui 原生应用骨架（与 Tauri 版并存）

## 背景与目标

Phase 1 已把数据层/业务逻辑抽离为不依赖 Tauri 的 `mnemo_core`。
Phase 2 的目标是：

> 引入 egui 工具链，建立一个可独立运行的原生窗口应用 `mnemo-egui`，验证 egui 编译链与中文字体渲染方案，且**不修改、不破坏现有 Tauri 版本**。

## 关键决策

### 1. 新建独立 crate，而非在 Tauri crate 内加 bin

计划阶段曾考虑在 `src-tauri/src/bin/` 下新增 egui 入口，实际实现改为**独立 crate `mnemo-egui/`**，原因：

- Tauri 的 lib 是 `staticlib/cdylib/rlib`，在同 crate 内加 egui bin 会把整个 Tauri/tao 栈链接进 egui 二进制，体积与构建时间浪费；
- Tauri（tao）与 eframe（winit）属于两套窗口/事件循环实现，放在同一二进制存在潜在运行时冲突；
- 独立 crate 与 Phase 4「删除 Tauri」的目标天然契合，后续只需让 `mnemo-egui` 依赖 `mnemo-core`。

### 2. 中文字体方案：运行时按优先级解析

egui 内置字体仅含拉丁字符，中文会显示为方框。实现 `fonts.rs` 按优先级解析：

1. 环境变量 `MNEMO_FONT` 指定的字体文件；
2. 打包字体 `assets/fonts/NotoSansSC-Regular.ttf`（可执行文件同级 / 工作目录 / 源码目录）；
3. 各平台系统字体：Windows（微软雅黑 `msyh.ttc`、黑体 `simhei.ttf`）、macOS（苹方 `PingFang.ttc`）、Linux（Noto CJK、文泉驿）。

采用「运行时解析 + 系统回退」而非编译期 `include_bytes!` 的原因：`include_bytes!` 要求字体文件在编译前存在，否则 crate 无法编译；运行时方案可立即构建运行，同时保留放置 `NotoSansSC-Regular.ttf` 即优先加载的打包能力。字体作为回退追加在默认字体之后（`Proportional`/`Monospace`），拉丁字符仍使用 egui 默认字体。

> 版本注意：`egui::FontData` 含 `index` 字段，可正确处理 `.ttc` 字体集合（取 face 0），系统 `.ttc` 字体可用。

### 3. 复用 Tauri 的数据库路径

`paths.rs` 使用 `dirs::data_dir()` 拼接 `com.longanl.mnemo`，与 Tauri `app_data_dir()` 在三平台语义完全一致：

| 平台 | 路径 |
|---|---|
| Windows | `%APPDATA%\com.longanl.mnemo\commands.db` |
| macOS | `~/Library/Application Support/com.longanl.mnemo/commands.db` |
| Linux | `$XDG_DATA_HOME` 或 `~/.local/share/com.longanl.mnemo/commands.db` |

## 改动内容

### 新增文件

| 文件 | 作用 |
|---|---|
| `mnemo-egui/Cargo.toml` | 独立 crate 配置（eframe 0.36、dirs 7） |
| `mnemo-egui/.gitignore` | 忽略 `/target/` |
| `mnemo-egui/src/main.rs` | egui 应用入口，最小窗口：标题、搜索框、中英文文本、字体来源、数据库路径 |
| `mnemo-egui/src/fonts.rs` | CJK 字体解析（env → 打包 → 系统回退） |
| `mnemo-egui/src/paths.rs` | 复用 Tauri 的应用数据目录与 `commands.db` 路径 |
| `mnemo-egui/assets/fonts/README.md` | 打包字体放置说明 |

### 未改动

- `src-tauri/`（Tauri 版本）完全未动，仍可正常构建；
- `src/`（React 前端）未动。

## 依赖变更

新增 crate `mnemo-egui` 的依赖（**仅修改配置文件，未执行安装**）：

```toml
eframe = "0.36"
dirs = "7"
```

## 验证方式

Phase 2 未接入 `mnemo_core`，属骨架验证阶段。按项目规则，本次未主动执行构建；请手动运行：

```powershell
cd mnemo-egui
cargo run
```

验收点：
1. 弹出标题为 `Mnemo` 的窗口（900×650，最小 500×400）；
2. 中文文本正常显示（非方框）；界面底部显示「字体来源」；
3. 「数据库路径」显示 `...\com.longanl.mnemo\commands.db`，与 Tauri 版一致。

> 若需打包字体，将 `NotoSansSC-Regular.ttf` 放入 `mnemo-egui/assets/fonts/` 即可自动优先加载。

## 遗留事项 / 下一步

- 本阶段为最小窗口，尚未接入 `mnemo_core` 与完整交互（Phase 3 完成）；
- Phase 3 将处理 `mnemo-egui` 对核心库的依赖（届时把 `mnemo_core` 抽为独立 crate，双方通过 path 依赖），并实现列表/搜索/编辑/查看与键盘快捷键。

## 相关文件

- `mnemo-egui/Cargo.toml`
- `mnemo-egui/src/main.rs`
- `mnemo-egui/src/fonts.rs`
- `mnemo-egui/src/paths.rs`
- `mnemo-egui/assets/fonts/README.md`
