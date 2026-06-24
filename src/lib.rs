#![allow(clippy::module_inception)]

/// Upstream pi version whose RPC protocol this crate targets.
pub const COMPATIBLE_PI_VERSION: &str = "0.80.2";

pub mod session;
pub mod types;
