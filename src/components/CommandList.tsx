import type { Ref } from "react";
import { useTranslation } from "react-i18next";
import type { Command } from "../types";
import CommandItem from "./CommandItem";

interface Props {
  commands: Command[];
  selectedIndex: number;
  listRef: Ref<HTMLUListElement>;
  onOpen: (cmd: Command) => void;
  onCopy: (cmd: Command) => void;
  onEdit: (cmd: Command) => void;
  onDelete: (cmd: Command) => void;
}

// CommandList 是主列表容器，负责向用户展示当前筛选结果，并同步当前选中项。
// 这个列表在键盘操作中承担“导航 + 执行”的核心职责：上下键移动、回车触发动作。
export default function CommandList({
  commands,
  selectedIndex,
  listRef,
  onOpen,
  onCopy,
  onEdit,
  onDelete,
}: Props) {
  const { t } = useTranslation();
  if (commands.length === 0) {
    return <div className="empty">{t("empty.noCommands")}</div>;
  }

  return (
    <ul className="list" ref={listRef}>
      {commands.map((c, i) => (
        <CommandItem
          key={c.id}
          command={c}
          selected={i === selectedIndex}
          onOpen={onOpen}
          onCopy={onCopy}
          onEdit={onEdit}
          onDelete={onDelete}
        />
      ))}
    </ul>
  );
}
