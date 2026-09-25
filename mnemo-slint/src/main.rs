// Mnemo（Slint 版）入口。
// 窗口行为（无边框 / 置顶 / 失焦隐藏 / 最小化 / Esc 隐藏）+ 系统托盘 + 数据层装配。
// 查询（list / LIKE 过滤）已接入 mnemo-core；增删改、复制、导入导出、i18n 见后续步骤。

use std::cell::Cell;
use std::rc::Rc;

use slint::winit_030::{winit, EventResult, WinitWindowAccessor};
use slint::{ComponentHandle, SharedString};

slint::include_modules!();

// UI 侧模块：数据层（SQLite 绑定与列表刷新）/ 路径解析（复用 com.longanl.mnemo 目录）
mod data;
mod paths;

// Slint 没有 monospace 通用族关键字，代码字体必须给具体族名，按平台选择。
#[cfg(target_os = "windows")]
const CODE_FONT_FAMILY: &str = "Consolas";
#[cfg(target_os = "macos")]
const CODE_FONT_FAMILY: &str = "Menlo";
#[cfg(all(unix, not(target_os = "macos")))]
const CODE_FONT_FAMILY: &str = "Noto Sans Mono";

/// 窗口交互的共享状态（只在 UI 线程访问，`Rc<Cell<..>>` 足够）。
///
/// 为什么需要它：Slint 的 `on_winit_window_event` 是**覆盖语义**
/// （i-slint-backend-winit 内部执行 `window_event_filter.set(Some(..))`），
/// 一个窗口只能注册一个钩子，因此"失焦隐藏"与"Esc 隐藏"必须写在同一个回调里。
#[derive(Clone, Copy)]
struct WindowFlags {
    /// 托盘图标是否创建成功。为 false 时禁止隐藏窗口——没有托盘就没有唤回入口。
    tray_ready: bool,
    /// 最小化进行中：抑制"失去焦点自动隐藏"。
    /// 最小化必然伴随 Focused(false)，不抑制的话"最小化到任务栏"会变成"隐藏到托盘"。
    minimizing: bool,
}

impl WindowFlags {
    fn new() -> Self {
        Self {
            tray_ready: true,
            minimizing: false,
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    // 失焦自动隐藏依赖 winit 的事件钩子，这里显式锁定 winit 后端。
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .select()?;

    // 与旧版（Tauri / egui）沿用同一应用标识，便于桌面环境归组。
    let _ = slint::set_xdg_app_id("com.longanl.mnemo");

    let flags = Rc::new(Cell::new(WindowFlags::new()));

    let ui = MainWindow::new()?;
    apply_code_font(&ui);
    install_callbacks(&ui);

    // 数据层：打开 commands.db（含建表/迁移）并做首次查询，把真实数据绑定给 State.commands
    let app_data = data::AppData::open();
    data::refresh_list(&ui, &app_data, "");
    install_data_bindings(&ui, &app_data);

    install_window_controls(&ui, flags.clone());
    install_window_event_hook(&ui, flags.clone());

    // 托盘必须早于窗口显示：无边框窗口一旦被隐藏，托盘是唯一的唤回入口。
    let tray = match Tray::new() {
        Ok(tray) => {
            install_tray(&tray, &ui, flags.clone());
            Some(tray)
        }
        Err(err) => {
            // 没有托盘 → 隐藏窗口后无法唤回。退化：禁用失焦隐藏，关闭/Esc 直接退出。
            eprintln!("[mnemo] 托盘图标创建失败：{err}");
            eprintln!("[mnemo] 退化策略：关闭按钮 / Esc 直接退出进程，失焦自动隐藏已禁用");
            let mut f = flags.get();
            f.tray_ready = false;
            flags.set(f);
            None
        }
    };

    ui.show()?;
    if let Some(tray) = &tray {
        tray.show()?;
    }
    // 托盘不在窗口组件树里，作用域结束就会被销毁（图标随之消失），这里显式持有到事件循环结束。
    let _keep_tray_alive = tray;

    // 使用 run_event_loop_until_quit：窗口隐藏（失焦自动隐藏 / Esc / 关闭按钮）后事件循环仍需继续，
    // 只有托盘菜单的 Quit（或"无托盘"退化路径）才退出。
    slint::run_event_loop_until_quit().expect("Slint event loop failed");
    Ok(())
}

// 把代码字体写入 Theme（P1 可扩展为按语言/系统字体回退）。
fn apply_code_font(ui: &MainWindow) {
    ui.global::<Theme>()
        .set_code_font_family(SharedString::from(CODE_FONT_FAMILY));
}

// 搜索框输入 → 重新查询并刷新列表模型。
// 每次输入都触发查询（本地 SQLite + LIKE 足够快，暂不做防抖）；关键词转义交给 mnemo-core。
fn install_data_bindings(ui: &MainWindow, app_data: &Rc<data::AppData>) {
    let app_data = app_data.clone();
    let ui_weak = ui.as_weak();
    ui.on_query_changed(move |query| {
        if let Some(ui) = ui_weak.upgrade() {
            data::refresh_list(&ui, &app_data, query.as_str());
        }
    });
}

// 尚未接入业务的回调：只打印日志，用于验证 UI → Rust 的接线（替换见后续步骤）。
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

// 窗口控制按钮（工具栏最右侧）：最小化到任务栏 / 关闭（= 隐藏到托盘，不退出进程）。
fn install_window_controls(ui: &MainWindow, flags: Rc<Cell<WindowFlags>>) {
    // 最小化
    {
        let flags = flags.clone();
        let ui_weak = ui.as_weak();
        ui.on_minimize_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                // 先置抑制标志再去最小化：最小化会触发 Focused(false)，
                // 而 winit 事件是随后派发的，因此这里置位一定能被钩子看到。
                let mut f = flags.get();
                f.minimizing = true;
                flags.set(f);
                ui.window().set_minimized(true);
            }
        });
    }

    // 关闭：隐藏窗口，进程与托盘继续运行（退出走托盘菜单）
    {
        let flags = flags.clone();
        let ui_weak = ui.as_weak();
        ui.on_hide_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                hide_or_quit(&ui, &flags);
            }
        });
    }
}

