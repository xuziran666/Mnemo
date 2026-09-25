// Mnemo（Slint 版）入口：窗口行为、系统托盘、数据层与业务回调的装配。
// 分层：
//   data.rs     —— SQLite 连接、查询缓存、CRUD 包装、列表模型刷新
//   strings.rs  —— 把 i18n 文案写进 Slint 的 Strings / State / 托盘菜单
//   i18n.rs     —— 语言资源（locales/*.json，构建期嵌入二进制）
//   settings.rs —— 语言选择持久化（与旧版 egui 共用 settings.json）
//   paths.rs    —— 数据目录（com.longanl.mnemo）
//   ui/*.slint  —— 视图与弹层；键盘捕获在 .slint（FocusScope + KeyBinding），Esc 在本文件（winit 钩子）

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use mnemo_core::models::{NewCommand, KIND_SNIPPET};
use slint::winit_030::{winit, EventResult, WinitWindowAccessor};
use slint::{ComponentHandle, SharedString};

slint::include_modules!();

mod data;
mod i18n;
mod paths;
mod settings;
mod strings;

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
    /// 原生文件对话框（导入/导出）打开中：同样抑制失焦隐藏。
    /// rfd 的对话框会让主窗口真失焦，不抑制的话窗口会藏到对话框背后。
    dialog_open: bool,
}

impl WindowFlags {
    fn new() -> Self {
        Self {
            tray_ready: true,
            minimizing: false,
            dialog_open: false,
        }
    }
}

// 原生文件对话框打开/关闭时切换抑制标志（见 WindowFlags::dialog_open）。
fn set_dialog_open(flags: &Rc<Cell<WindowFlags>>, open: bool) {
    let mut f = flags.get();
    f.dialog_open = open;
    flags.set(f);
}

