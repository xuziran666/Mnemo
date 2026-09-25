mod fonts;
mod paths;

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;

// Phase 2 的最小 egui 应用：仅用于验证 egui 工具链与中文字体方案，
// 尚未接入 mnemo_core。后续 Phase 3 将替换为完整的列表/编辑/查看界面。
struct MnemoApp {
    query: String,
    font_note: String,
    db_path: Option<PathBuf>,
}

impl MnemoApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (font_note, fonts) = install_fonts();
        cc.egui_ctx.set_fonts(fonts);
        Self {
            query: String::new(),
            font_note,
            db_path: paths::database_path(),
        }
    }
}

// 在 egui 默认字体（拉丁）基础上追加中文字体作为回退，避免中文显示为方框。
fn install_fonts() -> (String, egui::FontDefinitions) {
    let mut fonts = egui::FontDefinitions::default();
    match fonts::resolve_cjk_font() {
        Some(font) => {
            let note = format!("{} ({})", font.name, font.source);
            fonts.font_data.insert(
                "cjk".to_owned(),
                Arc::new(egui::FontData::from_owned(font.bytes)),
            );
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts
                    .families
                    .entry(family)
                    .or_default()
                    .push("cjk".to_owned());
            }
            (note, fonts)
        }
        None => ("egui built-in (Latin only)".to_owned(), fonts),
    }
}

impl eframe::App for MnemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mnemo · Phase 2 验证");
            ui.add_space(4.0);
            ui.label("中文渲染测试：搜索命令、代码片段、笔记、导入、导出。");
            ui.label("English check: The quick brown fox jumps over the lazy dog.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("搜索");
                ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("搜索命令...")
                        .desired_width(f32::INFINITY),
                );
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            ui.label(format!("字体来源: {}", self.font_note));
            match &self.db_path {
                Some(path) => {
                    ui.label(format!("数据库路径: {}", path.display()));
                }
                None => {
                    ui.colored_label(egui::Color32::RED, "无法解析应用数据目录");
                }
            }
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Mnemo")
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([500.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Mnemo",
        options,
        Box::new(|cc| Ok(Box::new(MnemoApp::new(cc)))),
    )
}
