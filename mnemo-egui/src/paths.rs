use std::path::PathBuf;

// 与 Tauri 的 bundle identifier 保持一致，用于复用同一份 commands.db。
pub const APP_IDENTIFIER: &str = "com.longanl.mnemo";

// 解析应用数据目录，结果与 Tauri 的 `app_data_dir()` 一致：
// - Windows: %APPDATA%\com.longanl.mnemo
// - macOS:   ~/Library/Application Support/com.longanl.mnemo
// - Linux:   $XDG_DATA_HOME/com.longanl.mnemo 或 ~/.local/share/com.longanl.mnemo
// dirs::data_dir() 在各平台返回的正是上述基础目录。
pub fn app_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join(APP_IDENTIFIER))
}

// 数据库文件路径，沿用旧版本文件名，保证老用户数据无缝迁移。
pub fn database_path() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("commands.db"))
}
