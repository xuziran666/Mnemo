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

pub fn save_lang(lang: Lang) {
    let Some(path) = settings_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or_else(|| json!({}));
    if !value.is_object() {
        value = json!({});
    }
    value["lang"] = json!(lang.code());

    if let Err(err) = std::fs::write(&path, value.to_string()) {
        eprintln!("[mnemo] 写入设置失败：{err}");
    }
}
