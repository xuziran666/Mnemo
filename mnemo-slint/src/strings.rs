use slint::ComponentHandle;

use crate::i18n::I18n;
use crate::{MainWindow, Strings, Tray};

// 把语言资源写进界面文案与托盘菜单。
//
// 新增一条文案的固定三步：
// 1) locales/en.json 与 zh.json 各加一个 key（如 toast.saved）；
// 2) ui/state.slint 的 Strings 里加同名 kebab-case 属性（如 toast-saved）；
// 3) 在本文件的 apply() 里加一行 setter。
pub fn apply(ui: &MainWindow, i18n: &I18n) {
    let strings = ui.global::<Strings>();

    // 搜索与工具栏
    strings.set_search_placeholder(i18n.t("search.placeholder").into());
    strings.set_lang_title(i18n.t("lang.title").into());
    // 语言按钮显示"可切换到的语言"（英文界面显示"中文"，中文界面显示"EN"）
    strings.set_lang_button(i18n.t("lang.button").into());
    strings.set_io_import(i18n.t("io.import").into());
    strings.set_io_export(i18n.t("io.export").into());
    strings.set_add_title(i18n.t("add.title").into());

    // 卡片
    strings.set_badge_code(i18n.t("badge.snippet").into());
    strings.set_badge_note(i18n.t("badge.note").into());
    strings.set_action_view(i18n.t("actions.view").into());
    strings.set_action_copy(i18n.t("actions.copy").into());
    strings.set_action_edit(i18n.t("actions.edit").into());
    strings.set_action_delete(i18n.t("actions.delete").into());
    strings.set_empty(i18n.t("empty.noCommands").into());

    // 编辑器
    strings.set_editor_new(i18n.t("editor.new").into());
    strings.set_editor_edit(i18n.t("editor.edit").into());
    strings.set_editor_kind_snippet(i18n.t("editor.kindSnippet").into());
    strings.set_editor_kind_note(i18n.t("editor.kindNote").into());
    strings.set_editor_title_placeholder(i18n.t("editor.titlePlaceholder").into());
    strings.set_editor_content_placeholder(i18n.t("editor.contentPlaceholder").into());
    strings.set_editor_note_placeholder(i18n.t("editor.notePlaceholder").into());
    strings.set_editor_tags_placeholder(i18n.t("editor.tagsPlaceholder").into());
    strings.set_editor_cancel(i18n.t("editor.cancel").into());
    strings.set_editor_save(i18n.t("editor.save").into());

    // 删除确认框
    strings.set_confirm_title(i18n.t("confirm.title").into());

    // 查看弹层（复用 viewer.* 文案）
    strings.set_viewer_edit(i18n.t("viewer.edit").into());
    strings.set_viewer_close(i18n.t("viewer.cancel").into());
}

// 托盘菜单文案（菜单在窗口之外，需要单独应用一次）。
pub fn apply_tray(tray: &Tray, i18n: &I18n) {
    tray.set_label_show(i18n.t("tray.show").into());
    tray.set_label_quit(i18n.t("tray.quit").into());
}
