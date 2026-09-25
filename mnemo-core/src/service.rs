use rusqlite::Connection;

use crate::models::{
    Command, ExportCommand, ExportData, ImportResult, NewCommand, KIND_NOTE, KIND_SNIPPET,
};

// kind 只允许 KIND_SNIPPET / KIND_NOTE；其它值（例如外部 JSON 写错、上层传错）一律按 Snippet 处理。
// 选择"归一化"而不是"报错"，是为了导入外部备份时尽量把可用数据救回来。
fn normalize_kind(kind: i64) -> i64 {
    if kind == KIND_NOTE {
        KIND_NOTE
    } else {
        KIND_SNIPPET
    }
}

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
            normalize_kind(input.kind),
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
                normalize_kind(input.kind),
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
// 目标不存在时返回错误：否则上层会误报"已删除"（UI 会弹 Toast，但库里什么都没变）。
pub fn delete(conn: &Connection, id: i64) -> Result<(), String> {
    let deleted = conn
        .execute("DELETE FROM commands WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    if deleted == 0 {
        return Err(format!("command {id} not found"));
    }
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
// 整批写入放在一个事务里：中途失败（约束冲突、磁盘问题）会整体回滚，不留半份数据。
pub fn import(conn: &Connection, json: String) -> Result<ImportResult, String> {
    let data: ExportData = serde_json::from_str(&json).map_err(|e| format!("invalid JSON: {e}"))?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;

    // service 的函数签名是 &Connection，而 rusqlite 的 transaction() 需要 &mut self，
    // 因此用 unchecked_transaction()：同样是事务语义，出错时 drop 会自动回滚。
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    let mut imported = 0usize;
    let mut skipped = 0usize;
    for item in data.commands {
        if item.title.trim().is_empty() || item.content.trim().is_empty() {
            skipped += 1;
            continue;
        }
        let created_at = item.created_at.unwrap_or(now);
        tx.execute(
            "INSERT INTO commands (title, content, note, tags, kind, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                item.title,
                item.content,
                item.note,
                item.tags,
                normalize_kind(item.kind),
                created_at,
            ],
        )
        .map_err(|e| e.to_string())?;
        imported += 1;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(ImportResult { imported, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    // 内存库 + 与生产完全同一套建表/迁移逻辑（db::init_schema 是 crate 内可见）
    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::init_schema(&conn).unwrap();
        conn
    }

    fn new_command(title: &str, content: &str, kind: i64) -> NewCommand {
        NewCommand {
            title: title.to_string(),
            content: content.to_string(),
            note: None,
            tags: None,
            kind,
        }
    }

    #[test]
    fn create_returns_inserted_row_and_list_sees_it() {
        let conn = memory_db();
        let created =
            create(&conn, new_command("推分支", "git push origin HEAD", KIND_SNIPPET)).unwrap();

        assert!(created.id > 0);
        assert_eq!(created.kind, KIND_SNIPPET);
        assert_eq!(created.title, "推分支");

        let rows = list(&conn, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].content, "git push origin HEAD");
    }

    #[test]
    fn create_and_update_normalize_unknown_kind() {
        let conn = memory_db();
        // 99 不是合法 kind：需归一化为 Snippet，否则 UI 的 kind 分支会渲染成"未知类型"
        assert_eq!(
            create(&conn, new_command("t", "c", 99)).unwrap().kind,
            KIND_SNIPPET
        );
        assert_eq!(
            create(&conn, new_command("t2", "c2", KIND_NOTE)).unwrap().kind,
            KIND_NOTE
        );

        let snippet = create(&conn, new_command("t3", "c3", KIND_SNIPPET)).unwrap();
        assert_eq!(
            update(&conn, snippet.id, new_command("t4", "c4", -1))
                .unwrap()
                .kind,
            KIND_SNIPPET
        );
    }

    #[test]
    fn update_keeps_created_at_and_reports_missing_row() {
        let conn = memory_db();
        let created = create(&conn, new_command("t", "c", KIND_SNIPPET)).unwrap();
        let updated = update(&conn, created.id, new_command("t2", "c2", KIND_NOTE)).unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.title, "t2");
        assert_eq!(updated.content, "c2");
        assert_eq!(updated.kind, KIND_NOTE);
        // update 不应改动 created_at（否则列表排序会被打乱）
        assert_eq!(updated.created_at, created.created_at);

        let err = update(&conn, 4242, new_command("x", "y", KIND_SNIPPET)).unwrap_err();
        assert!(err.contains("not found"), "unexpected error: {err}");
    }

    #[test]
    fn delete_reports_missing_row() {
        let conn = memory_db();
        let created = create(&conn, new_command("t", "c", KIND_SNIPPET)).unwrap();

        assert!(delete(&conn, created.id).is_ok());
        assert!(list(&conn, None).unwrap().is_empty());

        // 再删一次必须报错：否则上层会误报"已删除"
        let err = delete(&conn, created.id).unwrap_err();
        assert!(err.contains("not found"), "unexpected error: {err}");
    }

    #[test]
    fn list_filters_across_text_columns_and_ignores_blank_query() {
        let conn = memory_db();
        let mut with_note = new_command("部署", "kubectl apply -f k8s/", KIND_SNIPPET);
        with_note.note = Some("生产环境用".to_string());
        with_note.tags = Some("k8s,ops".to_string());
        create(&conn, with_note).unwrap();
        create(&conn, new_command("回滚", "kubectl rollout undo", KIND_SNIPPET)).unwrap();

        assert_eq!(list(&conn, None).unwrap().len(), 2);
        // 空白关键词等价于不过滤
        assert_eq!(list(&conn, Some("   ")).unwrap().len(), 2);
        // title / content / note / tags 都能命中
        assert_eq!(list(&conn, Some("部署")).unwrap().len(), 1);
        assert_eq!(list(&conn, Some("apply")).unwrap().len(), 1);
        assert_eq!(list(&conn, Some("生产")).unwrap().len(), 1);
        assert_eq!(list(&conn, Some("ops")).unwrap().len(), 1);
        assert_eq!(list(&conn, Some("不存在的关键词")).unwrap().len(), 0);
    }

    #[test]
    fn list_escapes_like_wildcards() {
        let conn = memory_db();
        create(&conn, new_command("percent", "50% 折扣", KIND_NOTE)).unwrap();
        create(&conn, new_command("plain", "5012", KIND_NOTE)).unwrap();
        create(&conn, new_command("underscore", "a_b", KIND_NOTE)).unwrap();
        create(&conn, new_command("wildcard", "aXb", KIND_NOTE)).unwrap();

        // % 必须按字面量匹配：不转义的话 "50%" 会命中 "5012"
        let hits = list(&conn, Some("50%")).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "percent");

        // _ 同理：不转义的话 "a_b" 会命中 "aXb"
        let hits = list(&conn, Some("a_b")).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "underscore");
    }

    #[test]
    fn list_orders_by_created_at_desc() {
        let conn = memory_db();
        conn.execute_batch(
            "INSERT INTO commands (title, content, kind, created_at) VALUES
                ('old', 'c', 1, 100),
                ('new', 'c', 1, 200),
                ('mid', 'c', 1, 150);",
        )
        .unwrap();

        let titles: Vec<String> = list(&conn, None)
            .unwrap()
            .into_iter()
            .map(|command| command.title)
            .collect();
        assert_eq!(titles, vec!["new", "mid", "old"]);
    }

    #[test]
    fn import_skips_invalid_rows_and_normalizes_kind() {
        let conn = memory_db();
        let json = serde_json::json!({
            "version": 1,
            "commands": [
                { "title": "good", "content": "echo ok", "kind": 99 },
                { "title": "   ", "content": "no title" },
                { "title": "no content", "content": "" },
                { "title": "note", "content": "body", "kind": 2 }
            ]
        })
        .to_string();

        let result = import(&conn, json).unwrap();
        assert_eq!(result.imported, 2);
        assert_eq!(result.skipped, 2);

        let rows = list(&conn, None).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows.iter().find(|c| c.title == "good").unwrap().kind,
            KIND_SNIPPET
        );
        assert_eq!(
            rows.iter().find(|c| c.title == "note").unwrap().kind,
            KIND_NOTE
        );
    }

    #[test]
    fn import_rejects_invalid_json_without_touching_data() {
        let conn = memory_db();
        create(&conn, new_command("keep", "me", KIND_SNIPPET)).unwrap();

        assert!(import(&conn, "{ not json".to_string()).is_err());
        // 原有数据不受影响
        assert_eq!(list(&conn, None).unwrap().len(), 1);
    }

    #[test]
    fn import_rolls_back_whole_batch_on_failure() {
        let conn = memory_db();
        // 人为制造唯一约束，让第二条 INSERT 必然失败；第一条也必须被一起回滚
        conn.execute_batch("CREATE UNIQUE INDEX test_unique_title ON commands(title);")
            .unwrap();
        let json = serde_json::json!({
            "version": 1,
            "commands": [
                { "title": "dup", "content": "first" },
                { "title": "dup", "content": "second" }
            ]
        })
        .to_string();

        assert!(import(&conn, json).is_err());
        // 事务整体回滚：一条都不该落库
        assert!(list(&conn, None).unwrap().is_empty());
    }

    #[test]
    fn export_and_import_round_trip() {
        let source = memory_db();
        let mut snippet = new_command("导出", "echo hi", KIND_SNIPPET);
        snippet.note = Some("备注".to_string());
        snippet.tags = Some("a,b".to_string());
        create(&source, snippet).unwrap();
        create(&source, new_command("笔记", "正文", KIND_NOTE)).unwrap();

        let json = export(&source).unwrap();

        let target = memory_db();
        let result = import(&target, json).unwrap();
        assert_eq!(result.imported, 2);
        assert_eq!(result.skipped, 0);

        let rows = list(&target, None).unwrap();
        let mut summary: Vec<(String, i64, Option<String>, Option<String>)> = rows
            .iter()
            .map(|c| (c.title.clone(), c.kind, c.note.clone(), c.tags.clone()))
            .collect();
        summary.sort();
        assert_eq!(
            summary,
            vec![
                (
                    "导出".to_string(),
                    KIND_SNIPPET,
                    Some("备注".to_string()),
                    Some("a,b".to_string())
                ),
                ("笔记".to_string(), KIND_NOTE, None, None),
            ]
        );

        // created_at 随导出带走并被保留（备份恢复后排序不会丢）
        let origin: std::collections::HashMap<String, i64> = list(&source, None)
            .unwrap()
            .into_iter()
            .map(|command| (command.title, command.created_at))
            .collect();
        for row in rows {
            assert_eq!(origin.get(&row.title), Some(&row.created_at));
        }
    }
}