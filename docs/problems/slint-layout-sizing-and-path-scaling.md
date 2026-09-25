# Slint 列表卡片的高度推导与 Path 图标缩放

## 问题现象

用 Slint 还原设计图的卡片列表时遇到两类问题：

1. **高度不可控**：卡片（`CommandCard`）如果只写内部布局而不显式给定高度，会出现「卡片高度等于父容器高度（铺满整屏）」或「高度为 0」两种情况；列表项高度也无法确定，导致卡片之间无法得到设计图要求的 8px 间距。
2. **图标变形**：用 `Path` 绘制的箭头三角形被异常放大并错位（导入/导出图标的下三角变成了占满 16×16 的大三角）。

## 问题原因

1. Slint 的尺寸规则有两条关键约束：
   - **不在布局内的元素，`width`/`height` 默认等于父元素的 100%**。所以「卡片内容是自然高度」这件事不会自动成立，它默认会去撑满父元素。
   - **布局内的子元素，其 `x`/`y`/`width`/`height` 由布局接管，应改用布局项属性 `min-*`/`max-*`/`preferred-*`**。用 `width`/`height` 固定在布局里既不可靠也无法阻止被拉伸。
   - ⚠️ **`min-*`/`max-*` 只对"布局项"生效，非布局子元素不读取它们**：编译器把它们 lower 成
     layout-constraint 属性（`i-slint-compiler/passes/materialize_fake_properties.rs:196-204` 的
     `layout_constraint_prop`），只有布局会消费。所以**非布局子元素必须显式写 `width`/`height`**，
     否则尺寸退化为父元素的 100%（注意 `height` 与 `min-height`/`max-height` 互斥，不能同时写）。
     **本项目实际踩过这个坑**：自定义顶栏（`TitleBar.slint`）最初只写了 `min/max-height: 34px`，
     于是整块顶栏高度 = 整窗高；里面的 `IconButton` 同样只有 `min/max-width/height`，尺寸也变成 100%，
     导致 (a) 顶栏内容（名字/图标）落到窗口中间，(b) 带 `danger: true` 的关闭按钮在**整个窗口**范围内都算
     `has-hover`，鼠标一到空白处整窗就变成 `danger-hover-bg` 红色。修法：顶栏写显式 `height`，
     三个窗口按钮放进 `HorizontalLayout`（成为布局项后 min/max 才生效）。
   - `ListView`/`ScrollView` 都没有 `spacing` 属性（两者属性集相同），卡片间距必须自己造：本项目用内部 `VerticalLayout` 的 `spacing` 实现。
2. `Path` 的官方文档明确写着：coordinates 处于 path 自身的"虚拟坐标系"，**"If the width and height properties are non-zero, then the entire shape is fit into these bounds - by scaling accordingly."**。即 `Path` 会把命令图形缩放到元素的 `width`/`height` 边界内。当命令包围盒（例如 x∈[3.2,12.8]、y∈[6.6,11.4]）与元素尺寸（16×16）不一致时，图形就会被拉伸放大并偏离预期位置。

## 解决方案

**尺寸约定（写入各组件）**

- 列表：`ScrollView` + 内部 `VerticalLayout`：
  - `spacing: Theme.gap-card` 提供卡片之间 8px 间距；
  - `width: card-scroll.visible-width` 显式跟随可视宽度，否则 `ScrollView` 的 `content-width` 会退化成内容最小宽度、卡片变窄；
  - 卡片不写 `width`，宽度交给布局拉伸（在布局里写 `width` 会覆盖布局分配、撑破 padding）。
- 卡片高度：卡片始终是布局子元素，而布局只读取 `min-*`/`max-*`/`preferred-*`，所以三者都绑定到同一个自然高度：

```slint
private property <length> natural-height: card-content.height + Theme.pad-card * 2;
min-height: root.natural-height;
max-height: root.natural-height;
preferred-height: root.natural-height;
```

- 内容布局收缩到自然高度：`height: self.preferred-height;`（必须带 `self.` 前缀）。
- 布局内的原子组件（`Badge`、`Tag`、`TextButton`、`IconButton`、`LangMenu`、`SearchBox`、`Toolbar`）一律用 `min-*`/`max-*` 固定尺寸（例如 `min-width: label.preferred-width + 16px; max-width: ...`），既不依赖隐式推导，也不会被布局拉伸。
- 卡片内部子元素靠 `vertical-alignment: center` 自行居中，而不是靠布局的交叉轴对齐。
- 窗口尺寸用 `preferred-width/height`（900×650）+ `min-width/height`（500×400）；**不能再用 `width/height`**，见下方约束表。

