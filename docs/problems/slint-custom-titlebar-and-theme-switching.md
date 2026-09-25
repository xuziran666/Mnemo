# 自定义顶栏（无系统标题栏）与亮/暗主题切换

## 问题现象 / 背景

要把无边框窗口改造成「100% 自绘顶栏 + 工具栏精简」，涉及三类硬约束：

1. 顶栏既要**可拖动**（拖动区只覆盖顶栏），又要**双击切换最大化**，而 Slint 的 `WindowMoveArea`
   与 `TouchArea` 在同一区域会打架。
2. 主题要能**整体切换**亮/暗两套配色，且不能去改二十多个组件文件。
3. 最大化状态有**三个来源**（顶栏按钮、顶栏双击、外部如 Win+↑/任务栏菜单），必须保持同步。

## 关键决策与依据

### 1. 顶栏拖动：不用 `WindowMoveArea`，改用 `TouchArea` + 阈值 + Rust `drag_window()`

原因（源码依据）：`WindowMoveArea` 记录按下位置发生在 `input_event_filter_before_children`
（`i-slint-core-1.18.1/items/input_items.rs:311-323`）——也就是**事件目标元素自身**的过滤阶段。
双击最大化必须在顶栏铺一个 `TouchArea`，一旦它成为事件目标，按下事件就不会再落到
`WindowMoveArea` 上，**拖动直接失效**；反过来把 `TouchArea` 放到下层，`WindowMoveArea`
会 `EventAccepted` 掉按下，双击也永远收不到。

因此顶栏自己实现同一套语义：

```slint
title-drag := TouchArea {
    double-clicked => { root.maximize-clicked(); }
    pointer-event(event) => {
        if (event.kind == PointerEventKind.down && event.button == PointerEventButton.left) {
            root.press-x = title-drag.mouse-x;   // PointerEvent 没有坐标字段
            root.press-y = title-drag.mouse-y;
            root.pressed = true;
            root.dragging = false;
        } else if (event.kind == PointerEventKind.move && root.pressed && !root.dragging) {
            if (…超过 4px…) { root.drag-requested(); }   // → Rust: winit drag_window()
        }
    }
}
```

Rust 侧 `on_drag_requested` → `ui.window().with_winit_window(|w| w.drag_window())`。
`WindowMoveArea` 内部调用的也正是 `internal.start_window_move()` → `winit_window.drag_window()`
（`i-slint-backend-winit-1.18.1/winitwindowadapter.rs:2036-2040`），所以行为一致（无边框窗口靠窗口系统拖动）。

### 2. `PointerEvent` 没有坐标 → 用 `TouchArea.mouse-x` / `mouse-y`

Slint 的 `PointerEvent` 只有 4 个字段：`kind` / `button` / `modifiers` / `touch-finger-id`
（`i-slint-core` 里构造该结构体的地方可证）。最初写 `event.position.x` 直接编译失败
（`Cannot access the field 'position'`）。正确做法是读 `TouchArea` 自带的
`out property <length> mouse-x / mouse-y`（`builtin_elements.rs:1304-1306`），它是相对元素自身的坐标，
按下与移动处在同一坐标系，可以直接做阈值判断。

### 3. 自审发现的 bug：`move` 事件必须用 `pressed` 门闩

`pointer-event` 的 `move` 分支**不保证只在按下时派发**。不加门闩的话，悬停移动会拿着
`press-x=0` 与当前鼠标位置比较，超过 4px 就误触发窗口拖动（窗口会"跟着鼠标乱飘"）。
所以按下时置 `pressed = true`、抬起/取消时复位，`move` 分支再判 `pressed && !dragging`。

### 4. 主题：单一状态源 `is-dark` + 全量条件绑定

`Theme` 新增 `in-out property <bool> is-dark`，**所有颜色令牌都改成
`is-dark ? 暗色 : 亮色` 的条件绑定**（当前 25 条）。这样做的关键好处是
**消费方（所有组件）一行都不用改**——组件仍然写 `Theme.card-bg`，切换 `is-dark` 时
Slint 的绑定系统会让整棵树自动重算并重绘，不需要任何手动刷新或通知机制。

亮色配色的校准依据（需求要求）：

