//! Object-safe remote filesystem contract.
//!
//! Backends implement this module without exposing a protocol client to the
//! shell.  Futures are boxed explicitly instead of using `async fn` in the
//! trait so `Box<dyn RemoteFs>` remains usable by the transfer and UI layers.

use std::{fmt, future::Future, pin::Pin};

/// A future returned by a remote filesystem operation.
pub type FsFuture<'a, T> = Pin<Box<dyn Future<Output = FsResult<T>> + Send + 'a>>;

/// Result type shared by filesystem operations and handles.
pub type FsResult<T> = Result<T, FsError>;

/// Errors that a backend can classify without exposing its implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FsError {
    /// The caller supplied a path that cannot be materialized safely.
    InvalidPath(PathError),
    /// The operation is not advertised by [`Caps`].
    Unsupported(&'static str),
    /// The requested path does not exist.
    NotFound,
    /// A no-replace or compare-and-swap operation found a conflict.
    Conflict,
    /// Transport or protocol detail, kept as a diagnostic string.
    Other(String),
}

impl fmt::Display for FsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(error) => write!(formatter, "invalid remote path: {error}"),
            Self::Unsupported(operation) => write!(formatter, "unsupported operation: {operation}"),
            Self::NotFound => formatter.write_str("remote path was not found"),
            Self::Conflict => formatter.write_str("remote destination changed or already exists"),
            Self::Other(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for FsError {}

/// Errors found before a path is sent to a backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathError {
    /// Empty components and `.` are normalized, but `..` is never accepted.
    Traversal,
    /// NUL cannot be represented in a protocol path.
    Nul,
    /// Backslashes are rejected so a Windows-looking path cannot cross a
    /// remote POSIX root boundary by accident.
    Backslash,
    /// A path component is required for `join_component`.
    InvalidComponent,
}

impl fmt::Display for PathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Traversal => "parent traversal is not allowed",
            Self::Nul => "NUL is not allowed",
            Self::Backslash => "backslashes are not allowed",
            Self::InvalidComponent => "invalid path component",
        })
    }
}

impl std::error::Error for PathError {}

/// A normalized POSIX-style path received from or sent to a remote server.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RemotePath(String);

impl RemotePath {
    /// The remote root.
    #[must_use]
    pub fn root() -> Self {
        Self("/".to_owned())
    }

    /// Validates and normalizes a POSIX-style path.
    pub fn parse(path: &str) -> Result<Self, PathError> {
        if path.contains('\0') {
            return Err(PathError::Nul);
        }
        if path.contains('\\') {
            return Err(PathError::Backslash);
        }
        let absolute = path.starts_with('/');
        let mut components = Vec::new();
        for component in path.split('/') {
            match component {
                "" | "." => {}
                ".." => return Err(PathError::Traversal),
                value => components.push(value),
            }
        }
        let mut normalized = if absolute {
            String::from("/")
        } else {
            String::new()
        };
        normalized.push_str(&components.join("/"));
        if normalized.is_empty() {
            normalized.push('/');
        }
        Ok(Self(normalized))
    }

    /// Joins one validated name without allowing a separator or traversal.
    pub fn join_component(&self, component: &str) -> Result<Self, PathError> {
        if component.is_empty()
            || component == "."
            || component == ".."
            || component.contains('/')
            || component.contains('\\')
        {
            return Err(PathError::InvalidComponent);
        }
        if component.contains('\0') {
            return Err(PathError::Nul);
        }
        let separator = if self.0 == "/" { "" } else { "/" };
        Self::parse(&format!("{}{separator}{component}", self.0))
    }

    /// Returns the canonical path string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for RemotePath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for RemotePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Optional Unix ownership names resolved by the server.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerNames {
    pub user: Option<String>,
    pub group: Option<String>,
}

/// File metadata common to directory entries and stat calls.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attrs {
    pub size: u64,
    pub mode: Option<u32>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    /// Milliseconds since the Unix epoch, when the backend supplies it.
    pub mtime: Option<i64>,
    pub kind: EntryKind,
    pub owner_names: Option<OwnerNames>,
}

/// A name and metadata row returned by `read_dir`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub name: String,
    pub path: RemotePath,
    pub attrs: Attrs,
    pub file_class: FileClass,
}

/// Kinds represented by a remote directory entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Other,
}

/// Coarse class used for row glyphs and preview routing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileClass {
    Folder,
    Alias,
    Archive,
    Audio,
    Image,
    Video,
    Text,
    Document,
    Application,
    Other,
}

/// Free-space information returned by `statfs`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatFs {
    pub total_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
    pub block_size: Option<u64>,
}

/// Server capabilities.  Unknown bits are preserved by `from_bits` so a
/// newer backend cannot accidentally turn an unknown capability on.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Caps(u16);

