//! Types from `packages/coding-agent/src/core/compaction/compaction.ts`.

use serde::{Deserialize, Serialize};

/// Result from context compaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompactionResult {
  pub summary: String,
  #[serde(rename = "firstKeptEntryId")]
  pub first_kept_entry_id: String,
  #[serde(rename = "tokensBefore")]
  pub tokens_before: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub details: Option<serde_json::Value>,
}
