//! Types from `packages/coding-agent/src/core/bash-executor.ts`.

use serde::{Deserialize, Serialize};

/// Result from bash command execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BashResult {
  pub output: String,
  #[serde(rename = "exitCode")]
  pub exit_code: Option<f64>,
  pub cancelled: bool,
  pub truncated: bool,
  #[serde(
    rename = "fullOutputPath",
    default,
    skip_serializing_if = "Option::is_none"
  )]
  pub full_output_path: Option<String>,
}
