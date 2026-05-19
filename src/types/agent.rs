//! Types from `packages/agent/src/types.ts`.
//!
//! Agent message and event types. `AgentMessage` includes variants added by
//! `packages/coding-agent/src/core/messages.ts` via declaration merging.
//! `AgentEvent` includes variants added by
//! `packages/coding-agent/src/core/agent-session.ts` (`AgentSessionEvent`).

use serde::{Deserialize, Serialize};

use super::ai::*;
use super::compaction::CompactionResult;

// ============================================================================
// ThinkingLevel
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThinkingLevel {
  #[serde(rename = "off")]
  Off,
  #[serde(rename = "minimal")]
  Minimal,
  #[serde(rename = "low")]
  Low,
  #[serde(rename = "medium")]
  Medium,
  #[serde(rename = "high")]
  High,
  #[serde(rename = "xhigh")]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum AgentMessage {
  // -- From packages/ai/src/types.ts (Message = UserMessage | AssistantMessage | ToolResultMessage) --
  #[serde(rename = "user")]
  User {
    content: UserContent,
    timestamp: f64,
  },
  #[serde(rename = "assistant")]
  Assistant {
    content: Vec<ContentBlock>,
    api: String,
    provider: String,
    model: String,
    #[serde(
      rename = "responseModel",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    response_model: Option<String>,
    #[serde(
      rename = "responseId",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    response_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    diagnostics: Option<Vec<AssistantMessageDiagnostic>>,
    usage: Usage,
    #[serde(rename = "stopReason")]
    stop_reason: StopReason,
    #[serde(
      rename = "errorMessage",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    error_message: Option<String>,
    timestamp: f64,
  },
  #[serde(rename = "toolResult")]
  ToolResult {
    #[serde(rename = "toolCallId")]
    tool_call_id: String,
    #[serde(rename = "toolName")]
    tool_name: String,
    content: Vec<ContentBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
    #[serde(rename = "isError")]
    is_error: bool,
    timestamp: f64,
  },

  // -- From packages/coding-agent/src/core/messages.ts (declaration-merged into CustomAgentMessages) --
  #[serde(rename = "bashExecution")]
  BashExecution {
    command: String,
    output: String,
    #[serde(rename = "exitCode")]
    exit_code: Option<f64>,
    cancelled: bool,
    truncated: bool,
    #[serde(
      rename = "fullOutputPath",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    full_output_path: Option<String>,
    timestamp: f64,
    #[serde(
      rename = "excludeFromContext",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    exclude_from_context: Option<bool>,
  },
  #[serde(rename = "custom")]
  Custom {
    #[serde(rename = "customType")]
    custom_type: String,
    content: serde_json::Value, // string | (TextContent | ImageContent)[]
    display: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
    timestamp: f64,
  },
  #[serde(rename = "branchSummary")]
  BranchSummary {
    summary: String,
    #[serde(rename = "fromId")]
    from_id: String,
    timestamp: f64,
  },
  #[serde(rename = "compactionSummary")]
  CompactionSummary {
    summary: String,
    #[serde(rename = "tokensBefore")]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
  // -- From packages/agent/src/types.ts --

  // Agent lifecycle
  #[serde(rename = "agent_start")]
  AgentStart,
  #[serde(rename = "agent_end")]
  AgentEnd { messages: Vec<AgentMessage> },

  // Turn lifecycle
  #[serde(rename = "turn_start")]
  TurnStart,
  #[serde(rename = "turn_end")]
  TurnEnd {
    message: AgentMessage,
    #[serde(rename = "toolResults")]
    tool_results: Vec<AgentMessage>, // always ToolResult variants
  },

  // Message lifecycle
  #[serde(rename = "message_start")]
  MessageStart { message: AgentMessage },
  #[serde(rename = "message_update")]
  MessageUpdate {
    message: AgentMessage,
    #[serde(rename = "assistantMessageEvent")]
    assistant_message_event: AssistantMessageEvent,
  },
  #[serde(rename = "message_end")]
  MessageEnd { message: AgentMessage },

  // Tool execution
  #[serde(rename = "tool_execution_start")]
  ToolExecutionStart {
    #[serde(rename = "toolCallId")]
    tool_call_id: String,
    #[serde(rename = "toolName")]
    tool_name: String,
    args: serde_json::Value,
  },
  #[serde(rename = "tool_execution_update")]
  ToolExecutionUpdate {
    #[serde(rename = "toolCallId")]
    tool_call_id: String,
    #[serde(rename = "toolName")]
    tool_name: String,
    args: serde_json::Value,
    #[serde(rename = "partialResult")]
    partial_result: serde_json::Value,
  },
  #[serde(rename = "tool_execution_end")]
  ToolExecutionEnd {
    #[serde(rename = "toolCallId")]
    tool_call_id: String,
    #[serde(rename = "toolName")]
    tool_name: String,
    result: serde_json::Value,
    #[serde(rename = "isError")]
    is_error: bool,
  },

  // -- From packages/coding-agent/src/core/agent-session.ts (AgentSessionEvent extensions) --
  #[serde(rename = "queue_update")]
  QueueUpdate {
    steering: Vec<String>,
    #[serde(rename = "followUp")]
    follow_up: Vec<String>,
  },
  #[serde(rename = "compaction_start")]
  CompactionStart { reason: CompactionReason },
  #[serde(rename = "session_info_changed")]
  SessionInfoChanged {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
  },
  #[serde(rename = "thinking_level_changed")]
  ThinkingLevelChanged { level: ThinkingLevel },
  #[serde(rename = "compaction_end")]
  CompactionEnd {
    reason: CompactionReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    result: Option<CompactionResult>,
    aborted: bool,
    #[serde(rename = "willRetry")]
    will_retry: bool,
    #[serde(
      rename = "errorMessage",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    error_message: Option<String>,
  },
  #[serde(rename = "auto_retry_start")]
  AutoRetryStart {
    attempt: f64,
    #[serde(rename = "maxAttempts")]
    max_attempts: f64,
    #[serde(rename = "delayMs")]
    delay_ms: f64,
    #[serde(rename = "errorMessage")]
    error_message: String,
  },
  #[serde(rename = "auto_retry_end")]
  AutoRetryEnd {
    success: bool,
    attempt: f64,
    #[serde(
      rename = "finalError",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    final_error: Option<String>,
  },

  // -- From packages/coding-agent/src/modes/rpc/rpc-mode.ts (untyped in TS, only exists on the wire) --
  #[serde(rename = "extension_error")]
  ExtensionError {
    #[serde(rename = "extensionPath")]
    extension_path: String,
    event: String,
    error: String,
  },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompactionReason {
  #[serde(rename = "manual")]
  Manual,
  #[serde(rename = "threshold")]
  Threshold,
  #[serde(rename = "overflow")]
  Overflow,
}
