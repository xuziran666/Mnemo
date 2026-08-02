import { useEffect, useState } from "react";
import { KIND_NOTE, type Command, type NewCommand } from "../types";
import EntryEditor from "./EntryEditor";
import NoteView from "./NoteView";

interface Props {
  command: Command;
  onCopy: (cmd: Command) => void;
  onSave: (input: NewCommand, id: number) => Promise<void>;
  onExit: () => void;
}

export default function Viewer({ command, onCopy, onSave, onExit }: Props) {
  const [editing, setEditing] = useState(false);
  const isNote = command.kind === KIND_NOTE;

  useEffect(() => {
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
  }, []);

  useEffect(() => {
    if (editing) return;
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Enter") {
        e.preventDefault();
        onCopy(command);
      } else if (e.key === "Escape") {
        e.preventDefault();
        onExit();
      } else if (e.key.toLowerCase() === "e") {
        e.preventDefault();
        setEditing(true);
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [editing, command, onCopy, onExit]);

  if (editing) {
    return (
      <EntryEditor
        initial={command}
        onSave={async (input, id) => {
          await onSave(input, id ?? command.id);
          setEditing(false);
        }}
        onCancel={() => setEditing(false)}
      />
    );
  }

  return (
    <div className="viewer">
      <div className="viewer-header">
        <span className={isNote ? "badge note" : "badge"}>{isNote ? "知识" : "代码"}</span>
        <h1 className="viewer-title">{command.title}</h1>
      </div>
      <div className="viewer-body">
        {isNote ? (
          <NoteView content={command.content} />
        ) : (
          <pre className="viewer-content">{command.content}</pre>
        )}
        {command.note && <div className="viewer-note">{command.note}</div>}
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
      <div className="viewer-actions">
        <button type="button" className="btn primary" onClick={() => onCopy(command)}>
          复制
        </button>
        <button type="button" className="btn" onClick={() => setEditing(true)}>
          编辑
        </button>
      </div>
      <div className="viewer-hints">
        <span className="hint">
          <kbd>Enter</kbd> 复制
        </span>
        <span className="hint">
          <kbd>E</kbd> 编辑
        </span>
        <span className="hint">
          <kbd>Esc</kbd> 返回
        </span>
      </div>
    </div>
  );
}
