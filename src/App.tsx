import { useCallback, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { createCommand, deleteCommand, listCommands, updateCommand } from "./api";
import CommandList from "./components/CommandList";
import EntryEditor from "./components/EntryEditor";
import SearchBox from "./components/SearchBox";
import Viewer from "./components/Viewer";
import { useClampSelectedIndex } from "./hooks/useClampSelectedIndex";
import { useCommandHotkeys } from "./hooks/useCommandHotkeys";
import { useLoadCommands } from "./hooks/useLoadCommands";
import { useScrollSelectedIntoView } from "./hooks/useScrollSelectedIntoView";
import type { Command, NewCommand } from "./types";
import "./App.css";

export default function App() {
  const [query, setQuery] = useState("");
  const [commands, setCommands] = useState<Command[]>([]);
  const [editing, setEditing] = useState<Command | "new" | null>(null);
  const [viewing, setViewing] = useState<Command | null>(null);
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
    enabled: !editing && !viewing,
    commands,
    selectedIndex,
    listActiveRef: listActive,
    searchRef,
    setSelectedIndex,
    onOpen: setViewing,
    onOpenAdd: () => setEditing("new"),
    onEdit: setEditing,
    onCopy: handleCopy,
    onClose: () => {
      void getCurrentWindow().close();
    },
  });

  function showToast(message: string) {
    setToast(message);
    if (toastTimer.current) window.clearTimeout(toastTimer.current);
    toastTimer.current = window.setTimeout(() => setToast(null), 1200);
  }

  function exitViewer() {
    setViewing(null);
    listActive.current = false;
    searchRef.current?.focus();
  }

  async function handleCopy(cmd: Command) {
    try {
      await writeText(cmd.content);
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

  async function handleSave(input: NewCommand, id?: number): Promise<Command | undefined> {
    let saved: Command;
    if (id != null) {
      saved = await updateCommand(id, input);
    } else {
      saved = await createCommand(input);
    }
    await load(query);
    return saved;
  }

  if (editing !== null) {
    return (
      <EntryEditor
        initial={editing === "new" ? null : editing}
        onSave={async (input, id) => {
          await handleSave(input, id);
          setEditing(null);
        }}
        onCancel={() => setEditing(null)}
      />
    );
  }

  if (viewing) {
    return (
      <Viewer
        command={viewing}
        onExit={exitViewer}
        onSave={async (input, id) => {
          const saved = await handleSave(input, id);
          if (saved) {
            setViewing(saved);
            showToast("Saved");
          }
        }}
      />
    );
  }

  return (
    <div className="app">
      <div className="toolbar">
        <SearchBox query={query} onChange={setQuery} inputRef={searchRef} />
        <button className="add" title="Add command" onClick={() => setEditing("new")}>
          +
        </button>
      </div>
      <CommandList
        commands={commands}
        selectedIndex={selectedIndex}
        listRef={listRef}
        onOpen={setViewing}
        onCopy={handleCopy}
        onEdit={setEditing}
        onDelete={handleDelete}
      />
      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}
