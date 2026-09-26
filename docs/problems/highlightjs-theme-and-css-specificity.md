# 深色主题加载浅色代码高亮，且 App.css 的 code 规则把语法高亮压成单色

## 问题现象

1. 笔记里的代码块在深色背景上几乎看不清，代码文字对比度极低。
2. 换成暗色主题后**语法高亮依然不生效** —— 整段代码不论关键字、字符串、注释，
   全部是同一个颜色。

## 问题原因

两个独立问题叠加。

**问题 1：主题选错**

`NoteView.tsx` 引入的是 `highlight.js/styles/github.css`（浅色主题，
背景 `#fff`、正文 `#24292e`），而全站是深色：`:root` 里 `--bg: #1e1e1e`、
`--panel: #2a2a2a`，`.markdown-body pre` 背景是 `#333`。
浅色主题的 token 配色是按白底调的，叠在深色底上就是低对比度。

**问题 2：CSS 特异性覆盖（更隐蔽）**

`App.css` 里原本有：

```css
.markdown-body code {
  background: #333;
  border-radius: 4px;
  padding: 1px 5px;
  font-family: ...;
  font-size: 12px;
  color: #9ecbff;   /* ← 问题在这 */
}
```

这条规则的选择器特异性是 **(0,1,1)**（1 个 class + 1 个类型选择器），
而 highlight.js 主题里所有令牌色是 **.hljs-keyword / .hljs-string / .hljs-comment**，
特异性只有 **(0,1,0)**。于是 App.css 的 `color: #9ecbff` 赢了全部令牌色，
代码无论什么 token 都渲染成同一个蓝色。

换句话说：**即使当初就选了正确的主题，高亮也一直是失效的**，
因为这条为"行内代码"设计的规则误伤了代码块里的 `code`。

## 解决方案

### 1. 换暗色主题

```ts
import "highlight.js/styles/github-dark.css";
```

`github-dark.css`（`highlight.js@11.11.1` 的 `src/styles/github-dark.css`）背景 `#0d1117`，
token 配色为浅字深底，与应用深色风格一致。

### 2. 把行内代码规则限定在 `pre` 之外

```css
.markdown-body :not(pre) > code { ... }
```

`:not(pre)` 自身不增加特异性，规则最终是 (0,1,2)，但**只匹配父元素不是 `pre` 的 `code`**：
- 行内代码 `` `x` `` 渲染成 `<p><code>x</code></p>`，父元素是 `p` → 命中
- 代码块渲染成 `<pre><code class="hljs">`，父元素是 `pre` → 不命中，交给主题着色

`App.css` 里原有的 `.markdown-body pre code { background: none; padding: 0; }`
特异性 (0,1,2)，仍然盖过主题的 `.hljs` 背景，让 `pre` 自己的 `#333` 透出来 —— 这个行为
是想要的，保持不变。

## 为什么采用这个方案

- `github-dark` 而不是 `a11y-dark` / `nord`：`a11y-dark` 对比度更高但饱和度低，
  `github-dark` 的 token 区分度更好，且和 `--panel: #2a2a2a` 的中性灰不冲突。
- 用 `:not(pre) >` 而不是提高 `.hljs` 的特异性（比如 `.markdown-body pre code.hljs`）：
  前者从"选择器作用域"解决问题，语义也更清楚 —— 行内代码和代码块本来就是两套样式。
- 没有为此引入 CSS-in-JS 或 CSS Modules：为了两条规则改变样式架构不划算。

## 解决了什么问题

代码块在深色背景下清晰可读，语法高亮（关键字/字符串/注释/数字等）恢复不同颜色。

## 相关文件

- `src/components/NoteView.tsx:7` — 主题切换
- `src/App.css:308` — `.markdown-body code` → `.markdown-body :not(pre) > code`
