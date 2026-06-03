//! Types from `packages/agent/src/types.ts`.
//!
//! Agent message and event types. `AgentMessage` includes variants added by
//! `packages/coding-agent/src/core/messages.ts` via declaration merging.
//! `AgentEvent` includes variants added by
//! `packages/coding-agent/src/core/agent-session.ts` (`AgentSessionEvent`).

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

use super::ai::*;
use super::compaction::CompactionResult;

// ============================================================================
// ThinkingLevel
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, AsRefStr, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ThinkingLevel {
  Off,
  Minimal,
  Low,
  Medium,
  High,
  XHigh,
}

// ============================================================================
// AgentMessage
// ============================================================================

/// The resolved `AgentMessage` union.
///
/// In TypeScript, `AgentMessage = Message | CustomAgentMessages[...]` where
/// `Message = UserMessage | AssistantMessage | ToolResultMessage` is defined in
/// `packages/ai/src/types.ts`, and the custom message types are added by
/// `packages/coding-agent/src/core/messages.ts` via declaration merging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(
  tag = "role",
  rename_all = "camelCase",
  rename_all_fields = "camelCase"
)]
#[strum(serialize_all = "camelCase")]
pub enum AgentMessage {
  // -- From packages/ai/src/types.ts (Message = UserMessage | AssistantMessage | ToolResultMessage) --
  User {
    content: UserContent,
    timestamp: f64,
  },
  Assistant {
    content: Vec<ContentBlock>,
    api: String,
    provider: String,
    model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    response_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    response_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    diagnostics: Option<Vec<AssistantMessageDiagnostic>>,
    usage: Usage,
    stop_reason: StopReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
    timestamp: f64,
  },
  ToolResult {
    tool_call_id: String,
    tool_name: String,
    content: Vec<ContentBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
    is_error: bool,
    timestamp: f64,
  },

  // -- From packages/coding-agent/src/core/messages.ts (declaration-merged into CustomAgentMessages) --
  BashExecution {
    command: String,
    output: String,
    exit_code: Option<f64>,
    cancelled: bool,
    truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    full_output_path: Option<String>,
    timestamp: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    exclude_from_context: Option<bool>,
  },
  Custom {
    custom_type: String,
    content: serde_json::Value, // string | (TextContent | ImageContent)[]
    display: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
    timestamp: f64,
  },
  BranchSummary {
    summary: String,
    from_id: String,
    timestamp: f64,
  },
  CompactionSummary {
    summary: String,
    tokens_before: f64,
    timestamp: f64,
  },
}

// ============================================================================
// AgentEvent
// ============================================================================

/// The resolved `AgentSessionEvent` union.
///
/// In TypeScript, `AgentEvent` is defined in `packages/agent/src/types.ts`, and
/// `AgentSessionEvent = AgentEvent | ...` in
/// `packages/coding-agent/src/core/agent-session.ts` extends it with additional
/// variants. We define the full union here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(
  tag = "type",
  rename_all = "snake_case",
  rename_all_fields = "camelCase"
)]
#[strum(serialize_all = "snake_case")]
pub enum AgentEvent {
  // -- From packages/agent/src/types.ts --

  // Agent lifecycle
  AgentStart,
  AgentEnd {
    messages: Vec<AgentMessage>,
  },

  // Turn lifecycle
  TurnStart,
  TurnEnd {
    message: AgentMessage,
    tool_results: Vec<AgentMessage>, // always ToolResult variants
  },

  // Message lifecycle
  MessageStart {
    message: AgentMessage,
  },
  MessageUpdate {
    message: AgentMessage,
    assistant_message_event: AssistantMessageEvent,
  },
  MessageEnd {
    message: AgentMessage,
  },

  // Tool execution
  ToolExecutionStart {
    tool_call_id: String,
    tool_name: String,
    args: serde_json::Value,
  },
  ToolExecutionUpdate {
    tool_call_id: String,
    tool_name: String,
    args: serde_json::Value,
    partial_result: serde_json::Value,
  },
  ToolExecutionEnd {
    tool_call_id: String,
    tool_name: String,
    result: serde_json::Value,
    is_error: bool,
  },

  // -- From packages/coding-agent/src/core/agent-session.ts (AgentSessionEvent extensions) --
  QueueUpdate {
    steering: Vec<String>,
    follow_up: Vec<String>,
  },
  CompactionStart {
    reason: CompactionReason,
  },
  SessionInfoChanged {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
  },
  ThinkingLevelChanged {
    level: ThinkingLevel,
  },
  CompactionEnd {
    reason: CompactionReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    result: Option<CompactionResult>,
    aborted: bool,
    will_retry: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
  },
  AutoRetryStart {
    attempt: f64,
    max_attempts: f64,
    delay_ms: f64,
    error_message: String,
  },
  AutoRetryEnd {
    success: bool,
    attempt: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    final_error: Option<String>,
  },

  // -- From packages/coding-agent/src/modes/rpc/rpc-mode.ts (untyped in TS, only exists on the wire) --
  ExtensionError {
    extension_path: String,
    event: String,
    error: String,
  },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CompactionReason {
  Manual,
  Threshold,
  Overflow,
}