**编译期确认的硬约束（实现据此定型）**

| 约束 | 编译器报错 | 采用写法 |
| --- | --- | --- |
| 布局子元素的 `width`/`height` 被布局接管 | 无报错（静默覆盖：会撑破布局 padding） | 改用 `min-*`/`max-*`/`preferred-*` |
| `width` 与 `min-width`/`max-width` 互斥（`Window` 同样适用） | `Cannot specify both 'width' and 'min-width'` | 窗口用 `preferred-*` + `min-*` |
| 引用自身尺寸属性必须带 `self.` | `Unknown unqualified identifier 'preferred-height'. Did you mean 'self.preferred-height'?` | `height: self.preferred-height;` |
| `ScrollView` 内布局的 `content-width/height` 默认取最小尺寸 | 无报错（视觉上卡片变窄） | `width: card-scroll.visible-width;` |
| `Path` 不支持 `rotation-angle` | `Unknown property rotation-angle in Path` | 箭头改用 `Text` 切换 `▼`/`▲` |

**Path 图标**

- 图标统一按 16×16 坐标编写，但 `Path` 元素的 `x`/`y`/`width`/`height` 设为**与其 `commands` 包围盒完全一致**，使缩放系数为 1，不依赖 `viewbox-*`：

```slint
// 三角形包围盒：x∈[3.2,12.8], y∈[6.6,11.4]
Path {
    x: 3.2px; y: 6.6px; width: 9.6px; height: 4.8px;
    commands: "M 8 11.4 L 3.2 6.6 L 12.8 6.6 Z";
    fill: root.glyph-color;
}
```

## 为什么采用这个方案

- Path 也可以用 `viewbox-x/y/width/height` 做 1:1 映射，但该组属性的单位与默认值在文档中不够明确（float、默认值未列出），误用后是**静默缩放**（不报错、只是画错），排查成本高；让包围盒与元素尺寸一致是零假设的写法。
- 需要"旋转"的箭头（语言下拉）改用一个 `Text` 切换字符（`popup.is-open ? "▲" : "▼"`）：`Path` 元素不支持 `rotation-angle`，编译器直接报 `Unknown property rotation-angle in Path`。
- 尺寸方面没有选择「依赖 Slint 隐式推荐尺寸」：那正是本问题的成因；也没有选择「把卡片再包一层 Layout」：卡片脱离布局单独使用时就会失去自然高度。
- 列表容器选 `ScrollView` 而不是 `ListView`：`ListView` 只实例化可见项（更适合长列表），`ScrollView` 会实例化所有子元素（官方文档明确提示数据量大时性能下降）。当前实现用 `VerticalLayout.spacing` 表达 8px 间距最直观；**若后续条目数量上千，应改回 `ListView`**，并把 8px 间距做进列表项内部（列表项高度 = 卡片高度 + `Theme.gap-card`）。

## 解决了什么问题

- 卡片高度、卡片间距、徽章/标签/按钮尺寸在任何窗口尺寸下都可预期，且不受渲染器差异影响。
- 图标不会再被拉伸变形（导入/导出/新建三个 `Path` 图标均可控）。
- 遗留/待实机验证项：
  - `PopupWindow` 的 `x`/`y` 究竟相对「父元素」还是「窗口」——官方页面未写明（只列了 `close-policy`/`is-open`，`x/y/width/height` 属公共属性），当前按「相对父元素」实现，若实机发现下拉位置偏移，改用 `absolute-position` 计算。
  - `Path` 的 `fit` 模式（文档中存在 `fit` 属性）在本项目未显式设置；因包围盒与元素尺寸一致，`fill`/`contain` 两种模式下结果相同。

## 相关文件

- `mnemo-slint/ui/components/CommandCard.slint`（自然高度 `natural-height`、`self.preferred-height`、`min/max/preferred-height`）
- `mnemo-slint/ui/main.slint`（`ScrollView` + `VerticalLayout` 列表、窗口 `preferred-*`/`min-*`）
- `mnemo-slint/ui/components/{Badge,Tag,TextButton,IconButton,LangMenu,SearchBox}.slint`（`min-*`/`max-*` 固定尺寸）
- `mnemo-slint/ui/components/IconButton.slint`（`Path` 包围盒对齐）
- `mnemo-slint/ui/components/LangMenu.slint`（`Text` 下拉箭头、`PopupWindow`）
