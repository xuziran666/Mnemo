# Phase 1：抽离纯 Rust 核心库（mnemo_core）

## 背景与目标

Mnemo 目标是从 Tauri（Rust 后端 + React 前端）迁移到纯 Rust（egui）桌面应用。
采用逐步迁移策略，Phase 1 的目标是：

> 在不破坏现有 Tauri 版本的前提下，把数据层与业务逻辑抽离为不依赖 Tauri 的纯 Rust 核心库，供后续 egui 版本直接复用。

## 改动内容

### 新增文件

| 文件 | 作用 |
|---|---|
| `src-tauri/src/mnemo_core/mod.rs` | 核心库门面（facade），对外暴露统一的 DB/CRUD/导入导出 API |
| `src-tauri/src/mnemo_core/models.rs` | 数据模型：`Command`、`NewCommand`、`ImportResult`、`ExportData`、`ExportCommand` 与 `KIND_*` 常量 |
| `src-tauri/src/mnemo_core/db.rs` | SQLite 打开、schema 初始化与兼容迁移（含原有单元测试） |
| `src-tauri/src/mnemo_core/service.rs` | 纯业务逻辑：搜索、增删改查、导入导出，全部基于 `&Connection` |

### 修改文件

| 文件 | 改动 |
|---|---|
| `src-tauri/src/lib.rs` | 仅保留 Tauri 启动/插件注册/setup 与 `invoke_handler`；setup 改用 `mnemo_core::open_db` |
| `src-tauri/src/commands.rs` | 保留 `Db` 类型与 6 个 `#[tauri::command]` 处理器；处理器内部改为调用 `mnemo_core` 门面 |

### 目录结构（Phase 1 后）

```
src-tauri/src/
├── lib.rs              # Tauri 入口（薄壳）
├── main.rs             # 二进制入口
├── commands.rs         # Tauri command 处理器 + Db 连接包装
├── db.rs               # ⚠ 旧文件，已被 mnemo_core/db.rs 取代，暂未删除
└── mnemo_core/         # 纯 Rust 核心库（不依赖 Tauri）
    ├── mod.rs          # 门面 API
    ├── models.rs       # 数据模型
    ├── db.rs           # SQLite 连接与迁移
    └── service.rs      # 业务逻辑
```

## 关键设计决策

1. **模块命名为 `mnemo_core` 而非 `core`**
   `core` 是 Rust 保留标识符（指向标准库 `core` crate），直接 `mod core;` 会引发命名冲突与解析错误，故命名为 `mnemo_core`。

2. **Tauri 层只做薄封装**
   `commands.rs` 中的处理器只负责加锁 `State<Db>` 并转发给 `mnemo_core` 门面。这样未来 egui 版本可直接调用 `mnemo_core`，无需任何 Tauri 依赖。

3. **业务函数接收 `&Connection` 而非 `State<Db>`**
   `service` 层签名如 `list(conn: &Connection, query: Option<&str>)`，与框架解耦，便于单元测试和跨框架复用。

4. **错误类型保持 `Result<_, String>`**
   沿用原有约定，避免为 Phase 1 引入自定义错误类型而扩大改动范围；后续如需可再引入 `thiserror`。

5. **保留原有单元测试**
   `mnemo_core/db.rs` 内保留 3 个迁移测试（新建库、旧库迁移、command→content 重命名），确保迁移逻辑未在搬运中损坏。

## 验证

- `cargo check` 通过，**零 warning**。
- 旧的 React 前端 API（`list_commands` / `create_command` / `update_command` / `delete_command` / `export_commands` / `import_commands`）签名与行为保持不变，前端无需改动。

> 注：按项目规则，未主动运行 `cargo test`。如需验证迁移逻辑，请手动执行：
> `cd src-tauri && cargo test`

## 遗留事项

1. `src-tauri/src/db.rs` 已成为孤儿文件（无 `mod db;` 声明，不参与编译）。建议在 Phase 4 清理 Tauri 时一并删除；因涉及删除文件，此处暂不处理。
2. 尚未引入 egui 依赖，Phase 2 将新增 egui 原生二进制。

## 相关文件

- `src-tauri/src/mnemo_core/mod.rs`
- `src-tauri/src/mnemo_core/models.rs`
- `src-tauri/src/mnemo_core/db.rs`
- `src-tauri/src/mnemo_core/service.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands.rs`

## 下一步（Phase 2）

新增 egui 依赖与原生二进制入口（并存于 Tauri 版本），实现最小可运行窗口，验证 egui 工具链与中文字体方案。
