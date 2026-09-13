import { useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { confirm, open, save } from "@tauri-apps/plugin-dialog";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import { createCommand, deleteCommand, exportCommands, importCommands, listCommands, updateCommand } from "./api";
import CommandList from "./components/CommandList";
import { ExportIcon, ImportIcon, PlusIcon } from "./components/icons";
import EntryEditor from "./components/EntryEditor";
import SearchBox from "./components/SearchBox";
import Viewer from "./components/Viewer";
import { useClampSelectedIndex } from "./hooks/useClampSelectedIndex";
import { useCommandHotkeys } from "./hooks/useCommandHotkeys";
import { useLoadCommands } from "./hooks/useLoadCommands";
import { useScrollSelectedIntoView } from "./hooks/useScrollSelectedIntoView";
import type { Command, NewCommand } from "./types";
import "./App.css";

// 根 App 组件是搜索、选择、编辑和数据输入输出的控制中心。
// 它决定了 UI 显示的是列表、新建/编辑表单，还是笔记查看器。
export default function App() {
  const { t, i18n } = useTranslation();
  const [query, setQuery] = useState("");
  const [commands, setCommands] = useState<Command[]>([]);
  const [editing, setEditing] = useState<Command | "new" | null>(null);
  const [viewing, setViewing] = useState<Command | null>(null);
  const [toast, setToast] = useState<string | null>(null);// 消息提示
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const toastTimer = useRef<number | null>(null);
  const listActive = useRef(false);
  const searchRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  // load() 是从 UI 到 Tauri 后端的桥梁，并在每次变更后刷新列表。
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
    onDelete: handleDelete,
    onClose: () => {
      void getCurrentWindow().close();
    },
  });

  // 针对复制/导入/导出操作的小型短暂反馈，匹配以键盘为主的用户体验。
  function showToast(message: string) {
    setToast(message);
    if (toastTimer.current) window.clearTimeout(toastTimer.current);
    toastTimer.current = window.setTimeout(() => setToast(null), 1200);
  }

  // 从查看器返回时，应将焦点恢复到搜索框，同时保持列表状态不变。
  function exitViewer() {
    setViewing(null);
    listActive.current = false;
    searchRef.current?.focus();
  }

  // 复制一段代码片段时，应将原始内容推送到系统剪贴板并立即关闭应用程序。
  async function handleCopy(cmd: Command) {
    try {
      await writeText(cmd.content);
      await getCurrentWindow().close();
    } catch {
      showToast(t("toast.copyFailed"));
    }
  }

  async function handleDelete(cmd: Command) {
    if (!(await confirm(t("confirm.delete", { title: cmd.title })))) return;
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

  // 导出将完整的本地数据库状态序列化到用户选择的 JSON 文件中。
  async function handleExport() {
    try {
      const json = await exportCommands();
      const path = await save({
        defaultPath: "mnemo-export.json",
        filters: [{ name: t("io.exportFilter"), extensions: ["json"] }],
      });
      if (!path) return;
      await writeFile(path, new TextEncoder().encode(json));
      showToast(t("toast.exported"));
    } catch {
      showToast(t("toast.exportFailed"));
    }
  }

  // 导入从磁盘读取 JSON 导出，然后用合并结果刷新列表。
  async function handleImport() {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: t("io.importFilter"), extensions: ["json"] }],
      });
      if (!path) return;
      const data = await readFile(path as string);
      const json = new TextDecoder().decode(data);
      const result = await importCommands(json);
      await load(query);
      if (result.skipped > 0) {
        showToast(t("toast.importSkipped", { imported: result.imported, skipped: result.skipped }));
      } else {
        showToast(t("toast.imported", { count: result.imported }));
      }
    } catch {
      showToast(t("toast.importFailed"));
    }
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
            showToast(t("toast.saved"));
          }
        }}
      />
    );
  }

  return (
    <div className="app">
      <div className="toolbar">
        <SearchBox query={query} onChange={setQuery} inputRef={searchRef} />
        <button
          className="lang"
          title={t("lang.title")}
          onClick={() => void i18n.changeLanguage(i18n.language.startsWith("zh") ? "en" : "zh")}
        >
          {t("lang.button")}
        </button>
        <button className="add" title={t("io.import")} onClick={() => void handleImport()}>
          <ImportIcon />
        </button>
        <button className="add" title={t("io.export")} onClick={() => void handleExport()}>
          <ExportIcon />
        </button>
        <button
          className="add"
          title={`${t("add.title")} (Ctrl+N)`}
          onClick={() => setEditing("new")}
        >
          <PlusIcon />
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
