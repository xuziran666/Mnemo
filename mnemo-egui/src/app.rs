use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui;
use mnemo_core::models::{Command, NewCommand, KIND_NOTE, KIND_SNIPPET};
use mnemo_core::rusqlite::Connection;

use crate::fonts;
use crate::i18n::{I18n, Lang};
use crate::paths;

// 当前顶层视图：列表 / 编辑 / 查看。
enum View {
    List,
    Editor,
    Viewer,
}

// 需要对选中条目执行的动作（来自快捷键或鼠标点击）。
#[derive(Clone, Copy)]
enum UiAction {
    Open,
    Copy,
    Edit,
    Delete,
}

// 界面主题：日间 / 夜间，与 egui 的 Theme 一一对应并持久化。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Theme {
    Light,
    Dark,
}

impl Theme {
    fn code(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

// 编辑器缓存字段，独立于 Command，便于在编辑期间自由修改。
#[derive(Default)]
struct EditorState {
    id: Option<i64>,
    kind: i64,
    title: String,
    content: String,
    note: String,
    tags: String,
}

pub struct MnemoApp {
    db: Result<Connection, String>,
    i18n: I18n,

    query: String,
    commands: Vec<Command>,
    selected: Option<usize>,
    // 键盘是否处于“列表激活”状态（由搜索框按 Enter 进入）。
    list_active: bool,
    view: View,
    theme: Theme,

    editor: EditorState,
    editor_from_viewer: bool,
    editor_focus_pending: bool,
    viewing: Option<Command>,

    confirm_delete: Option<Command>,
    pending_action: Option<(UiAction, Command)>,
    toast: Option<(String, Instant)>,
    clipboard: Option<arboard::Clipboard>,

    search_id: Option<egui::Id>,
    search_has_focus: bool,
    focus_search_pending: bool,
    blur_search_pending: bool,
    focused_once: bool,
    dirty: bool,
    scroll_to_selected: bool,
}

impl MnemoApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_fonts(install_fonts());
        // 首次运行没有设置文件时：语言按系统推断，主题跟随系统当前主题。
        let (lang, theme) = load_settings().unwrap_or_else(|| {
            let theme = match cc.egui_ctx.theme() {
                egui::Theme::Dark => Theme::Dark,
                egui::Theme::Light => Theme::Light,
            };
            (Lang::from_system(), theme)
        });
        apply_theme(&cc.egui_ctx, theme);
        let mut app = Self {
            db: open_database(),
            i18n: I18n::new(lang),
            query: String::new(),
            commands: Vec::new(),
            selected: None,
            list_active: false,
            view: View::List,
            theme,
            editor: EditorState::default(),
            editor_from_viewer: false,
            editor_focus_pending: false,
            viewing: None,
            confirm_delete: None,
            pending_action: None,
            toast: None,
            clipboard: None,
            search_id: None,
            search_has_focus: false,
            focus_search_pending: false,
            blur_search_pending: false,
            focused_once: false,
            dirty: false,
            scroll_to_selected: false,
        };
        app.reload();
        app
    }

    // ---------- 数据 ----------

    fn reload(&mut self) {
        let query = self.query.clone();
        let result = match &self.db {
            Ok(conn) => mnemo_core::list(conn, Some(query.as_str())),
            Err(_) => return,
        };
        if let Ok(commands) = result {
            self.commands = commands;
            if self.commands.is_empty() {
                self.selected = None;
            } else if let Some(index) = self.selected {
                if index >= self.commands.len() {
                    self.selected = Some(self.commands.len() - 1);
                }
            }
        }
    }

    fn selected_cmd(&self) -> Option<Command> {
        self.selected.and_then(|index| self.commands.get(index).cloned())
    }

    // ---------- 视图切换 ----------

    fn start_new(&mut self) {
        self.editor = EditorState {
            id: None,
            kind: KIND_SNIPPET,
            ..Default::default()
        };
        self.editor_from_viewer = false;
        self.editor_focus_pending = true;
        self.view = View::Editor;
    }

