//! Protocol-selection policy for remote sessions.
//!
//! A site requests [`RequestedProtocol::Auto`], `Xsync`, or `Sftp`. Auto is
//! deliberately narrow: it falls back only when the xsync attempt returns the
//! typed [`XsyncAttemptError::XsyncUnavailable`] condition (for example, the
//! remote has no `xs` executable). Authentication, host-key, protocol, and
//! transport failures are not silently converted into SFTP.
//!
//! This module performs no network or SFTP/xsync work. The caller supplies a
//! closure for the xsync attempt, which also makes the forced-SFTP rule
//! mechanically testable: the closure is never called in that mode.

use std::fmt;

/// Protocol requested by a site or quick-connect target.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RequestedProtocol {
    /// Try xsync first, falling back only on typed unavailability.
    #[default]
    Auto,
    /// Require xsync; no fallback is permitted.
    Xsync,
    /// Require SFTP; xsync is never attempted.
    Sftp,
}

/// Protocol selected after applying a request and (if allowed) an xsync
/// attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChosenProtocol {
    /// xsync v3 over the authenticated SSH session.
    Xsync,
    /// SFTP over the authenticated SSH session.
    Sftp,
}

impl ChosenProtocol {
    /// Text used in the protocol badge before the host-key algorithm suffix.
    #[must_use]
    pub const fn badge_name(self) -> &'static str {
        match self {
            Self::Xsync => "XSYNC",
            Self::Sftp => "SFTP",
        }
    }
}

/// The only xsync failure that Auto may convert into SFTP.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XsyncAttemptError {
    /// The remote did not provide a usable xsync server executable.
    XsyncUnavailable { detail: String },
    /// Any other failure: authentication, transport, protocol, or server
    /// error. Auto must return this instead of hiding it behind SFTP.
    Failed { detail: String },
}

impl XsyncAttemptError {
    /// Constructs the typed unavailable result used by Auto fallback.
    #[must_use]
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self::XsyncUnavailable {
            detail: detail.into(),
        }
    }

    /// Constructs a non-fallback xsync failure.
    #[must_use]
    pub fn failed(detail: impl Into<String>) -> Self {
        Self::Failed {
            detail: detail.into(),
        }
    }

    /// Returns the diagnostic text without changing its classification.
    #[must_use]
    pub fn detail(&self) -> &str {
        match self {
            Self::XsyncUnavailable { detail } | Self::Failed { detail } => detail,
        }
    }
}

impl fmt::Display for XsyncAttemptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::XsyncUnavailable { detail } => {
                write!(formatter, "xsync unavailable: {detail}")
            }
            Self::Failed { detail } => write!(formatter, "xsync failed: {detail}"),
        }
    }
}

impl std::error::Error for XsyncAttemptError {}

/// Errors returned when the requested policy cannot be satisfied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolSelectionError {
    /// Forced Xsync cannot proceed because xsync is unavailable.
    XsyncUnavailable { detail: String },
    /// Xsync failed for a reason that must not fall back to SFTP.
    XsyncFailed { detail: String },
}

impl fmt::Display for ProtocolSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::XsyncUnavailable { detail } => {
                write!(formatter, "xsync unavailable: {detail}")
            }
            Self::XsyncFailed { detail } => write!(formatter, "xsync failed: {detail}"),
        }
    }
}

impl std::error::Error for ProtocolSelectionError {}

/// The selected protocol and whether Auto performed its fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolDecision {
    /// The request that produced this decision.
    pub requested: RequestedProtocol,
    /// The protocol the session should open.
    pub chosen: ChosenProtocol,
    /// True only when Auto converted `XsyncUnavailable` into SFTP.
    pub fell_back: bool,
}

impl ProtocolDecision {
    /// Returns badge/tooltip data for the app shell without depending on UI
    /// types. `key_algorithm` is the already-selected SSH host-key algorithm.
    #[must_use]
    pub fn presentation(self, key_algorithm: &str) -> ProtocolPresentation {
        let algorithm = if key_algorithm.is_empty() {
            "unknown"
        } else {
            key_algorithm
        };
        let tooltip = if self.chosen == ChosenProtocol::Sftp && self.fell_back {
            Some("xs not found on host; using SFTP".to_owned())
        } else {
            None
        };
        ProtocolPresentation {
            badge: format!("{} · {algorithm}", self.chosen.badge_name()),
            tone: match self.chosen {
                ChosenProtocol::Xsync => BadgeTone::Teal,
                ChosenProtocol::Sftp => BadgeTone::Amber,
            },
            tooltip,
        }
    }
}

/// Palette token for a protocol badge. It is data, not a UI dependency.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BadgeTone {
    /// Trusted xsync path.
    Teal,
    /// SFTP path or Auto fallback.
    Amber,
}

