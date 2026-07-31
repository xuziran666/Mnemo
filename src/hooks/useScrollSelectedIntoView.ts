import { useEffect, type RefObject } from "react";
import type { Command } from "../types";

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
