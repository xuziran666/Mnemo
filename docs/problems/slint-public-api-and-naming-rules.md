# Slint 1.18 的命名与公开 API 约束（编译期踩坑）

## 问题现象

`mnemo-slint` 首次 `cargo build` 后，暴露出 4 类只有编译器才能发现的约束（错误信息均为 Slint 编译器原文）：

| # | 错误信息 | 位置 |
| --- | --- | --- |
| 1 | `Unknown property rotation-angle in Path` | `LangMenu.slint` 的 `Path` 箭头 |
| 2 | `Unknown element 'ScrollView'` | `main.slint` 使用 `ListView`/`ScrollView` 处 |
| 3 | `'Theme' does not have a property 'padding'` | `main.slint` 手写 `Theme.padding` |
| 4 | `Cannot override property 'color'` + `warning: The property 'color' has been deprecated` | `IconButton.slint` 的 `IconGlyph` |
| 5 | `cannot find type 'Theme' / 'Tray' in this scope`（Rust 侧） | `src/main.rs` |

## 问题原因

1. **`Path` 不支持 `rotation-angle`**：它只接受几何/描边类属性（`commands`、`fill`、`stroke*`、`viewbox-*`、`fit`），没有通用变换属性，因此不能靠旋转实现箭头翻转。
2. **视图组件在 std-widgets 里，不是语言内置**：`ListView`、`ScrollView`、`LineEdit`、`Button` 等都属于 std-widgets，必须 `import { ... } from "std-widgets.slint";`。错误信息可能报在内部实现（`ListView` 内部基于 `ScrollView`，于是提示 `Unknown element 'ScrollView'`），容易误导。
3. **全局单例的属性必须逐项声明**：`Theme.padding` 从未在 `theme.slint` 里定义（现有的是 `pad-window`、`pad-card`、`gap-*`）。
4. **不能覆盖继承来的（已弃用）属性名**：`IconGlyph inherits Rectangle`，而 `Rectangle` 有一个已弃用的 `color` 属性（等价于 `background`）。自定义组件里再声明 `in property <color> color` 会被判定为「覆盖属性」，直接编译失败；对应的绑定 `color: ...` 也会给出弃用警告。
5. **`slint::include_modules!()` 只重导出「被编译文件自身导出」的符号**：build.rs 只编译 `ui/main.slint`，因此只有 `MainWindow` 被重导出；`theme.slint`/`state.slint` 里的全局单例（`Theme`/`State`/`Strings`）、`tray.slint` 里的组件（`Tray`）以及其它文件里的 struct/enum 对 Rust 不可见。生成代码里对 struct/enum 甚至给出了明确提示：*"not part of the public API. Re-export it from your main .slint file to make it public."*

## 解决方案

```slint
// main.slint -------------------------------
// 1) 视图组件来自 std-widgets，必须显式导入
import { ScrollView } from "std-widgets.slint";

// 2) 其它文件里的全局单例 / 组件 / 结构体必须在根组件所在文件重导出，
//    否则 Rust 侧 include_modules!() 看不到它们
export { Theme } from "theme.slint";
export { State, Strings, CommandItem, LangOption } from "state.slint";
export { IconKind } from "components/IconButton.slint";
export { Tray } from "tray.slint";
```

```slint
// IconButton.slint ---------------------------
// 3) 避开 Rectangle 继承来的已弃用 color 属性：改名为 glyph-color
component IconGlyph inherits Rectangle {
    in property <color> glyph-color: Theme.text;
    // ...
    Rectangle { background: root.glyph-color; }
}
```

```slint
// LangMenu.slint -----------------------------
// 4) Path 不能旋转 → 用 Text 字符切换
caret := Text {
    text: popup.is-open ? "▲" : "▼";
    color: Theme.text-muted;
    font-size: 10px;
    horizontal-alignment: center;
    vertical-alignment: center;
}
```

另有一处同类问题（命名歧义）：`TouchArea { mouse-cursor: text; }` 建议写成 `MouseCursor.text`，显式带上枚举命名空间，避免与同名属性/标识符混淆。

## 为什么采用这个方案

- `Path` 旋转：备选是引入 `viewbox` + 手写旋转后的路径点，代码可读性差且难以维护；用 `Text` 切换字符零成本、语义直观（▼/▲ 是通用字形，Windows/macOS/Linux 常见字体都有）。
- 重导出而不是把 `Tray`/`Theme` 塞进 `main.slint`：保持"一个文件一件事"的结构（theme / state / tray / components 分离），只用一个文件承担公开 API 出口，符合 Slint 的模块约定。
- 改名 `glyph-color` 而不是保留 `color` 并试图转义：Slint 没有属性名转义机制，且保留会持续产生弃用警告。

## 解决了什么问题

- `cargo check -p mnemo-slint` 从 11 条编译错误/告警收敛到 **0 error / 0 warning**。
- 沉淀出本项目在 Slint 1.18 下的四条约定：视图组件必须从 std-widgets 导入；公开 API 只在 `main.slint` 重导出；自定义组件不要用 `color`/`background` 等继承属性名；能显式限定命名空间（`MouseCursor.text`、`self.preferred-height`）就显式限定。
- 副作用：`import { ScrollView } from "std-widgets.slint"` 会把 std-widgets（fluent 样式）整体链接进二进制，体积略增；本项目只需要滚动容器，可接受。

## 相关文件

- `mnemo-slint/ui/main.slint`（std-widgets 导入、重导出块）
- `mnemo-slint/ui/components/IconButton.slint`（`glyph-color`）
- `mnemo-slint/ui/components/LangMenu.slint`（`Text` 箭头）
- `mnemo-slint/ui/components/SearchBox.slint`（`MouseCursor.text`）
- `mnemo-slint/ui/theme.slint`（token 名称清单：无 `padding`，用 `pad-*`/`gap-*`）
- `mnemo-slint/src/main.rs`（`ui.global::<Theme>()`、`Tray::new()` 依赖重导出）
