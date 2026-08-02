export const KIND_SNIPPET = 1;
export const KIND_NOTE = 2;

export interface Command {
  id: number;
  title: string;
  content: string;
  note?: string | null;
  tags?: string | null;
  kind: number;
  created_at: number;
}

export interface NewCommand {
  title: string;
  content: string;
  note?: string | null;
  tags?: string | null;
  kind: number;
}
