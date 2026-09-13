import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";

// 记录当前窗口高度，用于编辑框打开/关闭时恢复原始窗口尺寸。
export async function captureWindowHeight(): Promise<number> {
  const win = getCurrentWindow();
  const size = await win.outerSize();
  const sf = await win.scaleFactor();
  return size.toLogical(sf).height;
}

// 编辑内容时动态调整窗口尺寸，让文本输入区域更自然地扩展，而不需要额外滚动弹窗。
export async function fitWindowHeight(targetHeight: number): Promise<void> {
  const win = getCurrentWindow();
  const size = await win.outerSize();
  const sf = await win.scaleFactor();
  const logical = size.toLogical(sf);
  const max = Math.max(400, Math.floor(window.screen.availHeight * 0.9));
  const target = Math.min(Math.round(targetHeight), max);
  if (Math.abs(target - logical.height) > 4) {
    await win.setSize(new LogicalSize(logical.width, target));
  }
}

// 退出编辑态时恢复原窗口高度，避免界面尺寸在用户编辑后持续留偏大状态。
export async function restoreWindowHeight(originalHeight: number): Promise<void> {
  const win = getCurrentWindow();
  const size = await win.outerSize();
  const sf = await win.scaleFactor();
  const logical = size.toLogical(sf);
  if (Math.abs(originalHeight - logical.height) > 4) {
    await win.setSize(new LogicalSize(logical.width, originalHeight));
  }
}
