import { useState } from "react";
import type { NewCommand } from "../types";

interface Props {
  onClose: () => void;
  onSave: (input: NewCommand) => void;
}

export default function AddDialog({ onClose, onSave }: Props) {
  const [title, setTitle] = useState("");
  const [command, setCommand] = useState("");
  const [note, setNote] = useState("");
  const [tags, setTags] = useState("");

  const canSave = title.trim().length > 0 && command.trim().length > 0;

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!canSave) return;
    onSave({
      title: title.trim(),
      command: command.trim(),
      note: note.trim() || null,
      tags: tags.trim() || null,
    });
  }

  return (
    <div className="overlay" onClick={onClose}>
      <form className="dialog" onClick={(e) => e.stopPropagation()} onSubmit={handleSubmit}>
        <h2>New Command</h2>
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
          value={command}
          onChange={(e) => setCommand(e.target.value)}
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
