#![forbid(unsafe_code)]
#![deny(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

//! Headless services shared by the xsaber desktop shell and command-line tools.

pub mod diagnostics;
pub mod hostkeys;
pub mod secrets;
pub mod session;

pub use diagnostics::{Diagnostics, LogMessage, LoggingError, LoggingGuard, init_logging};
pub use hostkeys::{HostKey, HostKeysError, KnownHosts, Trust};
pub use secrets::Secret;
pub use session::{
    ConnectionState, Event as SessionEvent, SessionConfig, SessionEvents, SessionState,
};
