// Mnemo（Slint 版）入口。
// P0 范围：窗口行为（无边框 / 置顶 / 失焦隐藏）+ 系统托盘 + 回调接线。
// 业务逻辑（查询、增删改、导入导出、i18n）在 P1 接入 mnemo-core，当前回调只输出日志。

use std::cell::Cell;
use std::rc::Rc;

use slint::winit_030::{winit, EventResult, WinitWindowAccessor};
use slint::{ComponentHandle, SharedString};

slint::include_modules!();

// Slint 没有 monospace 通用族关键字，代码字体必须给具体族名，按平台选择。
#[cfg(target_os = "windows")]
const CODE_FONT_FAMILY: &str = "Consolas";
#[cfg(target_os = "macos")]
const CODE_FONT_FAMILY: &str = "Menlo";
#[cfg(all(unix, not(target_os = "macos")))]
const CODE_FONT_FAMILY: &str = "Noto Sans Mono";

fn main() -> Result<(), slint::PlatformError> {
    // 失焦自动隐藏依赖 winit 的事件钩子，这里显式锁定 winit 后端。
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .select()?;

    // 与旧版（Tauri / egui）沿用同一应用标识，便于桌面环境归组。
    let _ = slint::set_xdg_app_id("com.longanl.mnemo");

    let ui = MainWindow::new()?;
    apply_code_font(&ui);
    install_callbacks(&ui);
    install_focus_hide(&ui);

    let tray = Tray::new()?;
    install_tray(&tray, &ui);

    ui.show()?;
    tray.show()?;

    // 使用 run_event_loop_until_quit：窗口隐藏（失焦自动隐藏）后事件循环仍需继续，
    // 只有托盘菜单的 Quit 调用 slint::quit_event_loop() 才退出。
    slint::run_event_loop_until_quit().expect("Slint event loop failed");
    Ok(())
}

// 把代码字体写入 Theme（P1 可扩展为按语言/系统字体回退）。
fn apply_code_font(ui: &MainWindow) {
    ui.global::<Theme>()
        .set_code_font_family(SharedString::from(CODE_FONT_FAMILY));
}

// P0：回调只打印日志，用于验证 UI → Rust 的接线；P1 替换为 mnemo-core 调用。
fn install_callbacks(ui: &MainWindow) {
    ui.on_card_activated(|id| eprintln!("[mnemo] card-activated id={id}"));
    ui.on_copy_requested(|id| eprintln!("[mnemo] copy-requested id={id}"));
    ui.on_edit_requested(|id| eprintln!("[mnemo] edit-requested id={id}"));
    ui.on_delete_requested(|id| eprintln!("[mnemo] delete-requested id={id}"));
    ui.on_import_clicked(|| eprintln!("[mnemo] import-clicked"));
    ui.on_export_clicked(|| eprintln!("[mnemo] export-clicked"));
    ui.on_new_clicked(|| eprintln!("[mnemo] new-clicked"));
    ui.on_lang_selected(|code| eprintln!("[mnemo] lang-selected {code}"));
}

// 托盘：唤回窗口 / 退出应用。
fn install_tray(tray: &Tray, ui: &MainWindow) {
    let ui_weak = ui.as_weak();
    tray.on_show_window(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let _ = ui.show();
        }
    });
    tray.on_quit(|| {
        let _ = slint::quit_event_loop();
    });
}

// 失去焦点自动隐藏：拦截 winit 的 Focused 事件。
// armed 标志确保窗口至少获得过一次焦点后才允许隐藏，避免启动瞬间被误判为失焦。
fn install_focus_hide(ui: &MainWindow) {
    let armed = Rc::new(Cell::new(false));
    ui.window()
        .on_winit_window_event(move |window, event| {
            if let winit::event::WindowEvent::Focused(focused) = event {
                if *focused {
                    armed.set(true);
                } else if armed.get() {
                    // TODO(P1)：接入 rfd 文件对话框（导入/导出）与外部弹层时，
                    // 需要用 suppress 标志临时禁用这里，否则打开对话框会让窗口自动隐藏。
                    let _ = window.hide();
                }
            }
            EventResult::Propagate
        });
}
