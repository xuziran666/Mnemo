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
5. 唤回入口用系统托盘：`export component Tray inherits SystemTrayIcon`（Slint 默认已启用 `system-tray` feature，**不引入新依赖**），`clicked` 回调里 `ui.show()`。

## 为什么采用这个方案

| 备选方案 | 未采用的原因 |
| --- | --- |
| 自建/包装 `WindowAdapter` | 需要实现平台层接口并与 winit 版本强耦合，代码量与维护成本远高于事件钩子 |
| 通过 `raw-window-handle` 取 HWND 后挂 Win32 消息钩子 | 平台特定实现，macOS/Linux 需要各写一套 |
| 定时轮询前台窗口（`GetForegroundWindow`） | 有延迟；更重要的是打开原生文件对话框时窗口会真失焦，轮询无法区分，必然误隐藏 |
| 全局热键（`global-hotkey`）唤回 | P0 阶段引入新依赖会带来跨平台权限与兼容性问题，先不做，P1 再评估 |

选中的方案是 Slint 官方文档给出的 winit 互操作入口，跨平台一致、代码量最小；代价是 feature 名带 `unstable` 前缀且与 winit 0.30 绑定。

## 解决了什么问题

- 实现了「失焦自动隐藏」，并且窗口可以通过托盘重新唤回（无边框 + 无系统标题栏的情况下，这是唯一的召回入口）。
- 剩余限制与副作用：
  - 隐藏状态下没有全局热键，只能靠托盘找回；
  - 打开原生文件对话框（`rfd`，P1 的导入/导出）会让窗口真失焦，**必须**在代码中加抑制标志临时禁用隐藏（`src/main.rs` 的 `install_focus_hide` 内已留 TODO）；
  - Linux 托盘依赖桌面环境的 StatusNotifierItem 实现（GNOME 需扩展）。

## 相关文件

- `mnemo-slint/Cargo.toml`（feature `unstable-winit-030`）
- `mnemo-slint/src/main.rs`（`BackendSelector` / `install_focus_hide` / `install_tray` / `run_event_loop_until_quit`）
- `mnemo-slint/ui/main.slint`（`no-frame` / `always-on-top` / `resize-border-width` / `WindowMoveArea`）
- `mnemo-slint/ui/tray.slint`（`SystemTrayIcon` 唤回入口）
