# 导入没有事务，且反序列化过严导致 skipped 机制形同虚设

## 问题现象

1. 导入一份较大的备份文件，中途失败（例如磁盘满、数据库被锁），库里留下**导入了一半**的数据，
   且界面提示的 `skipped` 数量与实际写入对不上。
2. 备份文件里只要有**一条**记录缺 `title` 或 `content` 字段，整个导入直接失败并提示
   `invalid JSON`，其他完全正常的记录也导不进去 —— 但提示文案和 `skipped` 字段的存在
   又暗示"坏数据应该被跳过"。

## 问题原因

**问题 1：`import_commands` 逐条 `execute`，没有事务。**

```rust
for item in data.commands {
    if ... { skipped += 1; continue; }
    conn.execute("INSERT INTO commands ...",
        .map_err(|e| e.to_string())?;   // ← 中途失败直接返回，已写入的不回滚
    imported += 1;
}
```

SQLite 默认每条语句一个隐式事务，逐条 execute 之间没有原子性。
`imported` / `skipped` 是纯内存计数，函数返回 `Err` 时它们也一起被丢弃，
用户既不知道已经写进去多少，也没有办法回退。

**问题 2：`ExportCommand` 的 `title` / `content` 是必填字段。**

```rust
pub struct ExportCommand {
    pub title: String,      // ← 没有 #[serde(default)]
    pub content: String,    // ← 没有 #[serde(default)]
    ...
}
```

serde 的 derive 对缺字段的默认行为是**整条记录反序列化失败**，向上冒泡成
`serde_json::from_str` 的错误。而 `note` / `tags` / `created_at` 因为类型是
`Option<T>`，serde 会隐式当作可选处理 —— 所以只有 title/content 会触发。

于是 `import_commands` 里的容错判断：

```rust
if item.title.trim().is_empty() || item.content.trim().is_empty() {
    skipped += 1;
    continue;
}
```

只能覆盖"字段存在但值为空字符串"这一种情况，永远覆盖不到"字段缺失"。
注释写的"跳过空内容或缺失关键字段的数据"中，后半句从未生效。

## 解决方案

### 1. 包事务

```rust
let mut conn = state.lock().map_err(|e| e.to_string())?;
let tx = conn.transaction().map_err(|e| e.to_string())?;
// ... tx.execute(...)
tx.commit().map_err(|e| e.to_string())?;
```

`Connection::transaction()` 需要 `&mut self`，`MutexGuard<Connection>` 通过
`DerefMut` 满足（因此 `conn` 要声明为 `mut`）。事务对象在 `commit()` 前被 drop
会自动 rollback，错误提前返回也不会留下半套数据。

### 2. 字段容错

```rust
#[serde(default)]
pub title: String,
#[serde(default)]
pub content: String,
```

`String` 的 `Default` 是空串，所以缺字段的记录反序列化后是 `""`，
正好被既有的 `trim().is_empty()` 判断捕获并计入 `skipped`。
容错判断和 `skipped` 语义自此一致。

`note` / `tags` / `created_at` 保持原样（`Option<T>` 已被 serde 隐式可选），
`kind` 已有 `#[serde(default = "default_kind")]`。

注意 `ExportData.commands` **不加** `#[serde(default)]`：
缺整个数组说明这不是本应用导出的文件，属于格式错误，应该明确报错而不是静默导入 0 条。

## 为什么采用这个方案

- **`#[serde(default)]` 而不是给每条记录单独 `try_into` / 收集错误**：
  后者要重写整个解析流程（先解析成 `Vec<serde_json::Value>` 再逐条尝试），
  收益只是能区分"缺字段"和"空字符串"两种跳过原因，而 UI 上两者都只体现为一个 `skipped` 数字。
- **保留 JSON 语法错误为硬失败**：文件损坏和"内容不完整"是两类问题，
  前者必须让用户知道，后者应该尽量抢救。
- **事务而非"先校验全部再逐条写"**：预校验无法覆盖并发写入和磁盘故障，
  事务是唯一能保证原子性的手段。

## 解决了什么问题

- 导入要么全部成功，要么完全不变，不再有半套数据。
- 单条记录缺 `title` / `content` 会被跳过并计入 `skipped`，其余记录正常导入，
  与 `import_commands` 的注释和 `ImportResult` 的语义一致。

遗留：导入没有去重，重复导入同一份文件会产生重复记录。这属于产品决策
（"导入即追加"还是"按标题+内容去重"），未在本次改动范围内。

## 相关文件

- `src-tauri/src/commands.rs` — `import_commands`（事务）、`ExportCommand`（`#[serde(default)]`）
- `src/App.tsx:133-142` — `handleImport` 的 `imported` / `importSkipped` 提示分支
- `src/types.ts` — `ImportResult`
