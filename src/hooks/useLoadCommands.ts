import { useEffect, type Dispatch, type SetStateAction } from "react";

// 连续输入时每次按键都会触发一次 IPC 查询。
// 这里加防抖把请求合并成一次，避免大库下高频往返拖慢界面。
const DEBOUNCE_MS = 120;

export function useLoadCommands(
  query: string,
  load: (q: string) => Promise<void>,
  setSelectedIndex: Dispatch<SetStateAction<number>>,
) {
  useEffect(() => {
    setSelectedIndex(-1);
    const timer = window.setTimeout(() => {
      void load(query);
    }, DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [query, load, setSelectedIndex]);
}
