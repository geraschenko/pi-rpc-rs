//! RPC command and response types.
//!
//! Mirrors packages/coding-agent/src/modes/rpc/rpc-types.ts.
//! Types are ordered to match the TypeScript source — a developer can open both
//! files and scroll them together to see how they correspond.

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

use super::agent::*;
use super::agent_session::*;
use super::ai::*;
use super::bash_executor::*;
use super::compaction::*;
use super::source_info::*;

fn is_false(value: &bool) -> bool {
  !*value
}

// ============================================================================
// Small enums for limited-option string fields
// ============================================================================

/// How to queue a prompt when the agent is already streaming.
///
/// See [`PiSession::prompt`](crate::session::PiSession::prompt) for details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StreamingBehavior {
  /// Interrupt after current tool call, skip remaining tools.
  /// Same as calling [`PiSession::steer`](crate::session::PiSession::steer).
  #[serde(rename = "steer")]
  Steer,
  /// Queue until agent finishes all tool calls and steering messages.
  /// Same as calling [`PiSession::follow_up`](crate::session::PiSession::follow_up).
  #[serde(rename = "followUp")]
  FollowUp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueueMode {
  #[serde(rename = "all")]
  All,
  #[serde(rename = "one-at-a-time")]
  OneAtATime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlashCommandSource {
  #[serde(rename = "extension")]
  Extension,
  #[serde(rename = "prompt")]
  Prompt,
  #[serde(rename = "skill")]
  Skill,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotifyType {
  #[serde(rename = "info")]
  Info,
  #[serde(rename = "warning")]
  Warning,
  #[serde(rename = "error")]
  Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WidgetPlacement {
  #[serde(rename = "aboveEditor")]
  AboveEditor,
  #[serde(rename = "belowEditor")]
  BelowEditor,
}

// ============================================================================
// RpcCommand (stdin → pi)
// ============================================================================

/// An RPC command to send to pi.
///
/// Serializes to `{"id": "...", "type": "<command>", ...fields}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcCommand {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(flatten)]
  pub kind: RpcCommandKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(tag = "type", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RpcCommandKind {
  // -- Prompting --
  Prompt {
    message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    images: Option<Vec<ContentBlock>>, // Image variants only
    #[serde(
      rename = "streamingBehavior",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    streaming_behavior: Option<StreamingBehavior>,
  },
  Steer {
    message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    images: Option<Vec<ContentBlock>>,
  },
  FollowUp {
    message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    images: Option<Vec<ContentBlock>>,
  },
  Abort,
  NewSession {
    #[serde(
      rename = "parentSession",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    parent_session: Option<String>,
  },

  // -- State --
  GetState,

  // -- Model --
  SetModel {
    provider: String,
    #[serde(rename = "modelId")]
    model_id: String,
  },
  CycleModel,
  GetAvailableModels,

  // -- Thinking --
  SetThinkingLevel {
    level: ThinkingLevel,
  },
  CycleThinkingLevel,

  // -- Queue modes --
  SetSteeringMode {
    mode: QueueMode,
  },
  SetFollowUpMode {
    mode: QueueMode,
  },

  // -- Compaction --
  Compact {
    #[serde(
      rename = "customInstructions",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    custom_instructions: Option<String>,
  },
  SetAutoCompaction {
    enabled: bool,
  },

  // -- Retry --
  SetAutoRetry {
    enabled: bool,
  },
  AbortRetry,

  // -- Bash --
  Bash {
    command: String,
    #[serde(
      rename = "excludeFromContext",
      default,
      skip_serializing_if = "is_false"
    )]
    exclude_from_context: bool,
  },
  AbortBash,

  // -- Session --
  GetSessionStats,
  ExportHtml {
    #[serde(
      rename = "outputPath",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    output_path: Option<String>,
  },
  SwitchSession {
    #[serde(rename = "sessionPath")]
    session_path: String,
  },
  Fork {
    #[serde(rename = "entryId")]
    entry_id: String,
  },
  Clone,
  GetForkMessages,
  GetLastAssistantText,
  SetSessionName {
    name: String,
  },

  // -- Messages --
  GetMessages,

  // -- Commands --
  GetCommands,
}

// ============================================================================
// RpcSlashCommand
// ============================================================================

/// A slash command available for invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcSlashCommand {
  pub name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub source: SlashCommandSource,
  #[serde(rename = "sourceInfo")]
  pub source_info: SourceInfo,
}

// ============================================================================
// RpcSessionState
// ============================================================================

/// Current session state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcSessionState {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub model: Option<Model>,
  #[serde(rename = "thinkingLevel")]
  pub thinking_level: ThinkingLevel,
  #[serde(rename = "isStreaming")]
  pub is_streaming: bool,
  #[serde(rename = "isCompacting")]
  pub is_compacting: bool,
  #[serde(rename = "steeringMode")]
  pub steering_mode: QueueMode,
  #[serde(rename = "followUpMode")]
  pub follow_up_mode: QueueMode,
  #[serde(
    rename = "sessionFile",
    default,
    skip_serializing_if = "Option::is_none"
  )]
  pub session_file: Option<String>,
  #[serde(rename = "sessionId")]
  pub session_id: String,
  #[serde(
    rename = "sessionName",
    default,
    skip_serializing_if = "Option::is_none"
  )]
  pub session_name: Option<String>,
  #[serde(rename = "autoCompactionEnabled")]
  pub auto_compaction_enabled: bool,
  #[serde(rename = "messageCount")]
  pub message_count: f64,
  #[serde(rename = "pendingMessageCount")]
  pub pending_message_count: f64,
}

