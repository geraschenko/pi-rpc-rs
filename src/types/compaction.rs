//! Types from `packages/coding-agent/src/core/compaction/compaction.ts`.

use serde::{Deserialize, Serialize};

/// Result from context compaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactionResult {
  pub summary: String,
  pub first_kept_entry_id: String,
  pub tokens_before: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub estimated_tokens_after: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub details: Option<serde_json::Value>,
}
