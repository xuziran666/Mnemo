import { useEffect, type Dispatch, type RefObject, type SetStateAction } from "react";
import { KIND_SNIPPET, type Command } from "../types";

// 全局键盘绑定，使以搜索为先的工作流程保持轻量和快速。
interface HotkeyOptions {
  enabled: boolean;
  commands: Command[];
  selectedIndex: number;
  listActiveRef: RefObject<boolean>;
  searchRef: RefObject<HTMLInputElement | null>;
  setSelectedIndex: Dispatch<SetStateAction<number>>;
  onOpen: (cmd: Command) => void;
  onOpenAdd: () => void;
  onEdit: (cmd: Command) => void;
  onCopy: (cmd: Command) => void;
  onDelete: (cmd: Command) => void;
  onClose: () => void;
}

export function useCommandHotkeys({
  enabled,
  commands,
  selectedIndex,
  listActiveRef,
  searchRef,
  setSelectedIndex,
  onOpen,
  onOpenAdd,
  onEdit,
  onCopy,
  onDelete,
  onClose,
}: HotkeyOptions) {
  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (!enabled) return;
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        if (!listActiveRef.current) return;
        e.preventDefault();
        setSelectedIndex((i) => {
          if (commands.length === 0) return -1;
          if (e.key === "ArrowDown") return Math.min(i + 1, commands.length - 1);
          return Math.max(0, i - 1);
        });
        return;
      }
      if (e.key === "Enter") {
        // 在搜索框中按回车键会激活列表选择；当选择一个结果时，
        // 片段会立即被复制，并在查看器中打开一个笔记。
        if (document.activeElement === searchRef.current) {
          listActiveRef.current = true;
          searchRef.current?.blur();
          setSelectedIndex(0);
        } else if (listActiveRef.current) {
          const cmd = commands[selectedIndex];
          if (cmd) {
            if (cmd.kind === KIND_SNIPPET) onCopy(cmd);
            else onOpen(cmd);
          }
        }
        return;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
        return;
      }
      if (e.key === "s" || e.key === "S") {
        if (document.activeElement === searchRef.current) return;
        e.preventDefault();
        listActiveRef.current = false;
        searchRef.current?.focus();
        return;
      }
      if (e.key === "r" || e.key === "R") {
        if (!listActiveRef.current) return;
        e.preventDefault();
        const cmd = commands[selectedIndex];
        if (cmd) onEdit(cmd);
        return;
      }
      if (e.key === "c" || e.key === "C") {
        if (!listActiveRef.current) return;
        e.preventDefault();
        const cmd = commands[selectedIndex];
        if (cmd && cmd.kind === KIND_SNIPPET) onCopy(cmd);
        return;
      }
      if (e.key === "v" || e.key === "V") {
        if (!listActiveRef.current) return;
        e.preventDefault();
        const cmd = commands[selectedIndex];
        if (cmd) onOpen(cmd);
        return;
      }
      if (e.key === "d" || e.key === "D") {
        if (!listActiveRef.current) return;
        e.preventDefault();
        const cmd = commands[selectedIndex];
        if (cmd) onDelete(cmd);
        return;
      }
      if (e.key.toLowerCase() === "n" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        onOpenAdd();
      }
    }
    // 监听全局键盘事件，支持列表导航、打开、编辑、复制和关闭。
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [
    enabled,
    commands,
    selectedIndex,
    listActiveRef,
    searchRef,
    setSelectedIndex,
    onOpen,
    onOpenAdd,
    onEdit,
    onCopy,
    onDelete,
    onClose,
  ]);
}
