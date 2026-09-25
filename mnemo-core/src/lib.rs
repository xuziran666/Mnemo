pub mod db;
pub mod models;
pub mod service;

// 重导出 rusqlite，让上层 crate 无需单独声明该依赖即可持有 Connection 类型。
pub use rusqlite;

use db::open;
use models::{Command, ImportResult, NewCommand};

/// 打开应用数据目录下的 SQLite 数据库，并执行 schema 初始化与兼容迁移。
/// 这是纯 Rust 核心入口，不依赖任何 GUI 框架。
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
