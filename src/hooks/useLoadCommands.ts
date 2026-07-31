import { useEffect, type Dispatch, type SetStateAction } from "react";

export function useLoadCommands(
  query: string,
  load: (q: string) => Promise<void>,
  setSelectedIndex: Dispatch<SetStateAction<number>>,
) {
  useEffect(() => {
    load(query);
    setSelectedIndex(-1);
  }, [query, load, setSelectedIndex]);
}
