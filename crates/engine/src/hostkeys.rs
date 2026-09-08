//! App-private OpenSSH-compatible host-key trust.
//!
//! This module contains the policy and file format used by the SSH session
//! layer. It deliberately does not depend on russh: the session's
//! `check_server_key` callback can turn the key it receives into a [`HostKey`]
//! and pass it to [`check`]. Keeping that decision in one place means the SFTP
//! and xsync connection paths cannot accidentally grow different trust rules.
//!
//! The file is compatible with the ordinary, unhashed `known_hosts` entries
//! that this app writes, but it is app-private and is never the user's
//! `~/.ssh/known_hosts`. Hashed entries and `@cert-authority` entries are
//! skipped. We do not verify OpenSSH host certificates against certificate
//! authorities here; certificate support remains a session-layer blocker until
//! a certificate parser and CA policy are selected.

use std::{
    env, fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use base64::Engine as _;
use sha2::{Digest as _, Sha256};

/// The default SSH port, and the port for which `known_hosts` omits brackets.
pub const DEFAULT_SSH_PORT: u16 = 22;

/// One host key as an ordinary `known_hosts` entry records it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostKey {
    /// Hostname as the user entered it. Matching is case-insensitive.
    pub host: String,
    /// Port to which the key belongs.
    pub port: u16,
    /// Key algorithm, for example `ssh-ed25519`.
    pub algorithm: String,
    /// Public-key blob, base64 encoded as `known_hosts` stores it.
    pub key: String,
}

impl HostKey {
    /// Returns the OpenSSH SHA-256 fingerprint slot used by the prompt.
    ///
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(&self.key) else {
            return "SHA256:<unreadable key>".to_owned();
        };
        let digest = Sha256::digest(raw);
        let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(digest);
        format!("SHA256:{encoded}")
    }

    /// The host pattern used by OpenSSH for this host and port.
    fn pattern(&self) -> String {
        if self.port == DEFAULT_SSH_PORT {
            self.host.to_lowercase()
        } else {
            format!("[{}]:{}", self.host.to_lowercase(), self.port)
        }
    }

    fn to_line(&self) -> Result<String> {
        validate_field("host", &self.host)?;
        validate_field("algorithm", &self.algorithm)?;
        validate_field("key", &self.key)?;
        Ok(format!(
            "{} {} {}",
            self.pattern(),
            self.algorithm,
            self.key
        ))
    }
}

/// The result of checking keys offered during an SSH handshake.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Trust {
    /// At least one offered key is already trusted.
    Trusted,
    /// The host has no accepted key. The UI may ask the user about this key.
    Unknown { offered: HostKey },
    /// A key for an already-known algorithm changed. This is never a prompt.
    Changed { offered: HostKey, stored: HostKey },
    /// The host offered a key explicitly marked `@revoked`.
    Revoked { offered: HostKey },
}

/// Errors raised while reading or updating the trust store.
#[derive(Debug)]
pub enum HostKeysError {
    /// An operating-system error, annotated with the path being accessed.
    Io { path: PathBuf, source: io::Error },
    /// A host key field could inject or corrupt a `known_hosts` line.
    InvalidField { field: &'static str },
    /// The SSH callback supplied no host key.
    NoOfferedKeys { host: String, port: u16 },
    /// A caller tried to trust a replacement without first forgetting the old
    /// key. This prevents an accidental Changed -> Trusted click-through.
    ChangedKey {
        host: String,
        port: u16,
        algorithm: String,
        offered: String,
        stored: String,
    },
    /// A caller tried to trust a key that is explicitly revoked.
    RevokedKey {
        host: String,
        port: u16,
        algorithm: String,
    },
}

impl fmt::Display for HostKeysError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::InvalidField { field } => {
                write!(
                    formatter,
                    "host-key {field} contains whitespace or a newline"
                )
            }
            Self::NoOfferedKeys { host, port } => {
                write!(formatter, "{host}:{port} offered no SSH host keys")
            }
            Self::ChangedKey {
                host,
                port,
                algorithm,
                ..
            } => write!(
                formatter,
                "host key for {host}:{port} changed for algorithm {algorithm}; forget it first"
            ),
            Self::RevokedKey {
                host,
                port,
                algorithm,
            } => write!(
                formatter,
                "host key for {host}:{port} is revoked for algorithm {algorithm}"
            ),
        }
    }
}

