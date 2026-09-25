mod app;
mod fonts;
mod i18n;
mod paths;

use eframe::egui;

// 将打包的 PNG 图标解码为 egui 的 IconData；失败时回退到默认图标。
fn window_icon() -> Option<egui::IconData> {
    eframe::icon_data::from_png_bytes(include_bytes!("../assets/icons/icon.png")).ok()
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Mnemo")
        .with_inner_size([900.0, 650.0])
        .with_min_inner_size([500.0, 400.0]);
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Mnemo",
        options,
        Box::new(|cc| Ok(Box::new(app::MnemoApp::new(cc)))),
    )
}
