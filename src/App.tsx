import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
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
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const toastTimer = useRef<number | null>(null);
  const listActive = useRef(false);
  const searchRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const load = useCallback(async (q: string) => {
    const result = await listCommands(q);
    setCommands(result);
  }, []);

  useEffect(() => {
    load(query);
    setSelectedIndex(-1);
  }, [query, load]);

  function showToast(message: string) {
    setToast(message);
    if (toastTimer.current) window.clearTimeout(toastTimer.current);
    toastTimer.current = window.setTimeout(() => setToast(null), 1200);
  }

  useEffect(() => {
    if (selectedIndex >= 0 && listRef.current) {
      const el = listRef.current.children[selectedIndex] as HTMLElement | undefined;
      el?.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex, commands]);

  useEffect(() => {
    if (selectedIndex >= commands.length) {
      setSelectedIndex(commands.length - 1);
    }
  }, [commands, selectedIndex]);

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (showAdd) return;
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        if (!listActive.current) return;
        e.preventDefault();
        setSelectedIndex((i) => {
          if (commands.length === 0) return -1;
          if (e.key === "ArrowDown") return Math.min(i + 1, commands.length - 1);
          return Math.max(0, i - 1);
        });
        return;
      }
      if (e.key === "Enter") {
        if (document.activeElement === searchRef.current) {
          listActive.current = true;
          searchRef.current?.blur();
          setSelectedIndex(0);
        } else if (listActive.current) {
          const cmd = commands[selectedIndex];
          if (cmd) handleCopy(cmd);
        }
        return;
      }
      if (e.key === "s" || e.key === "S") {
        listActive.current = false;
        searchRef.current?.focus();
        return;
      }
      if (e.key.toLowerCase() === "n" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        setShowAdd(true);
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  });

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
