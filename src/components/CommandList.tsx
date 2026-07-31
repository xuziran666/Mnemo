import type { Command } from "../types";
import CommandItem from "./CommandItem";

interface Props {
  commands: Command[];
  onCopy: (cmd: Command) => void;
  onDelete: (cmd: Command) => void;
}

export default function CommandList({ commands, onCopy, onDelete }: Props) {
  if (commands.length === 0) {
    return <div className="empty">No commands found</div>;
  }

  return (
    <ul className="list">
      {commands.map((c) => (
        <CommandItem
          key={c.id}
          command={c}
          onCopy={onCopy}
          onDelete={onDelete}
        />
      ))}
    </ul>
  );
}