impl std::error::Error for HostKeysError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidField { .. }
            | Self::NoOfferedKeys { .. }
            | Self::ChangedKey { .. }
            | Self::RevokedKey { .. } => None,
        }
    }
}

type Result<T> = std::result::Result<T, HostKeysError>;

/// A trust store backed by an app-private `known_hosts`-format file.
#[derive(Clone, Debug)]
pub struct KnownHosts {
    path: PathBuf,
}

impl KnownHosts {
    /// Creates a store at `path`; the file and parent directory may not exist.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Uses a platform app-data convention at `<app-data>/xsaber/known_hosts`.
    pub fn app_default() -> Result<Self> {
        let base = app_data_dir().ok_or_else(|| HostKeysError::Io {
            path: PathBuf::from("<platform app-data>"),
            source: io::Error::new(
                io::ErrorKind::NotFound,
                "this platform has no application-data directory",
            ),
        })?;
        Ok(Self::new(base.join("xsaber").join("known_hosts")))
    }

    /// Returns the path used by the store.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads all usable entries matching `host` and `port`.
    ///
    /// A missing file is an empty store. Hashed and `@cert-authority` entries
    /// are intentionally omitted because this module cannot safely validate
    /// them or certificate chains.
    pub fn stored(&self, host: &str, port: u16) -> Result<Vec<StoredKey>> {
        let text = read_file(&self.path)?;
        let wanted = HostKey {
            host: host.to_owned(),
            port,
            algorithm: String::new(),
            key: String::new(),
        }
        .pattern();
        Ok(text
            .lines()
            .filter_map(parse_line)
            .filter(|entry| entry.matches(&wanted))
            .collect())
    }

    /// Records a key as trusted using an atomic same-directory replacement.
    ///
    /// A replacement for an already-known algorithm is rejected. The caller
    /// must explicitly call [`KnownHosts::forget`] first, so a Changed result
    /// cannot be accepted by an incidental prompt handler.
    pub fn trust(&self, key: &HostKey) -> Result<()> {
        let entries = self.stored(&key.host, key.port)?;
        if let Some(existing) = entries
            .iter()
            .find(|entry| entry.key.algorithm == key.algorithm && entry.key.key == key.key)
        {
            if existing.revoked {
                return Err(HostKeysError::RevokedKey {
                    host: key.host.clone(),
                    port: key.port,
                    algorithm: key.algorithm.clone(),
                });
            }
            return Ok(());
        }
        if let Some(existing) = entries
            .iter()
            .find(|entry| !entry.revoked && entry.key.algorithm == key.algorithm)
        {
            return Err(HostKeysError::ChangedKey {
                host: key.host.clone(),
                port: key.port,
                algorithm: key.algorithm.clone(),
                offered: key.key.clone(),
                stored: existing.key.key.clone(),
            });
        }

        let line = key.to_line()?;
        let mut text = read_file(&self.path)?;
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&line);
        text.push('\n');
        self.write_atomic(&text)
    }

    /// Removes all entries for one exact host and port.
    ///
    /// This is the deliberate second step required before accepting a changed
    /// key. It is not called by [`check`] and therefore cannot be reached by a
    /// Changed prompt click.
    pub fn forget(&self, host: &str, port: u16) -> Result<()> {
        let text = read_file(&self.path)?;
        if text.is_empty() {
            return Ok(());
        }
        let wanted = HostKey {
            host: host.to_owned(),
            port,
            algorithm: String::new(),
            key: String::new(),
        }
        .pattern();
        let kept: Vec<&str> = text
            .lines()
            .filter(|line| parse_line(line).is_none_or(|entry| !entry.matches(&wanted)))
            .collect();
        let mut out = kept.join("\n");
        if !out.is_empty() {
            out.push('\n');
        }
        self.write_atomic(&out)
    }

    fn write_atomic(&self, text: &str) -> Result<()> {
        let parent = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;

        let temporary = temporary_path(&self.path);
        let write_result = (|| {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary)
                .map_err(|source| io_error(&temporary, source))?;
            set_private_permissions(&file, &temporary)?;
            file.write_all(text.as_bytes())
                .map_err(|source| io_error(&temporary, source))?;
            file.sync_all()
                .map_err(|source| io_error(&temporary, source))?;
            fs::rename(&temporary, &self.path).map_err(|source| io_error(&self.path, source))
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        write_result
    }
}

