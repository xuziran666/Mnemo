use serde::{Deserialize, Serialize};

// 条目类型：Snippet（代码片段，复制到剪贴板）与 Note（笔记，纯文本查看）。
pub const KIND_SNIPPET: i64 = 1;
// KIND_NOTE 由后续 UI 层判定条目类型时使用，核心库阶段暂无调用点。
#[allow(dead_code)]
pub const KIND_NOTE: i64 = 2;

// Command 表示本地 SQLite 数据库中的持久化记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub note: Option<String>,
    pub tags: Option<String>,
    pub kind: i64,
    pub created_at: i64,
}

// NewCommand 是在创建或编辑记录时使用的输入形状。
#[derive(Debug, Clone, Deserialize)]
pub struct NewCommand {
    pub title: String,
    pub content: String,
    pub note: Option<String>,
    pub tags: Option<String>,
    pub kind: i64,
}

// ImportResult 报告了导入和跳过了多少条记录。
#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
}

// ExportData 是导出 JSON 的顶层结构，version 字段用于未来格式演进。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    #[serde(default)]
    pub version: i64,
    pub commands: Vec<ExportCommand>,
}

// ExportCommand 是导出时的条目形状，note/tags 可选，kind 有默认值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportCommand {
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(default = "default_kind")]
    pub kind: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

fn default_kind() -> i64 {
    KIND_SNIPPET
}