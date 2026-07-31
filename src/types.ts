export interface Command {
  id: number;
  title: string;
  command: string;
  note?: string | null;
  tags?: string | null;
  created_at: number;
}

export interface NewCommand {
  title: string;
  command: string;
  note?: string | null;
  tags?: string | null;
}