fn app_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var_os("APPDATA").map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
    }
    #[cfg(not(any(unix, target_os = "windows")))]
    {
        None
    }
}

/// One parsed entry from a `known_hosts` file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredKey {
    /// The key material and algorithm.
    pub key: HostKey,
    /// Whether the line used the `@revoked` marker.
    pub revoked: bool,
    patterns: Vec<String>,
}

impl StoredKey {
    fn matches(&self, pattern: &str) -> bool {
        self.patterns.iter().any(|candidate| candidate == pattern)
    }
}

/// Decides whether the keys received by an SSH handshake are usable.
///
/// The caller should display a prompt only for [`Trust::Unknown`]. In
/// particular, [`Trust::Changed`] is an unconditional failure until the user
/// explicitly forgets the old entry elsewhere.
pub fn check(known: &KnownHosts, host: &str, port: u16, offered: &[HostKey]) -> Result<Trust> {
    let preferred = preferred(offered).ok_or_else(|| HostKeysError::NoOfferedKeys {
        host: host.to_owned(),
        port,
    })?;
    let stored = known.stored(host, port)?;

    if let Some(revoked) = stored.iter().find(|entry| {
        entry.revoked
            && offered
                .iter()
                .any(|key| key.algorithm == entry.key.algorithm && key.key == entry.key.key)
    }) {
        return Ok(Trust::Revoked {
            offered: HostKey {
                host: host.to_owned(),
                port,
                algorithm: revoked.key.algorithm.clone(),
                key: revoked.key.key.clone(),
            },
        });
    }

    let live: Vec<&StoredKey> = stored.iter().filter(|entry| !entry.revoked).collect();
    if live.is_empty() {
        return Ok(Trust::Unknown {
            offered: preferred.clone(),
        });
    }

    if live.iter().any(|entry| {
        offered
            .iter()
            .any(|key| key.algorithm == entry.key.algorithm && key.key == entry.key.key)
    }) {
        return Ok(Trust::Trusted);
    }

    if let Some(entry) = live.iter().find_map(|entry| {
        offered
            .iter()
            .find(|key| key.algorithm == entry.key.algorithm)
            .map(|key| (key, *entry))
    }) {
        return Ok(Trust::Changed {
            offered: entry.0.clone(),
            stored: HostKey {
                host: host.to_owned(),
                port,
                algorithm: entry.1.key.algorithm.clone(),
                key: entry.1.key.key.clone(),
            },
        });
    }

    Ok(Trust::Unknown {
        offered: preferred.clone(),
    })
}

fn preferred(offered: &[HostKey]) -> Option<&HostKey> {
    const PREFERENCE: [&str; 4] = [
        "ssh-ed25519",
        "ecdsa-sha2-nistp256",
        "rsa-sha2-512",
        "ssh-rsa",
    ];
    PREFERENCE
        .iter()
        .find_map(|algorithm| offered.iter().find(|key| key.algorithm == *algorithm))
        .or_else(|| offered.first())
}