    fn start_edit(&mut self, id: i64) {
        let command = self
            .commands
            .iter()
            .find(|c| c.id == id)
            .cloned()
            .or_else(|| self.viewing.clone().filter(|c| c.id == id));
        let Some(command) = command else { return };
        self.editor = EditorState {
            id: Some(command.id),
            kind: command.kind,
            title: command.title,
            content: command.content,
            note: command.note.unwrap_or_default(),
            tags: command.tags.unwrap_or_default(),
        };
        self.editor_from_viewer = matches!(self.view, View::Viewer);
        self.editor_focus_pending = true;
        self.view = View::Editor;
    }

    fn open_viewer(&mut self, id: i64) {
        if let Some(command) = self.commands.iter().find(|c| c.id == id).cloned() {
            self.viewing = Some(command);
            self.view = View::Viewer;
        }
    }

    fn exit_viewer(&mut self) {
        self.view = View::List;
        self.viewing = None;
        self.list_active = false;
        self.focus_search_pending = true;
    }

    // ---------- 动作 ----------

    fn save_editor(&mut self) {
        let title = self.editor.title.trim().to_string();
        let content = self.editor.content.trim().to_string();
        if title.is_empty() || content.is_empty() {
            return;
        }
        let input = NewCommand {
            title,
            content,
            note: trimmed_opt(&self.editor.note),
            tags: trimmed_opt(&self.editor.tags),
            kind: self.editor.kind,
        };
        let result = match &self.db {
            Ok(conn) => match self.editor.id {
                Some(id) => mnemo_core::update(conn, id, input),
                None => mnemo_core::create(conn, input),
            },
            Err(error) => Err(error.clone()),
        };
        match result {
            Ok(saved) => {
                self.reload();
                if self.editor_from_viewer {
                    self.viewing = Some(saved);
                    self.view = View::Viewer;
                } else {
                    self.view = View::List;
                }
                let message = self.i18n.t("toast.saved");
                self.show_toast(message);
            }
            Err(error) => self.show_toast(error),
        }
    }

    fn perform_delete(&mut self, id: i64) {
        let result = match &self.db {
            Ok(conn) => mnemo_core::delete(conn, id),
            Err(error) => Err(error.clone()),
        };
        match result {
            Ok(()) => self.reload(),
            Err(_) => {
                let message = self.i18n.t("toast.deleteFailed");
                self.show_toast(message);
            }
        }
    }

    fn copy_to_clipboard(&mut self, content: &str, ctx: &egui::Context) {
        let result = match self.clipboard() {
            Ok(clipboard) => clipboard.set_text(content.to_owned()).map_err(|e| e.to_string()),
            Err(error) => Err(error),
        };
        match result {
            Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Err(_) => {
                let message = self.i18n.t("toast.copyFailed");
                self.show_toast(message);
            }
        }
    }

    fn clipboard(&mut self) -> Result<&mut arboard::Clipboard, String> {
        if self.clipboard.is_none() {
            self.clipboard = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
        }
        Ok(self.clipboard.as_mut().expect("clipboard initialized"))
    }

