import { useEffect, type RefObject } from "react";
import type { Command } from "../types";

// 让当前高亮项始终在可视区域内，避免长列表下选中项被滚动挡住。
export function useScrollSelectedIntoView(
  selectedIndex: number,
  commands: Command[],
  listRef: RefObject<HTMLUListElement | null>,
) {
  useEffect(() => {
    if (selectedIndex >= 0 && listRef.current) {
      const el = listRef.current.children[selectedIndex] as HTMLElement | undefined;
      el?.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex, commands, listRef]);
}
