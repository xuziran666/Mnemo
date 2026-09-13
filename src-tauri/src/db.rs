use rusqlite::Connection;
use std::path::PathBuf;

// SQLite 连接的入口，负责打开数据库并在启动时初始化表结构与兼容迁移逻辑。
pub fn open(path: PathBuf) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(path)?;
    init_schema(&conn)?;
    Ok(conn)
}

// 初次启动时创建 commands 表；后续执行 migrate 确保旧版本数据库字段兼容新 schema。
fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS commands (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            note TEXT,
            tags TEXT,
            kind INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER
        );",
    )?;
    migrate(conn)?;
    Ok(())
}

fn column_names(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    conn.prepare("PRAGMA table_info(commands)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()
}

// 数据库迁移逻辑用于向后兼容旧版本表结构，比如把 command 字段转换为 content，并补齐 kind 字段。
fn migrate(conn: &Connection) -> Result<(), rusqlite::Error> {
    let names = column_names(conn)?;
    if !names.iter().any(|name| name == "kind") {
        conn.execute_batch(
            "ALTER TABLE commands ADD COLUMN kind INTEGER NOT NULL DEFAULT 1;",
        )?;
    }
    let has_content = names.iter().any(|name| name == "content");
    let has_command = names.iter().any(|name| name == "command");
    if !has_content {
        if has_command {
            conn.execute_batch("ALTER TABLE commands RENAME COLUMN command TO content;")?;
        } else {
            conn.execute_batch("ALTER TABLE commands ADD COLUMN content TEXT NOT NULL DEFAULT '';")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn columns(conn: &Connection) -> Vec<String> {
        conn.prepare("PRAGMA table_info(commands)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    #[test]
    fn fresh_database_has_kind_and_content() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        let names = columns(&conn);
        assert!(names.contains(&"kind".to_string()));
        assert!(names.contains(&"content".to_string()));
        assert!(!names.contains(&"command".to_string()));
        conn.execute(
            "INSERT INTO commands (title, content, kind, created_at) VALUES ('t', 'c', 2, 0)",
            [],
        )
        .unwrap();
        let (kind, content): (i64, String) = conn
            .query_row("SELECT kind, content FROM commands", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(kind, 2);
        assert_eq!(content, "c");
    }

    #[test]
    fn original_database_is_migrated() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE commands (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                command TEXT NOT NULL,
                note TEXT,
                tags TEXT,
                created_at INTEGER
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO commands (title, command, created_at) VALUES ('t', 'git pull', 0)",
            [],
        )
        .unwrap();
        init_schema(&conn).unwrap();
        let names = columns(&conn);
        assert!(names.contains(&"kind".to_string()));
        assert!(names.contains(&"content".to_string()));
        assert!(!names.contains(&"command".to_string()));
        let content: String = conn
            .query_row("SELECT content FROM commands", [], |r| r.get(0))
            .unwrap();
        assert_eq!(content, "git pull");
    }

    #[test]
    fn kind_migrated_database_renames_command_to_content() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE commands (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                command TEXT NOT NULL,
                note TEXT,
                tags TEXT,
                kind INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO commands (title, command, kind, created_at) VALUES ('t', 'ssh x', 2, 0)",
            [],
        )
        .unwrap();
        init_schema(&conn).unwrap();
        let names = columns(&conn);
        assert!(names.contains(&"content".to_string()));
        assert!(!names.contains(&"command".to_string()));
        let (kind, content): (i64, String) = conn
            .query_row("SELECT kind, content FROM commands", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(kind, 2);
        assert_eq!(content, "ssh x");
    }
}
