# Tauri v2 窗口 setSize 需要显式 ACL 权限

## 问题现象

打开「新建命令」或「编辑」界面（`EntryEditor`）时，窗口高度完全不会变化，始终保持
`tauri.conf.json` 里配置的 650px。编辑长内容时不会自动变高，正文超出后只能在小窗口里滚动。

代码里确实调用了 `win.setSize()`，且没有任何报错提示，功能看起来"接好了但没生效"。

## 问题原因

Tauri v2 的所有前端 API 调用都要经过 capability ACL 校验。`src-tauri/capabilities/default.json` 原本只授予：

```json
"core:default",
"core:window:allow-close"
```

关键点在于 `core:default` 的展开规则（来自 `tauri` crate 的 `acl-manifests.json`）：

```
core:default
  ├── core:path:default
  ├── core:event:default
  ├── core:window:default   ← 这里
  ├── core:webview:default
  ├── core:app:default
  ├── core:image:default
  ├── core:resources:default
  ├── core:menu:default
  └── core:tray:default
```

而 `core:window:default` **不包含** `allow-set-size`。它只授予只读类命令
（`allow-outer-size`、`allow-scale-factor`、`allow-is-focused` 等 28 个）和
`allow-internal-toggle-maximize`。任何写操作（`set-size`、`set-position`、`close`、
`minimize`、`maximize`）都不在默认集里，必须逐个显式声明。

所以 `windowFit.ts` 里的 `setSize` 在运行时被 ACL 拒绝，Promise reject。又因为调用链是
`void fit()` → `void fitWindowHeight(...)`，没有任何 catch，错误只表现为一条 unhandled
rejection（控制台里都容易被忽略），UI 上完全无感知。

注意 `close` 当初被正确声明了（`allow-close`），说明作者知道要显式声明，只是漏了 `setSize`。
这类"漏一个写权限"的 bug 很难靠读代码发现，因为 TypeScript 侧完全合法。

## 解决方案

`src-tauri/capabilities/default.json` 补一条：

```json
"core:window:allow-set-size"
```

## 为什么采用这个方案

- 这是唯一解法：ACL 是 Tauri v2 的安全模型，无法通过调用方式绕过。
- 不用"把 capability 换成 `core:window:allow-all`"这类宽泛授权 —— 最小权限原则，
  应用只需要 `setSize` 和 `close`。
- 不改成"前端计算高度 + CSS 处理"绕开原生窗口调整 —— 需求本身就是让原生窗口跟随内容高度，
  这是 README 宣称的特性。

## 解决了什么问题

编辑器窗口按内容自动伸缩恢复可用。

遗留风险：`EntryEditor` 里 `fitWindowHeight` / `restoreWindowHeight` 仍然是 `void` 调用，
将来若因其它原因失败依旧会静默。后续若要加 lint 或测试，建议对 capability 与前端 API 调用
做一次交叉核对（本次排查说明"前端调了某个 window API"和"capability 声明了它"之间没有任何编译期关联）。

## 相关文件

- `src-tauri/capabilities/default.json` — 补 `core:window:allow-set-size`
- `src/windowFit.ts:21,32` — 实际调用 `setSize` 的位置
- `src/components/EntryEditor.tsx:75` — `void fit()` 导致错误被吞
- `src-tauri/gen/schemas/acl-manifests.json` — 排查依据（生成物，不入库）
