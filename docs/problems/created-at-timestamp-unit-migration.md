# created_at 秒级精度导致排序不确定，改为毫秒并迁移存量数据

## 问题现象

新建两条命令后，列表里刚保存的那条**不一定排在最上面**。尤其是连续快速保存多条时，
顺序看起来是乱的。导入旧备份后，新导入的记录被排到列表最末尾。

## 问题原因

三个原因叠加。

**1. 秒级精度 + 无次级排序键**

`commands.rs` 用 `.as_secs()` 写入 `created_at`，而所有查询只有 `ORDER BY created_at DESC`。
SQLite 对 `created_at` 相同的行不保证相对顺序（实际取决于索引/页布局），所以同一秒内
创建的多条记录顺序是任意的。用户的"刚保存就应该在最上面"预期不成立。

**2. 毫秒与秒混存不可比**

改成毫秒后引入新问题：数据库里已有的行是秒级（如 `1700000000`），新写入的是毫秒级
（如 `1700000000123`）。秒级数值远小于毫秒级，直接比较会把所有历史数据当成"1970 年左右"
排到最后。**从旧版本导出的 JSON 备份也带秒级 `created_at`**，导入后同样落到末尾。

**3. 无索引**

`created_at` 上没有索引，每次查询都是全表扫 + 排序。

## 解决方案

### 1. 统一毫秒

`commands.rs` 抽出 `now_millis()`，替换原先两处重复的 `SystemTime` 计算：

```rust
fn now_millis() -> Result<i64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .map_err(|e| e.to_string())
}
```

### 2. 加次级排序键

三处 `ORDER BY`（列表带搜索 / 列表全量 / 导出）统一改为
`ORDER BY created_at DESC, id DESC`。`id` 是自增主键，插入顺序即 id 顺序，
因此同毫秒内的记录变成"后插入的在前"，符合直觉且完全确定。

### 3. 存量数据迁移

`db.rs` 新增 `migrate_created_at_unit()`，在 `migrate()` 末尾执行：

```sql
UPDATE commands SET created_at = created_at * 1000
WHERE created_at > 0 AND created_at < 100000000000
```

- 分界值 `1e11`：作为毫秒约等于 1973 年，作为秒约等于 5138 年，
  任何真实时间戳都不会同时满足两种解释，因此判别是安全的。
- `created_at > 0`：`0` 表示"未设置"，不能被放大。
- **幂等**：转换后数值已超过分界，重复执行不命中任何行。这一点有单测覆盖
  （`seconds_created_at_is_migrated_to_millis` 里连续调两次 `init_schema`）。

### 4. 导入时归一化

`db.rs` 导出 `normalize_timestamp(value)`，供 `import_commands` 处理
`item.created_at`（来自旧版本 JSON 备份的秒级值）：

```rust
let created_at = item.created_at.map(normalize_timestamp).unwrap_or(now);
```

`migrate_created_at_unit` 的 SQL 条件和 `normalize_timestamp` 共用同一个
`SECONDS_TIMESTAMP_LIMIT` 常量，避免两处判别逻辑漂移。

### 5. 索引

```sql
CREATE INDEX IF NOT EXISTS idx_commands_created_at ON commands(created_at DESC, id DESC)
```

列顺序与 `ORDER BY` 完全一致，SQLite 可以直接按索引顺序输出，无需额外排序。

## 为什么采用这个方案

- **改毫秒而不是加 `rowid` 排序**：加次级键（方案里的第 2 点）其实是更小的改动，
  光靠 `id DESC` 就能解决"刚保存排最上面"。改成毫秒的收益是让 `created_at` 本身
  能表达真实先后（导出给其他工具时更有意义），且未来若有"按时间范围筛选"的需求不再需要迁移。
  两者不冲突，所以都做了。
- **用固定分界值而不是"看数量级"**：比值判断更直观，且常量集中定义后只有一个可信来源。
- **迁移写成幂等**：SQLite 的 `ALTER TABLE` 不能重复执行，但 `UPDATE` 可以。
  幂等迁移让"启动时执行"这种最省事的策略是安全的。

## 解决了什么问题

- 列表顺序完全确定，"刚保存的在最上面"成立。
- 历史数据库与旧备份不再因时间戳单位不同而排到末尾。
- 列表查询走索引，避免全表扫 + 排序。

遗留：`created_at` 只在创建时写入，编辑不会更新，所以"最近修改的"无法排序。
若需要，得加 `updated_at` 字段并做一次迁移。

## 相关文件

- `src-tauri/src/commands.rs` — `now_millis` / 三处 `ORDER BY` / `normalize_timestamp` 调用
- `src-tauri/src/db.rs` — `SECONDS_TIMESTAMP_LIMIT` / `normalize_timestamp` /
  `migrate_created_at_unit` / `idx_commands_created_at`
- `src-tauri/src/db.rs` — 新增单测 `seconds_created_at_is_migrated_to_millis`、
  `millis_created_at_is_left_untouched`、`created_at_index_is_created`、
  `normalize_timestamp_scales_seconds_only`
- `.github/workflows/ci.yml` — 补 `cargo test`，否则这些迁移单测永远不会执行
