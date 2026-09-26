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
    // 列表默认按 created_at 倒序，这个复合索引让排序直接走索引而不是全表扫 + 排序。
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_commands_created_at ON commands(created_at DESC, id DESC);",
    )?;
    migrate(conn)?;
    Ok(())
}

fn column_names(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    conn.prepare("PRAGMA table_info(commands)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()
}

// 区分秒级和毫秒级时间戳的分界：1e11 毫秒约等于 1973 年，1e11 秒约等于 5138 年。
const SECONDS_TIMESTAMP_LIMIT: i64 = 100_000_000_000;

// 把外部来源（历史数据库、旧版本导出的 JSON）带来的秒级时间戳统一换算成毫秒。
// 两种单位混存时数值不可比：秒级值远小于毫秒级值，会被当成极旧的数据排到列表末尾。
// 0 视为「未设置」，保持原样。
pub fn normalize_timestamp(value: i64) -> i64 {
    if value > 0 && value < SECONDS_TIMESTAMP_LIMIT {
        value * 1000
    } else {
        value
    }
}

// created_at 早期用秒级时间戳存储，同一秒内新建的多条记录排序不确定（ORDER BY 无次级键）。
// 现统一使用毫秒。此处把历史秒级数据放大到毫秒。
// 该迁移是幂等的：转换后数值已超过分界，重复执行不会命中任何行。
fn migrate_created_at_unit(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE commands SET created_at = created_at * 1000
         WHERE created_at > 0 AND created_at < ?1",
        [SECONDS_TIMESTAMP_LIMIT],
    )?;
    Ok(())
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
    migrate_created_at_unit(conn)?;
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

    #[test]
    fn seconds_created_at_is_migrated_to_millis() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE commands (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                kind INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO commands (title, content, created_at) VALUES ('a', 'x', 1700000000)",
            [],
        )
        .unwrap();
        init_schema(&conn).unwrap();
        let created_at: i64 = conn
            .query_row("SELECT created_at FROM commands", [], |r| r.get(0))
            .unwrap();
        assert_eq!(created_at, 1_700_000_000_000);

        // 迁移必须幂等：再次执行不应把毫秒值再放大一次。
        init_schema(&conn).unwrap();
        let again: i64 = conn
            .query_row("SELECT created_at FROM commands", [], |r| r.get(0))
            .unwrap();
        assert_eq!(again, 1_700_000_000_000);
    }

    #[test]
    fn millis_created_at_is_left_untouched() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO commands (title, content, created_at) VALUES ('a', 'x', 1700000000123)",
            [],
        )
        .unwrap();
        init_schema(&conn).unwrap();
        let created_at: i64 = conn
            .query_row("SELECT created_at FROM commands", [], |r| r.get(0))
            .unwrap();
        assert_eq!(created_at, 1_700_000_000_123);
    }

    #[test]
    fn created_at_index_is_created() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'index' AND tbl_name = 'commands'")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(indexes.contains(&"idx_commands_created_at".to_string()));
    }

    #[test]
    fn normalize_timestamp_scales_seconds_only() {
        assert_eq!(normalize_timestamp(1_700_000_000), 1_700_000_000_000);
        assert_eq!(normalize_timestamp(1_700_000_000_123), 1_700_000_000_123);
        // 0 表示未设置，不能被放大。
        assert_eq!(normalize_timestamp(0), 0);
        assert_eq!(normalize_timestamp(-1), -1);
    }
}
