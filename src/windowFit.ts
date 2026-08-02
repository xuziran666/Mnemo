import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";

export async function captureWindowHeight(): Promise<number> {
  const win = getCurrentWindow();
  const size = await win.outerSize();
  const sf = await win.scaleFactor();
  return size.toLogical(sf).height;
}

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

export async function restoreWindowHeight(originalHeight: number): Promise<void> {
  const win = getCurrentWindow();
  const size = await win.outerSize();
  const sf = await win.scaleFactor();
  const logical = size.toLogical(sf);
  if (Math.abs(originalHeight - logical.height) > 4) {
    await win.setSize(new LogicalSize(logical.width, originalHeight));
  }
}
