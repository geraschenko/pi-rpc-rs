//! Types from `packages/coding-agent/src/core/agent-session.ts`.
//!
//! The `AgentSessionEvent` type from this file extends `AgentEvent` with
//! additional variants; those are included directly in `AgentEvent` in
//! `agent.rs` rather than being a separate type here.

use serde::{Deserialize, Serialize};

/// Session statistics (token usage and cost).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
  pub session_file: Option<String>,
  pub session_id: String,
  pub user_messages: f64,
  pub assistant_messages: f64,
  pub tool_calls: f64,
  pub tool_results: f64,
  pub total_messages: f64,
  pub tokens: SessionTokens,
  pub cost: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub context_usage: Option<ContextUsage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextUsage {
  pub tokens: Option<f64>,
  pub context_window: f64,
  pub percent: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTokens {
  pub input: f64,
  pub output: f64,
  pub cache_read: f64,
  pub cache_write: f64,
  pub total: f64,
}
