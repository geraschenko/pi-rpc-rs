//! Error types for pi-rpc-rs.

use std::fmt;

/// Errors that can occur when interacting with a pi session.
#[derive(Debug)]
pub enum PiError {
  /// Pi process exited unexpectedly.
  ProcessExited { code: Option<i32>, stderr: String },
  /// Command returned success: false.
  CommandFailed { command: String, error: String },
  /// JSON serialization/deserialization error.
  Json(serde_json::Error),
  /// IO error (broken pipe, etc.).
  Io(std::io::Error),
  /// Response timeout.
  Timeout,
  /// Pi process is not running.
  NotRunning,
}

impl fmt::Display for PiError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PiError::ProcessExited { code, stderr } => {
        write!(f, "pi process exited (code: {code:?}): {stderr}")
      }
      PiError::CommandFailed { command, error } => {
        write!(f, "command '{command}' failed: {error}")
      }
      PiError::Json(e) => write!(f, "JSON error: {e}"),
      PiError::Io(e) => write!(f, "IO error: {e}"),
      PiError::Timeout => write!(f, "response timeout"),
      PiError::NotRunning => write!(f, "pi process is not running"),
    }
  }
}

impl std::error::Error for PiError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      PiError::Json(e) => Some(e),
      PiError::Io(e) => Some(e),
      _ => None,
    }
  }
}

impl From<serde_json::Error> for PiError {
  fn from(e: serde_json::Error) -> Self {
    PiError::Json(e)
  }
}

impl From<std::io::Error> for PiError {
  fn from(e: std::io::Error) -> Self {
    PiError::Io(e)
  }
}
