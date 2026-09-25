use rusqlite::Connection;
use std::sync::Mutex;

use crate::mnemo_core::models::{Command, ImportResult, NewCommand};

// Db 是全局数据库连接的新类型包装，挂载在 Tauri app 的状态中，供所有 invoke handler 共享同一连接。
pub type Db = Mutex<Connection>;

// list_commands 既能返回全部命令，也能按关键词过滤 title/content/note/tags。
#[tauri::command]
pub fn list_commands(query: Option<String>, state: tauri::State<Db>) -> Result<Vec<Command>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::mnemo_core::list(&conn, query.as_deref())
}

// create_command 用于新增命令，并返回新插入记录的完整对象，给前端立即刷新列表。
#[tauri::command]
pub fn create_command(input: NewCommand, state: tauri::State<Db>) -> Result<Command, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::mnemo_core::create(&conn, input)
}

// update_command 只修改指定 id 的记录，若目标不存在则返回明确错误，避免静默失败。
#[tauri::command]
pub fn update_command(
    id: i64,
    input: NewCommand,
    state: tauri::State<Db>,
) -> Result<Command, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::mnemo_core::update(&conn, id, input)
}

// delete_command 执行从本地数据库中删除一条记录，删除后前端会重新拉取列表。
#[tauri::command]
pub fn delete_command(id: i64, state: tauri::State<Db>) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::mnemo_core::delete(&conn, id)
}

// export_commands 将所有本地记录序列化为 JSON，供用户导出备份或迁移到另一台设备。
#[tauri::command]
pub fn export_commands(state: tauri::State<Db>) -> Result<String, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::mnemo_core::export(&conn)
}

// import_commands 接收导出的 JSON，并跳过空内容或缺失关键字段的数据，保证备份文件的可恢复性。
#[tauri::command]
pub fn import_commands(json: String, state: tauri::State<Db>) -> Result<ImportResult, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::mnemo_core::import(&conn, json)
}