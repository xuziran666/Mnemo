import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation();
  const isNote = command.kind === KIND_NOTE;
  return (
    <li className={selected ? "item selected" : "item"}>
      <div
        className="item-main"
        onClick={() => (isNote ? onOpen(command) : onCopy(command))}
      >
        <div className="item-title">
          <span className={isNote ? "badge note" : "badge"}>
            {isNote ? t("badge.note") : t("badge.snippet")}
          </span>
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
          title={t("actions.view")}
          onClick={(e) => {
            e.stopPropagation();
            onOpen(command);
          }}
        >
          {t("actions.view")}
        </button>
        <button
          className="act copy"
          title={t("actions.copy")}
          onClick={(e) => {
            e.stopPropagation();
            onCopy(command);
          }}
        >
          {t("actions.copy")}
        </button>
        <button
          className="act edit"
          title={t("actions.edit")}
          onClick={(e) => {
            e.stopPropagation();
            onEdit(command);
          }}
        >
          {t("actions.edit")}
        </button>
        <button
          className="act delete"
          title={t("actions.delete")}
          onClick={(e) => {
            e.stopPropagation();
            onDelete(command);
          }}
        >
          {t("actions.delete")}
        </button>
      </div>
    </li>
  );
}
