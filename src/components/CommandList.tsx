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