| 令牌 | 暗色 | 亮色 | 说明 |
| --- | --- | --- | --- |
| `bg` | `#1e1e1e` | `#f0f0f0` | 主背景严禁纯白，用带灰度的浅灰 |
| `card-bg` | `#2a2a2a` | `#ffffff` | 卡片与背景形成微弱对比 |
| `border` | `#3a3a3a` | `#dcdcdc` | 卡片/控件 1px 浅灰描边 |
| `text` | `#e6e6e6` | `#1a1a1a` | 正文严禁纯黑 |
| `text-muted` | `#9a9a9a` | `#6b6b6b` | 次要文本 |
| `accent` | `#5b7cfa` | `#3b5bdb` | 亮色下加深，保证白底对比度 |
| `code` | `#9ecbff` | `#0b5394` | 代码文本改深蓝，否则白底不可读 |
| `tag-bg` | `#333333` | `#e8e8e8` | 标签改浅灰底 + 深灰字 |
| `hover-*` | 亮色系 | 统一加深 | 卡片文字按钮悬停色在亮色下重新校准 |
| `overlay-bg` | `#00000099` | `#00000040` | 亮色弹层遮罩更淡 |
| `toast-bg` | `#000000e6` | `#ffffff` | 亮色用白底深字（否则深字压深底不可读） |

另外**必须新增** `on-solid-text`（两套主题都是白）：实心强调/危险按钮的白字不能复用
`badge-text`——一旦 `badge-text` 跟随主题变深，蓝/红实底上就会出现深色文字。
`SolidButton` / `IconButton`（danger 悬停）/ `KindChip` 都改用它。

### 5. 最大化状态：三个来源统一收口

- 顶栏按钮与顶栏双击都走同一个回调 `maximize-requested` → Rust `toggle_maximize()`；
- `toggle_maximize` 用**意图值**（`!is_maximized()`）同时写窗口与 `State.maximized`：
  `set_maximized` 是异步的，紧接着读回可能还是旧值，所以不能"写完再读"；
- 外部变化（Win+↑、任务栏菜单、系统快捷键）靠 winit 钩子里的
  `WindowEvent::Resized` 回到真实值再同步 `State.maximized`（有变化才写，避免无谓重绘）。

### 6. 设置持久化复用旧版字段

`settings.json` 与旧版 egui 共用同一目录同一文件，因此：

- 主题写 `"theme": "dark" | "light"`、语言写 `"lang": "en" | "zh"`（与 egui 完全同名同值）；
- 写入用「读-改-写」单字段更新，切主题不会把对方的 lang 清掉，反之亦然。

## 解决了什么问题

- 顶栏（34px）三区布局：左侧图标 + "Mnemo"，中间可拖动/双击最大化，右侧最小化 / 最大化还原 / 关闭
  （关闭按钮 `danger: true` 悬停变红，三个按钮用 `flat: true` 常态透明，贴近系统标题栏观感）。
- 工具栏精简为：搜索框 → 主题切换 → 语言切换 → 导入 → 导出 → 新建；语言下拉（`LangMenu`）文件保留但不再被引用。
- 主题与语言都在 Rust 侧切换并落盘，重启后保持。
- 遗留 / 已知限制：
  - 主题与语言按钮的图标语义不同（主题显示"当前"、语言显示"要切换到的"），按需求原文实现，若要统一改一行；
  - `🌙`/`☀` 是文本字形，渲染依赖系统字体（Windows 的 Segoe UI Emoji/Symbol 正常）；
  - 弹层打开时顶栏被遮罩挡住（连最小化/关闭也被挡住）——这是模态语义的有意选择；
  - 最大化状态下拖动被禁用（需求明确要求）；
  - 顶栏双击仅在拖动区（按钮左侧）生效，与系统标题栏一致（按钮区域双击不切换最大化）。

## 相关文件

- `mnemo-slint/ui/components/TitleBar.slint`（新增：顶栏 + 拖动/双击 + 三个窗口按钮）
- `mnemo-slint/ui/components/{Toolbar,IconButton,SolidButton,EditorPanel}.slint`（按钮与图标调整）
- `mnemo-slint/ui/theme.slint`（`is-dark` + 25 条条件绑定 + 新令牌）
- `mnemo-slint/ui/{main,state}.slint`（顶栏接入、回调调整、`State.maximized`、`Strings.lang-button`）
- `mnemo-slint/src/{main,settings,i18n,strings}.rs`（主题/语言切换与持久化、拖动、最大化同步）
