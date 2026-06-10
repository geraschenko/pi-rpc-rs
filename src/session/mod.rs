//! Pi session management — process lifecycle, event streaming, command dispatch.

pub mod error;
mod impl_rpc_methods;
mod session;

pub use error::PiError;
pub use session::{PiSession, PiSessionConfig, PiVersionCheck, SessionPersistence};
