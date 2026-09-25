mod commands;
mod mnemo_core;

use commands::Db;
use tauri::Manager;

// Tauri 应用启动入口，负责初始化插件、数据库连接和命令注册。
// 这里的关键点是：在 setup 阶段创建 app_data_dir 下的 SQLite 文件，并把连接挂载到全局应用状态中。
// Tauri command handler 只做薄封装，实际业务逻辑委托给 mnemo_core，确保 Rust 侧可独立复用。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 在 setup 阶段创建 app_data_dir 下的 SQLite 文件，并把连接挂载到全局应用状态中。
            let dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            // 创建 app_data_dir 目录
            std::fs::create_dir_all(&dir).expect("failed to create app data dir");
            // 打开 SQLite 数据库连接
            let conn =
                mnemo_core::open_db(dir.join("commands.db")).expect("failed to open database");
            app.manage(Db::new(conn));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_commands,
            commands::create_command,
            commands::update_command,
            commands::delete_command,
            commands::export_commands,
            commands::import_commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}