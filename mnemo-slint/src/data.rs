use std::cell::RefCell;
use std::rc::Rc;

use mnemo_core::models::Command;
use mnemo_core::rusqlite::Connection;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::paths;
use crate::{CommandItem, MainWindow, State};

// 数据层：持有 SQLite 连接、缓存最近一次查询结果，并负责刷新列表模型。
//
// 线程模型：Slint 的所有回调都在 UI 线程（主线程）执行，因此这里用 Rc/RefCell 共享即可，
// 既不需要 Mutex，也不会跨线程使用 rusqlite::Connection。
pub struct AppData {
    // 数据库打开失败时为 None：此时列表为空，写操作以失败提示反馈（见后续步骤）。
    conn: Option<Connection>,
    // 最近一次查询结果（原始 Command）：复制要取正文、删除要取标题、编辑要回填表单。
    commands: RefCell<Vec<Command>>,
}

impl AppData {
    // 打开应用数据目录下的 commands.db（含建表与旧库迁移）。
    // 任何失败都只记录日志并退化为空列表，不让 UI 崩掉。
    pub fn open() -> Rc<Self> {
        let conn = match paths::database_path() {
            Some(path) => {
                // 首次运行时目录可能不存在，先确保父目录可用（与旧版 egui 实现一致）。
                if let Some(parent) = path.parent() {
                    if let Err(err) = std::fs::create_dir_all(parent) {
                        eprintln!("[mnemo] 创建数据目录失败：{err}");
                    }
                }
                match mnemo_core::open_db(path) {
                    Ok(conn) => Some(conn),
                    Err(err) => {
                        eprintln!("[mnemo] 打开数据库失败：{err}");
                        None
                    }
                }
            }
            None => {
                eprintln!("[mnemo] 无法解析应用数据目录");
                None
            }
        };
        Rc::new(Self {
            conn,
            commands: RefCell::new(Vec::new()),
        })
    }

    // 按关键词查询并更新缓存。
    // 关键词里的 % _ \ 转义由 mnemo_core::service::list 内部完成，
    // 这里必须传原始关键词，否则会被转义两次而匹配不到数据。
    fn reload(&self, query: &str) {
        if let Some(conn) = self.conn.as_ref() {
            let trimmed = query.trim();
            let filter = if trimmed.is_empty() { None } else { Some(trimmed) };
            match mnemo_core::list(conn, filter) {
                Ok(rows) => *self.commands.borrow_mut() = rows,
                Err(err) => eprintln!("[mnemo] 查询失败：{err}"),
            }
        }
    }
}

// 查询并刷新 State.commands，同时修复选中项。
pub fn refresh_list(ui: &MainWindow, data: &AppData, query: &str) {
    data.reload(query);

    let items: Vec<CommandItem> = data.commands.borrow().iter().map(to_item).collect();
    let ids: Vec<i32> = items.iter().map(|item| item.id).collect();

    let state = ui.global::<State>();
    // 直接重建整个模型：本地命令库体量下最简单可靠；条目上万时再考虑增量更新。
    state.set_commands(ModelRc::from(Rc::new(VecModel::from(items))));

    // 选中项修复：0 表示无选中；原选中项被过滤掉（或列表为空）时退回第一条。
    let selected = state.get_selected_id();
    if !ids.contains(&selected) {
        state.set_selected_id(ids.first().copied().unwrap_or(0));
    }
}

// Command → CommandItem：note/tags 在库里是可空字符串，UI 侧统一为空串与字符串数组。
fn to_item(command: &Command) -> CommandItem {
    CommandItem {
        id: command.id as i32,
        title: command.title.as_str().into(),
        content: command.content.as_str().into(),
        note: command.note.clone().unwrap_or_default().into(),
        tags: split_tags(command.tags.as_deref()),
        kind: command.kind as i32,
    }
}

// tags 存的是逗号分隔字符串，解析规则与旧版 egui 的 tag_row 保持一致。
fn split_tags(tags: Option<&str>) -> ModelRc<SharedString> {
    let list: Vec<SharedString> = tags
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(SharedString::from)
        .collect();
    ModelRc::from(Rc::new(VecModel::from(list)))
}