/// Presentation data consumed by a UI badge or session tab.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolPresentation {
    /// Badge text, for example `XSYNC · ssh-ed25519`.
    pub badge: String,
    /// Badge palette token.
    pub tone: BadgeTone,
    /// Fallback explanation, when Auto selected SFTP after unavailability.
    pub tooltip: Option<String>,
}

/// Applies the requested protocol policy.
///
/// The closure is called only for Auto and forced Xsync. Forced SFTP returns
/// immediately and never evaluates it. A successful closure means xsync is
/// available; an unavailable result is consumed only by Auto, while every
/// other error is surfaced.
pub fn select_protocol<F>(
    requested: RequestedProtocol,
    try_xsync: F,
) -> Result<ProtocolDecision, ProtocolSelectionError>
where
    F: FnOnce() -> Result<(), XsyncAttemptError>,
{
    match requested {
        RequestedProtocol::Sftp => Ok(ProtocolDecision {
            requested,
            chosen: ChosenProtocol::Sftp,
            fell_back: false,
        }),
        RequestedProtocol::Auto => match try_xsync() {
            Ok(()) => Ok(ProtocolDecision {
                requested,
                chosen: ChosenProtocol::Xsync,
                fell_back: false,
            }),
            Err(XsyncAttemptError::XsyncUnavailable { .. }) => Ok(ProtocolDecision {
                requested,
                chosen: ChosenProtocol::Sftp,
                fell_back: true,
            }),
            Err(XsyncAttemptError::Failed { detail }) => {
                Err(ProtocolSelectionError::XsyncFailed { detail })
            }
        },
        RequestedProtocol::Xsync => match try_xsync() {
            Ok(()) => Ok(ProtocolDecision {
                requested,
                chosen: ChosenProtocol::Xsync,
                fell_back: false,
            }),
            Err(XsyncAttemptError::XsyncUnavailable { detail }) => {
                Err(ProtocolSelectionError::XsyncUnavailable { detail })
            }
            Err(XsyncAttemptError::Failed { detail }) => {
                Err(ProtocolSelectionError::XsyncFailed { detail })
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn auto_chooses_xsync_when_the_attempt_succeeds() -> Result<(), Box<dyn std::error::Error>> {
        let decision = select_protocol(RequestedProtocol::Auto, || Ok(()))?;
        assert_eq!(decision.chosen, ChosenProtocol::Xsync);
        assert!(!decision.fell_back);
        let presentation = decision.presentation("ssh-ed25519");
        assert_eq!(presentation.badge, "XSYNC · ssh-ed25519");
        assert_eq!(presentation.tone, BadgeTone::Teal);
        assert_eq!(presentation.tooltip, None);
        Ok(())
    }

    #[test]
    fn auto_falls_back_only_on_typed_unavailability() -> Result<(), Box<dyn std::error::Error>> {
        let decision = select_protocol(RequestedProtocol::Auto, || {
            Err(XsyncAttemptError::unavailable("xs was not found"))
        })?;
        assert_eq!(decision.chosen, ChosenProtocol::Sftp);
        assert!(decision.fell_back);
        let presentation = decision.presentation("ssh-rsa");
        assert_eq!(presentation.badge, "SFTP · ssh-rsa");
        assert_eq!(presentation.tone, BadgeTone::Amber);
        assert_eq!(
            presentation.tooltip.as_deref(),
            Some("xs not found on host; using SFTP")
        );
        Ok(())
    }

    #[test]
    fn auto_surfaces_other_xsync_failures_without_fallback() {
        let result = select_protocol(RequestedProtocol::Auto, || {
            Err(XsyncAttemptError::failed("permission denied"))
        });
        assert!(matches!(
            result,
            Err(ProtocolSelectionError::XsyncFailed { detail }) if detail == "permission denied"
        ));
    }

    #[test]
    fn forced_sftp_never_evaluates_the_xsync_attempt() -> Result<(), Box<dyn std::error::Error>> {
        let called = Cell::new(false);
        let decision = select_protocol(RequestedProtocol::Sftp, || {
            called.set(true);
            Err(XsyncAttemptError::failed("must not run"))
        })?;
        assert_eq!(decision.chosen, ChosenProtocol::Sftp);
        assert!(!decision.fell_back);
        assert!(!called.get());
        assert_eq!(decision.presentation("ssh-ed25519").tooltip, None);
        Ok(())
    }

    #[test]
    fn forced_xsync_does_not_fallback_when_unavailable() {
        let result = select_protocol(RequestedProtocol::Xsync, || {
            Err(XsyncAttemptError::unavailable("no xs executable"))
        });
        assert!(matches!(
            result,
            Err(ProtocolSelectionError::XsyncUnavailable { detail }) if detail == "no xs executable"
        ));
    }

    #[test]
    fn an_empty_key_algorithm_uses_a_safe_display_fallback()
    -> Result<(), Box<dyn std::error::Error>> {
        let decision = select_protocol(RequestedProtocol::Sftp, || Ok(()))?;
        assert_eq!(decision.presentation("").badge, "SFTP · unknown");
        Ok(())
    }
}