    fn toggle_lang(&mut self) {
        let next = match self.i18n.lang() {
            Lang::En => Lang::Zh,
            Lang::Zh => Lang::En,
        };
        self.i18n.set_lang(next);
        save_settings(next, self.theme);
    }

    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.theme = match self.theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
        apply_theme(ctx, self.theme);
        save_settings(self.i18n.lang(), self.theme);
    }

    fn do_export(&mut self) {
        let json = match &self.db {
            Ok(conn) => mnemo_core::export(conn),
            Err(error) => Err(error.clone()),
        };
        let Ok(json) = json else {
            let message = self.i18n.t("toast.exportFailed");
            self.show_toast(message);
            return;
        };
        let filter = self.i18n.t("io.exportFilter");
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("mnemo-export.json")
            .add_filter(filter, &["json"])
            .save_file()
        {
            if std::fs::write(&path, json).is_ok() {
                let message = self.i18n.t("toast.exported");
                self.show_toast(message);
            } else {
                let message = self.i18n.t("toast.exportFailed");
                self.show_toast(message);
            }
        }
    }

    fn do_import(&mut self) {
        let filter = self.i18n.t("io.importFilter");
        let Some(path) = rfd::FileDialog::new().add_filter(filter, &["json"]).pick_file() else {
            return;
        };
        let json = match std::fs::read_to_string(&path) {
            Ok(json) => json,
            Err(_) => {
                let message = self.i18n.t("toast.importFailed");
                self.show_toast(message);
                return;
            }
        };
        let result = match &self.db {
            Ok(conn) => mnemo_core::import(conn, json),
            Err(error) => Err(error.clone()),
        };
        match result {
            Ok(outcome) => {
                self.reload();
                let imported = outcome.imported.to_string();
                let skipped = outcome.skipped.to_string();
                let message = if outcome.skipped > 0 {
                    self.i18n.t_args(
                        "toast.importSkipped",
                        &[("imported", imported.as_str()), ("skipped", skipped.as_str())],
                    )
                } else {
                    self.i18n
                        .t_args("toast.imported", &[("count", imported.as_str())])
                };
                self.show_toast(message);
            }
            Err(_) => {
                let message = self.i18n.t("toast.importFailed");
                self.show_toast(message);
            }
        }
    }

    fn show_toast(&mut self, message: String) {
        self.toast = Some((message, Instant::now() + Duration::from_millis(1500)));
    }

    fn move_selection(&mut self, delta: i32) {
        if self.commands.is_empty() {
            self.selected = None;
            return;
        }
        let last = self.commands.len() as i32 - 1;
        let current = self.selected.map(|i| i as i32).unwrap_or(-1);
        let next = (current + delta).clamp(0, last);
        self.selected = Some(next as usize);
        self.scroll_to_selected = true;
    }

    fn apply_pending(&mut self, ctx: &egui::Context) {
        if let Some((action, command)) = self.pending_action.take() {
            match action {
                UiAction::Open => self.open_viewer(command.id),
                UiAction::Copy => {
                    if command.kind != KIND_NOTE {
                        self.copy_to_clipboard(&command.content, ctx);
                    }
                }
                UiAction::Edit => self.start_edit(command.id),
                UiAction::Delete => self.confirm_delete = Some(command),
            }
        }
    }

    fn apply_focus(&mut self, ctx: &egui::Context) {
        if self.focus_search_pending {
            if let Some(id) = self.search_id {
                ctx.memory_mut(|memory| memory.request_focus(id));
            }
            self.focus_search_pending = false;
        }
        if self.blur_search_pending {
            if let Some(id) = self.search_id {
                ctx.memory_mut(|memory| memory.surrender_focus(id));
            }
            self.blur_search_pending = false;
        }
    }

    // ---------- 列表视图 ----------

    fn ui_list(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let hint = self.i18n.t("search.placeholder");
                let button_width = 5.0 * 32.0 + ui.spacing().item_spacing.x * 6.0;
                let width = (ui.available_width() - button_width).max(120.0);

                // 左侧：语言切换
                if ui
                    .button(self.i18n.t("lang.button"))
                    .on_hover_text(self.i18n.t("lang.title"))
                    .clicked()
                {
                    self.toggle_lang();
                }

                // 中间：搜索框
                let response = ui.add_sized(
                    [width, 26.0],
                    egui::TextEdit::singleline(&mut self.query).hint_text(hint),
                );
                self.search_id = Some(response.id);
                self.search_has_focus = response.has_focus();
                if response.changed() {
                    self.dirty = true;
                }
                if !self.focused_once {
                    response.request_focus();
                    self.focused_once = true;
                }

                // 搜索框右侧：主题切换
                let (theme_icon, theme_tip) = match self.theme {
                    Theme::Light => ("🌙", self.i18n.t("theme.toDark")),
                    Theme::Dark => ("☀", self.i18n.t("theme.toLight")),
                };
                if ui.button(theme_icon).on_hover_text(theme_tip).clicked() {
                    self.toggle_theme(ui.ctx());
                }
                if ui
                    .button("⇩")
                    .on_hover_text(self.i18n.t("io.import"))
                    .clicked()
                {
                    self.do_import();
                }
                if ui
                    .button("⇧")
                    .on_hover_text(self.i18n.t("io.export"))
                    .clicked()
                {
                    self.do_export();
                }
                if ui
                    .button("＋")
                    .on_hover_text(self.i18n.t("add.title"))
                    .clicked()
                {
                    self.start_new();
                }
            });
            ui.separator();
            self.ui_list_body(ui);
        });

        if self.dirty {
            self.dirty = false;
            self.reload();
        }

        self.handle_list_hotkeys(&ctx);
        self.apply_pending(&ctx);
        self.apply_focus(&ctx);
    }

    fn ui_list_body(&mut self, ui: &mut egui::Ui) {
        if self.commands.is_empty() {
            ui.add_space(24.0);
            let empty = self.i18n.t("empty.noCommands");
            ui.vertical_centered(|ui| {
                ui.weak(empty);
            });
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let len = self.commands.len();
                for index in 0..len {
                    let command = self.commands[index].clone();
                    let selected = Some(index) == self.selected;
                    let mut action: Option<UiAction> = None;

                    let fill = if selected {
                        ui.visuals().selection.bg_fill
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    let frame = egui::Frame::NONE
                        .fill(fill)
                        .inner_margin(egui::Margin::same(6))
                        .corner_radius(egui::CornerRadius::same(4))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let is_note = command.kind == KIND_NOTE;
                                let (badge, color) = if is_note {
                                    (self.i18n.t("badge.note"), note_color())
                                } else {
                                    (self.i18n.t("badge.snippet"), snippet_color())
                                };

                                let main = ui.scope(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(color, badge);
                                        ui.vertical(|ui| {
                                            ui.strong(command.title.as_str());
                                            if !is_note {
                                                ui.monospace(command.content.as_str());
                                            }
                                            if let Some(note) = &command.note {
                                                ui.weak(note.as_str());
                                            }
                                            if let Some(tags) = &command.tags {
                                                tag_row(ui, tags);
                                            }
                                        });
                                    });
                                });
                                let main_response = main.response.interact(egui::Sense::click());
                                if main_response.clicked() {
                                    action = Some(if is_note {
                                        UiAction::Open
                                    } else {
                                        UiAction::Copy
                                    });
                                }
                                main_response.on_hover_cursor(egui::CursorIcon::PointingHand);

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button(self.i18n.t("actions.delete")).clicked() {
                                            action = Some(UiAction::Delete);
                                        }
                                        if ui.button(self.i18n.t("actions.edit")).clicked() {
                                            action = Some(UiAction::Edit);
                                        }
                                        if ui.button(self.i18n.t("actions.copy")).clicked() {
                                            action = Some(UiAction::Copy);
                                        }
                                        if ui.button(self.i18n.t("actions.view")).clicked() {
                                            action = Some(UiAction::Open);
                                        }
                                    },
                                );
                            });
                        });

                    if selected && self.scroll_to_selected {
                        frame.response.scroll_to_me(Some(egui::Align::Center));
                    }

                    if let Some(action) = action {
                        self.pending_action = Some((action, command));
                    }
                }
            });

        self.scroll_to_selected = false;
    }

    fn handle_list_hotkeys(&mut self, ctx: &egui::Context) {
        let search_focused = self.search_has_focus;
        let mut focus_search = false;
        let mut activate_from_search = false;
        let mut activate = false;
        let mut move_down = false;
        let mut move_up = false;
        let mut new = false;
        let mut close = false;
        let mut command_action: Option<UiAction> = None;

        ctx.input(|input| {
            if input.modifiers.command && input.key_pressed(egui::Key::N) {
                new = true;
                return;
            }
            if input.key_pressed(egui::Key::Escape) {
                close = true;
                return;
            }
            if input.key_pressed(egui::Key::Enter) {
                if search_focused {
                    activate_from_search = true;
                } else if self.list_active {
                    activate = true;
                }
                return;
            }
            if search_focused {
                return;
            }
            if input.key_pressed(egui::Key::S) {
                focus_search = true;
                return;
            }
            if self.list_active {
                if input.key_pressed(egui::Key::ArrowDown) {
                    move_down = true;
                } else if input.key_pressed(egui::Key::ArrowUp) {
                    move_up = true;
                } else if input.key_pressed(egui::Key::R) {
                    command_action = Some(UiAction::Edit);
                } else if input.key_pressed(egui::Key::C) {
                    command_action = Some(UiAction::Copy);
                } else if input.key_pressed(egui::Key::V) {
                    command_action = Some(UiAction::Open);
                } else if input.key_pressed(egui::Key::D) {
                    command_action = Some(UiAction::Delete);
                }
            }
        });

        if new {
            self.start_new();
            return;
        }
        if close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if focus_search {
            self.list_active = false;
            self.focus_search_pending = true;
        }
        if activate_from_search {
            self.list_active = true;
            self.blur_search_pending = true;
            if !self.commands.is_empty() {
                self.selected = Some(0);
                self.scroll_to_selected = true;
            }
        }
        if move_down {
            self.move_selection(1);
        }
        if move_up {
            self.move_selection(-1);
        }
        if let Some(action) = command_action {
            if let Some(command) = self.selected_cmd() {
                self.pending_action = Some((action, command));
            }
        }
        if activate {
            if let Some(command) = self.selected_cmd() {
                let action = if command.kind == KIND_NOTE {
                    UiAction::Open
                } else {
                    UiAction::Copy
                };
                self.pending_action = Some((action, command));
            }
        }
    }

    // ---------- 编辑视图 ----------

    fn ui_editor(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let mut do_cancel = false;
        let mut do_save = false;
        ctx.input(|input| {
            if input.key_pressed(egui::Key::Escape) {
                do_cancel = true;
            }
            if input.modifiers.command && input.key_pressed(egui::Key::S) {
                do_save = true;
            }
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let snippet = self.i18n.t("editor.kindSnippet");
                let note = self.i18n.t("editor.kindNote");
                if ui
                    .selectable_label(self.editor.kind == KIND_SNIPPET, snippet)
                    .clicked()
                {
                    self.editor.kind = KIND_SNIPPET;
                }
                if ui
                    .selectable_label(self.editor.kind == KIND_NOTE, note)
                    .clicked()
                {
                    self.editor.kind = KIND_NOTE;
                }
            });
            ui.add_space(6.0);

            let title_hint = self.i18n.t("editor.titlePlaceholder");
            let title_response = ui.add(
                egui::TextEdit::singleline(&mut self.editor.title)
                    .hint_text(title_hint)
                    .desired_width(f32::INFINITY),
            );
            if self.editor_focus_pending {
                title_response.request_focus();
                self.editor_focus_pending = false;
            }

            ui.add_space(6.0);
            let content_hint = self.i18n.t("editor.contentPlaceholder");
            ui.add(
                egui::TextEdit::multiline(&mut self.editor.content)
                    .hint_text(content_hint)
                    .desired_rows(10)
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(6.0);
            let note_hint = self.i18n.t("editor.notePlaceholder");
            ui.add(
                egui::TextEdit::singleline(&mut self.editor.note)
                    .hint_text(note_hint)
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(6.0);
            let tags_hint = self.i18n.t("editor.tagsPlaceholder");
            ui.add(
                egui::TextEdit::singleline(&mut self.editor.tags)
                    .hint_text(tags_hint)
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(10.0);
            let can_save = !self.editor.title.trim().is_empty()
                && !self.editor.content.trim().is_empty();
            ui.horizontal(|ui| {
                if ui.button(self.i18n.t("editor.cancel")).clicked() {
                    do_cancel = true;
                }
                if ui
                    .add_enabled(can_save, egui::Button::new(self.i18n.t("editor.save")))
                    .clicked()
                {
                    do_save = true;
                }
            });
        });

        if do_cancel {
            self.view = View::List;
            return;
        }
        if do_save {
            self.save_editor();
        }
    }

    // ---------- 查看视图 ----------

    fn ui_viewer(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let Some(command) = self.viewing.clone() else {
            self.view = View::List;
            return;
        };
        let mut do_exit = false;
        let mut do_edit = false;
        ctx.input(|input| {
            if input.key_pressed(egui::Key::Escape) {
                do_exit = true;
            }
            if input.key_pressed(egui::Key::Enter) && !input.modifiers.command {
                do_edit = true;
            }
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(6.0);
            let is_note = command.kind == KIND_NOTE;
            let (badge, color) = if is_note {
                (self.i18n.t("badge.note"), note_color())
            } else {
                (self.i18n.t("badge.snippet"), snippet_color())
            };
            ui.horizontal(|ui| {
                ui.colored_label(color, badge);
                ui.heading(command.title.as_str());
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if is_note {
                        ui.label(command.content.as_str());
                    } else {
                        ui.monospace(command.content.as_str());
                    }
                    if let Some(note) = &command.note {
                        ui.add_space(8.0);
                        ui.weak(note.as_str());
                    }
                    if let Some(tags) = &command.tags {
                        ui.add_space(8.0);
                        tag_row(ui, tags);
                    }
                });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button(self.i18n.t("viewer.cancel")).clicked() {
                    do_exit = true;
                }
                if ui.button(self.i18n.t("viewer.edit")).clicked() {
                    do_edit = true;
                }
            });
        });

        if do_exit {
            self.exit_viewer();
            return;
        }
        if do_edit {
            self.start_edit(command.id);
        }
    }

    // ---------- 弹层 ----------

    fn ui_confirm(&mut self, ctx: &egui::Context) {
        let Some(command) = self.confirm_delete.clone() else {
            return;
        };
        let mut do_close = false;
        let mut do_confirm = false;
        egui::Window::new(self.i18n.t("actions.delete"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                let message = self
                    .i18n
                    .t_args("confirm.delete", &[("title", command.title.as_str())]);
                ui.label(message);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(self.i18n.t("editor.cancel")).clicked() {
                        do_close = true;
                    }
                    if ui.button(self.i18n.t("actions.delete")).clicked() {
                        do_confirm = true;
                    }
                });
            });

        if do_close {
            self.confirm_delete = None;
        }
        if do_confirm {
            self.confirm_delete = None;
            self.perform_delete(command.id);
        }
    }

    fn ui_toast(&mut self, ctx: &egui::Context) {
        let expired = matches!(&self.toast, Some((_, until)) if Instant::now() >= *until);
        if expired {
            self.toast = None;
        }
        let Some((message, _)) = self.toast.clone() else {
            return;
        };
        egui::Area::new(egui::Id::new("mnemo_toast"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -24.0))
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(egui::Color32::from_black_alpha(220))
                    .inner_margin(egui::Margin::same(8))
                    .corner_radius(egui::CornerRadius::same(6))
                    .show(ui, |ui| {
                        ui.colored_label(egui::Color32::WHITE, message);
                    });
            });
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

