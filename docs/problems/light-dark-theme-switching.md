# 界面只有深色主题，浅色状态下所有代码展示色不可读

## 问题现象

应用只有一套深色配色，`:root` 里写死 `#1e1e1e` / `#2a2a2a`。
在浅色系统（Windows 浅色模式、macOS 浅色）下启动，窗口边框是浅色的、内容区是深色的，
视觉上割裂；用户也没有任何办法切到浅色。

## 问题原因

`src/App.css` 的变量表只有 6 项（`--bg` `--panel` `--border` `--text` `--muted` `--accent`），
这 6 项已经覆盖了所有按钮、输入框、边框。但**状态色和代码展示色全部是硬编码**：

| 硬编码 | 出现位置 | 性质 |
|---|---|---|
| `#333` ×7 | `.item.selected` `.tag` `.act:hover` `.markdown-body th` `.hint kbd`（UI 表面）<br>`.markdown-body pre` `.markdown-body :not(pre) > code`（代码块） | 同一个值承担了两种语义 |
| `#9ecbff` ×4 | `.item-command` `.viewer-content` `:not(pre) > code` `.badge` | 代码文字色 |
| `#000` + `#fff` | `.toast` | toast 底/字 |
| `rgba(255,255,255,.03)` | `.markdown-body tr:nth-child(even) td` | 表格斑马纹 |
| `#1f3a8a` / `#3b1f6e` / `#d0b4ff` | `.badge` / `.badge.note` | 徽标 |
| `#6a8cff` | `.btn.primary:hover` | 主按钮 hover |
| `#7bf0b0` / `#ffb86c` / `#ff6b6b` | `.act.copy:hover` / `.act.edit:hover` / `.act.delete:hover` | 语义色 |

共 19 处 / 11 种颜色。

关键约束是 `#7bf0b0`、`#ffb86c`、`#ff6b6b` 这组：它们是**为深色底挑的荧光色**，
在白底上对比度约 1.3:1，属于不可读。所以浅色主题不能只做背景反转，
必须为这组重新取值。

## 解决方案

### 1. 变量表扩到 19 项

新增 13 个变量承接上表。其中 `--surface-2` 与 `--code-bg` **必须拆开**：
虽然现在都是 `#333`，但前者是纯 UI 表面（浅色下应偏灰 `#eaeef2`），
后者要严格对齐 highlight.js 官方主题的底色（`github.css` 是 `#f6f8fa`），
合成一个变量会导致代码块底色和官方主题对不上。

### 2. 浅色块用 `[data-theme="light"]`，不新建主题注册表

```css
[data-theme="light"] { --bg: #ffffff; ... }
```

这个块**必须放在 `:root` 之后**：`:root` 和 `[data-theme="light"]` 特异性都是 (0,1,0)，
相同特异性下靠源码顺序决胜。放前面会被 `:root` 覆盖。

### 3. highlight.js 主题整体换 stylesheet

这是唯一的实现级难点。`NoteView.tsx` 原本静态 `import "highlight.js/styles/github-dark.css"`，
注释里直接写「必须用暗色主题」。

两套官方主题的令牌选择器（`.hljs-keyword` 等）**都是裸类名，特异性同为 (0,1,0)**，
所以无法靠外层包裹（`[data-theme="light"] .hljs-keyword`）来区分——两个都静态引入会互相覆盖，
且胜负由引入顺序决定，不可靠。

采用的方案：用 Vite 的 `?url` 拿到资源地址，再改一个常驻 `<head>` 的 `<link>` 的 href：

```ts
import hljsDark from "highlight.js/styles/github-dark.css?url";
import hljsLight from "highlight.js/styles/github.css?url";
const HLJS_HREF: Record<Theme, string> = { dark: hljsDark, light: hljsLight };
```

`index.html` 里放 `<link id="hljs-theme" rel="stylesheet" />`（**不带 href**）。

构建产物验证通过，两套主题各自成为独立资源：

```
dist/assets/github-CDab0zVI.css       1.06 kB   底色 #fff
dist/assets/github-dark-Dfs9RUU9.css  1.07 kB   底色 #0d1117
```

**`<link>` 上不能写 `href=""`**：空的 href 会被浏览器解析成当前文档 URL，
产生一次把 HTML 当样式表的无效请求。

### 4. KaTeX 不需要处理

