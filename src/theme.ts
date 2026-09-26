import { getCurrentWindow } from "@tauri-apps/api/window";

export type Theme = "light" | "dark";

const STORAGE_KEY = "mnemo-theme";
const listeners = new Set<(t: Theme) => void>();

// 以 <html data-theme> 为准：index.html 的内联脚本才是首屏唯一可信来源，
// 它在首次访问时读一次系统偏好（matchMedia) 决定初值，之后完全由用户手动决定，
// 这里不再重复解析 localStorage，避免两处逻辑漂移。
export function readTheme(): Theme {
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

export function onThemeChange(fn: (t: Theme) => void): () => void {
  listeners.add(fn);
  return () => {
    listeners.delete(fn);
  };
}

export function setTheme(t: Theme) {
  localStorage.setItem(STORAGE_KEY, t);
  paint(t);
}

function paint(t: Theme) {
  const el = document.documentElement;
  // data-theme 供 App.css 里的 [data-theme="light"] 变量块使用。
  el.dataset.theme = t;
  // colorScheme 驱动 WebView 原生控件（滚动条、文本选区、caret）的明暗，
  // 与 data-theme 是两回事，漏设会出现应用深色但滚动条浅色的割裂。
  el.style.colorScheme = t;
  // 原生窗口边框/标题栏主题由 Tauri 单独管理，不受 CSS 影响，必须显式同步。
  void getCurrentWindow().setTheme(t).catch(() => undefined);
  for (const fn of listeners) fn(t);
}

// main.tsx 首帧调用一次，重新应用当前主题。
// 此时 data-theme 已由 index.html 内联脚本写好，这里主要补两件事：
// 让 colorScheme 与原生窗口主题和首次渲染保持一致（内联脚本只设了 CSS 侧）。
export function syncNativeTheme() {
  paint(readTheme());
}
