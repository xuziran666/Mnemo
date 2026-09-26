# 笔记里的链接会让 WebView 原地导航，无法回到应用

## 问题现象

在查看器（Viewer）里打开一条包含 Markdown 链接的笔记，点击链接后窗口直接变成了目标网页，
地址栏式的内容替换了整个界面，应用失去响应且**没有任何返回应用的途径**，只能杀进程重启。

## 问题原因

`NoteView.tsx` 用 `react-markdown` 渲染，默认会把 `[text](url)` 渲染成真实的 `<a href>`，
`App.css` 里只有 `.markdown-body a { color: var(--accent); }` 这条颜色规则，
既没有 `target` 属性，也没有任何 `onClick` 拦截。

Tauri 应用的 WebView 只有一个页面（`tauri://localhost` 或 `devUrl`）。
普通 `<a href>` 的默认行为就是让当前页面导航过去 —— 浏览器里这会开新标签页，
但在 Tauri 里没有标签页概念，于是导航直接替换了应用界面。

`tauri.conf.json` 里 `"csp": null` 也意味着没有 CSP 层面的兜底。

## 解决方案

用 opener 插件接管外链点击。

### 1. 依赖

`package.json` / `src-tauri/Cargo.toml` 增加 `@tauri-apps/plugin-opener` / `tauri-plugin-opener`，
`lib.rs` 注册 `.plugin(tauri_plugin_opener::init())`。

### 2. 权限（最小化）

```json
"opener:allow-open-url",
"opener:allow-default-urls"
```

`opener:default` 里还含 `allow-reveal-item-in-dir`（在文件管理器中显示文件），
本应用完全不需要，所以没有直接用 `opener:default`，而是只授予 `open_url`
+ `allow-default-urls`（后者把 URL 作用域限定在 `https/http/mailto/tel`）。

### 3. 自定义链接渲染

```tsx
const EXTERNAL_URL = /^(https?:|mailto:|tel:)/i;

const components: Components = {
  a({ href, title, children }) {
    return (
      <a href={href} title={title} onClick={(e) => {
        e.preventDefault();
        if (href && EXTERNAL_URL.test(href)) {
          void openUrl(href).catch(() => undefined);
        }
      }}>{children}</a>
    );
  },
};
```

## 为什么采用这个方案

- **`preventDefault` 是必须的**，不是可选优化。只要不阻止默认行为，WebView 就已经导航了。
- **不靠 `target="_blank"`**：Tauri v2 对 `new window` 请求的处理依赖平台实现，
  Linux/Windows 上不可靠，官方推荐做法就是 `preventDefault` + `openUrl`。
- **用 `EXTERNAL_URL` 白名单而不是黑名单**：相对地址（如 `#anchor`、`./foo`）在
  `tauri://localhost` 下没有意义，如果放行同样会破坏界面。正则只放行
  `http/https/mailto/tel`，其余一律忽略。
- **没有手写 `rehype` 插件去过滤 URL**：`react-markdown` v10 默认的 `urlTransform`
  （`defaultUrlTransform`）已经把 `javascript:`、`data:` 等危险协议剥掉了，
  重复实现是多余的。真正缺的只是"点击不导航"。
- **没有顺手收紧 CSP**：`"csp": null` 是独立的加固项，涉及 KaTeX / highlight.js
  的内联样式，改动风险需要单独评估，不应和本问题混在一起。

## 解决了什么问题

笔记里的 http/https/mailto/tel 链接用系统默认程序打开，应用界面不再被导航走掉；
非白名单协议和相对地址被静默忽略，不会破坏 WebView。

## 相关文件

- `src/components/NoteView.tsx` — `EXTERNAL_URL` / `components.a` / `openUrl`
- `src-tauri/capabilities/default.json` — `opener:allow-open-url` + `opener:allow-default-urls`
- `src-tauri/src/lib.rs` — 注册 `tauri_plugin_opener`
- `src-tauri/Cargo.toml` / `package.json` — 新增依赖
- `src-tauri/tauri.conf.json:23` — `"csp": null`，相关但本次未改
