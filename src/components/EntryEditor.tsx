import { useEffect, useRef, useState } from "react";
import { KIND_NOTE, KIND_SNIPPET, type Command, type NewCommand } from "../types";
import AutoGrowTextarea from "./AutoGrowTextarea";
import { captureWindowHeight, fitWindowHeight, restoreWindowHeight } from "../windowFit";

interface Props {
  initial?: Command | null;
  onSave: (input: NewCommand, id?: number) => Promise<void>;
  onCancel: () => void;
}

export default function EntryEditor({ initial, onSave, onCancel }: Props) {
  const [title, setTitle] = useState(initial?.title ?? "");
  const [content, setContent] = useState(initial?.content ?? "");
  const [note, setNote] = useState(initial?.note ?? "");
  const [tags, setTags] = useState(initial?.tags ?? "");
  const [kind, setKind] = useState(initial?.kind ?? KIND_SNIPPET);
  const rootRef = useRef<HTMLDivElement>(null);
  const fieldsRef = useRef<HTMLDivElement>(null);
  const titleRef = useRef<HTMLTextAreaElement>(null);
  const originalHeight = useRef<number | null>(null);
  const fitTimer = useRef<number | null>(null);

  useEffect(() => {
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
    const el = titleRef.current;
    if (el) {
      el.focus();
      el.setSelectionRange(el.value.length, el.value.length);
    }
  }, []);

  useEffect(() => {
    void captureWindowHeight().then((h) => {
      originalHeight.current = h;
    });
    const fields = fieldsRef.current;
    if (!fields) return;
    const observer = new ResizeObserver(() => scheduleFit());
    observer.observe(fields);
    scheduleFit();
    return () => {
      observer.disconnect();
      if (fitTimer.current) window.clearTimeout(fitTimer.current);
      if (originalHeight.current != null) {
        void restoreWindowHeight(originalHeight.current);
        originalHeight.current = null;
      }
    };
  }, []);

  function scheduleFit() {
    if (fitTimer.current) window.clearTimeout(fitTimer.current);
    fitTimer.current = window.setTimeout(() => {
      fitTimer.current = null;
      void fit();
    }, 60);
  }

  async function fit() {
    const root = rootRef.current;
    if (!root) return;
    await fitWindowHeight(root.scrollHeight + 4);
  }

  function canSave() {
    return title.trim().length > 0 && content.trim().length > 0;
  }

  async function save() {
    if (!canSave()) return;
    await onSave(
      {
        title: title.trim(),
        content: content.trim(),
        note: note.trim() || null,
        tags: tags.trim() || null,
        kind,
      },
      initial?.id,
    );
  }

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.preventDefault();
        onCancel();
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
        e.preventDefault();
        void save();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [title, content, note, tags, kind, initial, onSave, onCancel]);

  return (
    <div className="viewer" ref={rootRef}>
      <div className="editor-fields" ref={fieldsRef}>
        <div className="seg">
          <button
            type="button"
            className={kind === KIND_SNIPPET ? "seg-btn active" : "seg-btn"}
            onClick={() => setKind(KIND_SNIPPET)}
          >
            Snippet
          </button>
          <button
            type="button"
            className={kind === KIND_NOTE ? "seg-btn active" : "seg-btn"}
            onClick={() => setKind(KIND_NOTE)}
          >
            Note
          </button>
        </div>
        <AutoGrowTextarea
          ref={titleRef}
          className="field"
          placeholder="Title"
          rows={1}
          maxHeight={80}
          value={title}
          onKeyDown={(e) => {
            if (e.key === "Enter") e.preventDefault();
          }}
          onChange={(e) => setTitle(e.target.value)}
        />
        <AutoGrowTextarea
          className="field"
          placeholder="Content"
          rows={4}
          maxHeight={420}
          value={content}
          onChange={(e) => setContent(e.target.value)}
        />
        <AutoGrowTextarea
          className="field"
          placeholder="Note (optional)"
          rows={1}
          maxHeight={120}
          value={note}
          onKeyDown={(e) => {
            if (e.key === "Enter") e.preventDefault();
          }}
          onChange={(e) => setNote(e.target.value)}
        />
        <AutoGrowTextarea
          className="field"
          placeholder="Tags (comma separated, optional)"
          rows={1}
          maxHeight={120}
          value={tags}
          onChange={(e) => setTags(e.target.value)}
        />
      </div>
      <div className="editor-actions">
        <button type="button" className="btn" onClick={onCancel}>
          取消
        </button>
        <button
          type="button"
          className="btn primary"
          disabled={!canSave()}
          onClick={() => void save()}
        >
          保存
        </button>
      </div>
      <div className="viewer-hints">
        <span className="hint">
          <kbd>Ctrl+S</kbd> 保存
        </span>
        <span className="hint">
          <kbd>Esc</kbd> 取消
        </span>
      </div>
    </div>
  );
}
