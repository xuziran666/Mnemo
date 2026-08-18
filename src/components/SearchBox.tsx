import type { Ref } from "react";
import { useTranslation } from "react-i18next";

interface Props {
  query: string;
  onChange: (value: string) => void;
  inputRef: Ref<HTMLInputElement>;
}

export default function SearchBox({ query, onChange, inputRef }: Props) {
  const { t } = useTranslation();
  return (
    <input
      ref={inputRef}
      className="search"
      type="text"
      placeholder={t("search.placeholder")}
      value={query}
      autoFocus
      onChange={(e) => onChange(e.target.value)}
    />
  );
}
