use std::fmt;

use zeroize::Zeroizing;

/// Sensitive material that is safe to carry through tracing fields.
///
/// The value can be deliberately accessed by code that needs to authenticate,
/// but both [`Debug`] and [`Display`] always produce the same redacted marker.
/// Keeping redaction at this boundary means filters and subscribers cannot
/// accidentally turn a password into a log record.
#[derive(Clone, Eq, PartialEq)]
pub struct Secret {
    value: Zeroizing<String>,
}

impl Secret {
    /// Wraps a sensitive string.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: Zeroizing::new(value.into()),
        }
    }

    /// Names a value used as a password while retaining one redaction type.
    pub fn password(value: impl Into<String>) -> Self {
        Self::new(value)
    }

    /// Names a value used as a passphrase while retaining one redaction type.
    pub fn passphrase(value: impl Into<String>) -> Self {
        Self::new(value)
    }

    /// Names private-key or other key material while retaining one redaction type.
    pub fn key_material(value: impl Into<String>) -> Self {
        Self::new(value)
    }

    /// Borrows the secret for the authentication operation that requires it.
    pub fn expose(&self) -> &str {
        self.value.as_str()
    }

    /// Returns whether the wrapped value is empty.
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted>")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted>")
    }
}