`katex.min.css` **零硬编码颜色**，只用了 `border-color: currentColor`，
正文颜色全部靠继承。所以公式颜色自动跟随 `--text`，无需任何改动。

（这一步纠正了实施前的错误判断：曾认为 KaTeX 是深色主题下的现存可读性问题，
实测不成立。）

### 5. 三处必须同时写

| 位置 | 作用 | 漏掉的后果 |
|---|---|---|
| `<html data-theme>` | 供 `[data-theme="light"]` 变量块使用 | 应用不换色 |
| `<html style="color-scheme">` | WebView 原生控件（滚动条、选区、caret） | 应用深色但滚动条浅色 |
| `getCurrentWindow().setTheme()` | 原生窗口边框/标题栏 | 系统深色 + 应用浅色割裂 |

第三项需要新增权限 `core:window:allow-set-theme`；
`theme()` / `onThemeChanged()` 所需的 `allow-theme` / `core:event:default`
已含在现有 `core:default` 与 `core:window:default` 里。

### 6. 首屏防闪烁靠内联脚本

主题必须在首屏 CSS 之前确定，否则浅色用户会看到一帧深色背景。
`index.html` 的 `<head>` 里放一段同步执行的 IIFE 读 localStorage 并写 `data-theme`。

因此 `src/theme.ts` 的 `readTheme()` **以 `document.documentElement.dataset.theme` 为准**，
不再自己解析 localStorage——避免两处解析逻辑漂移。

## 为什么采用这个方案

- **只做浅色/深色两态，不做主题注册表**：变量用 `[data-theme="light"]` 而非按名字索引，
  是因为没有多主题需求。将来要加主题，把选择器换成 `[data-theme="<name>"]` 并把取值抽成表即可，
  现有结构不需要推翻。
- **`?url` 换 href，而不是动态 `import()` 两个主题**：动态 import 会让两个 stylesheet
  同时留在 document 里，切换时靠插入顺序决胜，且旧样式表无法卸载。
  单 `<link>` 换 href 只有一份样式表，不累积、无 FOUC（`<link>` 从首屏就在 `<head>` 里）。
- **不手写约 35 条 `.hljs-*` → `var(--hljs-*)` 映射**：那要跟着 highlight.js 的类名清单走，
  属于重复实现上游；`?url` 直接复用官方主题文件。
- **`setTheme` 用 `.catch(() => undefined)` 兜底**：Tauri 文档说明 Linux 下 `setTheme` 是
  app-wide，GTK 运行时切主题偶有失效。失败时退化为「只有应用内 CSS 跟随，原生边框不变」，
  不崩、不阻塞切换。

## 解决了什么问题

- 工具栏新增主题按钮，可在浅色/深色间切换，选择持久化到 `localStorage`。
- 首次启动无记录时读一次系统偏好作为初值，之后完全由用户手动决定，不再跟随系统变化。
- 19 处硬编码颜色全部收敛为变量，`App.css` 里除变量定义块外零硬编码颜色。

副作用与遗留：
1. 浅色下 `.act.copy:hover` 等从荧光色变为沉稳色，是对比度修正而非回归。
2. 首次打开笔记时代码块可能有一瞬无样式（`<link>` 初始无 href，`NoteView` 挂载后才写入）。
   资源在 `<head>` 常驻，二次打开及切换均无闪烁。
3. `setTheme` 在 Linux/Wayland 上可能不生效于原生边框，属平台限制。
4. 未自定义滚动条样式，靠 `colorScheme` 渲染原生明暗滚动条。

## 相关文件

- `src/theme.ts`（新增）— `readTheme` / `setTheme` / `onThemeChange` / `syncNativeTheme`
- `index.html` — 首屏内联脚本 + `<link id="hljs-theme">`
- `src/main.tsx` — `syncNativeTheme()`
- `src/components/ThemeButton.tsx`（新增）— 切换按钮
- `src/components/icons.tsx` — `MoonIcon` / `SunIcon`
- `src/components/NoteView.tsx` — `?url` + 换 href + 挂载时补应用一次
- `src/App.css` — 19 个变量 + `[data-theme="light"]` 块 + 19 处硬编码替换
- `src/App.tsx` — 挂载 `ThemeButton`
- `src/i18n/locales/{en,zh}.json` — `theme.light` / `theme.dark`
- `src-tauri/capabilities/default.json` — `core:window:allow-set-theme`
