use rusqlite::Connection;
use std::path::PathBuf;

pub fn open(path: PathBuf) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(path)?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS commands (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            command TEXT NOT NULL,
            note TEXT,
            tags TEXT,
            created_at INTEGER
        );",
    )?;
    Ok(())
}
