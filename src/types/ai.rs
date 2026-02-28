//! Types from `packages/ai/src/types.ts`.
//!
//! Content blocks, messages, usage, models, and streaming events.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Content blocks
// ============================================================================

/// Union of TextContent | ThinkingContent | ImageContent | ToolCall.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
  #[serde(rename = "text")]
  Text {
    text: String,
    #[serde(
      rename = "textSignature",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    text_signature: Option<String>,
  },
  #[serde(rename = "thinking")]
  Thinking {
    thinking: String,
    #[serde(
      rename = "thinkingSignature",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    thinking_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    redacted: Option<bool>,
  },
  #[serde(rename = "image")]
  Image {
    data: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
  },
  #[serde(rename = "toolCall")]
  ToolCall {
    id: String,
    name: String,
    arguments: HashMap<String, serde_json::Value>,
    #[serde(
      rename = "thoughtSignature",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    thought_signature: Option<String>,
  },
}

// ============================================================================
// Usage
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Usage {
  pub input: f64,
  pub output: f64,
  #[serde(rename = "cacheRead")]
  pub cache_read: f64,
  #[serde(rename = "cacheWrite")]
  pub cache_write: f64,
  #[serde(rename = "totalTokens")]
  pub total_tokens: f64,
  pub cost: UsageCost,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageCost {
  pub input: f64,
  pub output: f64,
  #[serde(rename = "cacheRead")]
  pub cache_read: f64,
  #[serde(rename = "cacheWrite")]
  pub cache_write: f64,
  pub total: f64,
}

// ============================================================================
// StopReason
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StopReason {
  #[serde(rename = "stop")]
  Stop,
  #[serde(rename = "length")]
  Length,
  #[serde(rename = "toolUse")]
  ToolUse,
  #[serde(rename = "error")]
  Error,
  #[serde(rename = "aborted")]
  Aborted,
}

// ============================================================================
// Messages (UserMessage, AssistantMessage, ToolResultMessage)
// ============================================================================

/// UserMessage content: either a plain string or an array of content blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserContent {
  Text(String),
  Blocks(Vec<ContentBlock>),
}

// ============================================================================
// Model
// ============================================================================

/// Model definition. The generic `TApi` and conditional `compat` field from
/// TypeScript are erased — on the wire it's always `Model<any>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Model {
  pub id: String,
  pub name: String,
  pub api: String,
  pub provider: String,
  #[serde(rename = "baseUrl")]
  pub base_url: String,
  pub reasoning: bool,
  pub input: Vec<String>,
  pub cost: ModelCost,
  #[serde(rename = "contextWindow")]
  pub context_window: f64,
  #[serde(rename = "maxTokens")]
  pub max_tokens: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelCost {
  pub input: f64,
  pub output: f64,
  #[serde(rename = "cacheRead")]
  pub cache_read: f64,
  #[serde(rename = "cacheWrite")]
  pub cache_write: f64,
}

// ============================================================================
// AssistantMessageEvent
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssistantMessageEvent {
  #[serde(rename = "start")]
  Start { partial: Box<serde_json::Value> },
  #[serde(rename = "text_start")]
  TextStart {
    #[serde(rename = "contentIndex")]
    content_index: f64,
    partial: Box<serde_json::Value>,
  },
  #[serde(rename = "text_delta")]
  TextDelta {
    #[serde(rename = "contentIndex")]
    content_index: f64,
    delta: String,
    partial: Box<serde_json::Value>,
  },
  #[serde(rename = "text_end")]
  TextEnd {
    #[serde(rename = "contentIndex")]
    content_index: f64,
    content: String,
    partial: Box<serde_json::Value>,
  },
  #[serde(rename = "thinking_start")]
  ThinkingStart {
    #[serde(rename = "contentIndex")]
    content_index: f64,
    partial: Box<serde_json::Value>,
  },
  #[serde(rename = "thinking_delta")]
  ThinkingDelta {
    #[serde(rename = "contentIndex")]
    content_index: f64,
    delta: String,
    partial: Box<serde_json::Value>,
  },
  #[serde(rename = "thinking_end")]
  ThinkingEnd {
    #[serde(rename = "contentIndex")]
    content_index: f64,
    content: String,
    partial: Box<serde_json::Value>,
  },
  #[serde(rename = "toolcall_start")]
  ToolcallStart {
    #[serde(rename = "contentIndex")]
    content_index: f64,
    partial: Box<serde_json::Value>,
  },
  #[serde(rename = "toolcall_delta")]
  ToolcallDelta {
    #[serde(rename = "contentIndex")]
    content_index: f64,
    delta: String,
    partial: Box<serde_json::Value>,
  },
  #[serde(rename = "toolcall_end")]
  ToolcallEnd {
    #[serde(rename = "contentIndex")]
    content_index: f64,
    #[serde(rename = "toolCall")]
    tool_call: ContentBlock, // always the ToolCall variant
    partial: Box<serde_json::Value>,
  },
  #[serde(rename = "done")]
  Done {
    reason: StopReason,
    message: Box<serde_json::Value>,
  },
  #[serde(rename = "error")]
  Error {
    reason: StopReason,
    error: Box<serde_json::Value>,
  },
}