// ============================================================================
// RpcResponse (pi → stdout)
// ============================================================================

/// A response from pi to an RPC command.
///
/// On the wire: `{"type": "response", "id": "...", "command": "<cmd>", "success": true/false, ...}`.
/// We drop `type` (always "response") and collapse `success`+`data`/`error` into the kind enum.
#[derive(Debug, Clone, PartialEq)]
pub struct RpcResponse {
  pub id: Option<String>,
  pub kind: RpcResponseKind,
}

/// Response variants for each command. Success responses carry data (if any)
/// directly; the `Error` variant handles failures for any command.
#[derive(Debug, Clone, PartialEq)]
pub enum RpcResponseKind {
  // -- No-data success responses --
  Prompt,
  Steer,
  FollowUp,
  Abort,
  SetThinkingLevel,
  SetSteeringMode,
  SetFollowUpMode,
  SetAutoCompaction,
  SetAutoRetry,
  AbortRetry,
  AbortBash,
  SetSessionName,

  // -- Success responses with data --
  NewSession(NewSessionData),
  GetState(RpcSessionState),
  SetModel(Model),
  CycleModel(Option<CycleModelData>),
  GetAvailableModels(GetAvailableModelsData),
  CycleThinkingLevel(Option<CycleThinkingLevelData>),
  Compact(CompactionResult),
  Bash(BashResult),
  GetSessionStats(SessionStats),
  ExportHtml(ExportHtmlData),
  SwitchSession(SwitchSessionData),
  Fork(ForkData),
  Clone(CloneData),
  GetForkMessages(GetForkMessagesData),
  GetLastAssistantText(GetLastAssistantTextData),
  GetMessages(GetMessagesData),
  GetCommands(GetCommandsData),

  // -- Error (any command can fail) --
  Error { command: String, error: String },
}

