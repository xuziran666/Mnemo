import { invoke } from "@tauri-apps/api/core";
import type { Command, NewCommand } from "./types";

export function listCommands(query: string): Promise<Command[]> {
  return invoke("list_commands", { query });
}

export function createCommand(input: NewCommand): Promise<Command> {
  return invoke("create_command", { input });
}

export function deleteCommand(id: number): Promise<void> {
  return invoke("delete_command", { id });
}
