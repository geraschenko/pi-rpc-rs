//! Public RPC command methods on PiSession.
//!
//! Each method constructs an `RpcCommandKind`, calls `send_command`, and
//! unpacks the typed response. See `docs/session-api.md` for the full API.

use std::path::PathBuf;
use std::time::Duration;

use super::error::PiError;
use super::session::PiSession;
use crate::types::*;

/// Longer timeout for compact.
const COMPACT_TIMEOUT: Duration = Duration::from_secs(300);

/// Helper to convert an error response into `PiError::CommandFailed`,
/// or return an unexpected-response error if the variant doesn't match.
macro_rules! match_response {
  ($response:expr, $pattern:pat => $result:expr) => {
    match $response.kind {
      $pattern => Ok($result),
      RpcResponseKind::Error { command, error } => Err(PiError::CommandFailed { command, error }),
      other => Err(PiError::CommandFailed {
        command: format!("{:?}", other),
        error: "unexpected response variant".into(),
      }),
    }
  };
}

impl PiSession {
  // ========================================================================
  // Prompting
  // ========================================================================

  /// Send a prompt message to the agent.
  ///
  /// If the agent is idle, starts a new agent run with this message.
  ///
  /// If the agent is already streaming, `streaming_behavior` is **required**:
  /// - `StreamingBehavior::Steer` — equivalent to [`steer()`](Self::steer):
  ///   interrupts after the current tool call, skipping remaining tool calls.
  /// - `StreamingBehavior::FollowUp` — equivalent to [`follow_up()`](Self::follow_up):
  ///   queued until the agent finishes all tool calls and steering messages.
  ///
  /// Returns an error if the agent is streaming and no `streaming_behavior` is specified.
  pub async fn prompt(
    &self,
    message: &str,
    images: Option<Vec<ContentBlock>>,
    streaming_behavior: Option<StreamingBehavior>,
  ) -> Result<(), PiError> {
    let resp = self
      .send_command(RpcCommandKind::Prompt {
        message: message.to_string(),
        images,
        streaming_behavior,
      })
      .await?;
    match_response!(resp, RpcResponseKind::Prompt => ())
  }

  /// Queue a steering message to interrupt the agent mid-run.
  ///
  /// The message is injected after the **current tool call** finishes,
  /// **skipping any remaining tool calls** in the turn. The agent then
  /// produces a new assistant response incorporating the steer message.
  ///
  /// For simple prompts with no tool use, steer behaves like
  /// [`follow_up()`](Self::follow_up) — the message is delivered after
  /// the current assistant response completes.
  pub async fn steer(
    &self,
    message: &str,
    images: Option<Vec<ContentBlock>>,
  ) -> Result<(), PiError> {
    let resp = self
      .send_command(RpcCommandKind::Steer {
        message: message.to_string(),
        images,
      })
      .await?;
    match_response!(resp, RpcResponseKind::Steer => ())
  }

  /// Queue a follow-up message for after the agent finishes.
  ///
  /// The message is delivered only when the agent has **no more tool calls
  /// and no pending steering messages**. This effectively extends the
  /// conversation: the agent would normally emit `agent_end`, but instead
  /// processes the follow-up as a new user turn within the same agent run.
  ///
  /// Contrast with [`steer()`](Self::steer), which interrupts between tool
  /// calls and skips remaining ones.
  pub async fn follow_up(
    &self,
    message: &str,
    images: Option<Vec<ContentBlock>>,
  ) -> Result<(), PiError> {
    let resp = self
      .send_command(RpcCommandKind::FollowUp {
        message: message.to_string(),
        images,
      })
      .await?;
    match_response!(resp, RpcResponseKind::FollowUp => ())
  }

  /// Abort the current agent operation immediately.
  ///
  /// Unlike [`steer()`](Self::steer), which waits for the current tool call
  /// to finish, abort cancels the in-flight API call or tool execution right
  /// away. The agent emits `agent_end` with `stop_reason: "aborted"`.
  pub async fn abort(&self) -> Result<(), PiError> {
    let resp = self.send_command(RpcCommandKind::Abort).await?;
    match_response!(resp, RpcResponseKind::Abort => ())
  }

  // ========================================================================
  // Session management
  // ========================================================================

  /// Start a new session, optionally with a parent session path.
  pub async fn new_session(
    &self,
    parent_session: Option<String>,
  ) -> Result<NewSessionData, PiError> {
    let resp = self
      .send_command(RpcCommandKind::NewSession { parent_session })
      .await?;
    match_response!(resp, RpcResponseKind::NewSession(data) => data)
  }

