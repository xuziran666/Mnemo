import { KIND_NOTE, type Command } from "../types";

interface Props {
  command: Command;
  selected: boolean;
  onOpen: (cmd: Command) => void;
  onEdit: (cmd: Command) => void;
  onDelete: (cmd: Command) => void;
}

export default function CommandItem({ command, selected, onOpen, onEdit, onDelete }: Props) {
  const isNote = command.kind === KIND_NOTE;
  return (
    <li className={selected ? "item selected" : "item"}>
      <div className="item-main" onClick={() => onOpen(command)}>
        <div className="item-title">
          <span className={isNote ? "badge note" : "badge"}>{isNote ? "知识" : "代码"}</span>
          {command.title}
        </div>
        {!isNote && <pre className="item-command">{command.content}</pre>}
        {!isNote && command.note && <div className="item-note">{command.note}</div>}
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
        className="edit"
        title="Edit"
        onClick={(e) => {
          e.stopPropagation();
          onEdit(command);
        }}
      >
        ✎
      </button>
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
