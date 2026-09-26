import type { Ref } from "react";
import { useTranslation } from "react-i18next";

interface Props {
  query: string;
  onChange: (value: string) => void;
  inputRef: Ref<HTMLInputElement>;
  onFocus?: () => void;
}

// SearchBox 是全局搜索入口，整个应用的核心交互都由这个输入框发起。
// 它保持 autoFocus，确保用户一打开应用就可以直接输入关键字，符合 keyboard-first 的设计目标。
// onFocus 用于在鼠标点击回到输入框时把列表交还输入框：否则 listActive 仍为 true，
// 单字母快捷键会继续拦截输入，方向键也会被 preventDefault，用户无法正常打字。
export default function SearchBox({ query, onChange, inputRef, onFocus }: Props) {
  const { t } = useTranslation();
  return (
    <input
      ref={inputRef}
      className="search"
      type="text"
      placeholder={t("search.placeholder")}
      value={query}
      autoFocus
      onFocus={onFocus}
      onChange={(e) => onChange(e.target.value)}
    />
  );
}
