mod commands;
mod db;

use commands::{create_command, delete_command, export_commands, import_commands, list_commands, update_command, Db};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&dir).expect("failed to create app data dir");
            let conn =
                db::open(dir.join("commands.db")).expect("failed to open database");
            app.manage(Db::new(conn));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_commands,
            create_command,
            update_command,
            delete_command,
            export_commands,
            import_commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
