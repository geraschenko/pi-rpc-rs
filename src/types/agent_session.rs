//! Types from `packages/coding-agent/src/core/agent-session.ts`.
//!
//! The `AgentSessionEvent` type from this file extends `AgentEvent` with
//! additional variants; those are included directly in `AgentEvent` in
//! `agent.rs` rather than being a separate type here.

use serde::{Deserialize, Serialize};

/// Session statistics (token usage and cost).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStats {
  #[serde(rename = "sessionFile")]
  pub session_file: Option<String>,
  #[serde(rename = "sessionId")]
  pub session_id: String,
  #[serde(rename = "userMessages")]
  pub user_messages: f64,
  #[serde(rename = "assistantMessages")]
  pub assistant_messages: f64,
  #[serde(rename = "toolCalls")]
  pub tool_calls: f64,
  #[serde(rename = "toolResults")]
  pub tool_results: f64,
  #[serde(rename = "totalMessages")]
  pub total_messages: f64,
  pub tokens: SessionTokens,
  pub cost: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionTokens {
  pub input: f64,
  pub output: f64,
  #[serde(rename = "cacheRead")]
  pub cache_read: f64,
  #[serde(rename = "cacheWrite")]
  pub cache_write: f64,
  pub total: f64,
}
