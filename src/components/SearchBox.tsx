import type { Ref } from "react";

interface Props {
  query: string;
  onChange: (value: string) => void;
  inputRef: Ref<HTMLInputElement>;
}

export default function SearchBox({ query, onChange, inputRef }: Props) {
  return (
    <input
      ref={inputRef}
      className="search"
      type="text"
      placeholder="Search commands..."
      value={query}
      autoFocus
      onChange={(e) => onChange(e.target.value)}
    />
  );
}
