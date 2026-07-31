import type { Command } from "../types";

interface Props {
  command: Command;
  selected: boolean;
  onCopy: (cmd: Command) => void;
  onDelete: (cmd: Command) => void;
}

export default function CommandItem({ command, selected, onCopy, onDelete }: Props) {
  return (
    <li className={selected ? "item selected" : "item"}>
      <div className="item-main" onClick={() => onCopy(command)}>
        <div className="item-title">{command.title}</div>
        <pre className="item-command">{command.command}</pre>
        {command.note && <div className="item-note">{command.note}</div>}
        {command.tags && (
          <div className="item-tags">
            {command.tags
              .split(",")
              .map((t) => t.trim())
              .filter(Boolean)
              .map((t, i) => (
                <span key={i} className="tag">
                  {t}
                </span>
              ))}
          </div>
        )}
      </div>
      <button
        className="delete"
        title="Delete"
        onClick={(e) => {
          e.stopPropagation();
          onDelete(command);
        }}
      >
        ×
      </button>
    </li>
  );
}
