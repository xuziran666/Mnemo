mod app;
mod fonts;
mod i18n;
mod paths;

use eframe::egui;

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
        Box::new(|cc| Ok(Box::new(app::MnemoApp::new(cc)))),
    )
}
