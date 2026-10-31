# Slint 弹层焦点、Note 的 Enter 语义与跨平台输入修复（0.4.1）

> 后续调整：0.4.1 收尾已移除“关闭到托盘”，复制成功和关闭窗口都直接退出应用；
> 详见 [remove-close-to-tray.md](remove-close-to-tray.md)。本文中“隐藏窗口”的旧描述按当时行为保留。

## 问题现象

四个互相独立但都影响键盘主流程的问题：

1. **查看层/确认框快捷键穿透**：打开查看层后按 `Enter` 会复制并隐藏窗口，按 `r` 会在查看层上再叠一个编辑器，按 `d` 会弹删除确认；确认框里按 `Enter` 同样会复制关窗，无法用键盘确认删除。
2. **关闭编辑器后快捷键失效**：保存、取消或按 `Esc` 关闭编辑器后，`s`、`↑/↓`、`Enter` 等列表快捷键不再响应，必须点击列表或重新激活窗口才恢复。
3. **Note 的 `Enter` 与文档不一致**：README 承诺“`Enter` 打开 Note 查看层、查看层内 `Enter` 进入编辑”，实现却始终复制并关窗。
4. **跨平台输入**：Linux 下复制后粘贴可能为空；macOS 的 `Cmd+S` 不生效。

## 问题原因

### 1. 弹层没有接管焦点

查看层和确认框声明在 `content-scope` 之外，但没有自己的 `FocusScope`，也没有主动聚焦。焦点仍停留在 `content-scope`，而列表的 `KeyBinding` 只判断 `!toolbar.search-focused && selected-id != 0`，因此弹层打开时列表快捷键仍然生效。

### 2. 弹层销毁后焦点没有归还

Slint 的 `forward-focus` 只会在组件初始化时执行一次（生成代码里是 `user_init` 中的一次 `fn_focus()`），不会在弹层销毁后重新执行。编辑器自己的 `editor-scope` 持有焦点，保存/取消后整个 scope 被销毁，`WindowInner::key_event` 从失效的 `focus_item` 开始投递，列表快捷键自然收不到按键。

### 3. `Enter` 没有按 kind 分流

`main.slint` 的 `Return` 直接绑定 `copy-requested`，Rust 侧也不判断 `kind`；查看层没有 `Enter` 绑定。

### 4. 剪贴板实例被立即 drop

原实现每次复制都 `arboard::Clipboard::new()`，写入后闭包结束即 drop。`arboard` 文档明确说明：Linux（X11/Wayland）下剪贴板内容由最后写入的进程托管，最后一个 `Clipboard` 实例 drop 后内容可能失效；X11 还会尝试交给剪贴板管理器，但没有管理器时不保证。

### 5. macOS 快捷键漏绑

编辑器只声明了 `@keys(Control + S)`；Slint 中 macOS 的 Command 对应 `Meta`，因此 `Cmd+S` 不会触发保存。

## 解决方案

- `State` 新增 `modal-open: editor-open || viewing-open || confirm-open`，所有列表快捷键统一加 `!State.modal-open` 守卫。
- 查看层与确认框各自包裹一个 `FocusScope`，`init` 时主动聚焦：
  - 查看层：`Enter` = 编辑，`c` = 复制，`r` = 编辑；
  - 确认框：`Enter` = 确认删除。
- `MainWindow` 新增 `public function focus-list()`，在以下时机显式把焦点还给 `content-scope`：
  - 保存成功、取消编辑、`Esc` 关闭编辑器；
  - 关闭查看层/确认框（含 `Esc` 与按钮）；
  - 查看层「编辑」时条目已不存在。
- `Enter` 改走新的 `activate-requested` 回调，由 Rust 按 `KIND_NOTE` 分流：Note 打开查看层，其余复制并隐藏。
- 新增 `ClipboardHandle`，进程内只保留一个 `arboard::Clipboard` 实例，复制时复用。
- 编辑器补充 `@keys(Meta + S)`。
- README 同步更新 `Enter`/`v`/`Cmd+S` 的说明，并删除不存在的 `+` 快捷键条目。

## 为什么采用这个方案

- **用 `FocusScope` 而不是在 `.slint` 里判断弹层状态**：Slint 的按键投递基于焦点 item；让弹层真正持有焦点，既解决快捷键穿透，也让 `Enter`/`Ctrl+S` 的作用域自然正确，不需要在列表回调里堆条件。
- **显式 `focus-list()` 而不是依赖 `forward-focus`**：`forward-focus` 只在组件初始化时执行；弹层是动态创建/销毁的，必须在关闭路径上显式归还焦点。
- **`modal-open` 守卫作为第二道防线**：即使某个平台/焦点异常导致 `content-scope` 仍持有焦点，弹层打开时列表快捷键也不会误触发。
- **`activate-requested` 放在 Rust 侧**：列表缓存与 `kind` 都在 Rust 侧，避免在 `.slint` 里再维护一份“选中项类型”。
- **长生命周期 `Clipboard` 而不是 `SetExtLinux::wait()`**：`wait()` 会阻塞直到其他应用接管剪贴板，不符合“复制后立即隐藏窗口”的交互；复用实例不阻塞、不新增依赖，同时满足 X11/Wayland 的托管要求。
- **`Meta + S` 与 `Control + S` 并存**：与已有的 `Ctrl+N`/`Cmd+N` 做法一致，不做平台条件编译。

## 解决了什么问题

- 查看层/确认框内的 `Enter`、`r`、`d` 不再误操作背后的列表；确认框可以用 `Enter` 确认删除。
- 关闭编辑器/弹层后键盘快捷键立即恢复，不再需要鼠标点击。
- Note 的 `Enter` 行为与 README 一致；查看层内 `Enter` 可进入编辑。
- Linux 下复制后的剪贴板内容在进程存活期间保持可用。
- macOS 的 `Cmd+S` 可以保存。

### 已知限制

- 本次只修复高优先级体验问题；保存时 `trim()` 正文、`created_at` 空值/同秒排序、文件对话框置顶层级等中低优先级问题未处理。
- 按项目规则未在本地运行 `cargo test`/`cargo build`，由推送到默认分支后的 CI 验证。

## 相关文件

- `mnemo-slint/ui/state.slint`（`modal-open`）
- `mnemo-slint/ui/main.slint`（快捷键守卫、查看/确认 FocusScope、`Meta + S`、`focus-list()`）
- `mnemo-slint/src/main.rs`（`activate-requested`、`ClipboardHandle`、焦点归还）
- `README.md`、`README.zh-CN.md`（行为与快捷键说明）
- `.github/workflows/ci.yml`（默认分支 `slint` 触发 CI）
- `mnemo-core/Cargo.toml`、`mnemo-slint/Cargo.toml`、`Cargo.lock`（0.4.1）
