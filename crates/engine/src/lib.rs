#![forbid(unsafe_code)]
#![deny(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

//! Headless services shared by the xsaber desktop shell and command-line tools.

pub mod auth;
pub mod diagnostics;
pub mod hostkeys;
pub mod protocol;
pub mod remote_fs;
pub mod secrets;
pub mod session;
pub mod ssh_config;

pub use auth::{AuthLadder, AuthMethod, AuthMethodKind, AuthMethodSet, AuthOutcome, AuthRequest};
pub use diagnostics::{Diagnostics, LogMessage, LoggingError, LoggingGuard, init_logging};
pub use hostkeys::{HostKey, HostKeysError, KnownHosts, Trust};
pub use protocol::{ChosenProtocol, ProtocolDecision, RequestedProtocol, select_protocol};
pub use remote_fs::{Caps, FsError, RemoteFs, RemotePath};
pub use secrets::Secret;
pub use session::{
    ConnectionState, Event as SessionEvent, SessionConfig, SessionEvents, SessionState,
};
pub use ssh_config::{ResolvedSshConfig, SshConfig, SshConfigError};
