pub mod db;
pub mod models;
pub mod service;

use db::open;
use models::{Command, ImportResult, NewCommand};

/// 打开应用数据目录下的 SQLite 数据库，并执行 schema 初始化与兼容迁移。
/// 这是纯 Rust 核心入口，不依赖 Tauri，便于后续迁移到 egui 时直接复用。
pub fn open_db(path: std::path::PathBuf) -> Result<rusqlite::Connection, String> {
    open(path).map_err(|e| e.to_string())
}

pub fn list(conn: &rusqlite::Connection, query: Option<&str>) -> Result<Vec<Command>, String> {
    service::list(conn, query)
}

pub fn create(conn: &rusqlite::Connection, input: NewCommand) -> Result<Command, String> {
    service::create(conn, input)
}

pub fn update(conn: &rusqlite::Connection, id: i64, input: NewCommand) -> Result<Command, String> {
    service::update(conn, id, input)
}

pub fn delete(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    service::delete(conn, id)
}

pub fn export(conn: &rusqlite::Connection) -> Result<String, String> {
    service::export(conn)
}

pub fn import(conn: &rusqlite::Connection, json: String) -> Result<ImportResult, String> {
    service::import(conn, json)
}