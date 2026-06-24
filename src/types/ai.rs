//! Types from `packages/ai/src/types.ts`.
//!
//! Content blocks, messages, usage, models, and streaming events.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{AsRefStr, Display};

// ============================================================================
// Content blocks
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextSignatureV1 {
  pub v: u8,
  pub id: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub phase: Option<TextSignaturePhase>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TextSignaturePhase {
  Commentary,
  FinalAnswer,
}

/// Legacy text signature string or structured V1 metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextSignature {
  String(String),
  V1(TextSignatureV1),
}

/// Union of TextContent | ThinkingContent | ImageContent | ToolCall.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(
  tag = "type",
  rename_all = "camelCase",
  rename_all_fields = "camelCase"
)]
#[strum(serialize_all = "camelCase")]
pub enum ContentBlock {
  Text {
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text_signature: Option<TextSignature>,
  },
  Thinking {
    thinking: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    thinking_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    redacted: Option<bool>,
  },
  Image {
    data: String,
    mime_type: String,
  },
  ToolCall {
    id: String,
    name: String,
    arguments: HashMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    thought_signature: Option<String>,
  },
}

// ============================================================================
// Usage
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
  pub input: f64,
  pub output: f64,
  pub cache_read: f64,
  pub cache_write: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cache_write1h: Option<f64>,
  pub total_tokens: f64,
  pub cost: UsageCost,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageCost {
  pub input: f64,
  pub output: f64,
  pub cache_read: f64,
  pub cache_write: f64,
  pub total: f64,
}

// ============================================================================
// StopReason
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum StopReason {
  Stop,
  Length,
  ToolUse,
  Error,
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

/// Model definition. The generic `TApi` from TypeScript is erased — on the
/// wire RPC exposes this as `Model<any>`.
///
/// The `compat` field stores provider/API-specific compatibility flags. Its
/// TypeScript shape is conditional on `api`, so Rust preserves it as JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
  pub id: String,
  pub name: String,
  pub api: String,
  pub provider: String,
  pub base_url: String,
  pub reasoning: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub thinking_level_map: Option<HashMap<crate::types::ThinkingLevel, Option<String>>>,
  pub input: Vec<String>,
  pub cost: ModelCost,
  pub context_window: f64,
  pub max_tokens: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub headers: Option<HashMap<String, String>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub compat: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCost {
  pub input: f64,
  pub output: f64,
  pub cache_read: f64,
  pub cache_write: f64,
}

// ============================================================================
// Assistant message diagnostics
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticErrorInfo {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  pub message: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub stack: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub code: Option<DiagnosticErrorCode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiagnosticErrorCode {
  String(String),
  Number(f64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantMessageDiagnostic {
  #[serde(rename = "type")]
  pub type_: String,
  pub timestamp: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub error: Option<DiagnosticErrorInfo>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub details: Option<HashMap<String, serde_json::Value>>,
}

// ============================================================================
// AssistantMessageEvent
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(
  tag = "type",
  rename_all = "snake_case",
  rename_all_fields = "camelCase"
)]
#[strum(serialize_all = "snake_case")]
pub enum AssistantMessageEvent {
  Start {
    partial: Box<serde_json::Value>,
  },
  TextStart {
    content_index: f64,
    partial: Box<serde_json::Value>,
  },
  TextDelta {
    content_index: f64,
    delta: String,
    partial: Box<serde_json::Value>,
  },
  TextEnd {
    content_index: f64,
    content: String,
    partial: Box<serde_json::Value>,
  },
  ThinkingStart {
    content_index: f64,
    partial: Box<serde_json::Value>,
  },
  ThinkingDelta {
    content_index: f64,
    delta: String,
    partial: Box<serde_json::Value>,
  },
  ThinkingEnd {
    content_index: f64,
    content: String,
    partial: Box<serde_json::Value>,
  },
  ToolcallStart {
    content_index: f64,
    partial: Box<serde_json::Value>,
  },
  ToolcallDelta {
    content_index: f64,
    delta: String,
    partial: Box<serde_json::Value>,
  },
  ToolcallEnd {
    content_index: f64,
    tool_call: ContentBlock, // always the ToolCall variant
    partial: Box<serde_json::Value>,
  },
  Done {
    reason: StopReason,
    message: Box<serde_json::Value>,
  },
  Error {
    reason: StopReason,
    error: Box<serde_json::Value>,
  },
}