impl eframe::App for MnemoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if let Err(error) = &self.db {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add_space(20.0);
                ui.heading("Mnemo");
                ui.colored_label(egui::Color32::RED, error.as_str());
            });
            return;
        }

        match self.view {
            View::List => self.ui_list(ui),
            View::Editor => self.ui_editor(ui),
            View::Viewer => self.ui_viewer(ui),
        }
        self.ui_confirm(&ctx);
        self.ui_toast(&ctx);
    }
}

// ---------- 辅助函数 ----------

fn trimmed_opt(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn tag_row(ui: &mut egui::Ui, tags: &str) {
    let items: Vec<&str> = tags
        .split(',')
        .map(|tag| tag.trim())
        .filter(|tag| !tag.is_empty())
        .collect();
    if items.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        for tag in items {
            ui.label(egui::RichText::new(tag).small().color(egui::Color32::GRAY));
        }
    });
}

fn snippet_color() -> egui::Color32 {
    egui::Color32::from_rgb(0x3b, 0x6d, 0x8a)
}

fn note_color() -> egui::Color32 {
    egui::Color32::from_rgb(0x8a, 0x6d, 0x3b)
}

fn install_fonts() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    if let Some(font) = fonts::resolve_cjk_font() {
        eprintln!("[mnemo] CJK font: {} ({})", font.name, font.source);
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
    }
    fonts
}

fn open_database() -> Result<Connection, String> {
    let path = paths::database_path().ok_or_else(|| "无法解析应用数据目录".to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    mnemo_core::open_db(path)
}

fn settings_path() -> Option<std::path::PathBuf> {
    paths::app_data_dir().map(|dir| dir.join("settings.json"))
}

fn apply_theme(ctx: &egui::Context, theme: Theme) {
    ctx.set_theme(match theme {
        Theme::Light => egui::Theme::Light,
        Theme::Dark => egui::Theme::Dark,
    });
}

fn load_settings() -> Option<(Lang, Theme)> {
    let text = std::fs::read_to_string(settings_path()?).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    let lang = match value.get("lang")?.as_str()? {
        "zh" => Lang::Zh,
        "en" => Lang::En,
        _ => return None,
    };
    let theme = match value.get("theme").and_then(|v| v.as_str()) {
        Some("dark") => Theme::Dark,
        _ => Theme::Light,
    };
    Some((lang, theme))
}

fn save_settings(lang: Lang, theme: Theme) {
    if let Some(path) = settings_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let value = serde_json::json!({ "lang": lang.code(), "theme": theme.code() });
        let _ = std::fs::write(path, value.to_string());
    }
}
