import { KIND_NOTE, type Command } from "../types";

interface Props {
  command: Command;
  selected: boolean;
  onOpen: (cmd: Command) => void;
  onCopy: (cmd: Command) => void;
  onEdit: (cmd: Command) => void;
  onDelete: (cmd: Command) => void;
}

export default function CommandItem({ command, selected, onOpen, onCopy, onEdit, onDelete }: Props) {
  const isNote = command.kind === KIND_NOTE;
  return (
    <li className={selected ? "item selected" : "item"}>
      <div
        className="item-main"
        onClick={() => (isNote ? onOpen(command) : onCopy(command))}
      >
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
      <div className="item-actions">
        <button
          className="act view"
          title="查看"
          onClick={(e) => {
            e.stopPropagation();
            onOpen(command);
          }}
        >
          查看
        </button>
        <button
          className="act copy"
          title="复制"
          onClick={(e) => {
            e.stopPropagation();
            onCopy(command);
          }}
        >
          复制
        </button>
        <button
          className="act edit"
          title="修改"
          onClick={(e) => {
            e.stopPropagation();
            onEdit(command);
          }}
        >
          修改
        </button>
        <button
          className="act delete"
          title="删除"
          onClick={(e) => {
            e.stopPropagation();
            onDelete(command);
          }}
        >
          删除
        </button>
      </div>
    </li>
  );
}
