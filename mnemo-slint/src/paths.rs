use std::path::PathBuf;

// 与旧版（Tauri / egui）保持同一应用标识，复用同一份 commands.db。
//
// 说明：路径逻辑放在 UI 侧而不是 mnemo-core —— mnemo-core 的 open_db() 只接收完整路径，
// 由调用方决定数据存放位置（本文件是 mnemo-egui/src/paths.rs 的等价实现，不修改旧版代码）。
pub const APP_IDENTIFIER: &str = "com.longanl.mnemo";

// 应用数据目录，与 Tauri 的 app_data_dir() 一致：
// - Windows: %APPDATA%\com.longanl.mnemo
// - macOS:   ~/Library/Application Support/com.longanl.mnemo
// - Linux:   $XDG_DATA_HOME/com.longanl.mnemo 或 ~/.local/share/com.longanl.mnemo
pub fn app_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join(APP_IDENTIFIER))
}

// 数据库文件路径，沿用旧版文件名，保证老用户数据无缝迁移。
pub fn database_path() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("commands.db"))
}
