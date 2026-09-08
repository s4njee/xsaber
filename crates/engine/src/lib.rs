#![forbid(unsafe_code)]
#![deny(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

//! Headless services shared by the xsaber desktop shell and command-line tools.

pub mod diagnostics;
pub mod secrets;

pub use diagnostics::{Diagnostics, LogMessage, LoggingError, LoggingGuard, init_logging};
pub use secrets::Secret;
