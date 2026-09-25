# mnemo-core 数据完整性、JSON 导入导出与原生文件对话框

## 问题现象 / 背景

两个问题域放在一起记录：

1. **数据完整性缺口**（批次 C）：`mnemo-core` 的写操作缺少必要校验/事务——
   - `delete` 不检查影响行数：id 不存在也返回 `Ok`，UI 会弹「已删除」但库里什么都没变；
   - `create`/`update` 不校验 `kind`：非法值直接入库，UI 按 `kind` 分支渲染时会出现"未知类型"；
   - `import` 逐条 INSERT 且**没有事务**：中途失败会留下半份数据。
2. **导入/导出未接入**（批次 B）：工具栏两个按钮只打日志，全部 7 条相关文案（`io.*Filter`、`toast.imported/importSkipped/importFailed/exported/exportFailed`）备而不用。

## 解决方案与依据

### C1. `delete` 校验影响行数

```rust
let deleted = conn.execute("DELETE FROM commands WHERE id = ?1", [id]).map_err(...)?;
if deleted == 0 { return Err(format!("command {id} not found")); }
```

与 `update` 已有的 `updated == 0` 检查对齐：目标不存在时返回明确错误，
UI 侧（`toast.deleteFailed`）才会如实提示，而不是误报成功。

### C2. `kind` 归一化（而不是报错）

```rust
fn normalize_kind(kind: i64) -> i64 {
    if kind == KIND_NOTE { KIND_NOTE } else { KIND_SNIPPET }
}
```

`create` / `update` / `import` 三处写入前统一归一化。选择"归一化"而非"报错"的原因：
导入外部备份时，`kind` 写错不该导致整条数据被丢弃（`import` 本来就有"跳过空 title/content"的容错取向）；
对本应用内部的 UI 而言，它只会传 1/2，归一化是零成本的兜底。

### C3. `import` 用 `unchecked_transaction()` 而不是 `transaction()`

`rusqlite::Connection::transaction()` 需要 `&mut self`，而：
- `service::import` 的签名是 `&Connection`；
- 上层 `mnemo-slint::data::AppData` 把连接放在普通字段（不是 `RefCell`），且整份数据用 `Rc<AppData>` 共享，
  根本拿不到 `&mut Connection`。

改成 `&mut` 会连带改 lib.rs 与 UI 的持有方式。因此用 `conn.unchecked_transaction()`（rusqlite 0.31 提供，
`transaction.rs:466`）：**同样是事务语义**，出错时 `Transaction` 被 drop 会自动回滚，
只是把"借用检查"让位给调用者自觉（本项目单线程、单连接，风险为零）。

### C4. 补 `service.rs` 单元测试（原先 0 个）

11 个测试，全部用内存库 + 与生产同一套建表逻辑（`db::init_schema` 从私有改为 `pub(crate)`，
避免测试里复制 schema）。覆盖点：

- `create` 返回插入行 + `list` 能看到；
- `kind` 归一化（`99` → Snippet，`-1` → Snippet，`2` → Note）；
- `update` 保留 `created_at` + 目标不存在报错；
- `delete` 目标不存在报错（新行为）；
- `list` 过滤覆盖 title/content/note/tags，空白关键词等价于不过滤；
- **`list` 的 LIKE 转义**：造 `50% / 5012`、`a_b / aXb` 两组对照数据——
  不转义时 `50%` 会命中 `5012`、`a_b` 会命中 `aXb`，测试就会失败（把 `escape_like` 钉住）；
- 排序稳定为 `created_at DESC`（用显式 `created_at` 插入，避免同秒歧义）；
- `import`：跳过空 title/content、`kind` 归一化、非法 JSON 不影响既有数据；
- **`import` 事务回滚**：临时 `CREATE UNIQUE INDEX` 让第二条 INSERT 必然失败，
  断言第一条也必须回滚（`list(...).is_empty()`）——这是"没有事务就必然失败"的可判定测试；
- `export` → `import` 往返：字段（含 `created_at`）完整保留。

### B1. rfd 依赖

`rfd = "0.17"`，与 `mnemo-egui` **同版本**：既避免 lockfile 里出现两个版本，
也因为该 crate 已在 Cargo.lock / 本地缓存中，`cargo check` 无需联网。

### B2. 对话框期间的失焦抑制（关键）

`rfd` 的对话框是**阻塞式**的（调用期间主线程进入对话框自己的消息循环，与旧版 egui 一致），
且会让主窗口**真失焦** —— 不抑制的话，窗口会在对话框弹出后被"失焦自动隐藏"藏起来。

因此 `WindowFlags` 增加 `dialog_open`，并且**必须在调用对话框之前置位、返回后立即复位**：

```rust
set_dialog_open(&flags, true);
let path = rfd::FileDialog::new().add_filter(filter, &["json"]).save_file();
set_dialog_open(&flags, false);
```

失焦隐藏的判定相应变为：`armed && tray_ready && !minimizing && !dialog_open`。
（原先这里留的 TODO 已删除——它现在真的实现了。）

### B3. 文案与交互细节

- 7 条原本未使用的 key 现在用上了 6 条（`io.exportFilter`/`io.importFilter` 作为对话框过滤器名，
  `toast.imported`/`importSkipped`/`importFailed`/`exported`/`exportFailed` 作为结果提示）；
  导入成功且有条目被跳过时用 `toast.importSkipped`（带 `{{imported}}`/`{{skipped}}` 插值），与旧版完全一致。
- **用户取消对话框不弹 Toast**（不打扰）；只有失败才提示。
- 导入成功后按**当前搜索条件**刷新（复用 `data::refresh_current`，与删除/保存一致）。

### B4. 清理

- 删除 `install_placeholder_callbacks`：导入/导出已接入、查看弹层已做，**不再有"只打日志"的回调**；
- `card-activated`（点击卡片正文）刻意**不注册 Rust 处理函数**：它只用于选中，
  不沿用旧版"点正文即复制/查看"的行为（用户已确认，避免误关窗）。`main.slint` 里加了说明注释。

## 解决了什么问题

- `mnemo-core` 的三个数据完整性缺口全部补齐，且被测试钉住（跨 crate 的符号都没变，`mnemo-egui` 无需改动即可编译）。
- 导入/导出打通：导出全库 JSON、导入带回滚与跳过统计，两条路径都有 Toast 反馈。

## 遗留 / 可选后续

- `service::create`/`update` 仍未校验 title/content 非空（UI 已拦；若要让 core 成为唯一防线可再补）；
- `toast.copied` 仍未使用（复制成功后窗口立即隐藏，弹了也看不见）；
- `theme.toDark`/`theme.toLight`、`viewer.loading` 仍未使用（主题按钮用图形符号，查看层是同步渲染）；
- 「文案是否使用」的快速核对脚本用的是正则，**不识别跨行调用**
  （如 `t_args(\n    "toast.importSkipped",`），会把这类 key 误报为"未使用"——核对时需人工确认；

## 相关文件

- `mnemo-core/src/service.rs`（delete 影响行数 / normalize_kind / import 事务 / 11 个测试）
- `mnemo-core/src/{models,db}.rs`（KIND_NOTE 注释与 `init_schema` 可见性）
- `mnemo-slint/Cargo.toml`（新增 `rfd = "0.17"`）
- `mnemo-slint/src/data.rs`（`export()` / `import()` 包装）
- `mnemo-slint/src/main.rs`（`WindowFlags::dialog_open`、`install_io_callbacks`、删除占位回调）
- `mnemo-slint/ui/main.slint`（`card-activated` 语义说明）
