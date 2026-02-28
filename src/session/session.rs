//! PiSession — manages a `pi --mode rpc` child process.
//!
//! See `docs/session-api.md` for the full API design.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::JoinHandle;

use super::error::PiError;
use crate::types::{RpcCommand, RpcCommandKind, RpcEvent, RpcExtensionUIRequest, RpcResponse};

// ============================================================================
// Configuration
// ============================================================================

/// How session persistence is handled.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionPersistence {
  /// Default pi behavior (sessions saved to ~/.pi/agent/sessions/).
  Enabled,
  /// Pass --no-session flag.
  Disabled,
  /// Pass --session-dir with the given path.
  CustomDir(PathBuf),
}

impl Default for SessionPersistence {
  fn default() -> Self {
    SessionPersistence::Enabled
  }
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
    }
  }
}

// ============================================================================
// Default command timeout
// ============================================================================

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// ============================================================================
// PiSession
// ============================================================================

type Subscribers = Arc<Mutex<Vec<mpsc::UnboundedSender<RpcEvent>>>>;
type Pending = Arc<Mutex<HashMap<String, oneshot::Sender<RpcResponse>>>>;

/// A running pi RPC session.
///
/// Owns the child process and provides:
/// - `send_command` for sending RPC commands and awaiting responses
/// - `subscribe` for streaming events
/// - `kill` for explicit shutdown (also happens on drop)
pub struct PiSession {
  child: Option<Child>,
  writer: Arc<Mutex<BufWriter<tokio::process::ChildStdin>>>,
  subscribers: Subscribers,
  pending: Pending,
  next_id: AtomicU64,
  reader_task: Option<JoinHandle<()>>,
  /// Collects stderr in the background for error reporting.
  stderr_task: Option<JoinHandle<String>>,
}

impl PiSession {
  /// Spawn a new pi process in RPC mode.
  pub async fn spawn(config: PiSessionConfig) -> Result<PiSession, PiError> {
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
      .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdin = child.stdin.take().expect("stdin was piped");
    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");

    let writer = Arc::new(Mutex::new(BufWriter::new(stdin)));
    let subscribers: Subscribers = Arc::new(Mutex::new(Vec::new()));
    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));

    // Spawn stderr collector
    let stderr_task = tokio::spawn(async move {
      let mut reader = BufReader::new(stderr);
      let mut buf = String::new();
      loop {
        match reader.read_line(&mut buf).await {
          Ok(0) | Err(_) => break,
          Ok(_) => {}
        }
      }
      buf
    });

    // Spawn reader task
    let reader_subscribers = subscribers.clone();
    let reader_pending = pending.clone();
    let reader_task = tokio::spawn(async move {
      let reader = BufReader::new(stdout);
      let mut lines = reader.lines();

      loop {
        let line = match lines.next_line().await {
          Ok(Some(line)) => line,
          Ok(None) => break, // EOF — process exited
          Err(_) => break,
        };

        if line.is_empty() {
          continue;
        }

        // Parse as generic JSON first
        let value: serde_json::Value = match serde_json::from_str(&line) {
          Ok(v) => v,
          Err(_) => continue, // Skip unparseable lines
        };

        let type_str = value.get("type").and_then(|v| v.as_str()).unwrap_or("");

        if type_str == "response" {
          // Deserialize as RpcResponse
          let response: RpcResponse = match serde_json::from_value(value) {
            Ok(r) => r,
            Err(_) => continue,
          };

          // Look up pending command by id
          if let Some(ref id) = response.id {
            let mut pending = reader_pending.lock().await;
            if let Some(sender) = pending.remove(id) {
              let _ = sender.send(response);
            }
          }
        } else if type_str == "extension_ui_request" {
          // Deserialize as RpcExtensionUIRequest
          let request: RpcExtensionUIRequest = match serde_json::from_value(value) {
            Ok(r) => r,
            Err(_) => continue,
          };

          let event = RpcEvent::ExtensionUI(request);
          fan_out(&reader_subscribers, event).await;
        } else {
          // Deserialize as AgentEvent
          match serde_json::from_value(value) {
            Ok(agent_event) => {
              let event = RpcEvent::Agent(agent_event);
              fan_out(&reader_subscribers, event).await;
            }
            Err(_) => continue,
          }
        }
      }

      // Process exited — close all pending commands and subscribers
      let mut pending = reader_pending.lock().await;
      pending.clear(); // Dropping oneshot senders signals RecvError to waiters

      let mut subs = reader_subscribers.lock().await;
      subs.clear(); // Dropping senders closes the receivers
    });

    let session = PiSession {
      child: Some(child),
      writer,
      subscribers,
      pending,
      next_id: AtomicU64::new(1),
      reader_task: Some(reader_task),
      stderr_task: Some(stderr_task),
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
    if self.child.is_none() {
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
      Ok(Ok(response)) => Ok(response),
      Ok(Err(_)) => {
        // oneshot sender was dropped — process likely exited
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
    if let Some(ref mut child) = self.child {
      child.kill().await?;
      self.child = None;
    }
    Ok(())
  }

  /// Wait for the reader task to finish (useful after kill).
  pub async fn wait_closed(&mut self) {
    if let Some(task) = self.reader_task.take() {
      let _ = task.await;
    }
  }
}

impl Drop for PiSession {
  fn drop(&mut self) {
    if let Some(ref mut child) = self.child {
      // Best-effort kill. start_kill() is non-async and sends SIGKILL.
      let _ = child.start_kill();
    }
    // Abort the reader task so it doesn't outlive the session.
    if let Some(task) = self.reader_task.take() {
      task.abort();
    }
    if let Some(task) = self.stderr_task.take() {
      task.abort();
    }
  }
}

/// Fan out an event to all subscribers, removing dead ones.
async fn fan_out(subscribers: &Subscribers, event: RpcEvent) {
  let mut subs = subscribers.lock().await;
  subs.retain(|tx| tx.send(event.clone()).is_ok());
}
