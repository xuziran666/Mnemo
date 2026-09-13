import { useEffect, type Dispatch, type SetStateAction } from "react";

// 保证当前选中索引不会越界；当列表长度缩短后，选中位置会被强制收敛到有效范围内。
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
