//! PiSession — manages a `pi --mode rpc` child process.
//!
//! See `docs/session-api.md` for the full API design.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, ChildStderr, ChildStdout, Command};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::error::PiError;
use crate::COMPATIBLE_PI_VERSION;
use crate::types::{
  DeserializationErrorContext, JsonErrorInfo, RpcCommand, RpcCommandKind, RpcEvent,
  RpcExtensionUIRequest, RpcResponse, SessionEvent,
};

// ============================================================================
// Configuration
// ============================================================================

/// How session persistence is handled.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SessionPersistence {
  /// Default pi behavior (sessions saved to ~/.pi/agent/sessions/).
  #[default]
  Enabled,
  /// Pass --no-session flag.
  Disabled,
  /// Pass --session-dir with the given path.
  CustomDir(PathBuf),
}

/// How `PiSession::spawn` should handle installed pi version mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PiVersionCheck {
  Disabled,
  #[default]
  Warn,
  Error,
}

/// Configuration for spawning a pi session.
#[derive(Debug, Clone)]
pub struct PiSessionConfig {
  /// Path to the `pi` binary. Default: "pi" (found via PATH).
  pub pi_binary: String,
  /// LLM provider (--provider flag).
  pub provider: Option<String>,
  /// Model pattern or ID (--model flag).
  pub model: Option<String>,
  /// Session persistence mode.
  pub session_persistence: SessionPersistence,
  /// Working directory for the pi process.
  pub working_dir: Option<PathBuf>,
  /// Custom session directory (--session-dir flag).
  pub session_dir: Option<PathBuf>,
  /// Additional CLI arguments passed to pi.
  pub extra_args: Vec<String>,
  /// Whether to check `pi --version` against [`COMPATIBLE_PI_VERSION`] before spawning.
  pub version_check: PiVersionCheck,
}

impl Default for PiSessionConfig {
  fn default() -> Self {
    PiSessionConfig {
      pi_binary: "pi".into(),
      provider: None,
      model: None,
      session_persistence: SessionPersistence::default(),
      working_dir: None,
      session_dir: None,
      extra_args: vec![],
      version_check: PiVersionCheck::default(),
    }
  }
}

// ============================================================================
// Default command timeout
// ============================================================================

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

async fn check_pi_version(config: &PiSessionConfig) -> Result<(), PiError> {
  if config.version_check == PiVersionCheck::Disabled {
    return Ok(());
  }

  match installed_pi_version(config).await {
    Ok(actual) if actual == COMPATIBLE_PI_VERSION => Ok(()),
    Ok(actual) => handle_version_check_problem(
      config.version_check,
      PiError::VersionMismatch {
        expected: COMPATIBLE_PI_VERSION.to_string(),
        actual,
      },
    ),
    Err(err) => handle_version_check_problem(config.version_check, err),
  }
}

fn handle_version_check_problem(check: PiVersionCheck, err: PiError) -> Result<(), PiError> {
  match check {
    PiVersionCheck::Disabled => Ok(()),
    PiVersionCheck::Warn => {
      log::warn!("{err}");
      Ok(())
    }
    PiVersionCheck::Error => Err(err),
  }
}

async fn installed_pi_version(config: &PiSessionConfig) -> Result<String, PiError> {
  let mut cmd = Command::new(&config.pi_binary);
  cmd.arg("--version");

  if let Some(ref working_dir) = config.working_dir {
    cmd.current_dir(working_dir);
  }

  let output = cmd
    .output()
    .await
    .map_err(|err| PiError::VersionCheckFailed {
      message: format!("failed to run '{} --version': {err}", config.pi_binary),
    })?;

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);
  let combined = format!("{stdout}\n{stderr}");

  if !output.status.success() {
    return Err(PiError::VersionCheckFailed {
      message: format!(
        "'{} --version' exited with status {}: {}",
        config.pi_binary,
        output.status,
        combined.trim()
      ),
    });
  }

  parse_pi_version(&combined).ok_or_else(|| PiError::VersionCheckFailed {
    message: format!(
      "could not parse pi version from '{} --version' output: {}",
      config.pi_binary,
      combined.trim()
    ),
  })
}

