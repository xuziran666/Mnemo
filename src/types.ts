// “Snippet”和“Note”是Mnemo支持的两种内容模式。
// Snippet会被复制到剪贴板，而Note则以Markdown模式查看。
export const KIND_SNIPPET = 1;
export const KIND_NOTE = 2;

// Command 表示本地 SQLite 数据库中的持久化记录。
export interface Command {
  id: number;
  title: string;
  content: string;
  note?: string | null;
  tags?: string | null;
  kind: number;
  created_at: number;
}

// NewCommand 是在创建或编辑记录时使用的输入形状。
export interface NewCommand {
  title: string;
  content: string;
  note?: string | null;
  tags?: string | null;
  kind: number;
}

// ImportResult 报告了导入和跳过了多少条记录。
export interface ImportResult {
  imported: number;
  skipped: number;
}
