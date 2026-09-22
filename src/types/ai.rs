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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
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
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub reasoning: Option<f64>,
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
  Pending,
  Stop,
  Length,
  ToolUse,
  Error,
  Aborted,
  Deferred,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeferredHandle {
  pub provider: String,
  pub model_id: String,
  pub api: String,
  pub id: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub expires_at: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub poll_after_ms: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub data: Option<serde_json::Value>,
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

/// Instruction text carried by a system message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemContent {
  Text(String),
  Blocks(Vec<ContentBlock>),
}

/// Provider-visible tool declaration, without an executable implementation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
  pub name: String,
  pub description: String,
  pub parameters: serde_json::Value,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub constrained_sampling: Option<ConstrainedSampling>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConstrainedSampling {
  Disabled(False),
  Config(ConstrainedSamplingConfig),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "bool", into = "bool")]
pub struct False;

impl TryFrom<bool> for False {
  type Error = &'static str;
  fn try_from(value: bool) -> Result<Self, Self::Error> {
    if value {
      Err("expected false")
    } else {
      Ok(Self)
    }
  }
}

impl From<False> for bool {
  fn from(_: False) -> Self {
    false
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConstrainedSamplingConfig {
  JsonSchema {
    strict: StrictMode,
  },
  Grammar {
    variants: HashMap<GrammarFormat, String>,
  },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrictMode {
  Prefer,
  Require,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrammarFormat {
  OpenaiLark,
  OpenaiRegex,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolReference {
  pub name: String,
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
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub input_limits: Option<ModelInputLimits>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub prompt_cache: Option<ModelPromptCache>,
  pub cost: ModelCost,
  pub context_window: f64,
  pub max_tokens: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub sampling_params: Option<HashMap<String, serde_json::Value>>,
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
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub tiers: Option<Vec<ModelCostTier>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCostTier {
  pub input: f64,
  pub output: f64,
  pub cache_read: f64,
  pub cache_write: f64,
  pub input_tokens_above: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPromptCache {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub short: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub long: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInputLimits {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_request_bytes: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub images: Option<ModelImageInputLimits>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelImageInputLimits {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub resize: Option<ModelImageResizeOptions>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_per_message: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_per_request: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelImageResizeOptions {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_width: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_height: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_bytes: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub jpeg_quality: Option<f64>,
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
  Start,
  TextStart {
    content_index: f64,
  },
  TextDelta {
    content_index: f64,
    delta: String,
  },
  TextEnd {
    content_index: f64,
    content: String,
  },
  ThinkingStart {
    content_index: f64,
  },
  ThinkingDelta {
    content_index: f64,
    delta: String,
  },
  ThinkingEnd {
    content_index: f64,
    content: String,
  },
  ToolcallStart {
    content_index: f64,
    id: String,
    tool_name: String,
  },
  ToolcallDelta {
    content_index: f64,
    delta: String,
  },
  ToolcallEnd {
    content_index: f64,
    tool_call: ContentBlock, // always the ToolCall variant
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
