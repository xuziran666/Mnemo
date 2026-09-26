import { invoke } from "@tauri-apps/api/core";
import type { Command, ImportResult, NewCommand } from "./types";

// 前端不会直接与 SQLite 通信。
// 所有的持久化、导入/导出和查询都通过 Tauri 命令进行。
export function listCommands(query: string): Promise<Command[]> {
  return invoke("list_commands", { query });
}

// 列表返回的 content 是后端在 SQL 层裁剪过的预览长度，
// 复制 / 查看 / 编辑前需要按 id 取回完整记录。
export function getCommand(id: number): Promise<Command> {
  return invoke("get_command", { id });
}

export function createCommand(input: NewCommand): Promise<Command> {
  return invoke("create_command", { input });
}

export function updateCommand(id: number, input: NewCommand): Promise<Command> {
  return invoke("update_command", { id, input });
}

export function deleteCommand(id: number): Promise<void> {
  return invoke("delete_command", { id });
}

export function exportCommands(): Promise<string> {
  return invoke("export_commands");
}

export function importCommands(json: string): Promise<ImportResult> {
  return invoke("import_commands", { json });
}
