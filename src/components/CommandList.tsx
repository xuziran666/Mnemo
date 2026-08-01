import type { Ref } from "react";
import type { Command } from "../types";
import CommandItem from "./CommandItem";

interface Props {
  commands: Command[];
  selectedIndex: number;
  listRef: Ref<HTMLUListElement>;
  onCopy: (cmd: Command) => void;
  onEdit: (cmd: Command) => void;
  onDelete: (cmd: Command) => void;
}

export default function CommandList({
  commands,
  selectedIndex,
  listRef,
  onCopy,
  onEdit,
  onDelete,
}: Props) {
  if (commands.length === 0) {
    return <div className="empty">No commands found</div>;
  }

  return (
    <ul className="list" ref={listRef}>
      {commands.map((c, i) => (
        <CommandItem
          key={c.id}
          command={c}
          selected={i === selectedIndex}
          onCopy={onCopy}
          onEdit={onEdit}
          onDelete={onDelete}
        />
      ))}
    </ul>
  );
}
