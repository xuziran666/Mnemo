# 列表接口裁剪 content + 详情接口取全文，以及由此引入的数据丢失陷阱

## 问题现象

搜索是"每次按键都跑一次"的高频路径，但 `list_commands` 一次返回**所有行**的完整 `content`，
前端 `CommandItem` 又把完整正文塞进 `<pre class="item-command">`（无行数截断）。

库变大后每敲一个字符都要传输并渲染全量正文，界面明显卡顿。

## 问题原因

- `commands.rs` 的 `list_commands` 没有 `LIMIT`，`SELECT` 里直接取原始 `content` 列。
- `App.css` 的 `.item-command` 只有 `white-space: pre-wrap`，没有行数上限。
- 所有 Tauri command 都是同步 `fn`，在 Tauri v2 中同步 command 在**主线程**执行，
  全表扫 + 大字符串序列化会直接阻塞 UI 事件循环。
- `created_at` 上没有索引，`ORDER BY created_at DESC` 每次都是全表扫 + 排序。

本质是：列表渲染只需要"预览"，但接口按"详情"的粒度返回数据。

## 解决方案

### 1. SQL 层裁剪（后端）

`commands.rs` 新增预览表达式，只作用于列表接口：

```rust
const CONTENT_PREVIEW_CHARS: usize = 2000;

fn list_content_expr() -> String {
    format!(
        "CASE WHEN length(content) > {CONTENT_PREVIEW_CHARS} \
         THEN substr(content, 1, {CONTENT_PREVIEW_CHARS}) || '…' ELSE content END"
    )
}
```

列表两个分支改为 `SELECT id, title, {expr} AS content, ...`。

要点：
- `WHERE` 子句仍用原始 `content` 列（SQLite 不允许在 `WHERE` 里引用 SELECT 别名），
  所以**搜索依然能命中 2000 字符之后的关键词**，只是列表行显示的是开头预览。
- 用 `CASE WHEN` 而非 `substr(...) || '…'`，否则短于上限的记录也会被加上省略号。
- `length()` / `substr()` 对 TEXT 按字符计数，不会切坏多字节字符。

### 2. 新增详情接口

`get_command(id)` 返回完整 `content`，`create_command` / `update_command` 的回读同样保持完整。
注册到 `lib.rs` 的 `invoke_handler`，前端加 `getCommand()`。

### 3. 前端取全文后再操作

`App.tsx` 的打开 / 编辑 / 复制三条路径都先 `fetchFull(cmd)` 取回完整记录。

### 4. 索引

`db.rs` 的 `init_schema` 建复合索引，与 `ORDER BY` 完全对齐以避免额外排序：

```sql
CREATE INDEX IF NOT EXISTS idx_commands_created_at ON commands(created_at DESC, id DESC);
```

实测 `EXPLAIN QUERY PLAN` 输出 `SCAN commands USING INDEX idx_commands_created_at`。

## 关键陷阱：取全文失败时的回退会毁数据

第一版 `fetchFull` 写成了：

```ts
try { return await getCommand(cmd.id); } catch { return cmd; }   // ← 危险
```

这个 fallback 看起来无害，实际是**数据丢失路径**：`getCommand` 失败时返回列表里的
截断记录，用户在编辑器里一保存，`update_command` 就把被截断的正文写回数据库，原文被覆盖。

最终改为失败即中止：

```ts
async function fetchFull(cmd: Command): Promise<Command | null> {
  try { return await getCommand(cmd.id); } catch { return null; }
}
```

三个调用方各自判断 `null` 后 `showToast(t("toast.loadFailed"))` 并 return，不进入编辑态。

## 为什么采用这个方案

- **在 SQL 层裁剪而不是 CSS 裁剪**：CSS 只能省下渲染成本，省不下 IPC 传输和 JSON 序列化成本。
  而搜索是高频路径，传输量是主要瓶颈。
- **不复用 `list_commands` 的结果再补一次前端拼接**：预览与全文混在同一个结构里，
  前端无法区分"这就是全文"和"这是预览"，迟早会误用。
- **拆出 `get_command` 而不是给列表加 `full_content` 标志位**：多一个布尔字段意味着
  每个调用方都要写分支判断，且默认行为容易搞反。独立接口让"要全文"变成显式动作。
- **索引列顺序与 `ORDER BY` 一致**：只索引 `created_at` 时，同值行仍需额外排序。

## 解决了什么问题

- 列表传输量与渲染量从 O(库大小 × 正文长度) 降到 O(库大小 × 2000 字符)。
- 列表排序从全表扫 + 排序变成索引扫描。

副作用与遗留：
1. 搜索命中位置在 2000 字符之后时，列表行看不出为什么匹配（预览只显示开头）。
   要彻底解决需要在预览里高亮命中片段，属独立需求。
2. 打开/编辑/复制各多一次 IPC 往返。相对一次数据库读取可忽略。
3. 同步 command 跑主线程的问题**未解决**：大库下 `list_commands` 仍可能卡顿。
   根治需要把 command 改成 `async fn` + `tauri::async_runtime::spawn_blocking`。

## 相关文件

- `src-tauri/src/commands.rs` — `CONTENT_PREVIEW_CHARS` / `list_content_expr` / `get_command`
- `src-tauri/src/lib.rs` — 注册 `get_command`
- `src-tauri/src/db.rs` — `idx_commands_created_at`
- `src/api.ts` — `getCommand`
- `src/App.tsx` — `fetchFull` / `handleOpen` / `handleEdit` / `handleCopy`
- `src/i18n/locales/{en,zh}.json` — `toast.loadFailed`
