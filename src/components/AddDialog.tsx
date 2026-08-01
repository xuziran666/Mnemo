import { useState } from "react";
import type { Command, NewCommand } from "../types";

interface Props {
  command?: Command | null;
  onClose: () => void;
  onSave: (input: NewCommand, id?: number) => void;
}

export default function AddDialog({ command, onClose, onSave }: Props) {
  const [title, setTitle] = useState(command?.title ?? "");
  const [cmd, setCmd] = useState(command?.command ?? "");
  const [note, setNote] = useState(command?.note ?? "");
  const [tags, setTags] = useState(command?.tags ?? "");

  const canSave = title.trim().length > 0 && cmd.trim().length > 0;

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!canSave) return;
    onSave(
      {
        title: title.trim(),
        command: cmd.trim(),
        note: note.trim() || null,
        tags: tags.trim() || null,
      },
      command?.id,
    );
  }

  return (
    <div className="overlay" onClick={onClose}>
      <form className="dialog" onClick={(e) => e.stopPropagation()} onSubmit={handleSubmit}>
        <h2>{command ? "Edit Command" : "New Command"}</h2>
        <input
          className="field"
          placeholder="Title"
          value={title}
          autoFocus
          onChange={(e) => setTitle(e.target.value)}
        />
        <textarea
          className="field"
          placeholder="Command"
          rows={3}
          value={cmd}
          onChange={(e) => setCmd(e.target.value)}
        />
        <input
          className="field"
          placeholder="Note (optional)"
          value={note}
          onChange={(e) => setNote(e.target.value)}
        />
        <input
          className="field"
          placeholder="Tags (comma separated, optional)"
          value={tags}
          onChange={(e) => setTags(e.target.value)}
        />
        <div className="dialog-actions">
          <button type="button" className="btn" onClick={onClose}>
            Cancel
          </button>
          <button type="submit" className="btn primary" disabled={!canSave}>
            Save
          </button>
        </div>
      </form>
    </div>
  );
}
