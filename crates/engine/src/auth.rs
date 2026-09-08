//! Authentication policy for an SSH session.
//!
//! This module is deliberately transport-neutral.  A russh adapter can turn
//! [`AuthMethod`] values into calls on `client::Handle`, while the UI answers
//! [`AuthRequest`] values.  No method reads stdin or environment variables,
//! and all supplied secret material stays inside [`Secret`].

use std::path::PathBuf;

use crate::Secret;

/// A method the authentication ladder may attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthMethod {
    /// Use the SSH agent selected by the adapter (including Pageant on
    /// Windows when supported by that adapter).
    Agent,
    /// Use a configured private-key file.
    ConfiguredKey {
        /// Path selected by the user, never a shell command string.
        path: PathBuf,
        /// Already supplied passphrase, if the key was previously unlocked.
        /// An encrypted key with no passphrase asks the UI through
        /// [`AuthRequest::Passphrase`].
        passphrase: Option<Secret>,
    },
    /// Use the configured password.
    Password(Secret),
    /// Ask the server and UI to complete a keyboard-interactive exchange.
    KeyboardInteractive,
}

impl AuthMethod {
    /// Returns the stable kind used for server method bitsets and diagnostics.
    #[must_use]
    pub const fn kind(&self) -> AuthMethodKind {
        match self {
            Self::Agent => AuthMethodKind::Agent,
            Self::ConfiguredKey { .. } => AuthMethodKind::ConfiguredKey,
            Self::Password(_) => AuthMethodKind::Password,
            Self::KeyboardInteractive => AuthMethodKind::KeyboardInteractive,
        }
    }
}

/// Stable method identity independent of credentials.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthMethodKind {
    Agent,
    ConfiguredKey,
    Password,
    KeyboardInteractive,
}

impl AuthMethodKind {
    const fn bit(self) -> u8 {
        match self {
            Self::Agent => 1,
            Self::ConfiguredKey => 1 << 1,
            Self::Password => 1 << 2,
            Self::KeyboardInteractive => 1 << 3,
        }
    }
}

/// A set of authentication methods advertised by a server.
///
/// This is intentionally a small typed bitset instead of a string list.  The
/// russh adapter owns conversion from its `MethodSet`, and unknown future
/// server methods can be ignored without accidentally enabling a UI control.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AuthMethodSet(u8);

impl AuthMethodSet {
    /// No methods remain.
    pub const EMPTY: Self = Self(0);
    /// All methods understood by xsaber v1.
    pub const ALL: Self = Self(0b1111);

    /// Creates a one-method set.
    #[must_use]
    pub const fn only(method: AuthMethodKind) -> Self {
        Self(method.bit())
    }

    /// Adds one method to this set.
    #[must_use]
    pub const fn with(self, method: AuthMethodKind) -> Self {
        Self(self.0 | method.bit())
    }

    /// Returns whether a method is present.
    #[must_use]
    pub const fn contains(self, method: AuthMethodKind) -> bool {
        self.0 & method.bit() != 0
    }

    /// Returns the raw bit representation for adapter diagnostics.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// User-facing request emitted by an authentication adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthRequest {
    /// The configured key is encrypted and needs a passphrase.
    Passphrase { key_path: PathBuf },
    /// A server requested a password and no configured password is available.
    Password,
    /// The server supplied keyboard-interactive questions.
    KeyboardInteractive {
        name: String,
        instructions: String,
        prompts: Vec<AuthPrompt>,
    },
}

/// One keyboard-interactive question.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthPrompt {
    /// Text to show to the user.
    pub text: String,
    /// Whether the response may be shown while typing.
    pub echo: bool,
}

/// A response to an [`AuthRequest`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthResponse {
    /// A passphrase or password.  It is never represented as a plain String.
    Secret(Secret),
    /// Answers to keyboard-interactive prompts, in the same order received.
    KeyboardInteractive(Vec<Secret>),
    /// The user rejected the request.
    Reject,
}

/// The result of one server authentication attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthOutcome {
    /// Methods the server says may still be attempted.
    pub remaining_methods: AuthMethodSet,
    /// Whether the attempt succeeded as one step of a multi-factor exchange.
    pub partial_success: bool,
}

impl AuthOutcome {
    /// Constructs a server failure/continuation result.
    #[must_use]
    pub const fn failure(remaining_methods: AuthMethodSet, partial_success: bool) -> Self {
        Self {
            remaining_methods,
            partial_success,
        }
    }
}

/// Ordered authentication policy and its current server-guided position.
///
/// The initial order is always agent → configured key → password → keyboard
/// interactive.  A server's remaining-method set narrows that order after
/// each failed or partial attempt.  `partial_success` is retained in the
/// outcome for the adapter and tests: it means the attempted factor is done,
/// while the remaining factors must still be completed.
#[derive(Clone, Debug)]
pub struct AuthLadder {
    methods: Vec<AuthMethod>,
    allowed: AuthMethodSet,
    cursor: usize,
    current: Option<AuthMethodKind>,
}

