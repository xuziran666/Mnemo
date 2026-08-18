import { lazy, Suspense, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { KIND_NOTE, type Command, type NewCommand } from "../types";
import EntryEditor from "./EntryEditor";

const NoteView = lazy(() => import("./NoteView"));

interface Props {
  command: Command;
  onSave: (input: NewCommand, id: number) => Promise<void>;
  onExit: () => void;
}

export default function Viewer({ command, onSave, onExit }: Props) {
  const { t } = useTranslation();
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
        setEditing(true);
      } else if (e.key === "Escape") {
        e.preventDefault();
        onExit();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [editing, onExit]);

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
        <span className={isNote ? "badge note" : "badge"}>
          {isNote ? t("badge.note") : t("badge.snippet")}
        </span>
        <h1 className="viewer-title">{command.title}</h1>
      </div>
      <div className="viewer-body">
        {isNote ? (
          <Suspense fallback={<div className="viewer-content">{t("viewer.loading")}</div>}>
            <NoteView content={command.content} />
          </Suspense>
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
        <button type="button" className="btn" onClick={onExit}>
          {t("viewer.cancel")}
        </button>
        <button type="button" className="btn primary" onClick={() => setEditing(true)}>
          {t("viewer.edit")}
        </button>
      </div>
      <div className="viewer-hints">
        <span className="hint">
          <kbd>Enter</kbd> {t("viewer.edit")}
        </span>
        <span className="hint">
          <kbd>Esc</kbd> {t("viewer.cancel")}
        </span>
      </div>
    </div>
  );
}
