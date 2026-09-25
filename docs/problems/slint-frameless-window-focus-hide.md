# Slint 无边框窗口的失焦自动隐藏实现

## 问题现象

产品需求是「无边框窗口 + 始终置顶 + 失去焦点自动隐藏」，但在 Slint 1.18 中：

- `.slint` 语言层没有任何窗口焦点/激活状态的属性或回调：`Window` 只提供 `width/height`、`always-on-top`、`full-screen`、`minimized`、`maximized`、`no-frame`、`resize-border-width`、`title`、`icon` 等，没有 `focus` / `active` 相关成员。
- Rust 侧 `slint::Window` 的方法列表（`show/hide/set_position/set_size/on_close_requested/dispatch_event/dispatch_event_with_result/window_handle/...`）同样没有焦点的读取入口。
- 官方 issue [slint-ui/slint#3250](https://github.com/slint-ui/slint/issues/3250) 与此需求完全一致（Windows + Rust，想失焦隐藏窗口），官方未给出回调或 API。

## 问题原因

Slint 内部**确实**会产生窗口激活状态事件：`slint::platform::WindowEvent::WindowActiveChanged(bool)`（以及 winit 的 `WindowEvent::Focused(bool)`），但：

1. 这些事件的方向是「平台后端 → item 树」（由 backend 调用 `Window::dispatch_event_with_result()` 注入），**没有向应用层暴露订阅接口**；
2. `WindowEvent` 是 Rust 侧 `#[non_exhaustive]` 枚举，`.slint` 的绑定/条件表达式无法读取它。

因此，仅靠 `.slint` 语言无法感知窗口失焦，必须下探到后端事件层。

## 解决方案

采用 Slint 官方的 winit 互操作入口：

1. `mnemo-slint/Cargo.toml` 开启 feature：`slint = { version = "~1.18", features = ["unstable-winit-030"] }`（版本用 `~1.18` 是为了锁定 winit 0.30 对应的适配层，避免升级到 1.19 时该 feature 发生变更）。
2. `src/main.rs` 里显式锁定后端：`slint::BackendSelector::new().backend_name("winit".into()).select()?`。
3. 用 `WinitWindowAccessor::on_winit_window_event` 注册事件钩子，拦截 `winit::event::WindowEvent::Focused(false)` 后调用 `Window::hide()`：

```rust
use slint::winit_030::{winit, EventResult, WinitWindowAccessor};

let armed = Rc::new(Cell::new(false));
ui.window().on_winit_window_event(move |window, event| {
    if let winit::event::WindowEvent::Focused(focused) = event {
        if *focused {
            armed.set(true);          // 至少获得过一次焦点后才允许隐藏
        } else if armed.get() {
            let _ = window.hide();
        }
    }
    EventResult::Propagate
});
```

4. 事件循环改用 `slint::run_event_loop_until_quit()`：窗口隐藏后循环必须继续存活，只有托盘菜单的 Quit 才调用 `slint::quit_event_loop()`。
5. 唤回入口用系统托盘：`export component Tray inherits SystemTrayIcon`（Slint 默认 feature 里已带 `tray-icon`，**不引入新依赖**）。
   - ⚠️ 内置的 `SystemTrayIcon.clicked`（Windows 上左键点击图标）**不会生成 Rust 访问器**——生成的 `Tray` 只有 `on_show_window` / `on_quit` / `show` / `hide`。因此转发必须写在 `.slint` 侧：`clicked => { root.show-window(); }`，Rust 只需实现 `on_show_window`。
   - ⚠️ 只有 `icon` 被赋了非空 image 时才真正创建托盘图标（官方文档原文），图标资源必须随二进制嵌入（`@image-url`）。
   - 托盘创建失败必须退化：禁止隐藏窗口、关闭/Esc 直接退出，否则窗口隐藏后无法唤回，进程会变成僵尸。
6. **一个窗口只能注册一个 winit 事件钩子**：`on_winit_window_event` 是**覆盖语义**（i-slint-backend-winit 内部执行 `window_event_filter.set(Some(..))`），后注册的会顶掉先注册的。因此「失焦隐藏」与「Esc 隐藏」必须写在同一个闭包里，用 `Rc<Cell<WindowFlags>>` 共享状态。
7. **Esc 全局隐藏**：在同一个钩子里拦截 `winit::event::WindowEvent::KeyboardInput` + `Key::Named(NamedKey::Escape)`，处理后返回 `EventResult::PreventDefault`（不再交给 Slint，避免弹层再处理一次）。这样不依赖窗口内谁持有焦点。
8. **最小化与失焦隐藏会打架**：最小化必然触发 `Focused(false)`，不处理的话「最小化到任务栏」会被当成失焦而变成「隐藏到托盘」。做法：先置 `minimizing = true`，再调 `Window::set_minimized(true)`；在 `Focused(true)`（含从任务栏恢复）时清除该标志。
9. 窗口控制按钮：工具栏最右侧新增「最小化 / 关闭」（关闭 = `hide()`，**不退出进程**），`.slint` 回调 `minimize-requested` / `hide-requested` → Rust 侧 `Window::set_minimized(true)` / `hide()`。

## 为什么采用这个方案

| 备选方案 | 未采用的原因 |
| --- | --- |
| 自建/包装 `WindowAdapter` | 需要实现平台层接口并与 winit 版本强耦合，代码量与维护成本远高于事件钩子 |
| 通过 `raw-window-handle` 取 HWND 后挂 Win32 消息钩子 | 平台特定实现，macOS/Linux 需要各写一套 |
| 定时轮询前台窗口（`GetForegroundWindow`） | 有延迟；更重要的是打开原生文件对话框时窗口会真失焦，轮询无法区分，必然误隐藏 |
| 全局热键（`global-hotkey`）唤回 | P0 阶段引入新依赖会带来跨平台权限与兼容性问题，先不做，P1 再评估 |
| Esc 用 Slint 的 `FocusScope` + `key-pressed` 实现 | 该方式要求窗口内**某个元素持有焦点**，按键才会冒泡到 FocusScope；启动后焦点可能不在任何控件上（无边框窗口里这种情况很常见），不如 winit 钩子稳定。与「选中项」相关的按键（↑/↓/Enter/Ctrl+N）会在 P1 用 `FocusScope` 实现，Esc 这种全局键留在 Rust 侧 |
| 拖动区改用 `TouchArea` 自己实现拖动 | `WindowMoveArea` 由窗口系统执行移动（内部 `winit_window.drag_window()`），且「普通点击不移动窗口、子元素保持可交互」；自己实现要处理拖动阈值、多屏缩放与平台差异 |

选中的方案是 Slint 官方文档给出的 winit 互操作入口，跨平台一致、代码量最小；代价是 feature 名带 `unstable` 前缀且与 winit 0.30 绑定。

## 解决了什么问题

- 实现了「失焦自动隐藏」，并且窗口可以通过托盘重新唤回（无边框 + 无系统标题栏的情况下，这是唯一的召回入口）。
- P0 收尾补齐的基础交互：工具栏最右侧「最小化 / 关闭」按钮、全局 Esc 隐藏、左键点击托盘唤回、拖动区只吃工具栏空白处。
- 剩余限制与副作用：
  - 隐藏状态下没有全局热键，只能靠托盘（左键或菜单）找回；
  - 打开原生文件对话框（`rfd`，P1 的导入/导出）会让窗口真失焦，**必须**在代码中加抑制标志临时禁用隐藏（`src/main.rs` 的 `install_window_event_hook` 内已留 TODO）；
  - 托盘创建失败时应用退化为「关闭/Esc 即退出」（不隐藏），这是刻意选择：宁可少一个「隐藏」能力，也不能让窗口无法唤回；
  - `minimizing` 抑制标志依赖 `Focused(true)` 来清除，若某平台恢复窗口时不派发该事件，失焦自动隐藏会暂时失效直到下一次获得焦点（失败方向是安全的：不会误隐藏）；
  - Linux 托盘依赖桌面环境的 StatusNotifierItem 实现（GNOME 需扩展）。

## 相关文件

- `mnemo-slint/Cargo.toml`（feature `unstable-winit-030`）
- `mnemo-slint/src/main.rs`（`WindowFlags` / `install_window_controls` / `install_window_event_hook` / `install_tray` / `show_window` / `hide_or_quit` / `run_event_loop_until_quit`）
- `mnemo-slint/ui/main.slint`（`no-frame` / `always-on-top` / `resize-border-width` / `WindowMoveArea` / `minimize-requested` / `hide-requested`）
- `mnemo-slint/ui/components/Toolbar.slint`、`IconButton.slint`（最小化 / 关闭按钮与图标）
- `mnemo-slint/ui/tray.slint`（`SystemTrayIcon` 唤回入口 + `clicked` 转发）
