//! Types from `packages/coding-agent/src/core/source-info.ts`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SourceScope {
  #[serde(rename = "user")]
  User,
  #[serde(rename = "project")]
  Project,
  #[serde(rename = "temporary")]
  Temporary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SourceOrigin {
  #[serde(rename = "package")]
  Package,
  #[serde(rename = "top-level")]
  TopLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceInfo {
  pub path: String,
  pub source: String,
  pub scope: SourceScope,
  pub origin: SourceOrigin,
  #[serde(rename = "baseDir", default, skip_serializing_if = "Option::is_none")]
  pub base_dir: Option<String>,
}