fn main() -> Result<(), slint::PlatformError> {
    // 失焦自动隐藏依赖 winit 的事件钩子，这里显式锁定 winit 后端。
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .select()?;

    // 与旧版（Tauri / egui）沿用同一应用标识，便于桌面环境归组。
    let _ = slint::set_xdg_app_id("com.longanl.mnemo");

    let flags = Rc::new(Cell::new(WindowFlags::new()));

    // 语言与主题：优先用 settings.json 里的持久化值（与旧版 egui 共用同一字段），
    // 没有则语言按系统 locale 推断、主题默认暗色。
    let lang = settings::load_lang().unwrap_or_else(i18n::Lang::from_system);
    let is_dark = settings::load_theme().unwrap_or(true);
    let i18n = Rc::new(RefCell::new(i18n::I18n::new(lang)));
    // Toast 的"最后一次提示"令牌：连续提示时避免旧定时器把新提示提前关掉
    let toast_token = Rc::new(Cell::new(0u32));

    let ui = MainWindow::new()?;
    apply_code_font(&ui);
    // 主题状态必须在窗口显示前写入：界面所有颜色都是基于它的条件绑定
    ui.global::<Theme>().set_is_dark(is_dark);
    strings::apply(&ui, &i18n.borrow());

    // 数据层：打开 commands.db（含建表/迁移）并做首次查询，把真实数据绑定给 State.commands
    let app_data = data::AppData::open();
    data::refresh_list(&ui, &app_data, "");
    install_data_bindings(&ui, &app_data);
    install_business_callbacks(&ui, &app_data, &flags, &i18n, &toast_token);
    install_io_callbacks(&ui, &app_data, &flags, &i18n, &toast_token);

    install_window_controls(&ui, flags.clone());
    install_window_event_hook(&ui, flags.clone());

    // 托盘必须早于窗口显示：无边框窗口一旦被隐藏，托盘是唯一的唤回入口。
    let tray: Option<Rc<Tray>> = match Tray::new() {
        Ok(tray) => {
            install_tray(&tray, &ui, flags.clone());
            Some(Rc::new(tray))
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
    if let Some(tray) = &tray {
        strings::apply_tray(tray, &i18n.borrow());
    }
    install_preference_callbacks(&ui, tray.clone(), i18n.clone());

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

// ---------- 导入 / 导出（原生文件对话框）----------
// 说明：rfd 的对话框是阻塞式的（调用期间主线程进入对话框自己的消息循环），与旧版 egui 的做法一致。
// 期间必须抑制"失焦自动隐藏"（set_dialog_open），否则窗口会被藏到对话框背后。
fn install_io_callbacks(
    ui: &MainWindow,
    app_data: &Rc<data::AppData>,
    flags: &Rc<Cell<WindowFlags>>,
    i18n: &Rc<RefCell<i18n::I18n>>,
    toast_token: &Rc<Cell<u32>>,
) {
    // 导出：库 → JSON → 保存对话框 → 写文件
    {
        let app_data = app_data.clone();
        let flags = flags.clone();
        let i18n = i18n.clone();
        let token = toast_token.clone();
        let ui_weak = ui.as_weak();
        ui.on_export_clicked(move || {
            let Some(ui) = ui_weak.upgrade() else { return };

            let json = match app_data.export() {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("[mnemo] 导出失败：{err}");
                    let message = i18n.borrow().t("toast.exportFailed");
                    show_toast(&ui, &message, &token);
                    return;
                }
            };

            let filter = i18n.borrow().t("io.exportFilter");
            set_dialog_open(&flags, true);
            let path = rfd::FileDialog::new()
                .set_file_name("mnemo-export.json")
                .add_filter(filter, &["json"])
                .save_file();
            set_dialog_open(&flags, false);

            // 用户取消：直接返回（不弹 Toast 打扰）
            let Some(path) = path else { return };
            match std::fs::write(&path, json) {
                Ok(()) => {
                    let message = i18n.borrow().t("toast.exported");
                    show_toast(&ui, &message, &token);
                }
                Err(err) => {
                    eprintln!("[mnemo] 写出文件失败：{err}");
                    let message = i18n.borrow().t("toast.exportFailed");
                    show_toast(&ui, &message, &token);
                }
            }
        });
    }

    // 导入：打开对话框 → 读文件 → 写库 → 刷新列表 + 带条数的 Toast
    {
        let app_data = app_data.clone();
        let flags = flags.clone();
        let i18n = i18n.clone();
        let token = toast_token.clone();
        let ui_weak = ui.as_weak();
        ui.on_import_clicked(move || {
            let Some(ui) = ui_weak.upgrade() else { return };

            let filter = i18n.borrow().t("io.importFilter");
            set_dialog_open(&flags, true);
            let path = rfd::FileDialog::new()
                .add_filter(filter, &["json"])
                .pick_file();
            set_dialog_open(&flags, false);

            let Some(path) = path else { return };
            let json = match std::fs::read_to_string(&path) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("[mnemo] 读取文件失败：{err}");
                    let message = i18n.borrow().t("toast.importFailed");
                    show_toast(&ui, &message, &token);
                    return;
                }
            };

            match app_data.import(json) {
                Ok(result) => {
                    // 与删除/保存一致：保持当前搜索条件刷新列表
                    data::refresh_current(&ui, &app_data);
                    let imported = result.imported.to_string();
                    let skipped = result.skipped.to_string();
                    let message = {
                        let i18n = i18n.borrow();
                        if result.skipped > 0 {
                            i18n.t_args(
                                "toast.importSkipped",
                                &[("imported", imported.as_str()), ("skipped", skipped.as_str())],
                            )
                        } else {
                            i18n.t_args("toast.imported", &[("count", imported.as_str())])
                        }
                    };
                    show_toast(&ui, &message, &token);
                }
                Err(err) => {
                    eprintln!("[mnemo] 导入失败：{err}");
                    let message = i18n.borrow().t("toast.importFailed");
                    show_toast(&ui, &message, &token);
                }
            }
        });
    }
}

