use serde_json::Value;
use std::collections::HashMap;

// 支持的语言。后续新增语言时在此扩展，并添加对应 locales/<code>.json。
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Zh => "zh",
        }
    }

    // 根据系统 locale 粗略判断语言；无法判断时回退英文。
    pub fn from_system() -> Lang {
        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(value) = std::env::var(var) {
                if value.to_ascii_lowercase().starts_with("zh") {
                    return Lang::Zh;
                }
            }
        }
        Lang::En
    }
}

// 轻量级 i18n：启动时把嵌套 JSON 展平为 "a.b.c" -> 文本 的映射。
pub struct I18n {
    lang: Lang,
    en: HashMap<String, String>,
    zh: HashMap<String, String>,
}

impl I18n {
    pub fn new(lang: Lang) -> Self {
        Self {
            lang,
            en: load_table(include_str!("../locales/en.json")),
            zh: load_table(include_str!("../locales/zh.json")),
        }
    }

    pub fn lang(&self) -> Lang {
        self.lang
    }

    pub fn set_lang(&mut self, lang: Lang) {
        self.lang = lang;
    }

    // 取翻译文本；缺失时回退英文，再缺失则原样返回 key，便于定位漏配。
    pub fn t(&self, key: &str) -> String {
        let table = match self.lang {
            Lang::En => &self.en,
            Lang::Zh => &self.zh,
        };
        table
            .get(key)
            .or_else(|| self.en.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    // 支持 {{name}} 占位符替换，对应原前端的插值用法。
    pub fn t_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut text = self.t(key);
        for (name, value) in args {
            text = text.replace(&format!("{{{{{name}}}}}"), value);
        }
        text
    }
}

fn load_table(json: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        flatten(&value, "", &mut out);
    }
    out
}

fn flatten(value: &Value, prefix: &str, out: &mut HashMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let full = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten(child, &full, out);
            }
        }
        Value::String(text) => {
            out.insert(prefix.to_string(), text.clone());
        }
        _ => {}
    }
}
