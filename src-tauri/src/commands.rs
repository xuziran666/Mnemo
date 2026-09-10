use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

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

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[tauri::command]
pub fn list_commands(query: Option<String>, state: State<Db>) -> Result<Vec<Command>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;

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

#[tauri::command]
pub fn create_command(input: NewCommand, state: State<Db>) -> Result<Command, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
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
    pub title: String,
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

#[tauri::command]
pub fn export_commands(state: State<Db>) -> Result<String, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
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

#[tauri::command]
pub fn import_commands(json: String, state: State<Db>) -> Result<ImportResult, String> {
    let data: ExportData = serde_json::from_str(&json).map_err(|e| format!("invalid JSON: {e}"))?;
    let conn = state.lock().map_err(|e| e.to_string())?;
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