// ---------- 业务回调：复制 / 选中移动 / 增删改 ----------
fn install_business_callbacks(
    ui: &MainWindow,
    app_data: &Rc<data::AppData>,
    flags: &Rc<Cell<WindowFlags>>,
    i18n: &Rc<RefCell<i18n::I18n>>,
    toast_token: &Rc<Cell<u32>>,
) {
    // 复制：写入剪贴板成功后隐藏窗口（无边框小工具"用完即走"的行为）
    {
        let app_data = app_data.clone();
        let flags = flags.clone();
        let i18n = i18n.clone();
        let token = toast_token.clone();
        let ui_weak = ui.as_weak();
        ui.on_copy_requested(move |id| {
            let Some(ui) = ui_weak.upgrade() else { return };
            let Some(command) = app_data.find(id) else { return };
            // 每次复制都新建 Clipboard 实例：短生命周期在 Windows 上更稳妥（写入后由系统接管数据）。
            // 失败不能只打日志，必须给用户 Toast 反馈。
            let copied = arboard::Clipboard::new()
                .and_then(|mut clipboard| clipboard.set_text(command.content))
                .map_err(|err| err.to_string());
            match copied {
                Ok(()) => hide_or_quit(&ui, &flags),
                Err(err) => {
                    eprintln!("[mnemo] 写入剪贴板失败：{err}");
                    let message = i18n.borrow().t("toast.copyFailed");
                    show_toast(&ui, &message, &token);
                }
            }
        });
    }

    // ↑ / ↓：移动选中项
    {
        let app_data = app_data.clone();
        let ui_weak = ui.as_weak();
        ui.on_select_move(move |delta| {
            if let Some(ui) = ui_weak.upgrade() {
                move_selection(&ui, &app_data, delta);
            }
        });
    }

    // 编辑：把该条填进编辑器（用列表缓存，不再查库）
    {
        let app_data = app_data.clone();
        let ui_weak = ui.as_weak();
        ui.on_edit_requested(move |id| {
            if let Some(ui) = ui_weak.upgrade() {
                if let Some(command) = app_data.find(id) {
                    open_editor(&ui, Some(&command));
                }
            }
        });
    }

    // 查看：点卡片「查看」按钮或按 V —— 用列表缓存填充 State.viewing 后打开查看弹层
    {
        let app_data = app_data.clone();
        let ui_weak = ui.as_weak();
        ui.on_view_requested(move |id| {
            if let Some(ui) = ui_weak.upgrade() {
                if let Some(command) = app_data.find(id) {
                    open_viewer(&ui, &command);
                }
            }
        });
    }

    // 查看弹层里的「编辑」：先关掉查看再打开编辑器，避免两个遮罩叠在一起
    {
        let app_data = app_data.clone();
        let ui_weak = ui.as_weak();
        ui.on_viewer_edit(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let state = ui.global::<State>();
                let id = state.get_viewing().id;
                state.set_viewing_open(false);
                if let Some(command) = app_data.find(id) {
                    open_editor(&ui, Some(&command));
                }
            }
        });
    }

    // 新建：清空编辑器。已在编辑时忽略，避免 Ctrl+N 连按把用户已输入的内容清掉。
    {
        let ui_weak = ui.as_weak();
        ui.on_new_clicked(move || {
            if let Some(ui) = ui_weak.upgrade() {
                if !ui.global::<State>().get_editor_open() {
                    open_editor(&ui, None);
                }
            }
        });
    }

    // 删除：先弹二次确认（破坏性操作），点"取消"直接在 .slint 侧关闭弹层
    {
        let app_data = app_data.clone();
        let i18n = i18n.clone();
        let ui_weak = ui.as_weak();
        ui.on_delete_requested(move |id| {
            let Some(ui) = ui_weak.upgrade() else { return };
            let Some(command) = app_data.find(id) else { return };
            let message = i18n
                .borrow()
                .t_args("confirm.delete", &[("title", command.title.as_str())]);
            let state = ui.global::<State>();
            state.set_confirm_message(message.into());
            state.set_confirm_id(id);
            state.set_confirm_open(true);
        });
    }

    // 确认删除：写库 → 按当前搜索条件刷新 → Toast
    {
        let app_data = app_data.clone();
        let i18n = i18n.clone();
        let token = toast_token.clone();
        let ui_weak = ui.as_weak();
        ui.on_confirm_ok(move || {
            let Some(ui) = ui_weak.upgrade() else { return };
            let state = ui.global::<State>();
            let id = state.get_confirm_id();
            state.set_confirm_open(false);

            match app_data.delete(id as i64) {
                Ok(()) => {
                    data::refresh_current(&ui, &app_data);
                    let message = i18n.borrow().t("toast.deleted");
                    show_toast(&ui, &message, &token);
                }
                Err(err) => {
                    eprintln!("[mnemo] 删除失败：{err}");
                    let message = i18n.borrow().t("toast.deleteFailed");
                    show_toast(&ui, &message, &token);
                }
            }
        });
    }

    // 保存：title 与 content 必填；不满足时只提示、不关弹层、不写库
    {
        let app_data = app_data.clone();
        let i18n = i18n.clone();
        let token = toast_token.clone();
        let ui_weak = ui.as_weak();
        ui.on_editor_save(move || {
            let Some(ui) = ui_weak.upgrade() else { return };
            let state = ui.global::<State>();

            let title = state.get_editor_title().trim().to_string();
            let content = state.get_editor_content().trim().to_string();
            if title.is_empty() || content.is_empty() {
                let message = i18n.borrow().t("toast.invalid");
                show_toast(&ui, &message, &token);
                return;
            }

            let input = NewCommand {
                title,
                content,
                note: trimmed_opt(&state.get_editor_note()),
                tags: trimmed_opt(&state.get_editor_tags()),
                kind: state.get_editor_kind() as i64,
            };
            let id = state.get_editor_id();
            let saved = if id > 0 {
                app_data.update(id as i64, input)
            } else {
                app_data.create(input)
            };

            match saved {
                Ok(command) => {
                    // 保持当前搜索条件刷新，并选中刚保存的条目
                    data::refresh_current(&ui, &app_data);
                    // 若保存的条目不满足当前过滤条件，则不强行选中（refresh 已修正过选中项）
                    if app_data.find(command.id as i32).is_some() {
                        state.set_selected_id(command.id as i32);
                    }
                    state.set_editor_open(false);
                    let message = i18n.borrow().t("toast.saved");
                    show_toast(&ui, &message, &token);
                }
                Err(err) => {
                    eprintln!("[mnemo] 保存失败：{err}");
                    let message = i18n.borrow().t("toast.saveFailed");
                    show_toast(&ui, &message, &token);
                }
            }
        });
    }
}