// 唯一的 winit 窗口事件钩子（Slint 只允许注册一个，原因见 WindowFlags 注释）：
// 1) 失去焦点自动隐藏（armed 标志确保窗口至少获得过一次焦点，避免启动瞬间误判）
// 2) 最小化过程中抑制上述隐藏，否则"最小化到任务栏"会变成"隐藏到托盘"
// 3) Esc 隐藏窗口：在 Rust 侧拦截，不依赖 Slint 侧谁持有焦点，任何时候都生效
fn install_window_event_hook(ui: &MainWindow, flags: Rc<Cell<WindowFlags>>) {
    let armed = Rc::new(Cell::new(false));
    ui.window()
        .on_winit_window_event(move |window, event| {
            match event {
                winit::event::WindowEvent::Focused(true) => {
                    armed.set(true);
                    // 重新获得焦点（含从任务栏恢复）：结束"最小化抑制"
                    let mut f = flags.get();
                    f.minimizing = false;
                    flags.set(f);
                }
                winit::event::WindowEvent::Focused(false) => {
                    let f = flags.get();
                    if armed.get() && f.tray_ready && !f.minimizing {
                        // TODO(P1)：接入 rfd 文件对话框（导入/导出）时，需要用 suppress 标志
                        // 临时禁用这里，否则打开对话框会让窗口自动隐藏。
                        let _ = window.hide();
                    }
                }
                winit::event::WindowEvent::KeyboardInput { event: key_event, .. } => {
                    let is_escape =
                        matches!(key_event.state, winit::event::ElementState::Pressed)
                            && matches!(
                                &key_event.logical_key,
                                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape)
                            );
                    if is_escape {
                        // Esc：隐藏窗口，进程与托盘继续运行
                        if flags.get().tray_ready {
                            let _ = window.hide();
                        } else {
                            let _ = slint::quit_event_loop();
                        }
                        // 已处理：不再传给 Slint，避免弹层等再处理一次
                        return EventResult::PreventDefault;
                    }
                }
                _ => {}
            }
            EventResult::Propagate
        });
}

// 托盘：菜单"显示主界面" → 唤回窗口；菜单"退出" → 结束事件循环。
// 左键点击托盘图标同样会走到这里：tray.slint 里把内置的 SystemTrayIcon.clicked 转发给了
// show-window（内置回调不会生成 Rust 访问器，所以转发必须在 .slint 侧做）。
// SystemTrayIcon 只有在 icon 非空时才真正创建托盘图标；Windows 上左键走 clicked、右键弹菜单。
fn install_tray(tray: &Tray, ui: &MainWindow, flags: Rc<Cell<WindowFlags>>) {
    let ui_weak = ui.as_weak();
    tray.on_show_window(move || {
        if let Some(ui) = ui_weak.upgrade() {
            show_window(&ui, &flags);
        }
    });

    tray.on_quit(|| {
        let _ = slint::quit_event_loop();
    });
}

// 唤回窗口：从"隐藏"与"最小化"两种状态都能正确恢复。
fn show_window(ui: &MainWindow, flags: &Rc<Cell<WindowFlags>>) {
    // 清掉"最小化抑制"，避免恢复之后失焦自动隐藏被永久禁用
    let mut f = flags.get();
    f.minimizing = false;
    flags.set(f);

    if ui.window().is_minimized() {
        // 必须先取消最小化，否则 show() 之后窗口仍停留在任务栏最小化状态
        ui.window().set_minimized(false);
    }
    let _ = ui.show();
}

// 隐藏窗口；若托盘不可用（隐藏后无法唤回）则退化为退出进程。
fn hide_or_quit(ui: &MainWindow, flags: &Rc<Cell<WindowFlags>>) {
    if flags.get().tray_ready {
        let _ = ui.hide();
    } else {
        let _ = slint::quit_event_loop();
    }
}
