use std::path::{Path, PathBuf};

// 已解析出的中文（CJK）字体：字体名、字节内容与来源描述。
pub struct ResolvedFont {
    pub name: String,
    pub bytes: Vec<u8>,
    pub source: String,
}

// 按优先级解析可用的 CJK 字体：
// 1. 环境变量 MNEMO_FONT 指定的字体文件；
// 2. 随程序打包的 assets/fonts/NotoSansSC-Regular.ttf；
// 3. 各平台常见的系统中文字体。
// 全部失败时返回 None，此时 egui 回退到内置字体（仅支持拉丁字符，中文会显示为方框）。
pub fn resolve_cjk_font() -> Option<ResolvedFont> {
    if let Some(path) = std::env::var_os("MNEMO_FONT") {
        if let Some(font) = load(&PathBuf::from(path), "env:MNEMO_FONT") {
            return Some(font);
        }
    }

    for dir in bundled_dirs() {
        if let Some(font) = load(&dir.join("NotoSansSC-Regular.ttf"), "bundled") {
            return Some(font);
        }
    }

    for path in system_candidates() {
        if let Some(font) = load(&path, "system") {
            return Some(font);
        }
    }

    None
}

fn load(path: &Path, source: &str) -> Option<ResolvedFont> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.is_empty() {
        return None;
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("font")
        .to_owned();
    Some(ResolvedFont {
        name,
        bytes,
        source: source.to_owned(),
    })
}

// 依次尝试：可执行文件同级目录、当前工作目录、源码目录（开发期）。
fn bundled_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("assets/fonts"));
        }
    }
    dirs.push(PathBuf::from("assets/fonts"));
    dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts"));
    dirs
}

#[cfg(target_os = "windows")]
fn system_candidates() -> Vec<PathBuf> {
    let root = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let fonts = root.join("Fonts");
    ["msyh.ttc", "msyh.ttf", "simhei.ttf", "simsun.ttc", "Deng.ttf"]
        .iter()
        .map(|f| fonts.join(f))
        .collect()
}

#[cfg(target_os = "macos")]
fn system_candidates() -> Vec<PathBuf> {
    [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
    ]
    .iter()
    .map(PathBuf::from)
    .collect()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn system_candidates() -> Vec<PathBuf> {
    [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJKsc-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/truetype/arphic/uming.ttc",
    ]
    .iter()
    .map(PathBuf::from)
    .collect()
}