  /// Switch to an existing session by path.
  pub async fn switch_session(&self, session_path: &str) -> Result<SwitchSessionData, PiError> {
    let resp = self
      .send_command(RpcCommandKind::SwitchSession {
        session_path: session_path.to_string(),
      })
      .await?;
    match_response!(resp, RpcResponseKind::SwitchSession(data) => data)
  }

  /// Fork the session at the given entry ID.
  pub async fn fork(&self, entry_id: &str) -> Result<ForkData, PiError> {
    let resp = self
      .send_command(RpcCommandKind::Fork {
        entry_id: entry_id.to_string(),
      })
      .await?;
    match_response!(resp, RpcResponseKind::Fork(data) => data)
  }

  /// Clone the current active branch into a new session.
  ///
  /// This corresponds to pi's RPC command named `clone`, but the Rust method is
  /// named `clone_session` to avoid confusion with [`Clone::clone`].
  pub async fn clone_session(&self) -> Result<CloneData, PiError> {
    let resp = self.send_command(RpcCommandKind::Clone).await?;
    match_response!(resp, RpcResponseKind::Clone(data) => data)
  }

  /// Get messages that can be forked from.
  pub async fn get_fork_messages(&self) -> Result<GetForkMessagesData, PiError> {
    let resp = self.send_command(RpcCommandKind::GetForkMessages).await?;
    match_response!(resp, RpcResponseKind::GetForkMessages(data) => data)
  }

  /// Set the session name.
  pub async fn set_session_name(&self, name: &str) -> Result<(), PiError> {
    let resp = self
      .send_command(RpcCommandKind::SetSessionName {
        name: name.to_string(),
      })
      .await?;
    match_response!(resp, RpcResponseKind::SetSessionName => ())
  }

  // ========================================================================
  // State
  // ========================================================================

  /// Get the current session state.
  pub async fn get_state(&self) -> Result<RpcSessionState, PiError> {
    let resp = self.send_command(RpcCommandKind::GetState).await?;
    match_response!(resp, RpcResponseKind::GetState(data) => data)
  }

  /// Get all messages in the session.
  pub async fn get_messages(&self) -> Result<GetMessagesData, PiError> {
    let resp = self.send_command(RpcCommandKind::GetMessages).await?;
    match_response!(resp, RpcResponseKind::GetMessages(data) => data)
  }

  /// Get session statistics.
  pub async fn get_session_stats(&self) -> Result<SessionStats, PiError> {
    let resp = self.send_command(RpcCommandKind::GetSessionStats).await?;
    match_response!(resp, RpcResponseKind::GetSessionStats(data) => data)
  }

  /// Get the last assistant text.
  pub async fn get_last_assistant_text(&self) -> Result<GetLastAssistantTextData, PiError> {
    let resp = self
      .send_command(RpcCommandKind::GetLastAssistantText)
      .await?;
    match_response!(resp, RpcResponseKind::GetLastAssistantText(data) => data)
  }

  /// Export the session as HTML.
  pub async fn export_html(&self, output_path: Option<PathBuf>) -> Result<ExportHtmlData, PiError> {
    let resp = self
      .send_command(RpcCommandKind::ExportHtml {
        output_path: output_path.map(|p| p.to_string_lossy().into_owned()),
      })
      .await?;
    match_response!(resp, RpcResponseKind::ExportHtml(data) => data)
  }

  // ========================================================================
  // Model
  // ========================================================================

  /// Set the model by provider and model ID.
  pub async fn set_model(&self, provider: &str, model_id: &str) -> Result<Model, PiError> {
    let resp = self
      .send_command(RpcCommandKind::SetModel {
        provider: provider.to_string(),
        model_id: model_id.to_string(),
      })
      .await?;
    match_response!(resp, RpcResponseKind::SetModel(data) => data)
  }

  /// Cycle to the next model.
  pub async fn cycle_model(&self) -> Result<Option<CycleModelData>, PiError> {
    let resp = self.send_command(RpcCommandKind::CycleModel).await?;
    match_response!(resp, RpcResponseKind::CycleModel(data) => data)
  }

  /// Get all available models.
  pub async fn get_available_models(&self) -> Result<GetAvailableModelsData, PiError> {
    let resp = self
      .send_command(RpcCommandKind::GetAvailableModels)
      .await?;
    match_response!(resp, RpcResponseKind::GetAvailableModels(data) => data)
  }

