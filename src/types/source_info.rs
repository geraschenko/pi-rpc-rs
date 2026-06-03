//! Types from `packages/coding-agent/src/core/source-info.ts`.

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SourceScope {
  User,
  Project,
  Temporary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SourceOrigin {
  Package,
  TopLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
  pub path: String,
  pub source: String,
  pub scope: SourceScope,
  pub origin: SourceOrigin,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub base_dir: Option<String>,
}
