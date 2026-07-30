//! Types from `packages/coding-agent/src/core/session-manager.ts`.
//!
//! Session entries and tree nodes returned by RPC `get_entries` and `get_tree`.

use serde::{Deserialize, Serialize};

use super::agent::AgentMessage;
use super::ai::{ContentBlock, Usage};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessageEntry {
  pub id: String,
  pub parent_id: Option<String>,
  pub timestamp: String,
  pub message: AgentMessage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingLevelChangeEntry {
  pub id: String,
  pub parent_id: Option<String>,
  pub timestamp: String,
  pub thinking_level: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChangeEntry {
  pub id: String,
  pub parent_id: Option<String>,
  pub timestamp: String,
  pub provider: String,
  pub model_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactionEntry {
  pub id: String,
  pub parent_id: Option<String>,
  pub timestamp: String,
  pub summary: String,
  pub first_kept_entry_id: String,
  pub tokens_before: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub details: Option<serde_json::Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub usage: Option<Usage>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub from_hook: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchSummaryEntry {
  pub id: String,
  pub parent_id: Option<String>,
  pub timestamp: String,
  pub from_id: String,
  pub summary: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub details: Option<serde_json::Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub usage: Option<Usage>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub from_hook: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomEntry {
  pub id: String,
  pub parent_id: Option<String>,
  pub timestamp: String,
  pub custom_type: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomMessageContent {
  Text(String),
  Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomMessageEntry {
  pub id: String,
  pub parent_id: Option<String>,
  pub timestamp: String,
  pub custom_type: String,
  pub content: CustomMessageContent,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub details: Option<serde_json::Value>,
  pub display: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelEntry {
  pub id: String,
  pub parent_id: Option<String>,
  pub timestamp: String,
  pub target_id: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfoEntry {
  pub id: String,
  pub parent_id: Option<String>,
  pub timestamp: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEntry {
  Message(SessionMessageEntry),
  ThinkingLevelChange(ThinkingLevelChangeEntry),
  ModelChange(ModelChangeEntry),
  Compaction(CompactionEntry),
  BranchSummary(BranchSummaryEntry),
  Custom(CustomEntry),
  CustomMessage(CustomMessageEntry),
  Label(LabelEntry),
  SessionInfo(SessionInfoEntry),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTreeNode {
  pub entry: SessionEntry,
  pub children: Vec<SessionTreeNode>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub label: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub label_timestamp: Option<String>,
}
