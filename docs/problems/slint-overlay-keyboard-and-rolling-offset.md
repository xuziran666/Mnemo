# Slint 弹层层级、键盘焦点与滚动偏移（Step 2 业务接入）

## 问题现象 / 背景

把 Slint 骨架接到 `mnemo-core` 真实数据时，需要补上弹层（新建/编辑、删除确认）、键盘导航、
Toast 与 i18n。这些在 Slint 1.18 里各有一处容易踩空的地方：

1. 弹层如果用 `Dialog`（Slint 的独立窗口），主窗口的"失焦自动隐藏"钩子会把主窗口藏起来，
   于是只剩一个孤零零的编辑窗口，托盘唤回还会打断编辑。
2. 键盘导航要区分"列表快捷键"和"输入框打字"，而 `FocusScope` 只在特定条件下才收得到按键。
3. `↑/↓` 移动选中项需要"当前项的下标"，但 Slint 的 `for` 循环**没有下标变量**。
4. 滚动跟随选中项要写滚动位置，但 `viewport-y` 在 1.18 已 `@deprecated`。
5. Toast 需要 1.5s 自动消失，而 Slint 1.18 没有 `Timer` 元素。

## 关键决策与依据

### 1. 弹层：窗口内浮层（Overlay + 遮罩）

`main.slint` 里弹层是**全窗口的 `Rectangle` 兄弟节点，声明在内容之后**（层级规则：后面的兄弟在上）：

```slint
if (State.editor-open) : Rectangle {
    width: 100%; height: 100%;
    background: Theme.overlay-bg;
    TouchArea { }          // 遮罩：吃掉点击，防穿透 + 防误拖窗口
    EditorPanel { ... }    // 声明在遮罩之后 → 面板在遮罩之上，可正常接收输入
}
```

- 遮罩的 `TouchArea` 同时挡住了下层的卡片点击与 `WindowMoveArea`（否则在弹层上拖拽会移动窗口）。
- **点击遮罩不关闭弹层**：避免误点丢失正在编辑的内容，关闭入口只有"取消 / 保存 / Esc"。
- 不用 `Dialog` 的取舍：`Dialog` 是真正的第二个顶层窗口，与"失焦即隐藏"的产品行为直接冲突，
  且需要跨窗口同步主题与状态；窗口内浮层的代价是无法拖到主窗口之外（对表单场景无影响）。

### 2. 键盘：`FocusScope` 必须包住整棵内容树

```slint
export component MainWindow inherits Window {
    forward-focus: content-scope;   // 启动即拿到初始焦点，不用先点一下
    content-scope := FocusScope {
        KeyBinding { keys: @keys(DownArrow); enabled: !toolbar.search-focused; activated => {...} }
        ...
        VerticalLayout { ... }      // 搜索框、卡片列表都在作用域内部
    }
    // 弹层在作用域**之外**
    if (State.editor-open) : Rectangle { ... EditorPanel { ... } }
}
```

三个要点：

1. **不能用一个独立的小元素当键盘作用域**。官方文档：`FocusScope` 只在"自身 `has-focus`"或
   "包裹着 `has-focus` 的子元素"时才处理按键。搜索框聚焦时它自己会消费字母键，因此必须让
   FocusScope 成为搜索框的**祖先**，否则 Ctrl+N 之类的全局键在搜索框聚焦时会失效。
2. **键名必须用 Slint 的命名**（依据 `i-slint-common-1.18.1/key_codes.rs` 的映射表）：
   `DownArrow`/`UpArrow`（不是 `Down`/`Up`）、`Return`（不是 `Enter`）、字母必须大写（`S`/`N`）、
   修饰键是 `Control`（不是 `Ctrl`/`Command`）。写错会得到 `xxx not defined in the Keys namespace`。
3. **弹层在作用域之外 → 编辑时列表快捷键不会触发**。编辑表单的 `TextInput` 拿焦点后，按键
   只沿"输入框 → 弹层 → 窗口"冒泡，不会经过 `content-scope`，所以在标题里打 `s` 不会跳去搜索框，
   `↑/↓` 只移动文本光标不会改选中项。这是把弹层放在作用域外的**有意收益**。
4. 单键快捷键（`S`）另外加了 `enabled: !toolbar.search-focused`（`search-focused` 由 `Toolbar`
   从 `SearchBox` 的 `input-focused` 透出）：即使某天输入框不再消费按键，也不会抢键。