impl AuthLadder {
    /// Builds a ladder from explicit, app-owned credentials and preferences.
    #[must_use]
    pub fn new(
        use_agent: bool,
        configured_key: Option<(PathBuf, Option<Secret>)>,
        password: Option<Secret>,
        keyboard_interactive: bool,
    ) -> Self {
        let mut methods = Vec::with_capacity(4);
        if use_agent {
            methods.push(AuthMethod::Agent);
        }
        if let Some((path, passphrase)) = configured_key {
            methods.push(AuthMethod::ConfiguredKey { path, passphrase });
        }
        if let Some(password) = password {
            methods.push(AuthMethod::Password(password));
        }
        if keyboard_interactive {
            methods.push(AuthMethod::KeyboardInteractive);
        }
        Self {
            methods,
            allowed: AuthMethodSet::ALL,
            cursor: 0,
            current: None,
        }
    }

    /// Returns the next method, preserving the ladder order.
    ///
    /// Calling this again before [`Self::observe`] returns the same method;
    /// the adapter cannot accidentally skip a factor by polling twice.
    pub fn next_method(&mut self) -> Option<&AuthMethod> {
        if let Some(current) = self.current {
            return self.methods[self.cursor..]
                .iter()
                .find(|method| method.kind() == current);
        }
        while self.cursor < self.methods.len() {
            let method = &self.methods[self.cursor];
            if self.allowed.contains(method.kind()) {
                self.current = Some(method.kind());
                return Some(method);
            }
            self.cursor += 1;
        }
        None
    }

    /// Applies russh's remaining-method and partial-success result.
    pub fn observe(&mut self, outcome: AuthOutcome) {
        if self.current.is_some() {
            self.cursor = self.cursor.saturating_add(1);
            self.current = None;
        }
        self.allowed = outcome.remaining_methods;
    }

    /// Returns the configured methods, useful for a UI summary without
    /// exposing mutable planner internals.
    #[must_use]
    pub fn configured_methods(&self) -> &[AuthMethod] {
        &self.methods
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AuthLadder, AuthMethod, AuthMethodKind, AuthMethodSet, AuthOutcome, AuthPrompt,
        AuthRequest, AuthResponse,
    };
    use crate::Secret;
    use std::path::PathBuf;

    #[test]
    fn ladder_orders_configured_methods_without_implicit_credentials() {
        let mut ladder = AuthLadder::new(
            true,
            Some((PathBuf::from("/tmp/id_ed25519"), None)),
            Some(Secret::password("password-that-must-not-leak")),
            true,
        );
        assert_eq!(
            ladder.next_method().map(AuthMethod::kind),
            Some(AuthMethodKind::Agent)
        );
        ladder.observe(AuthOutcome::failure(AuthMethodSet::ALL, false));
        assert_eq!(
            ladder.next_method().map(AuthMethod::kind),
            Some(AuthMethodKind::ConfiguredKey)
        );
        ladder.observe(AuthOutcome::failure(AuthMethodSet::ALL, false));
        assert_eq!(
            ladder.next_method().map(AuthMethod::kind),
            Some(AuthMethodKind::Password)
        );
        ladder.observe(AuthOutcome::failure(AuthMethodSet::ALL, false));
        assert_eq!(
            ladder.next_method().map(AuthMethod::kind),
            Some(AuthMethodKind::KeyboardInteractive)
        );
        ladder.observe(AuthOutcome::failure(AuthMethodSet::EMPTY, false));
        assert!(ladder.next_method().is_none());
    }

    #[test]
    fn partial_success_continues_only_with_server_remaining_factors() {
        let mut ladder = AuthLadder::new(
            true,
            Some((PathBuf::from("id_ed25519"), None)),
            Some(Secret::password("pw")),
            true,
        );
        assert_eq!(
            ladder.next_method().map(AuthMethod::kind),
            Some(AuthMethodKind::Agent)
        );
        ladder.observe(AuthOutcome::failure(
            AuthMethodSet::only(AuthMethodKind::ConfiguredKey)
                .with(AuthMethodKind::KeyboardInteractive),
            true,
        ));
        assert_eq!(
            ladder.next_method().map(AuthMethod::kind),
            Some(AuthMethodKind::ConfiguredKey)
        );
        ladder.observe(AuthOutcome::failure(
            AuthMethodSet::only(AuthMethodKind::KeyboardInteractive),
            true,
        ));
        assert_eq!(
            ladder.next_method().map(AuthMethod::kind),
            Some(AuthMethodKind::KeyboardInteractive)
        );
    }

    #[test]
    fn requests_and_responses_keep_secret_values_redacted() {
        let request = AuthRequest::KeyboardInteractive {
            name: "otp".to_owned(),
            instructions: "Enter the code".to_owned(),
            prompts: vec![AuthPrompt {
                text: "Code: ".to_owned(),
                echo: false,
            }],
        };
        assert_eq!(request.clone(), request);
        let response = AuthResponse::KeyboardInteractive(vec![Secret::new("123456")]);
        let debug = format!("{response:?}");
        assert!(!debug.contains("123456"), "{debug}");
        assert!(debug.contains("<redacted>"), "{debug}");
    }

    #[test]
    fn method_debug_does_not_expose_password_or_passphrase() {
        let method = AuthMethod::ConfiguredKey {
            path: PathBuf::from("id_ed25519"),
            passphrase: Some(Secret::passphrase("private phrase")),
        };
        let debug = format!("{method:?}");
        assert!(!debug.contains("private phrase"), "{debug}");
        assert!(debug.contains("<redacted>"), "{debug}");
    }
}
