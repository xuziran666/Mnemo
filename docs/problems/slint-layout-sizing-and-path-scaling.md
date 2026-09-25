# Slint 列表卡片的高度推导与 Path 图标缩放

## 问题现象

用 Slint 还原设计图的卡片列表时遇到两类问题：

1. **高度不可控**：卡片（`CommandCard`）如果只写内部布局而不显式给定高度，会出现「卡片高度等于父容器高度（铺满整屏）」或「高度为 0」两种情况；列表项高度也无法确定，导致卡片之间无法得到设计图要求的 8px 间距。
2. **图标变形**：用 `Path` 绘制的箭头三角形被异常放大并错位（导入/导出图标的下三角变成了占满 16×16 的大三角）。

## 问题原因

1. Slint 的尺寸规则有两条关键约束：
   - **不在布局内的元素，`width`/`height` 默认等于父元素的 100%**。所以「卡片内容是自然高度」这件事不会自动成立，它默认会去撑满父元素。
   - **布局内的子元素，其 `x`/`y`/`width`/`height` 由布局接管，应改用布局项属性 `min-*`/`max-*`/`preferred-*`**。用 `width`/`height` 固定在布局里既不可靠也无法阻止被拉伸。
   - `ListView` 本身没有 `spacing` 属性（它的属性集与 `ScrollView` 相同），卡片间距必须自己造。
2. `Path` 的官方文档明确写着：coordinates 处于 path 自身的"虚拟坐标系"，**"If the width and height properties are non-zero, then the entire shape is fit into these bounds - by scaling accordingly."**。即 `Path` 会把命令图形缩放到元素的 `width`/`height` 边界内。当命令包围盒（例如 x∈[3.2,12.8]、y∈[6.6,11.4]）与元素尺寸（16×16）不一致时，图形就会被拉伸放大并偏离预期位置。

## 解决方案

**尺寸约定（写入各组件）**

- 非布局子元素（`CommandCard` 的根、`ListView` 的列表项）显式绑定高度，链条为：
  - 内容布局 `card-content: height: preferred-height`（收缩到自然高度）；
  - 卡片 `height: card-content.height + Theme.pad-card * 2`；
  - 列表项 `height: card.height + Theme.gap-card`（顺带实现 8px 卡片间距）。
- 布局内的原子组件（`Badge`、`Tag`、`TextButton`、`IconButton`、`LangMenu`、`SearchBox`、`Toolbar`）一律用 `min-*`/`max-*` 固定尺寸（例如 `min-width: label.preferred-width + 16px; max-width: ...`），既不依赖隐式推导，也不会被布局拉伸。
- 卡片内部子元素靠 `vertical-alignment: center` 自行居中，而不是靠布局的交叉轴对齐。

**Path 图标**

- 图标统一按 16×16 坐标编写，但 `Path` 元素的 `x`/`y`/`width`/`height` 设为**与其 `commands` 包围盒完全一致**，使缩放系数为 1，不依赖 `viewbox-*`：

```slint
// 三角形包围盒：x∈[3.2,12.8], y∈[6.6,11.4]
Path {
    x: 3.2px; y: 6.6px; width: 9.6px; height: 4.8px;
    commands: "M 8 11.4 L 3.2 6.6 L 12.8 6.6 Z";
    fill: root.color;
}
```

## 为什么采用这个方案

- Path 也可以用 `viewbox-x/y/width/height` 做 1:1 映射，但该组属性的单位与默认值在文档中不够明确（float、默认值未列出），误用后是**静默缩放**（不报错、只是画错），排查成本高；让包围盒与元素尺寸一致是零假设的写法。
- 尺寸方面没有选择「依赖 Slint 隐式推荐尺寸」或「把所有内容再包一层 Layout」：前者不可预期（就是本问题的成因），后者会引入额外填充/对齐的不确定性，且卡片本身并不在布局中，用显式高度链条更直观。

## 解决了什么问题

- 卡片高度、卡片间距、徽章/标签/按钮尺寸在任何窗口尺寸下都可预期，且不受渲染器差异影响。
- 图标不会再被拉伸变形（导入/导出/新建三个 `Path` 图标均可控）。
- 遗留/待实机验证项：
  - `PopupWindow` 的 `x`/`y` 究竟相对「父元素」还是「窗口」——官方页面未写明（只列了 `close-policy`/`is-open`，`x/y/width/height` 属公共属性），当前按「相对父元素」实现，若实机发现下拉位置偏移，改用 `absolute-position` 计算。
  - `Path` 的 `fit` 模式（文档中存在 `fit` 属性）在本项目未显式设置；因包围盒与元素尺寸一致，`fill`/`contain` 两种模式下结果相同。

## 相关文件

- `mnemo-slint/ui/components/CommandCard.slint`（高度链条、`card-content`）
- `mnemo-slint/ui/main.slint`（`ListView` 列表项高度与 8px 间距）
- `mnemo-slint/ui/components/{Badge,Tag,TextButton,IconButton,LangMenu,SearchBox}.slint`（`min-*`/`max-*` 固定尺寸）
- `mnemo-slint/ui/components/IconButton.slint`（`Path` 包围盒对齐）
- `mnemo-slint/ui/components/LangMenu.slint`（下拉箭头 `Path`、`PopupWindow`）
