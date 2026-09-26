# 全局单字母快捷键劫持系统组合键 + 鼠标聚焦搜索框后无法打字

## 问题现象

两个都稳定复现的交互缺陷：

1. 在列表里选中一条代码片段后按 `Ctrl+C`，应用没有执行常规复制，而是**把该片段写入剪贴板并关闭了窗口**。
   同理 `Ctrl+V` 变成"打开查看器"、`Ctrl+D` 变成"删除"、`Ctrl+R` 的刷新被吞掉。
2. 先在搜索框里输入内容并按 `Enter`（此时键盘焦点交给列表），然后**用鼠标点击搜索框**，
   再打字时输入的 `c` / `v` / `d` / `r` 全部被当成快捷键执行，字符根本没进输入框；方向键也失灵。

## 问题原因

两者同源于 `useCommandHotkeys` 的两个设计缺口。

**问题 1：单字母快捷键没有排除修饰键。**

原代码对 `s/r/c/v/d` 只判断 `listActiveRef.current`：

```ts
if (e.key === "c" || e.key === "C") {
  if (!listActiveRef.current) return;
  e.preventDefault();
  ...
}
```

而同一个函数里 `Ctrl+N` 的分支（`e.key.toLowerCase() === "n" && (e.ctrlKey || e.metaKey)`）
却正确判断了修饰键。这种不一致说明作者在写 `n` 时意识到了问题，但没回头修前面几个字母。
结果是 `Ctrl+C` 时 `e.key === "c"` 成立（Ctrl 不改变 `e.key` 的字母部分），直接命中复制分支。

**问题 2：`listActive` 只在键盘路径上复位。**

`listActive` 这个 ref 表达的是"焦点在列表还是在搜索框"。它只在两处被写：
- `Enter`（搜索框内）→ 置 `true`，并 `blur()` 搜索框
- `s` 快捷键 → 置 `false` 并 `focus()` 搜索框

`SearchBox` 没有 `onFocus` 回调。所以鼠标点回搜索框这个路径完全没有通知，
`listActive` 停留在 `true`，单字母快捷键和方向键继续被劫持。

顺带一个认知陷阱：`listActive` 初始为 `false`，而方向键导航要求 `listActive === true`，
所以刚打开应用时按 `↑`/`↓` 是无效的（光标在输入框里移动）。必须先在搜索框按 `Enter`。
这个行为本身是有意的（打字时方向键应该移动光标），但 README 描述成"打开就能方向键导航"，
与实现不符。

## 解决方案

1. `useCommandHotkeys.ts`：把 5 个单字母快捷键整体包进"无修饰键"判断里。
   用嵌套 `if` 而不是给每个分支加条件，避免漏改：

```ts
if (!e.ctrlKey && !e.metaKey && !e.altKey) {
  if (e.key === "s" || e.key === "S") { ... }
  if (e.key === "r") { ... }
  // c / v / d 同理
}
// 下面的 Ctrl+N 分支不受影响，会自然 fall through
```

2. `SearchBox.tsx`：新增可选 `onFocus` 回调，`App.tsx` 传入
   `() => { listActive.current = false; }`，让鼠标聚焦也走复位路径。

3. README（中英）把 `↑`/`↓` 的前置条件写清楚：搜索框内按 `Enter` 才进入列表导航模式。

## 为什么采用这个方案

- 修饰键判断放在最外层而不是分散到各分支：这类 bug 的本质是"同一类快捷键的判定条件不一致"，
  集中一处才能保证以后新增单字母快捷键自动继承该约束。
- 修 `listActive` 复位而不是给搜索框加"输入框永远优先"的特判：状态应该由真实焦点驱动，
  单一数据源比在消费端打补丁可靠。
- 没有把方向键改成"任何时候都能导航"：那会破坏输入框内的光标移动，属于改变交互模型。

## 解决了什么问题

`Ctrl/⌘+C/V/D/R/S` 恢复为系统原义；鼠标点回搜索框后可以正常打字和用方向键移动光标；
文档与实现一致。

遗留：`listActive` 仍是散落在多处写入的 ref，逻辑复杂度在上升。若后续增加列表内输入框或
更多焦点切换，建议改成由单一 `focusZone` 状态驱动。

## 相关文件

- `src/hooks/useCommandHotkeys.ts:68-107` — 修饰键判断
- `src/components/SearchBox.tsx` — 新增 `onFocus`
- `src/App.tsx:32` — `listActive` ref 与 onFocus 回调
- `README.md` / `README.zh-CN.md` — 快捷键表前置条件说明