fn parse_pi_version(output: &str) -> Option<String> {
  output.split_whitespace().find_map(|token| {
    let token = token
      .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-')
      .trim_start_matches('v');

    if is_semver_like(token) {
      Some(token.to_string())
    } else {
      None
    }
  })
}

fn is_semver_like(version: &str) -> bool {
  let mut parts = version.split('.');
  let (Some(major), Some(minor), Some(patch)) = (parts.next(), parts.next(), parts.next()) else {
    return false;
  };
  if parts.next().is_some() {
    return false;
  }

  let patch = patch.split_once('-').map_or(patch, |(patch, _)| patch);

  !major.is_empty()
    && !minor.is_empty()
    && !patch.is_empty()
    && major.chars().all(|c| c.is_ascii_digit())
    && minor.chars().all(|c| c.is_ascii_digit())
    && patch.chars().all(|c| c.is_ascii_digit())
}

// ============================================================================
// PiSession
// ============================================================================

type Subscribers = Arc<Mutex<Vec<mpsc::UnboundedSender<RpcEvent>>>>;
type Pending = Arc<Mutex<HashMap<String, oneshot::Sender<Result<RpcResponse, PiError>>>>>;

enum ProcessControl {
  Kill {
    ack: Option<oneshot::Sender<Result<(), PiError>>>,
  },
}

/// A running pi RPC session.
///
/// Owns the child process and provides:
/// - `send_command` for sending RPC commands and awaiting responses
/// - `subscribe` for streaming events
/// - `kill` for explicit shutdown (also happens on drop)
pub struct PiSession {
  writer: Arc<Mutex<BufWriter<tokio::process::ChildStdin>>>,
  subscribers: Subscribers,
  pending: Pending,
  next_id: AtomicU64,
  running: Arc<AtomicBool>,
  process_control_tx: mpsc::UnboundedSender<ProcessControl>,
  reader_cancel: CancellationToken,
  supervisor_cancel: CancellationToken,
  // The reader task parses pi's stdout JSON lines, deserializes them into RPC
  // response/event types, sends RPC responses to the caller waiting on the
  // matching request id, and broadcasts non-response events to subscribers.
  reader_task: Option<JoinHandle<()>>,
  // The supervisor task owns the child process: it handles PiSession::kill,
  // watches for process exit, captures stderr, notifies subscribers, and fails
  // any outstanding RPC requests when the process exits.
  supervisor_task: Option<JoinHandle<()>>,
}