### 3. `Esc` 留在 Rust 侧，并做"弹层优先"

`on_winit_window_event` 是覆盖语义（一个窗口只能注册一个钩子，见
[slint-frameless-window-focus-hide.md](slint-frameless-window-focus-hide.md)），所以 Esc 与失焦隐藏
一直在同一个闭包里。本次给 Esc 增加了优先级判断：

```rust
if is_escape {
    // 有弹层先关弹层（读 State.editor-open / confirm-open），没有才隐藏窗口
    let handled = ui_weak.upgrade().map(|ui| close_top_overlay(&ui)).unwrap_or(false);
    ...
    return EventResult::PreventDefault;   // 不再传给 Slint，避免二次处理
}
```

分工因此非常明确：**Rust 管 Esc（全局、与焦点无关）；Slint 的 FocusScope 管 ↑/↓/Enter/Ctrl+N/s**。

### 4. `↑/↓` 的下标换算放在 Rust

Slint 的 `for item in State.commands : CommandCard` **拿不到下标**，所以选中态只能用 id 表示
（`State.selected-id == item.id`）。如果为了按方向键而在 `.slint` 里再维护一份 `selected-index`，
搜索过滤/增删之后两份状态极易不同步。因此 `↑/↓` 只负责发一个 `select-move(±1)` 回调，
由 `main.rs::move_selection` 用列表缓存算下标：

- 有选中：`index + delta`，`clamp(0, len-1)`（到头就停）；
- 无选中（含被搜索过滤掉）：`↓` 取第一条、`↑` 取最后一条。

### 5. 滚动跟随选中项：`content-y`（不是 `viewport-y`）

- `viewport-*` 在 1.18 已标记 `@deprecated`（`i-slint-compiler/widgets/fluent/scrollview.slint:158-161`），
  用它会产生弃用警告；对应新名字是 `content-x/y/width/height`。
- 通过 `ScrollView` 读写时 `content-y` 是**正数滚动偏移**（`fluent/scrollview.slint` 里
  `content-y <=> vertical-bar.value`，且 `maximum = content-height - height`）。
- 上报几何的方式：卡片自身 `changed selected => { if (root.selected) { root.scroll-requested(root.y, root.height); } }`，
  `root.y` 相对滚动内容，正是需要的坐标系。
- **边界**：两个分支都要夹取到 `[0, content-height - 可视高度]`。内容不足一屏时
  `content-height - 可视高度` 为负，只夹上界会把滚动位置设成负值（首版就踩了这个坑）。

### 6. Toast：定时器在 Rust 侧 + token 防串

Slint 1.18 没有 `Timer` 元素，因此 `State.toast-visible` 由 Rust 控制：显示后
`slint::Timer::single_shot(1500ms)` 关闭。连续提示时用 `Rc<Cell<u32>>` 记一个自增 token，
旧定时器发现 token 已变就什么都不做，避免"新提示被旧定时器提前关掉"。
Toast 整块没有 `TouchArea`，所以它铺满窗口也不挡操作。

### 7. i18n：构建期嵌入 + 三点维护

- 资源：`mnemo-slint/locales/{en,zh}.json`，用 `include_str!` 嵌入二进制（无运行时文件 IO）。
- 新增一条文案固定三步：JSON 加 key → `state.slint` 的 `Strings` 加同名 kebab-case 属性 →
  `strings.rs::apply()` 加一行 setter。`Strings` 属性名与 JSON key 一一对应（`toast.saved` ↔ `toast-saved`）。
- `settings.json` 与旧版 egui **共用同一个文件**（同目录），因此写入采用"读-改-写"保留 `theme` 字段，
  避免把旧版的主题偏好清掉；读取时只取 `lang`，两端互不破坏。

### 8. 面板高度与窗口最小尺寸

编辑浮层自然高度约 424px（字段 + 按钮 + 间距 + 内边距）。窗口 `min-height` 因此从 400px 提到 500px，
否则窗口缩到最小高度时"保存"按钮会跑到窗口外（这是静态检查发现不了的布局耦合）。

## 解决了什么问题