  // ========================================================================
  // Thinking
  // ========================================================================

  /// Set the thinking level.
  pub async fn set_thinking_level(&self, level: ThinkingLevel) -> Result<(), PiError> {
    let resp = self
      .send_command(RpcCommandKind::SetThinkingLevel { level })
      .await?;
    match_response!(resp, RpcResponseKind::SetThinkingLevel => ())
  }

  /// Cycle to the next thinking level.
  pub async fn cycle_thinking_level(&self) -> Result<Option<CycleThinkingLevelData>, PiError> {
    let resp = self
      .send_command(RpcCommandKind::CycleThinkingLevel)
      .await?;
    match_response!(resp, RpcResponseKind::CycleThinkingLevel(data) => data)
  }

  // ========================================================================
  // Queue modes
  // ========================================================================

  /// Set the steering queue mode.
  pub async fn set_steering_mode(&self, mode: QueueMode) -> Result<(), PiError> {
    let resp = self
      .send_command(RpcCommandKind::SetSteeringMode { mode })
      .await?;
    match_response!(resp, RpcResponseKind::SetSteeringMode => ())
  }

  /// Set the follow-up queue mode.
  pub async fn set_follow_up_mode(&self, mode: QueueMode) -> Result<(), PiError> {
    let resp = self
      .send_command(RpcCommandKind::SetFollowUpMode { mode })
      .await?;
    match_response!(resp, RpcResponseKind::SetFollowUpMode => ())
  }

  // ========================================================================
  // Compaction
  // ========================================================================

  /// Compact the session, optionally with custom instructions.
  /// Uses a longer timeout since this involves an LLM call.
  pub async fn compact(
    &self,
    custom_instructions: Option<String>,
  ) -> Result<CompactionResult, PiError> {
    let resp = self
      .send_command_with_timeout(
        RpcCommandKind::Compact {
          custom_instructions,
        },
        COMPACT_TIMEOUT,
      )
      .await?;
    match_response!(resp, RpcResponseKind::Compact(data) => data)
  }

  /// Enable or disable auto-compaction.
  pub async fn set_auto_compaction(&self, enabled: bool) -> Result<(), PiError> {
    let resp = self
      .send_command(RpcCommandKind::SetAutoCompaction { enabled })
      .await?;
    match_response!(resp, RpcResponseKind::SetAutoCompaction => ())
  }

  // ========================================================================
  // Retry
  // ========================================================================

  /// Enable or disable auto-retry.
  pub async fn set_auto_retry(&self, enabled: bool) -> Result<(), PiError> {
    let resp = self
      .send_command(RpcCommandKind::SetAutoRetry { enabled })
      .await?;
    match_response!(resp, RpcResponseKind::SetAutoRetry => ())
  }

  /// Abort the current retry.
  pub async fn abort_retry(&self) -> Result<(), PiError> {
    let resp = self.send_command(RpcCommandKind::AbortRetry).await?;
    match_response!(resp, RpcResponseKind::AbortRetry => ())
  }

  // ========================================================================
  // Bash
  // ========================================================================

  /// Execute a bash command.
  pub async fn bash(&self, command: &str) -> Result<BashResult, PiError> {
    let resp = self
      .send_command(RpcCommandKind::Bash {
        command: command.to_string(),
      })
      .await?;
    match_response!(resp, RpcResponseKind::Bash(data) => data)
  }

  /// Abort the current bash command.
  pub async fn abort_bash(&self) -> Result<(), PiError> {
    let resp = self.send_command(RpcCommandKind::AbortBash).await?;
    match_response!(resp, RpcResponseKind::AbortBash => ())
  }

  // ========================================================================
  // Commands
  // ========================================================================

  /// Get available slash commands.
  pub async fn get_commands(&self) -> Result<GetCommandsData, PiError> {
    let resp = self.send_command(RpcCommandKind::GetCommands).await?;
    match_response!(resp, RpcResponseKind::GetCommands(data) => data)
  }

  // ========================================================================
  // Extension UI
  // ========================================================================

  /// Send an extension UI response.
  ///
  /// This is NOT an RPC command — it writes an `RpcExtensionUIResponse`
  /// directly to the pi process stdin.
  pub async fn respond_extension_ui(
    &self,
    response: RpcExtensionUIResponse,
  ) -> Result<(), PiError> {
    let json = serde_json::to_string(&response)?;
    self.write_json_line(&json).await
  }
}