impl PiSession {
  /// Spawn a new pi process in RPC mode.
  pub async fn spawn(config: PiSessionConfig) -> Result<PiSession, PiError> {
    check_pi_version(&config).await?;

    let mut cmd = Command::new(&config.pi_binary);
    cmd.arg("--mode").arg("rpc");

    if let Some(ref provider) = config.provider {
      cmd.arg("--provider").arg(provider);
    }
    if let Some(ref model) = config.model {
      cmd.arg("--model").arg(model);
    }

    match &config.session_persistence {
      SessionPersistence::Enabled => {}
      SessionPersistence::Disabled => {
        cmd.arg("--no-session");
      }
      SessionPersistence::CustomDir(path) => {
        cmd.arg("--session-dir").arg(path);
      }
    }

    if let Some(ref session_dir) = config.session_dir {
      cmd.arg("--session-dir").arg(session_dir);
    }

    if let Some(ref working_dir) = config.working_dir {
      cmd.current_dir(working_dir);
    }

    for arg in &config.extra_args {
      cmd.arg(arg);
    }

    cmd
      .stdin(std::process::Stdio::piped())
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .kill_on_drop(true);

    let mut child = cmd.spawn()?;

    let stdin = child.stdin.take().expect("stdin was piped");
    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    let writer = Arc::new(Mutex::new(BufWriter::new(stdin)));
    let subscribers: Subscribers = Arc::new(Mutex::new(Vec::new()));
    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));
    let running = Arc::new(AtomicBool::new(true));
    let (process_control_tx, process_control_rx) = mpsc::unbounded_channel();

    let reader_cancel = CancellationToken::new();
    let supervisor_cancel = CancellationToken::new();

    let reader_task = spawn_reader_task(
      stdout,
      subscribers.clone(),
      pending.clone(),
      reader_cancel.clone(),
    );
    let supervisor_task = spawn_supervisor_task(
      child,
      stderr,
      process_control_rx,
      subscribers.clone(),
      pending.clone(),
      running.clone(),
      supervisor_cancel.clone(),
    );

    let session = PiSession {
      writer,
      subscribers,
      pending,
      next_id: AtomicU64::new(1),
      running,
      process_control_tx,
      reader_cancel,
      supervisor_cancel,
      reader_task: Some(reader_task),
      supervisor_task: Some(supervisor_task),
    };

    // Wait for pi to be ready before returning. Pi doesn't emit any
    // startup event, so we probe with get_state(). Without this, callers
    // could send commands before pi has finished initializing, leading to
    // errors or a crashed process.
    session.send_command(RpcCommandKind::GetState).await?;

    Ok(session)
  }

  /// Subscribe to events from the pi process.
  ///
  /// Returns an unbounded receiver. Events are delivered from the point of
  /// subscription onward. The receiver is automatically cleaned up when dropped.
  pub async fn subscribe(&self) -> mpsc::UnboundedReceiver<RpcEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    self.subscribers.lock().await.push(tx);
    rx
  }

  /// Send an RPC command and await the correlated response.
  ///
  /// This is the internal helper used by all public command methods.
  pub(crate) async fn send_command(&self, command: RpcCommandKind) -> Result<RpcResponse, PiError> {
    self
      .send_command_with_timeout(command, DEFAULT_TIMEOUT)
      .await
  }

  /// Send an RPC command with a custom timeout.
  pub(crate) async fn send_command_with_timeout(
    &self,
    command: RpcCommandKind,
    timeout: Duration,
  ) -> Result<RpcResponse, PiError> {
    // Check if process is still running
    if !self.running.load(Ordering::Acquire) {
      return Err(PiError::NotRunning);
    }

    // Generate unique ID
    let id = self.next_id.fetch_add(1, Ordering::Relaxed).to_string();

    // Build command
    let rpc_command = RpcCommand {
      id: Some(id.clone()),
      kind: command,
    };

    // Create oneshot for response correlation
    let (tx, rx) = oneshot::channel();
    {
      let mut pending = self.pending.lock().await;
      pending.insert(id.clone(), tx);
    }

    // Serialize and write
    let json = serde_json::to_string(&rpc_command)?;
    {
      let mut writer = self.writer.lock().await;
      if let Err(e) = writer.write_all(json.as_bytes()).await {
        // Clean up pending entry
        self.pending.lock().await.remove(&id);
        return Err(PiError::Io(e));
      }
      if let Err(e) = writer.write_all(b"\n").await {
        self.pending.lock().await.remove(&id);
        return Err(PiError::Io(e));
      }
      if let Err(e) = writer.flush().await {
        self.pending.lock().await.remove(&id);
        return Err(PiError::Io(e));
      }
    }

    // Await response with timeout
    match tokio::time::timeout(timeout, rx).await {
      Ok(Ok(response)) => response,
      Ok(Err(_)) => {
        // oneshot sender was dropped without an explicit error.
        Err(PiError::NotRunning)
      }
      Err(_) => {
        // Timeout — clean up pending entry
        self.pending.lock().await.remove(&id);
        Err(PiError::Timeout)
      }
    }
  }

  /// Write a raw JSON line to the pi process stdin.
  ///
  /// Used by `respond_extension_ui` to send non-command messages.
  pub(crate) async fn write_json_line(&self, json: &str) -> Result<(), PiError> {
    let mut writer = self.writer.lock().await;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
  }

  /// Kill the pi process.
  pub async fn kill(&mut self) -> Result<(), PiError> {
    if !self.running.load(Ordering::Acquire) {
      return Ok(());
    }

    let (ack_tx, ack_rx) = oneshot::channel();
    if self
      .process_control_tx
      .send(ProcessControl::Kill { ack: Some(ack_tx) })
      .is_err()
    {
      self.running.store(false, Ordering::Release);
      return Ok(());
    }

    match ack_rx.await {
      Ok(result) => result,
      Err(_) => Ok(()),
    }
  }

  /// Wait for background tasks to finish (useful after kill).
  pub async fn wait_closed(&mut self) {
    if let Some(task) = self.reader_task.take() {
      let _ = task.await;
    }
    if let Some(task) = self.supervisor_task.take() {
      let _ = task.await;
    }
  }
}

