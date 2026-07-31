import { useCallback, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { createCommand, deleteCommand, listCommands } from "./api";
import AddDialog from "./components/AddDialog";
import CommandList from "./components/CommandList";
import SearchBox from "./components/SearchBox";
import { useClampSelectedIndex } from "./hooks/useClampSelectedIndex";
import { useCommandHotkeys } from "./hooks/useCommandHotkeys";
import { useLoadCommands } from "./hooks/useLoadCommands";
import { useScrollSelectedIntoView } from "./hooks/useScrollSelectedIntoView";
import type { Command, NewCommand } from "./types";
import "./App.css";

export default function App() {
  const [query, setQuery] = useState("");
  const [commands, setCommands] = useState<Command[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const toastTimer = useRef<number | null>(null);
  const listActive = useRef(false);
  const searchRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const load = useCallback(async (q: string) => {
    const result = await listCommands(q);
    setCommands(result);
  }, []);

  useLoadCommands(query, load, setSelectedIndex);
  useScrollSelectedIntoView(selectedIndex, commands, listRef);
  useClampSelectedIndex(selectedIndex, commands.length, setSelectedIndex);
  useCommandHotkeys({
    enabled: !showAdd,
    commands,
    selectedIndex,
    listActiveRef: listActive,
    searchRef,
    setSelectedIndex,
    onCopy: handleCopy,
    onOpenAdd: () => setShowAdd(true),
  });

  function showToast(message: string) {
    setToast(message);
    if (toastTimer.current) window.clearTimeout(toastTimer.current);
    toastTimer.current = window.setTimeout(() => setToast(null), 1200);
  }

  async function handleCopy(cmd: Command) {
    try {
      await navigator.clipboard.writeText(cmd.command);
      await getCurrentWindow().close();
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
        <SearchBox query={query} onChange={setQuery} inputRef={searchRef} />
        <button className="add" title="Add command" onClick={() => setShowAdd(true)}>
          +
        </button>
      </div>
      <CommandList
        commands={commands}
        selectedIndex={selectedIndex}
        listRef={listRef}
        onCopy={handleCopy}
        onDelete={handleDelete}
      />
      {showAdd && <AddDialog onClose={() => setShowAdd(false)} onSave={handleSave} />}
      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}
