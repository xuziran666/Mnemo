use std::collections::HashMap;

use serde_json::Value;

// 支持的语言：新增语言时在此扩展，并在 state.slint 的 langs 里加一项、加 locales/<code>.json。
#[derive(Clone, Copy, PartialEq, Eq)]
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

    // 下拉按钮显示"当前语言"，按惯例用该语言自己的写法（与 state.slint 的 langs 保持一致）。
    pub fn label(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Zh => "中文",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Lang::En),
            "zh" => Some(Lang::Zh),
            _ => None,
        }
    }

    // 按系统 locale 粗略判断语言；无法判断时回退英文（规则与旧版 egui 一致）。
    pub fn from_system() -> Self {
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

// 轻量级 i18n：构建期把 locales/*.json 嵌入二进制，启动时展平成 "a.b.c" -> 文本 的映射。
// 说明：本文件与 mnemo-egui/src/i18n.rs 是同一套做法的两份实现（两个 crate 各自拥有资源，
// 互不依赖）；mnemo-slint 的 locales 是 egui 版的超集，因此没有改旧版代码。
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

    // 支持 {{name}} 占位符替换（例如 confirm.delete 里的 {{title}}）。
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
