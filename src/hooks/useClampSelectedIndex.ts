import { useEffect, type Dispatch, type SetStateAction } from "react";

export function useClampSelectedIndex(
  selectedIndex: number,
  listLength: number,
  setSelectedIndex: Dispatch<SetStateAction<number>>,
) {
  useEffect(() => {
    if (selectedIndex >= listLength) {
      setSelectedIndex(listLength - 1);
    }
  }, [selectedIndex, listLength, setSelectedIndex]);
}