fn parse_line(line: &str) -> Option<StoredKey> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let mut fields = line.split_whitespace();
    let mut first = fields.next()?;
    let mut revoked = false;
    if first.starts_with('@') {
        match first {
            "@revoked" => revoked = true,
            // Certificate-authority verification is intentionally not done by
            // this module. Treat all other markers as unusable, too.
            _ => return None,
        }
        first = fields.next()?;
    }
    // HashKnownHosts entries cannot be matched without implementing the
    // OpenSSH HMAC-SHA1 scheme. Prompting is safer than guessing.
    if first.starts_with('|') {
        return None;
    }
    let algorithm = fields.next()?;
    let key = fields.next()?;
    let patterns: Vec<String> = first.split(',').map(str::to_lowercase).collect();
    if patterns.is_empty() || patterns.iter().any(|pattern| pattern.is_empty()) {
        return None;
    }
    Some(StoredKey {
        key: HostKey {
            host: patterns[0].clone(),
            port: DEFAULT_SSH_PORT,
            algorithm: algorithm.to_owned(),
            key: key.to_owned(),
        },
        revoked,
        patterns,
    })
}

fn read_file(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(source) => Err(io_error(path, source)),
    }
}

fn io_error(path: &Path, source: io::Error) -> HostKeysError {
    HostKeysError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn validate_field(field: &'static str, value: &str) -> Result<()> {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err(HostKeysError::InvalidField { field });
    }
    Ok(())
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temporary_path(path: &Path) -> PathBuf {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("known_hosts");
    let name = format!(".{file_name}.xsaber-{}-{counter}.tmp", std::process::id());
    path.with_file_name(name)
}

fn set_private_permissions(file: &File, path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = file
            .metadata()
            .map_err(|source| io_error(path, source))?
            .permissions();
        permissions.set_mode(0o600);
        file.set_permissions(permissions)
            .map_err(|source| io_error(path, source))?;
    }
    #[cfg(not(unix))]
    let _ = (file, path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    const ED25519: &str = "AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl";
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> io::Result<Self> {
            let number = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("xsaber-hostkeys-{}-{number}", std::process::id()));
            fs::create_dir_all(&path)?;
            Ok(Self(path))
        }

        fn store(&self) -> KnownHosts {
            KnownHosts::new(self.0.join("nested").join("known_hosts"))
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn key(algorithm: &str, material: &str) -> HostKey {
        HostKey {
            host: "nas.local".to_owned(),
            port: 22,
            algorithm: algorithm.to_owned(),
            key: material.to_owned(),
        }
    }

    #[test]
    fn fingerprint_uses_the_openssh_sha256_form() {
        assert_eq!(
            key("ssh-ed25519", ED25519).fingerprint(),
            "SHA256:+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU"
        );
    }

    #[test]
    fn undecodable_key_is_reported_without_panicking() {
        assert_eq!(
            key("ssh-ed25519", "not base64 at all!").fingerprint(),
            "SHA256:<unreadable key>"
        );
    }

    #[test]
    fn missing_file_is_an_empty_store() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = TestDir::new()?;
        assert!(dir.store().stored("nas.local", 22)?.is_empty());
        Ok(())
    }

    #[test]
    fn trust_round_trips_and_is_idempotent() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let dir = TestDir::new()?;
        let known = dir.store();
        known.trust(&key("ssh-ed25519", ED25519))?;
        known.trust(&key("ssh-ed25519", ED25519))?;
        let stored = known.stored("nas.local", 22)?;
        assert_eq!(stored.len(), 1);
        assert!(!stored[0].revoked);
        assert_eq!(
            fs::read_to_string(known.path())?,
            format!("nas.local ssh-ed25519 {ED25519}\n")
        );
        Ok(())
    }

    #[test]
    fn non_default_port_is_bracketed_and_case_insensitive()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = TestDir::new()?;
        let known = dir.store();
        let mut moved = key("ssh-ed25519", ED25519);
        moved.host = "NAS.Local".to_owned();
        moved.port = 2222;
        known.trust(&moved)?;
        assert_eq!(known.stored("nas.local", 2222)?.len(), 1);
        assert!(known.stored("nas.local", 22)?.is_empty());
        assert!(fs::read_to_string(known.path())?.starts_with("[nas.local]:2222 "));
        Ok(())
    }

    #[test]
    fn hashed_and_certificate_authority_lines_are_skipped()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = TestDir::new()?;
        let known = dir.store();
        fs::create_dir_all(known.path().parent().ok_or("parent")?)?;
        fs::write(
            known.path(),
            format!(
                "# comment\n|1|c2FsdA==|aGFzaA== ssh-ed25519 {ED25519}\n@cert-authority nas.local ssh-ed25519 {ED25519}\n"
            ),
        )?;
        assert!(known.stored("nas.local", 22)?.is_empty());
        Ok(())
    }

    #[test]
    fn revoked_entry_wins_even_if_another_key_is_offered()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = TestDir::new()?;
        let known = dir.store();
        fs::create_dir_all(known.path().parent().ok_or("parent")?)?;
        fs::write(
            known.path(),
            format!("@revoked nas.local ssh-ed25519 {ED25519}\n"),
        )?;
        let other = key("ssh-rsa", "cnNhIGtleQ==");
        assert!(matches!(
            check(
                &known,
                "nas.local",
                22,
                &[key("ssh-ed25519", ED25519), other]
            )?,
            Trust::Revoked { .. }
        ));
        Ok(())
    }

    #[test]
    fn tofu_states_are_distinct_and_changed_never_becomes_unknown()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = TestDir::new()?;
        let known = dir.store();
        let original = key("ssh-ed25519", ED25519);
        let replacement = key(
            "ssh-ed25519",
            "AAAAC3NzaC1lZDI1NTE5AAAAIAAAAAAAAAAAAAAAAAAA",
        );

        assert!(matches!(
            check(&known, "nas.local", 22, std::slice::from_ref(&original))?,
            Trust::Unknown { .. }
        ));
        known.trust(&original)?;
        assert_eq!(
            check(&known, "nas.local", 22, std::slice::from_ref(&original))?,
            Trust::Trusted
        );
        let changed = check(&known, "nas.local", 22, std::slice::from_ref(&replacement))?;
        assert!(matches!(changed, Trust::Changed { .. }));
        assert!(matches!(
            known.trust(&replacement),
            Err(HostKeysError::ChangedKey { .. })
        ));
        known.forget("nas.local", 22)?;
        known.trust(&replacement)?;
        Ok(())
    }

    #[test]
    fn no_offered_keys_is_an_error() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = TestDir::new()?;
        match check(&dir.store(), "nas.local", 22, &[]) {
            Err(HostKeysError::NoOfferedKeys { .. }) => Ok(()),
            other => Err(format!("expected no-offered-keys error, got {other:?}").into()),
        }
    }

    #[test]
    fn fields_cannot_inject_a_second_known_hosts_line()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = TestDir::new()?;
        let mut key = key("ssh-ed25519", ED25519);
        key.host = "nas.local\nother.local".to_owned();
        assert!(matches!(
            dir.store().trust(&key),
            Err(HostKeysError::InvalidField { field: "host" })
        ));
        Ok(())
    }

    #[test]
    fn forget_leaves_other_hosts_intact() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let dir = TestDir::new()?;
        let known = dir.store();
        known.trust(&key("ssh-ed25519", ED25519))?;
        let mut other = key("ssh-ed25519", ED25519);
        other.host = "other.local".to_owned();
        known.trust(&other)?;
        known.forget("nas.local", 22)?;
        assert!(known.stored("nas.local", 22)?.is_empty());
        assert_eq!(known.stored("other.local", 22)?.len(), 1);
        Ok(())
    }
}