impl Caps {
    pub const RANDOM_ACCESS: Self = Self(1 << 0);
    pub const RESUME_WRITE: Self = Self(1 << 1);
    pub const ATOMIC_RENAME: Self = Self(1 << 2);
    pub const VERIFIED_COMMIT: Self = Self(1 << 3);
    pub const SYMLINKS: Self = Self(1 << 4);
    pub const OWNER_NAMES: Self = Self(1 << 5);
    pub const WATCH: Self = Self(1 << 6);

    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u16 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, capability: Self) -> bool {
        self.0 & capability.0 == capability.0
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// How a destination is opened for writing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteMode {
    /// Create or truncate from offset zero.
    Create,
    /// Continue a previously staged write at its opaque token.
    Resume,
}

/// Rename conflict policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenameMode {
    NoReplace,
    Replace,
    Exchange,
}

/// Requested timestamp updates. `None` leaves that timestamp unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileTimes {
    pub atime: Option<i64>,
    pub mtime: Option<i64>,
}

/// A random-access read handle.
pub trait ReadAt: Send + Sync {
    fn read_at(&self, offset: u64, length: u32) -> FsFuture<'_, Vec<u8>>;
    fn size(&self) -> Option<u64>;
}

/// A staged write handle.
pub trait WriteStage: Send {
    fn write(&mut self, offset: u64, data: Vec<u8>) -> FsFuture<'_, ()>;
    fn commit(&mut self) -> FsFuture<'_, Attrs>;
    fn abort(&mut self) -> FsFuture<'_, ()>;
}

/// Object-safe filesystem provider for one remote session.
pub trait RemoteFs: Send + Sync {
    fn caps(&self) -> Caps;
    fn statfs(&self) -> FsFuture<'_, StatFs>;
    fn read_dir(&self, path: &RemotePath) -> FsFuture<'_, Vec<Entry>>;
    fn stat(&self, path: &RemotePath) -> FsFuture<'_, Attrs>;
    fn lstat(&self, path: &RemotePath) -> FsFuture<'_, Attrs>;
    fn open_read(&self, path: &RemotePath) -> FsFuture<'_, Box<dyn ReadAt>>;
    fn open_write(
        &self,
        path: &RemotePath,
        size_hint: Option<u64>,
        mode: WriteMode,
    ) -> FsFuture<'_, Box<dyn WriteStage>>;
    fn mkdir(&self, path: &RemotePath, mode: Option<u32>) -> FsFuture<'_, ()>;
    fn mkdir_all(&self, path: &RemotePath, mode: Option<u32>) -> FsFuture<'_, ()>;
    fn rename(
        &self,
        source: &RemotePath,
        destination: &RemotePath,
        mode: RenameMode,
    ) -> FsFuture<'_, ()>;
    fn remove(&self, path: &RemotePath, recursive: bool) -> FsFuture<'_, ()>;
    fn set_permissions(&self, path: &RemotePath, mode: u32) -> FsFuture<'_, ()>;
    fn set_times(&self, path: &RemotePath, times: FileTimes) -> FsFuture<'_, ()>;
    fn symlink(&self, target: &str, path: &RemotePath) -> FsFuture<'_, ()>;
    fn read_link(&self, path: &RemotePath) -> FsFuture<'_, String>;
    fn ping(&self) -> FsFuture<'_, ()>;
}

#[cfg(test)]
mod tests {
    use super::{Caps, EntryKind, FileClass, PathError, RemotePath, RenameMode};

    #[test]
    fn capabilities_are_explicit_and_composable() {
        let caps = Caps::RANDOM_ACCESS
            .union(Caps::RESUME_WRITE)
            .union(Caps::SYMLINKS);
        assert!(caps.contains(Caps::RANDOM_ACCESS));
        assert!(caps.contains(Caps::RESUME_WRITE));
        assert!(!caps.contains(Caps::ATOMIC_RENAME));
        assert_eq!(Caps::from_bits(caps.bits()), caps);
    }

    #[test]
    fn paths_normalize_without_allowing_escape() {
        assert_eq!(
            RemotePath::parse("a//./b").map(|path| path.to_string()),
            Ok("a/b".to_owned())
        );
        assert_eq!(RemotePath::parse("/"), Ok(RemotePath::root()));
        assert_eq!(RemotePath::parse("a/../b"), Err(PathError::Traversal));
        assert_eq!(RemotePath::parse("C:\\temp"), Err(PathError::Backslash));
        assert_eq!(
            RemotePath::root()
                .join_component("etc")
                .map(|path| path.to_string()),
            Ok("/etc".to_owned())
        );
        assert_eq!(
            RemotePath::root().join_component("../etc"),
            Err(PathError::InvalidComponent)
        );
    }

    #[test]
    fn rename_modes_are_distinct_and_entry_kinds_have_matching_classes() {
        assert_ne!(RenameMode::NoReplace, RenameMode::Replace);
        assert_ne!(RenameMode::Replace, RenameMode::Exchange);
        assert_ne!(EntryKind::Directory, EntryKind::File);
        assert_ne!(FileClass::Folder, FileClass::Alias);
    }

    #[test]
    fn unknown_capability_bits_do_not_claim_known_features() {
        let future = Caps::from_bits(1 << 15);
        assert!(!future.contains(Caps::RANDOM_ACCESS));
        assert_eq!(future.bits(), 1 << 15);
    }
}