- 弹层、键盘导航、Toast、i18n 全部落地，`cargo check -p mnemo-slint` 保持 0 error / 0 warning。
- 遗留与已知限制：
  - `toast.copied` 文案已备好但未使用：复制成功后窗口立即隐藏，弹了也看不见；
  - 导入/导出仍需 `rfd`（当前只打日志）；卡片单击进 Viewer 未做（保持"仅选中"）；
  - 搜索框内按 Enter 不会进入列表（旧版 egui 有此行为），当前仅依赖 `↓`；
  - `ScrollView` 会实例化全部卡片（不是懒加载），条目上千时应改回 `ListView`；
  - `forward-focus` 理论上会在窗口重新获得焦点时把焦点转回 `content-scope`，若实机发现"打开编辑器后
    输入框不自动聚焦"，改用弹层内的显式 `focus()` 调用。

## 后续补齐：查看弹层与键盘动作（批次 A）

补齐了旧版 egui 有、Slint 版还缺的四项：查看（Viewer）、R/C/V/D 单键动作、搜索框 Enter、编辑器 Ctrl+S。

### 查看弹层（第三个窗口内浮层）

- `ViewerPanel.slint` 沿用同一套"遮罩 + 居中面板"约定；面板尺寸**全部显式**
  （`width: 460px; height: 452px`，都在窗口 `min-*` 之内），内容区用 `ScrollView` +
  `vertical-stretch: 1` 占剩余高度，长代码/长笔记在面板内滚动。
- 数据用**结构体状态** `State.viewing: CommandItem`（Rust 在打开时填充），
  而不是在 `.slint` 里按 id 查表：`for` 循环没有下标、按下标取模型元素也不可靠，
  交给 Rust 用列表缓存一次性填好最省事；字段映射直接复用 `data::to_item`，
  保证与卡片是同一套 note/tags 解析规则（`to_item` 因此从私有改为 `pub`）。
- z 序：**查看 < 编辑 < 确认 < 提示**；点查看里的"编辑"时 Rust 先关查看再开编辑器，
  不叠两层遮罩（Esc 的 `close_top_overlay` 也按 编辑器 → 确认 → 查看 的顺序消费）。

### 搜索框 Enter：用「焦点转移」而不是「失焦 API」

计划里本来要加 `SearchBox.blur-input()`（`input.clear-focus()`），实现时发现**不必要**：
Slint 的焦点是**独占**的，`content-scope.focus()` 一步就把焦点从 TextInput 拿走，
搜索框自然失焦。少一个 API 依赖、少两处改动。
"选中第一条"也复用已有的 `select-move(1)`（Rust 在"当前无选中 + ↓"时正好取第一条），
避免在 `.slint` 里做"模型下标取值"这种不确定写法。

### 编辑器 Ctrl+S：必须单独一个 `FocusScope`

`Ctrl+S` 不能放进 `content-scope`——那会让**列表聚焦时**按 Ctrl+S 也去保存编辑器里的
残留字段（可能误创建/误覆盖条目）。因此编辑器浮层内单独包一个 `editor-scope`：

```slint
if (State.editor-open) : Rectangle {
    TouchArea { ... }                    // 遮罩
    editor-scope := FocusScope {         // 显式 100%x100%，面板在其中居中
        KeyBinding { keys: @keys(Control + S); activated => { root.editor-save(); } }
        EditorPanel { ... }
    }
}
```

`EditorPanel` 的 `init` 会聚焦标题输入框（位于该 scope 内），按键据此冒泡到 `editor-scope`；
而列表的 ↑/↓/R/C/V/D 在编辑时不会触发（弹层仍在 `content-scope` 之外）。

### R / C / V / D 单键动作

与 Enter 同一套守卫 `enabled: !toolbar.search-focused && (State.selected-id != 0)`，
分别触发 `edit-requested / copy-requested / view-requested / delete-requested`。
字母键必须大写（`key_codes.rs` 规则），键名是标识符时小写会被编译器要求改成大写。
搜索框聚焦时这些键由 TextInput 消费（输入框先拿到按键并 accept），列表聚焦时才生效。

## 相关文件

- `mnemo-slint/ui/main.slint`（FocusScope + KeyBinding、弹层层级、滚动跟随）
- `mnemo-slint/ui/components/{EditorPanel,ConfirmDialog,ViewerPanel,SolidButton,Toast}.slint`（新增）
- `mnemo-slint/ui/components/{SearchBox,Toolbar,CommandCard}.slint`（聚焦透出 / 上报几何 / View 按钮 / search-accepted）
- `mnemo-slint/ui/{state,theme}.slint`（编辑器/确认框/Toast 状态与新 token）
- `mnemo-slint/src/{main,data,strings,i18n,settings}.rs`（业务回调、i18n、持久化）
- `mnemo-slint/locales/{en,zh}.json`（语言资源）
