use rusqlite::Connection;

use crate::models::{Command, ExportCommand, ExportData, ImportResult, NewCommand};

// LIKE 搜索需要对 %、_ 和 \ 做转义，否则用户输入的关键字会被当成通配符，导致误匹配。
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
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

// list 既能返回全部命令，也能按关键词过滤 title/content/note/tags。
pub fn list(conn: &Connection, query: Option<&str>) -> Result<Vec<Command>, String> {
    let rows = match query {
        Some(q) if !q.trim().is_empty() => {
            let like = format!("%{}%", escape_like(q.trim()));
            let mut stmt = conn
                .prepare(
                    "SELECT id, title, content, note, tags, kind, created_at
                     FROM commands
                     WHERE title LIKE ?1 ESCAPE '\\'
                        OR content LIKE ?1 ESCAPE '\\'
                        OR note LIKE ?1 ESCAPE '\\'
                        OR tags LIKE ?1 ESCAPE '\\'
                     ORDER BY created_at DESC",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([&like], row_to_command)
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        }
        _ => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, title, content, note, tags, kind, created_at
                     FROM commands
                     ORDER BY created_at DESC",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], row_to_command)
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        }
    };

    Ok(rows)
}

// create 用于新增命令，并返回新插入记录的完整对象，给调用方立即刷新列表。
pub fn create(conn: &Connection, input: NewCommand) -> Result<Command, String> {
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

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

// update 只修改指定 id 的记录，若目标不存在则返回明确错误，避免静默失败。
pub fn update(conn: &Connection, id: i64, input: NewCommand) -> Result<Command, String> {
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

// delete 执行从本地数据库中删除一条记录。
pub fn delete(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM commands WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// export 将所有本地记录序列化为 JSON，供用户导出备份或迁移到另一台设备。
pub fn export(conn: &Connection) -> Result<String, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, content, note, tags, kind, created_at
             FROM commands
             ORDER BY created_at DESC",
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

// import 接收导出的 JSON，并跳过空内容或缺失关键字段的数据，保证备份文件的可恢复性。
pub fn import(conn: &Connection, json: String) -> Result<ImportResult, String> {
    let data: ExportData = serde_json::from_str(&json).map_err(|e| format!("invalid JSON: {e}"))?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;
    let mut imported = 0usize;
    let mut skipped = 0usize;
    for item in data.commands {
        if item.title.trim().is_empty() || item.content.trim().is_empty() {
            skipped += 1;
            continue;
        }
        let created_at = item.created_at.unwrap_or(now);
        conn.execute(
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
    Ok(ImportResult { imported, skipped })
}