impl Drop for PiSession {
  fn drop(&mut self) {
    let _ = self
      .process_control_tx
      .send(ProcessControl::Kill { ack: None });
    self.reader_cancel.cancel();
    self.supervisor_cancel.cancel();

    if let Some(task) = self.reader_task.take() {
      task.abort();
    }
    if let Some(task) = self.supervisor_task.take() {
      task.abort();
    }
  }
}

fn spawn_reader_task(
  stdout: ChildStdout,
  subscribers: Subscribers,
  pending: Pending,
  cancel: CancellationToken,
) -> JoinHandle<()> {
  tokio::spawn(async move {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    loop {
      let line = tokio::select! {
        _ = cancel.cancelled() => break,
        result = lines.next_line() => match result {
          Ok(Some(line)) => line,
          Ok(None) => break,
          Err(e) => {
            fan_out(
              &subscribers,
              RpcEvent::Session(SessionEvent::DeserializationError {
                context: DeserializationErrorContext::JsonLine,
                error: JsonErrorInfo {
                  message: e.to_string(),
                  line: 0,
                  column: 0,
                  category: "Io".into(),
                },
                line: None,
              }),
            )
            .await;
            break;
          }
        }
      };

      if line.is_empty() {
        continue;
      }

      // Parse as generic JSON first
      let value: serde_json::Value = match serde_json::from_str(&line) {
        Ok(v) => v,
        Err(e) => {
          fan_out(
            &subscribers,
            RpcEvent::Session(SessionEvent::DeserializationError {
              context: DeserializationErrorContext::JsonLine,
              error: JsonErrorInfo::from(&e),
              line: Some(line),
            }),
          )
          .await;
          continue;
        }
      };

      let type_str = value.get("type").and_then(|v| v.as_str()).unwrap_or("");

      if type_str == "response" {
        let response_id = value.get("id").and_then(|v| v.as_str()).map(str::to_string);

        // Deserialize as RpcResponse
        let response: RpcResponse = match serde_json::from_value(value) {
          Ok(r) => r,
          Err(e) => {
            let error_info = JsonErrorInfo::from(&e);
            if let Some(id) = response_id {
              let mut pending = pending.lock().await;
              if let Some(sender) = pending.remove(&id) {
                let _ = sender.send(Err(PiError::Json(e)));
              }
            }

            fan_out(
              &subscribers,
              RpcEvent::Session(SessionEvent::DeserializationError {
                context: DeserializationErrorContext::RpcResponse,
                error: error_info,
                line: Some(line),
              }),
            )
            .await;
            continue;
          }
        };

        // Look up pending command by id
        if let Some(ref id) = response.id {
          let mut pending = pending.lock().await;
          if let Some(sender) = pending.remove(id) {
            let _ = sender.send(Ok(response));
          }
        }
      } else if type_str == "extension_ui_request" {
        // Deserialize as RpcExtensionUIRequest
        let request: RpcExtensionUIRequest = match serde_json::from_value(value) {
          Ok(r) => r,
          Err(e) => {
            fan_out(
              &subscribers,
              RpcEvent::Session(SessionEvent::DeserializationError {
                context: DeserializationErrorContext::RpcExtensionUIRequest,
                error: JsonErrorInfo::from(&e),
                line: Some(line),
              }),
            )
            .await;
            continue;
          }
        };

        let event = RpcEvent::ExtensionUI(request);
        fan_out(&subscribers, event).await;
      } else {
        // Deserialize as AgentEvent
        match serde_json::from_value(value) {
          Ok(agent_event) => {
            let event = RpcEvent::Agent(agent_event);
            fan_out(&subscribers, event).await;
          }
          Err(e) => {
            fan_out(
              &subscribers,
              RpcEvent::Session(SessionEvent::DeserializationError {
                context: DeserializationErrorContext::AgentEvent,
                error: JsonErrorInfo::from(&e),
                line: Some(line),
              }),
            )
            .await;
          }
        }
      }
    }
  })
}

