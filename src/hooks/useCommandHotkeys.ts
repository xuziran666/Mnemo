import { useEffect, type Dispatch, type RefObject, type SetStateAction } from "react";
import type { Command } from "../types";

interface HotkeyOptions {
  enabled: boolean;
  commands: Command[];
  selectedIndex: number;
  listActiveRef: RefObject<boolean>;
  searchRef: RefObject<HTMLInputElement | null>;
  setSelectedIndex: Dispatch<SetStateAction<number>>;
  onCopy: (cmd: Command) => void;
  onOpenAdd: () => void;
  onEdit: (cmd: Command) => void;
}

export function useCommandHotkeys({
  enabled,
  commands,
  selectedIndex,
  listActiveRef,
  searchRef,
  setSelectedIndex,
  onCopy,
  onOpenAdd,
  onEdit,
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
        if (document.activeElement === searchRef.current) {
          listActiveRef.current = true;
          searchRef.current?.blur();
          setSelectedIndex(0);
        } else if (listActiveRef.current) {
          const cmd = commands[selectedIndex];
          if (cmd) onCopy(cmd);
        }
        return;
      }
      if (e.key === "s" || e.key === "S") {
        listActiveRef.current = false;
        searchRef.current?.focus();
        return;
      }
      if (e.key === "r" || e.key === "R") {
        if (!listActiveRef.current) return;
        const cmd = commands[selectedIndex];
        if (cmd) onEdit(cmd);
        return;
      }
      if (e.key.toLowerCase() === "n" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        onOpenAdd();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [
    enabled,
    commands,
    selectedIndex,
    listActiveRef,
    searchRef,
    setSelectedIndex,
    onCopy,
    onOpenAdd,
    onEdit,
  ]);
}
