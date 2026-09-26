use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::db::normalize_timestamp;

// Db 是全局数据库连接的新类型包装，挂载在 Tauri app 的状态中，供所有 invoke handler 共享同一连接。
pub type Db = Mutex<Connection>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub note: Option<String>,
    pub tags: Option<String>,
    pub kind: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewCommand {
    pub title: String,
    pub content: String,
    pub note: Option<String>,
    pub tags: Option<String>,
    pub kind: i64,
}

// LIKE 搜索需要对 %、_ 和 \ 做转义，否则用户输入的关键字会被当成通配符，导致误匹配。
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

// created_at 统一使用毫秒时间戳。秒级精度下同一秒内新建的多条记录会拿到相同值，
// 配合无次级排序键的 ORDER BY 会导致列表顺序不确定。
fn now_millis() -> Result<i64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .map_err(|e| e.to_string())
}

// 列表只用于渲染单行预览，正文可能有几十 KB。搜索是「每次按键都跑」的高频路径，
// 在 SQL 层裁剪可以避免把全文通过 IPC 传回并塞进 DOM。
// 详情路径（get_command / create_command / update_command 的回读）必须返回完整 content，
// 否则复制和查看会拿到被截断的正文。预览长度与 db.rs 的排序索引无关，改这里无需迁移。
const CONTENT_PREVIEW_CHARS: usize = 2000;

fn list_content_expr() -> String {
    format!(
        "CASE WHEN length(content) > {CONTENT_PREVIEW_CHARS} \
         THEN substr(content, 1, {CONTENT_PREVIEW_CHARS}) || '…' ELSE content END"
    )
}

// list_commands 既能返回全部命令，也能按关键词过滤 title/content/note/tags。
#[tauri::command]
pub fn list_commands(query: Option<String>, state: State<Db>) -> Result<Vec<Command>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let content = list_content_expr();

    let rows = match query {
        Some(q) if !q.trim().is_empty() => {
            let like = format!("%{}%", escape_like(q.trim()));
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT id, title, {content} AS content, note, tags, kind, created_at
                     FROM commands
                     WHERE title LIKE ?1 ESCAPE '\\'
                        OR content LIKE ?1 ESCAPE '\\'
                        OR note LIKE ?1 ESCAPE '\\'
                        OR tags LIKE ?1 ESCAPE '\\'
                     ORDER BY created_at DESC, id DESC"
                ))
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([&like], row_to_command)
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        }
        _ => {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT id, title, {content} AS content, note, tags, kind, created_at
                     FROM commands
                     ORDER BY created_at DESC, id DESC"
                ))
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], row_to_command)
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        }
    };

    Ok(rows)
}

// get_command 返回单条记录的完整正文，供查看器和编辑器使用（列表里的 content 是裁剪后的预览）。
#[tauri::command]
pub fn get_command(id: i64, state: State<Db>) -> Result<Command, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, content, note, tags, kind, created_at
             FROM commands WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_row([id], row_to_command).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => format!("command {id} not found"),
        other => other.to_string(),
    })
}

// create_command 用于新增命令，并返回新插入记录的完整对象，给前端立即刷新列表。
#[tauri::command]
pub fn create_command(input: NewCommand, state: State<Db>) -> Result<Command, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let created_at = now_millis()?;

    conn.execute(
        "INSERT INTO commands (title, content, note, tags, kind, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            input.title,
            input.content,
            input.note,
            input.tags,
            input.kind,
            created_at
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let mut stmt = conn
        .prepare(
            "SELECT id, title, content, note, tags, kind, created_at
             FROM commands WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_row([id], row_to_command)
        .map_err(|e| e.to_string())
}

// update_command 只修改指定 id 的记录，若目标不存在则返回明确错误，避免静默失败。
#[tauri::command]
pub fn update_command(
    id: i64,
    input: NewCommand,
    state: State<Db>,
) -> Result<Command, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;

    let updated = conn
        .execute(
            "UPDATE commands SET title = ?1, content = ?2, note = ?3, tags = ?4, kind = ?5
             WHERE id = ?6",
            rusqlite::params![
                input.title,
                input.content,
                input.note,
                input.tags,
                input.kind,
                id
            ],
        )
        .map_err(|e| e.to_string())?;

    if updated == 0 {
        return Err(format!("command {id} not found"));
    }

    let mut stmt = conn
        .prepare(
            "SELECT id, title, content, note, tags, kind, created_at
             FROM commands WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_row([id], row_to_command)
        .map_err(|e| e.to_string())
}

// delete_command 执行从本地数据库中删除一条记录，删除后前端会重新拉取列表。
#[tauri::command]
pub fn delete_command(id: i64, state: State<Db>) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM commands WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn row_to_command(row: &rusqlite::Row) -> rusqlite::Result<Command> {
    Ok(Command {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        note: row.get(3)?,
        tags: row.get(4)?,
        kind: row.get(5)?,
        created_at: row.get(6)?,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    #[serde(default)]
    pub version: i64,
    pub commands: Vec<ExportCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportCommand {
    // title / content 用 default 兜底：备份文件里缺字段的条目应当被计入 skipped 并跳过，
    // 而不是让整份文件反序列化失败、导致一条坏数据毁掉整个导入。
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(default = "default_kind")]
    pub kind: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

fn default_kind() -> i64 {
    1
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
}

// export_commands 将所有本地记录序列化为 JSON，供用户导出备份或迁移到另一台设备。
#[tauri::command]
pub fn export_commands(state: State<Db>) -> Result<String, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, content, note, tags, kind, created_at
             FROM commands
             ORDER BY created_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], row_to_command)
        .map_err(|e| e.to_string())?;
    let commands: Vec<ExportCommand> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|c| ExportCommand {
            title: c.title,
            content: c.content,
            note: c.note,
            tags: c.tags,
            kind: c.kind,
            created_at: Some(c.created_at),
        })
        .collect();
    let data = ExportData {
        version: 1,
        commands,
    };
    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

// import_commands 接收导出的 JSON，并跳过空内容或缺失关键字段的数据，保证备份文件的可恢复性。
// 整个导入包在一个事务里：逐条 execute 时若中途失败（例如磁盘满、库被锁），
// 没有事务会留下「导入了一半」的库，而 skipped 计数也无法与实际写入对应。
#[tauri::command]
pub fn import_commands(json: String, state: State<Db>) -> Result<ImportResult, String> {
    let data: ExportData = serde_json::from_str(&json).map_err(|e| format!("invalid JSON: {e}"))?;
    let mut conn = state.lock().map_err(|e| e.to_string())?;
    let now = now_millis()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut imported = 0usize;
    let mut skipped = 0usize;
    for item in data.commands {
        if item.title.trim().is_empty() || item.content.trim().is_empty() {
            skipped += 1;
            continue;
        }
        // 旧版本导出的 JSON 里 created_at 是秒级时间戳，必须归一化成毫秒，
        // 否则和本库已有的毫秒值放在一起比较时，这条记录会被排到列表末尾。
        let created_at = item.created_at.map(normalize_timestamp).unwrap_or(now);
        tx.execute(
            "INSERT INTO commands (title, content, note, tags, kind, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                item.title,
                item.content,
                item.note,
                item.tags,
                item.kind,
                created_at,
            ],
        )
        .map_err(|e| e.to_string())?;
        imported += 1;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(ImportResult { imported, skipped })
}