// 自定义顶栏的窗口控制：拖动 / 最小化 / 最大化还原 / 关闭（= 隐藏到托盘，不退出进程）。
fn install_window_controls(ui: &MainWindow, flags: Rc<Cell<WindowFlags>>) {
    // 拖动：调用 winit 的系统级窗口拖动（Slint 的 WindowMoveArea 内部用的也是同一个 API）。
    // 拖动阈值判断在 TitleBar.slint 里完成，这里只在确实要开始拖动时才被调用。
    {
        let ui_weak = ui.as_weak();
        ui.on_drag_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.window().with_winit_window(|window| {
                    if let Err(err) = window.drag_window() {
                        eprintln!("[mnemo] 启动窗口拖动失败：{err}");
                    }
                });
            }
        });
    }

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

    // 最大化 / 还原（顶栏按钮与双击顶栏空白走的是同一条路径）
    {
        let ui_weak = ui.as_weak();
        ui.on_maximize_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                toggle_maximize(&ui);
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
// 3) Esc：弹层（编辑器/确认框）打开时只关弹层，否则隐藏窗口。放在 Rust 侧拦截，
//    与 .slint 里的 FocusScope（↑/↓/Enter/Ctrl+N/s）分工明确、互不重叠。
fn install_window_event_hook(ui: &MainWindow, flags: Rc<Cell<WindowFlags>>) {
    let armed = Rc::new(Cell::new(false));
    // 判断弹层是否需要优先处理需要读 State，因此钩子里也持有窗口弱引用
    let ui_weak = ui.as_weak();
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
                    // 两种抑制：最小化中（否则最小化会变成隐藏）、原生文件对话框打开中（rfd 会让窗口真失焦）
                    if armed.get() && f.tray_ready && !f.minimizing && !f.dialog_open {
                        let _ = window.hide();
                    }
                }
                winit::event::WindowEvent::Resized(_) => {
                    // 最大化 / 还原也可能由外部触发（Win+↑、任务栏菜单、系统快捷键……），
                    // 这里把真实状态同步给 State，保证顶栏"最大化/还原"图标始终正确。
                    if let Some(ui) = ui_weak.upgrade() {
                        let maximized = window.is_maximized();
                        let state = ui.global::<State>();
                        if state.get_maximized() != maximized {
                            state.set_maximized(maximized);
                        }
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
                        // 弹层优先：编辑器 / 确认框打开时，Esc 只关弹层，不隐藏整个窗口
                        let handled_by_overlay = ui_weak
                            .upgrade()
                            .map(|ui| close_top_overlay(&ui))
                            .unwrap_or(false);
                        if !handled_by_overlay {
                            // 无弹层：隐藏窗口，进程与托盘继续运行
                            if flags.get().tray_ready {
                                let _ = window.hide();
                            } else {
                                let _ = slint::quit_event_loop();
                            }
                        }
                        // 已处理：不再传给 Slint
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

// ---------- 辅助函数 ----------

// ↑/↓ 移动选中项。
//
// 为什么下标计算放在 Rust 而不是 Slint：Slint 的 for 循环没有下标变量，选中态只能用 id 表示
// （State.selected-id），"当前项的下标"只能在持有列表缓存的这一侧换算；否则 .slint 里要额外维护
// 一份 selected-index，搜索过滤/增删之后极易与 id 失去同步。
fn move_selection(ui: &MainWindow, app_data: &data::AppData, delta: i32) {
    let state = ui.global::<State>();
    let ids = app_data.ids();
    if ids.is_empty() {
        state.set_selected_id(0);
        return;
    }

    let next = match app_data.index_of(state.get_selected_id()) {
        Some(index) => (index as i32 + delta).clamp(0, ids.len() as i32 - 1) as usize,
        // 当前没有选中（或选中项已被搜索过滤掉）：↓ 从第一条开始，↑ 从最后一条开始
        None if delta > 0 => 0,
        None => ids.len() - 1,
    };
    state.set_selected_id(ids[next]);
}

// 打开编辑器：Some = 编辑（回填字段），None = 新建（清空字段）。
fn open_editor(ui: &MainWindow, command: Option<&mnemo_core::models::Command>) {
    let state = ui.global::<State>();
    match command {
        Some(command) => {
            state.set_editor_id(command.id as i32);
            state.set_editor_kind(command.kind as i32);
            state.set_editor_title(command.title.as_str().into());
            state.set_editor_content(command.content.as_str().into());
            state.set_editor_note(command.note.clone().unwrap_or_default().into());
            state.set_editor_tags(command.tags.clone().unwrap_or_default().into());
        }
        None => {
            state.set_editor_id(0);
            state.set_editor_kind(KIND_SNIPPET as i32);
            state.set_editor_title(SharedString::default());
            state.set_editor_content(SharedString::default());
            state.set_editor_note(SharedString::default());
            state.set_editor_tags(SharedString::default());
        }
    }
    state.set_editor_open(true);
}

// 打开查看弹层：把该条从列表缓存填进 State.viewing（只读展示）。
// 复用 data::to_item，保证与列表卡片是同一套字段映射（note 空串、tags 数组解析规则一致）。
fn open_viewer(ui: &MainWindow, command: &mnemo_core::models::Command) {
    let state = ui.global::<State>();
    state.set_viewing(data::to_item(command));
    state.set_viewing_open(true);
}

// Esc 的"弹层优先"处理：返回 true 表示已被弹层消费（此时不隐藏窗口）。
// 弹层状态都在 State 全局里，Rust 侧能直接判定，因此 Esc 仍然只需要一个入口。
fn close_top_overlay(ui: &MainWindow) -> bool {
    let state = ui.global::<State>();
    if state.get_editor_open() {
        state.set_editor_open(false);
        true
    } else if state.get_confirm_open() {
        state.set_confirm_open(false);
        true
    } else if state.get_viewing_open() {
        state.set_viewing_open(false);
        true
    } else {
        false
    }
}

// 底部 Toast：显示 1.5s 后自动关闭（时长与旧版 egui 一致）。
// token 处理连续提示：旧定时器发现 token 已变就什么都不做，不会把新提示提前关掉。
// 注意：成功后不弹"已复制"——窗口会立即隐藏，弹了也看不见；失败才需要提示。
fn show_toast(ui: &MainWindow, text: &str, token: &Rc<Cell<u32>>) {
    let state = ui.global::<State>();
    state.set_toast_text(SharedString::from(text));
    state.set_toast_visible(true);

    let current = token.get().wrapping_add(1);
    token.set(current);
    let token = token.clone();
    let ui_weak = ui.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(1500), move || {
        if token.get() == current {
            if let Some(ui) = ui_weak.upgrade() {
                ui.global::<State>().set_toast_visible(false);
            }
        }
    });
}

// 主题与语言：都是"点击即切换"，切换后持久化到 settings.json（字段与旧版 egui 共用）。
fn install_preference_callbacks(
    ui: &MainWindow,
    tray: Option<Rc<Tray>>,
    i18n: Rc<RefCell<i18n::I18n>>,
) {
    // 主题：只切换 Theme 全局的 is-dark。界面所有颜色都是它的条件绑定，
    // Slint 的绑定系统会自动重算并重绘，不需要逐控件通知。
    {
        let ui_weak = ui.as_weak();
        ui.on_theme_clicked(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let theme = ui.global::<Theme>();
                let is_dark = !theme.get_is_dark();
                theme.set_is_dark(is_dark);
                settings::save_theme(is_dark);
            }
        });
    }

    // 语言：中英互换 → 重写界面与托盘文案 → 持久化。
    // 语言按钮与托盘菜单文案要立刻跟着变，所以托盘也要重新应用一次。
    {
        let ui_weak = ui.as_weak();
        ui.on_lang_clicked(move || {
            let next = i18n.borrow().lang().toggled();
            i18n.borrow_mut().set_lang(next);
            {
                let i18n = i18n.borrow();
                if let Some(ui) = ui_weak.upgrade() {
                    strings::apply(&ui, &i18n);
                }
                if let Some(tray) = &tray {
                    strings::apply_tray(tray, &i18n);
                }
            }
            settings::save_lang(next);
        });
    }
}

// 切换最大化 / 还原，并把结果同步到 State（顶栏图标与"最大化时禁止拖动"都依赖它）。
fn toggle_maximize(ui: &MainWindow) {
    let window = ui.window();
    let next = !window.is_maximized();
    window.set_maximized(next);
    ui.global::<State>().set_maximized(next);
}

// 空字符串 → None（备注/标签留空时不写库，与旧版 egui 的行为一致）。
fn trimmed_opt(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
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
