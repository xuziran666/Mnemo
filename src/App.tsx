import { useCallback, useEffect, useRef, useState } from "react";
import { createCommand, deleteCommand, listCommands } from "./api";
import AddDialog from "./components/AddDialog";
import CommandList from "./components/CommandList";
import SearchBox from "./components/SearchBox";
import type { Command, NewCommand } from "./types";
import "./App.css";

export default function App() {
  const [query, setQuery] = useState("");
  const [commands, setCommands] = useState<Command[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const toastTimer = useRef<number | null>(null);

  const load = useCallback(async (q: string) => {
    const result = await listCommands(q);
    setCommands(result);
  }, []);

  useEffect(() => {
    load(query);
  }, [query, load]);

  function showToast(message: string) {
    setToast(message);
    if (toastTimer.current) window.clearTimeout(toastTimer.current);
    toastTimer.current = window.setTimeout(() => setToast(null), 1200);
  }

  async function handleCopy(cmd: Command) {
    try {
      await navigator.clipboard.writeText(cmd.command);
      showToast("Copied!");
    } catch {
      showToast("Copy failed");
    }
  }

  async function handleDelete(cmd: Command) {
    if (!window.confirm(`Delete "${cmd.title}"?`)) return;
    await deleteCommand(cmd.id);
    await load(query);
  }

  async function handleSave(input: NewCommand) {
    await createCommand(input);
    setShowAdd(false);
    await load(query);
  }

  return (
    <div className="app">
      <div className="toolbar">
        <SearchBox query={query} onChange={setQuery} />
        <button className="add" title="Add command" onClick={() => setShowAdd(true)}>
          +
        </button>
      </div>
      <CommandList commands={commands} onCopy={handleCopy} onDelete={handleDelete} />
      {showAdd && <AddDialog onClose={() => setShowAdd(false)} onSave={handleSave} />}
      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}
