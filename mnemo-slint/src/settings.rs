use std::path::PathBuf;

use serde_json::{json, Value};

use crate::i18n::Lang;
use crate::paths;

// 语言设置的持久化。
//
// 注意：settings.json 与旧版 egui 共用同一目录下的同一文件，因此写入时采用"读-改-写"，
// 保留本应用不关心的字段（如 theme），避免把旧版的主题偏好清掉。
fn settings_path() -> Option<PathBuf> {
    paths::app_data_dir().map(|dir| dir.join("settings.json"))
}

pub fn load_lang() -> Option<Lang> {
    let text = std::fs::read_to_string(settings_path()?).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    Lang::from_code(value.get("lang")?.as_str()?)
}

// 读取主题偏好。字段名与取值（"dark"/"light"）沿用旧版 egui 的约定，
// 因此两个实现读写同一份设置；缺失或非法值返回 None（由调用方决定默认值）。
pub fn load_theme() -> Option<bool> {
    let text = std::fs::read_to_string(settings_path()?).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    match value.get("theme")?.as_str()? {
        "dark" => Some(true),
        "light" => Some(false),
        _ => None,
    }
}

pub fn save_lang(lang: Lang) {
    update_setting("lang", json!(lang.code()));
}

pub fn save_theme(is_dark: bool) {
    update_setting("theme", json!(if is_dark { "dark" } else { "light" }));
}

// 读-改-写单个字段：保留文件里其它应用的设置（例如 egui 版的 theme/lang），
// 避免"切一次主题就把对方的偏好清掉"。
fn update_setting(key: &str, value: Value) {
    let Some(path) = settings_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut root = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or_else(|| json!({}));
    if !root.is_object() {
        root = json!({});
    }
    root[key] = value;

    if let Err(err) = std::fs::write(&path, root.to_string()) {
        eprintln!("[mnemo] 写入设置失败：{err}");
    }
}