fn spawn_supervisor_task(
  mut child: Child,
  stderr: ChildStderr,
  mut control_rx: mpsc::UnboundedReceiver<ProcessControl>,
  subscribers: Subscribers,
  pending: Pending,
  running: Arc<AtomicBool>,
  cancel: CancellationToken,
) -> JoinHandle<()> {
  tokio::spawn(async move {
    let mut stderr_reader = BufReader::new(stderr);
    let mut stderr = String::new();
    let mut stderr_open = true;
    let mut exit_status = None;
    let mut kill_ack = None;

    while exit_status.is_none() {
      let mut stderr_line = String::new();
      tokio::select! {
        _ = cancel.cancelled() => {
          let _ = child.start_kill();
          return;
        }
        result = stderr_reader.read_line(&mut stderr_line), if stderr_open => {
          match result {
            Ok(0) => stderr_open = false,
            Ok(_) => stderr.push_str(&stderr_line),
            Err(_) => stderr_open = false,
          }
        }
        status = child.wait() => {
          exit_status = Some(status);
        }
        command = control_rx.recv() => {
          match command {
            Some(ProcessControl::Kill { ack }) => {
              kill_ack = ack;
              if let Err(e) = child.start_kill() {
                // start_kill can race with natural process exit. In that case,
                // still wait for and report the real exit status.
                if e.kind() != std::io::ErrorKind::InvalidInput {
                  if let Some(ack) = kill_ack.take() {
                    let _ = ack.send(Err(PiError::Io(e)));
                  }
                  return;
                }
              }
              exit_status = Some(child.wait().await);
            }
            None => {
              let _ = child.start_kill();
              exit_status = Some(child.wait().await);
            }
          }
        }
      }
    }

    // Once the child has exited, stderr should close. Drain any remaining data.
    if stderr_open {
      loop {
        let mut stderr_line = String::new();
        match stderr_reader.read_line(&mut stderr_line).await {
          Ok(0) => break,
          Ok(_) => stderr.push_str(&stderr_line),
          Err(_) => break,
        }
      }
    }

    let code = exit_status
      .and_then(|status| status.ok())
      .and_then(|status| status.code());
    running.store(false, Ordering::Release);

    fail_pending_process_exited(&pending, code, stderr.clone()).await;
    fan_out(
      &subscribers,
      RpcEvent::Session(SessionEvent::ProcessExited {
        code,
        stderr: stderr.clone(),
      }),
    )
    .await;

    if let Some(ack) = kill_ack {
      let _ = ack.send(Ok(()));
    }

    let mut subs = subscribers.lock().await;
    subs.clear();
  })
}

async fn fail_pending_process_exited(pending: &Pending, code: Option<i32>, stderr: String) {
  let mut pending = pending.lock().await;
  for (_, sender) in pending.drain() {
    let _ = sender.send(Err(PiError::ProcessExited {
      code,
      stderr: stderr.clone(),
    }));
  }
}

/// Fan out an event to all subscribers, removing dead ones.
async fn fan_out(subscribers: &Subscribers, event: RpcEvent) {
  let mut subs = subscribers.lock().await;
  subs.retain(|tx| tx.send(event.clone()).is_ok());
}
