interface Props {
  query: string;
  onChange: (value: string) => void;
}

export default function SearchBox({ query, onChange }: Props) {
  return (
    <input
      className="search"
      type="text"
      placeholder="Search commands..."
      value={query}
      autoFocus
      onChange={(e) => onChange(e.target.value)}
    />
  );
}
