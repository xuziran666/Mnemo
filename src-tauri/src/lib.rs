mod commands;
mod db;

use commands::{create_command, delete_command, list_commands, Db};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            delete_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