// -- Response data structs --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewSessionData {
  pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CycleModelData {
  pub model: Model,
  #[serde(rename = "thinkingLevel")]
  pub thinking_level: ThinkingLevel,
  #[serde(rename = "isScoped")]
  pub is_scoped: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetAvailableModelsData {
  pub models: Vec<Model>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CycleThinkingLevelData {
  pub level: ThinkingLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportHtmlData {
  pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchSessionData {
  pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForkData {
  pub text: String,
  pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloneData {
  pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetForkMessagesData {
  pub messages: Vec<ForkableMessage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForkableMessage {
  #[serde(rename = "entryId")]
  pub entry_id: String,
  pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetLastAssistantTextData {
  pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetMessagesData {
  pub messages: Vec<AgentMessage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCommandsData {
  pub commands: Vec<RpcSlashCommand>,
}

// -- Custom Deserialize for RpcResponse --

impl<'de> Deserialize<'de> for RpcResponse {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    use serde::de::Error;

    let value = serde_json::Value::deserialize(deserializer)?;
    let obj = value
      .as_object()
      .ok_or_else(|| D::Error::custom("expected object"))?;

    let id = obj.get("id").and_then(|v| v.as_str()).map(String::from);
    let command = obj
      .get("command")
      .and_then(|v| v.as_str())
      .ok_or_else(|| D::Error::missing_field("command"))?;
    let success = obj
      .get("success")
      .and_then(|v| v.as_bool())
      .ok_or_else(|| D::Error::missing_field("success"))?;

    let kind = if !success {
      let error = obj
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
      RpcResponseKind::Error {
        command: command.to_string(),
        error,
      }
    } else {
      match command {
        // No-data responses
        "prompt" => RpcResponseKind::Prompt,
        "steer" => RpcResponseKind::Steer,
        "follow_up" => RpcResponseKind::FollowUp,
        "abort" => RpcResponseKind::Abort,
        "set_thinking_level" => RpcResponseKind::SetThinkingLevel,
        "set_steering_mode" => RpcResponseKind::SetSteeringMode,
        "set_follow_up_mode" => RpcResponseKind::SetFollowUpMode,
        "set_auto_compaction" => RpcResponseKind::SetAutoCompaction,
        "set_auto_retry" => RpcResponseKind::SetAutoRetry,
        "abort_retry" => RpcResponseKind::AbortRetry,
        "abort_bash" => RpcResponseKind::AbortBash,
        "set_session_name" => RpcResponseKind::SetSessionName,

        // Responses with data
        "new_session" => RpcResponseKind::NewSession(data_field(obj)?),
        "get_state" => RpcResponseKind::GetState(data_field(obj)?),
        "set_model" => RpcResponseKind::SetModel(data_field(obj)?),
        "cycle_model" => RpcResponseKind::CycleModel(nullable_data_field(obj)?),
        "get_available_models" => RpcResponseKind::GetAvailableModels(data_field(obj)?),
        "cycle_thinking_level" => RpcResponseKind::CycleThinkingLevel(nullable_data_field(obj)?),
        "compact" => RpcResponseKind::Compact(data_field(obj)?),
        "bash" => RpcResponseKind::Bash(data_field(obj)?),
        "get_session_stats" => RpcResponseKind::GetSessionStats(data_field(obj)?),
        "export_html" => RpcResponseKind::ExportHtml(data_field(obj)?),
        "switch_session" => RpcResponseKind::SwitchSession(data_field(obj)?),
        "fork" => RpcResponseKind::Fork(data_field(obj)?),
        "clone" => RpcResponseKind::Clone(data_field(obj)?),
        "get_fork_messages" => RpcResponseKind::GetForkMessages(data_field(obj)?),
        "get_last_assistant_text" => RpcResponseKind::GetLastAssistantText(data_field(obj)?),
        "get_messages" => RpcResponseKind::GetMessages(data_field(obj)?),
        "get_commands" => RpcResponseKind::GetCommands(data_field(obj)?),

        other => return Err(D::Error::unknown_variant(other, COMMAND_NAMES)),
      }
    };

    Ok(RpcResponse { id, kind })
  }
}

/// Extract and deserialize the `data` field from a response object.
fn data_field<T, E>(obj: &serde_json::Map<String, serde_json::Value>) -> Result<T, E>
where
  T: serde::de::DeserializeOwned,
  E: serde::de::Error,
{
  let data = obj
    .get("data")
    .ok_or_else(|| E::missing_field("data"))?
    .clone();
  serde_json::from_value(data).map_err(E::custom)
}

/// Extract and deserialize the `data` field, which may be null.
fn nullable_data_field<T, E>(
  obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<Option<T>, E>
where
  T: serde::de::DeserializeOwned,
  E: serde::de::Error,
{
  match obj.get("data") {
    None | Some(serde_json::Value::Null) => Ok(None),
    Some(v) => serde_json::from_value(v.clone())
      .map(Some)
      .map_err(E::custom),
  }
}

const COMMAND_NAMES: &[&str] = &[
  "prompt",
  "steer",
  "follow_up",
  "abort",
  "new_session",
  "get_state",
  "set_model",
  "cycle_model",
  "get_available_models",
  "set_thinking_level",
  "cycle_thinking_level",
  "set_steering_mode",
  "set_follow_up_mode",
  "compact",
  "set_auto_compaction",
  "set_auto_retry",
  "abort_retry",
  "bash",
  "abort_bash",
  "get_session_stats",
  "export_html",
  "switch_session",
  "fork",
  "clone",
  "get_fork_messages",
  "get_last_assistant_text",
  "set_session_name",
  "get_messages",
  "get_commands",
];

// -- Custom Serialize for RpcResponse --

impl Serialize for RpcResponse {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;

    let mut map = serializer.serialize_map(None)?;

    map.serialize_entry("type", "response")?;

    if let Some(id) = &self.id {
      map.serialize_entry("id", id)?;
    }

    match &self.kind {
      // No-data success responses
      RpcResponseKind::Prompt => serialize_success(&mut map, "prompt", None::<&()>),
      RpcResponseKind::Steer => serialize_success(&mut map, "steer", None::<&()>),
      RpcResponseKind::FollowUp => serialize_success(&mut map, "follow_up", None::<&()>),
      RpcResponseKind::Abort => serialize_success(&mut map, "abort", None::<&()>),
      RpcResponseKind::SetThinkingLevel => {
        serialize_success(&mut map, "set_thinking_level", None::<&()>)
      }
      RpcResponseKind::SetSteeringMode => {
        serialize_success(&mut map, "set_steering_mode", None::<&()>)
      }
      RpcResponseKind::SetFollowUpMode => {
        serialize_success(&mut map, "set_follow_up_mode", None::<&()>)
      }
      RpcResponseKind::SetAutoCompaction => {
        serialize_success(&mut map, "set_auto_compaction", None::<&()>)
      }
      RpcResponseKind::SetAutoRetry => serialize_success(&mut map, "set_auto_retry", None::<&()>),
      RpcResponseKind::AbortRetry => serialize_success(&mut map, "abort_retry", None::<&()>),
      RpcResponseKind::AbortBash => serialize_success(&mut map, "abort_bash", None::<&()>),
      RpcResponseKind::SetSessionName => {
        serialize_success(&mut map, "set_session_name", None::<&()>)
      }

      // Success responses with data
      RpcResponseKind::NewSession(d) => serialize_success(&mut map, "new_session", Some(d)),
      RpcResponseKind::GetState(d) => serialize_success(&mut map, "get_state", Some(d)),
      RpcResponseKind::SetModel(d) => serialize_success(&mut map, "set_model", Some(d)),
      RpcResponseKind::CycleModel(d) => serialize_success(&mut map, "cycle_model", Some(d)),
      RpcResponseKind::GetAvailableModels(d) => {
        serialize_success(&mut map, "get_available_models", Some(d))
      }
      RpcResponseKind::CycleThinkingLevel(d) => {
        serialize_success(&mut map, "cycle_thinking_level", Some(d))
      }
      RpcResponseKind::Compact(d) => serialize_success(&mut map, "compact", Some(d)),
      RpcResponseKind::Bash(d) => serialize_success(&mut map, "bash", Some(d)),
      RpcResponseKind::GetSessionStats(d) => {
        serialize_success(&mut map, "get_session_stats", Some(d))
      }
      RpcResponseKind::ExportHtml(d) => serialize_success(&mut map, "export_html", Some(d)),
      RpcResponseKind::SwitchSession(d) => serialize_success(&mut map, "switch_session", Some(d)),
      RpcResponseKind::Fork(d) => serialize_success(&mut map, "fork", Some(d)),
      RpcResponseKind::Clone(d) => serialize_success(&mut map, "clone", Some(d)),
      RpcResponseKind::GetForkMessages(d) => {
        serialize_success(&mut map, "get_fork_messages", Some(d))
      }
      RpcResponseKind::GetLastAssistantText(d) => {
        serialize_success(&mut map, "get_last_assistant_text", Some(d))
      }
      RpcResponseKind::GetMessages(d) => serialize_success(&mut map, "get_messages", Some(d)),
      RpcResponseKind::GetCommands(d) => serialize_success(&mut map, "get_commands", Some(d)),

      // Error
      RpcResponseKind::Error { command, error } => {
        map.serialize_entry("command", command)?;
        map.serialize_entry("success", &false)?;
        map.serialize_entry("error", error)?;
        Ok(())
      }
    }?;

    map.end()
  }
}

fn serialize_success<S, T>(map: &mut S, command: &str, data: Option<&T>) -> Result<(), S::Error>
where
  S: serde::ser::SerializeMap,
  T: Serialize,
{
  map.serialize_entry("command", command)?;
  map.serialize_entry("success", &true)?;
  if let Some(data) = data {
    map.serialize_entry("data", data)?;
  }
  Ok(())
}

// ============================================================================
// RpcEvent (unified event type for subscribers)
// ============================================================================

/// Event received from pi's stdout (anything that isn't a response).
///
/// Subscribers receive these via `PiSession::subscribe()`.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum RpcEvent {
  Agent(AgentEvent),
  ExtensionUI(RpcExtensionUIRequest),
  Session(SessionEvent),
}

/// Events emitted by the Rust session wrapper itself, rather than by pi's
/// RPC stdout protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionEvent {
  /// The pi process stdout closed, indicating that the process exited.
  #[serde(rename = "session_process_exited")]
  ProcessExited { code: Option<i32>, stderr: String },
  /// A line from pi's stdout could not be parsed as the expected RPC message.
  #[serde(rename = "session_deserialization_error")]
  DeserializationError {
    context: DeserializationErrorContext,
    error: JsonErrorInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    line: Option<String>,
  },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeserializationErrorContext {
  JsonLine,
  RpcResponse,
  RpcExtensionUIRequest,
  AgentEvent,
}

/// Serializable summary of a serde_json error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonErrorInfo {
  pub message: String,
  pub line: usize,
  pub column: usize,
  pub category: String,
}

impl From<&serde_json::Error> for JsonErrorInfo {
  fn from(error: &serde_json::Error) -> Self {
    JsonErrorInfo {
      message: error.to_string(),
      line: error.line(),
      column: error.column(),
      category: format!("{:?}", error.classify()),
    }
  }
}

impl Serialize for RpcEvent {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      RpcEvent::Agent(event) => event.serialize(serializer),
      RpcEvent::ExtensionUI(req) => req.serialize(serializer),
      RpcEvent::Session(event) => event.serialize(serializer),
    }
  }
}

impl<'de> Deserialize<'de> for RpcEvent {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    let type_str = value.get("type").and_then(|v| v.as_str()).unwrap_or("");

    if type_str == "extension_ui_request" {
      let req: RpcExtensionUIRequest =
        serde_json::from_value(value).map_err(serde::de::Error::custom)?;
      Ok(RpcEvent::ExtensionUI(req))
    } else if type_str.starts_with("session_") {
      let event: SessionEvent = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
      Ok(RpcEvent::Session(event))
    } else {
      let event: AgentEvent = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
      Ok(RpcEvent::Agent(event))
    }
  }
}

// ============================================================================
// RpcExtensionUIRequest (pi → stdout)
// ============================================================================

/// Extension UI request (pi → stdout, needs response).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcExtensionUIRequest {
  pub id: String,
  #[serde(flatten)]
  pub kind: RpcExtensionUIRequestKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum RpcExtensionUIRequestKind {
  #[serde(rename = "select")]
  Select {
    title: String,
    options: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timeout: Option<f64>,
  },
  #[serde(rename = "confirm")]
  Confirm {
    title: String,
    message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timeout: Option<f64>,
  },
  #[serde(rename = "input")]
  Input {
    title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timeout: Option<f64>,
  },
  #[serde(rename = "editor")]
  Editor {
    title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prefill: Option<String>,
  },
  #[serde(rename = "notify")]
  Notify {
    message: String,
    #[serde(
      rename = "notifyType",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    notify_type: Option<NotifyType>,
  },
  #[serde(rename = "setStatus")]
  SetStatus {
    #[serde(rename = "statusKey")]
    status_key: String,
    #[serde(rename = "statusText")]
    status_text: Option<String>,
  },
  #[serde(rename = "setWidget")]
  SetWidget {
    #[serde(rename = "widgetKey")]
    widget_key: String,
    #[serde(rename = "widgetLines")]
    widget_lines: Option<Vec<String>>,
    #[serde(
      rename = "widgetPlacement",
      default,
      skip_serializing_if = "Option::is_none"
    )]
    widget_placement: Option<WidgetPlacement>,
  },
  #[serde(rename = "setTitle")]
  SetTitle { title: String },
  #[serde(rename = "set_editor_text")]
  SetEditorText { text: String },
}

// ============================================================================
// RpcExtensionUIResponse (stdin → pi)
// ============================================================================

/// Response to an extension UI request (stdin → pi).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcExtensionUIResponse {
  Value {
    #[serde(rename = "type")]
    type_: ExtensionUIResponseType,
    id: String,
    value: String,
  },
  Confirmed {
    #[serde(rename = "type")]
    type_: ExtensionUIResponseType,
    id: String,
    confirmed: bool,
  },
  Cancelled {
    #[serde(rename = "type")]
    type_: ExtensionUIResponseType,
    id: String,
    cancelled: bool, // always true
  },
}

/// Helper — always serializes to "extension_ui_response".
#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionUIResponseType;

impl Serialize for ExtensionUIResponseType {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str("extension_ui_response")
  }
}

impl<'de> Deserialize<'de> for ExtensionUIResponseType {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let s = String::deserialize(deserializer)?;
    if s == "extension_ui_response" {
      Ok(ExtensionUIResponseType)
    } else {
      Err(serde::de::Error::invalid_value(
        serde::de::Unexpected::Str(&s),
        &"extension_ui_response",
      ))
    }
  }
}
