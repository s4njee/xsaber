//! Small, side-effect-free OpenSSH configuration policy.
//!
//! This is the engine-owned boundary around SSH configuration. It parses the
//! fields xsaber can use for a direct connection and resolves a `Host` alias
//! without reading the environment or executing a command. `ProxyCommand` is
//! retained as data for the later russh stream adapter; this module never
//! invokes it.
//!
//! The complete v1 parser is intentionally not reproduced here. In
//! particular, `ProxyJump` and `Match` are rejected with a field-level error
//! instead of being silently ignored. The parent integration still needs to
//! wire the mandated `russh-config::parse_home` path before this is presented
//! as full `~/.ssh/config` support.

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// The port used when a matching `Host` block does not provide one.
pub const DEFAULT_SSH_PORT: u16 = 22;

/// A parsed `Host` block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostBlock {
    /// Patterns from the `Host` directive, in source order.
    pub patterns: Vec<String>,
    /// One-based source line for diagnostics.
    pub line: usize,
    /// Optional `HostName` override.
    pub host_name: Option<String>,
    /// Optional `User` override.
    pub user: Option<String>,
    /// Optional `Port` override.
    pub port: Option<u16>,
    /// `IdentityFile` values, in source order.
    pub identity_files: Vec<String>,
    /// `ProxyCommand` text, retained but never executed here.
    pub proxy_command: Option<String>,
}

impl HostBlock {
    fn new(patterns: Vec<String>, line: usize) -> Self {
        Self {
            patterns,
            line,
            host_name: None,
            user: None,
            port: None,
            identity_files: Vec::new(),
            proxy_command: None,
        }
    }

    fn matches(&self, alias: &str) -> bool {
        matches_patterns(&self.patterns, alias)
    }
}

/// Parsed SSH configuration. Parsing has no filesystem or environment side
/// effects; use [`SshConfig::from_path`] only when the caller explicitly
/// chooses a path.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SshConfig {
    blocks: Vec<HostBlock>,
}

impl SshConfig {
    /// Parses a configuration string.
    pub fn parse(text: &str) -> Result<Self, SshConfigError> {
        let mut blocks = Vec::new();
        let mut current: Option<HostBlock> = None;

        for (line_index, raw_line) in text.lines().enumerate() {
            let line = line_index + 1;
            let line_text = strip_comment(raw_line);
            let trimmed = line_text.trim();
            if trimmed.is_empty() {
                continue;
            }

            let (keyword, rest) = split_keyword(trimmed)
                .ok_or_else(|| SshConfigError::invalid(line, "directive", "missing keyword"))?;
            let keyword_lower = keyword.to_ascii_lowercase();

            if keyword_lower == "host" {
                let patterns = words(rest, line, "Host")?;
                if patterns.is_empty() {
                    return Err(SshConfigError::invalid(line, "Host", "requires a pattern"));
                }
                if let Some(previous) = current.take() {
                    blocks.push(previous);
                }
                current = Some(HostBlock::new(patterns, line));
                continue;
            }

            if keyword_lower == "match" {
                return Err(SshConfigError::unsupported(
                    line,
                    "Match",
                    "Match blocks are unsupported in xsaber v1",
                ));
            }
            if keyword_lower == "proxyjump" {
                return Err(SshConfigError::unsupported(
                    line,
                    "ProxyJump",
                    "ProxyJump is unsupported in xsaber v1",
                ));
            }

            // Directives before the first Host belong to the implicit global
            // block. OpenSSH applies those values to every alias.
            let block = current.get_or_insert_with(|| HostBlock::new(vec!["*".to_owned()], line));
            match keyword_lower.as_str() {
                "hostname" => set_once(
                    &mut block.host_name,
                    Some(single_word(rest, line, "HostName")?),
                ),
                "user" => set_once(&mut block.user, Some(single_word(rest, line, "User")?)),
                "port" => {
                    let value = single_word(rest, line, "Port")?;
                    let port = value.parse::<u16>().map_err(|_| {
                        SshConfigError::invalid(line, "Port", "must be a number from 1 to 65535")
                    })?;
                    if port == 0 {
                        return Err(SshConfigError::invalid(
                            line,
                            "Port",
                            "must be a number from 1 to 65535",
                        ));
                    }
                    if block.port.is_none() {
                        block.port = Some(port);
                    }
                }
                "identityfile" => {
                    block
                        .identity_files
                        .push(single_word(rest, line, "IdentityFile")?)
                }
                "proxycommand" => {
                    let command = rest.trim();
                    if command.is_empty() {
                        return Err(SshConfigError::invalid(
                            line,
                            "ProxyCommand",
                            "requires a command",
                        ));
                    }
                    set_once(&mut block.proxy_command, Some(command.to_owned()));
                }
                // Unknown directives are left to russh-config/the transport
                // adapter. Ignoring them is safer than inventing semantics.
                _ => {}
            }
        }

        if let Some(last) = current {
            blocks.push(last);
        }
        Ok(Self { blocks })
    }

    /// Reads and parses an explicitly supplied config path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, SshConfigError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| SshConfigError::io(path, source))?;
        Self::parse(&text)
    }

    /// Returns parsed blocks in source order.
    #[must_use]
    pub fn blocks(&self) -> &[HostBlock] {
        &self.blocks
    }

    /// Resolves an alias using OpenSSH's first-value-wins block ordering.
    ///
    /// Matching uses the supported `*` and `?` patterns and honors negated
    /// patterns (`!name`). Unknown pattern features are treated literally,
    /// which errs toward a failed match rather than a surprising connection.
    pub fn resolve(&self, alias: &str) -> ResolvedSshConfig {
        let mut resolved = ResolvedSshConfig {
            alias: alias.to_owned(),
            host: alias.to_owned(),
            user: None,
            port: DEFAULT_SSH_PORT,
            identity_files: Vec::new(),
            proxy_command: None,
        };
        let mut host_name_set = false;
        let mut user_set = false;
        let mut port_set = false;
        let mut proxy_command_set = false;

        for block in &self.blocks {
            if !block.matches(alias) {
                continue;
            }
            if !host_name_set && let Some(host_name) = &block.host_name {
                resolved.host = host_name.clone();
                host_name_set = true;
            }
            if !user_set && let Some(user) = &block.user {
                resolved.user = Some(user.clone());
                user_set = true;
            }
            if !port_set && let Some(port) = block.port {
                resolved.port = port;
                port_set = true;
            }
            if resolved.identity_files.is_empty() {
                resolved
                    .identity_files
                    .extend(block.identity_files.iter().cloned());
            }
            if !proxy_command_set && let Some(command) = &block.proxy_command {
                resolved.proxy_command = Some(command.clone());
                proxy_command_set = true;
            }
        }
        resolved
    }
}

/// The effective connection fields for one alias.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedSshConfig {
    /// Alias supplied by the caller.
    pub alias: String,
    /// Hostname or address to dial after `HostName` resolution.
    pub host: String,
    /// Optional remote user.
    pub user: Option<String>,
    /// SSH port, defaulting to 22.
    pub port: u16,
    /// Identity files accumulated from matching blocks.
    pub identity_files: Vec<String>,
    /// Proxy command text, never executed by this module.
    pub proxy_command: Option<String>,
}

/// Configuration parsing and policy errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SshConfigError {
    /// The explicitly selected file could not be read.
    Io { path: PathBuf, message: String },
    /// A directive is malformed.
    Invalid {
        line: usize,
        field: String,
        message: String,
    },
    /// A recognized but unsupported field was used.
    Unsupported {
        line: usize,
        field: String,
        message: String,
    },
}

impl SshConfigError {
    fn io(path: &Path, source: io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            message: source.to_string(),
        }
    }

    fn invalid(line: usize, field: &str, message: &str) -> Self {
        Self::Invalid {
            line,
            field: field.to_owned(),
            message: message.to_owned(),
        }
    }

    fn unsupported(line: usize, field: &str, message: &str) -> Self {
        Self::Unsupported {
            line,
            field: field.to_owned(),
            message: message.to_owned(),
        }
    }

    /// Returns the config field responsible for this error, when applicable.
    #[must_use]
    pub fn field(&self) -> Option<&str> {
        match self {
            Self::Io { .. } => None,
            Self::Invalid { field, .. } | Self::Unsupported { field, .. } => Some(field),
        }
    }

    /// Returns the one-based source line, when applicable.
    #[must_use]
    pub fn line(&self) -> Option<usize> {
        match self {
            Self::Io { .. } => None,
            Self::Invalid { line, .. } | Self::Unsupported { line, .. } => Some(*line),
        }
    }
}

impl fmt::Display for SshConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(formatter, "{}: {message}", path.display()),
            Self::Invalid {
                line,
                field,
                message,
            } => write!(formatter, "line {line}: {field}: {message}"),
            Self::Unsupported {
                line,
                field,
                message,
            } => write!(formatter, "line {line}: {field}: {message}"),
        }
    }
}

impl std::error::Error for SshConfigError {}

fn set_once<T>(slot: &mut Option<T>, value: Option<T>) {
    if slot.is_none() {
        *slot = value;
    }
}

fn split_keyword(line: &str) -> Option<(&str, &str)> {
    let mut split = line.splitn(2, char::is_whitespace);
    let keyword = split.next()?.trim();
    let rest = split.next().unwrap_or_default().trim();
    (!keyword.is_empty()).then_some((keyword, rest))
}

fn single_word(rest: &str, line: usize, field: &str) -> Result<String, SshConfigError> {
    let values = words(rest, line, field)?;
    match values.as_slice() {
        [value] if !value.is_empty() => Ok(value.clone()),
        [] => Err(SshConfigError::invalid(line, field, "requires a value")),
        _ => Err(SshConfigError::invalid(
            line,
            field,
            "requires exactly one value",
        )),
    }
}

fn words(value: &str, line: usize, field: &str) -> Result<Vec<String>, SshConfigError> {
    let mut output = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;

    for character in value.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        match (quote, character) {
            (_, '\\') => escaped = true,
            (Some(active), character) if active == character => quote = None,
            (Some(_), character) => current.push(character),
            (None, '\'' | '"') => quote = Some(character),
            (None, character) if character.is_whitespace() => {
                if !current.is_empty() {
                    output.push(std::mem::take(&mut current));
                }
            }
            (None, character) => current.push(character),
        }
    }
    if escaped || quote.is_some() {
        return Err(SshConfigError::invalid(
            line,
            field,
            "unterminated quote or escape",
        ));
    }
    if !current.is_empty() {
        output.push(current);
    }
    Ok(output)
}

fn strip_comment(line: &str) -> String {
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match (quote, character) {
            (_, '\\') => escaped = true,
            (Some(active), character) if active == character => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(character),
            (None, '#') => return line[..index].to_owned(),
            (None, _) => {}
        }
    }
    line.to_owned()
}

fn matches_patterns(patterns: &[String], alias: &str) -> bool {
    let alias = alias.to_ascii_lowercase();
    let mut positive = false;
    let mut matched = false;
    for pattern in patterns {
        let (negated, pattern) = pattern
            .strip_prefix('!')
            .map_or((false, pattern.as_str()), |pattern| (true, pattern));
        if pattern.is_empty() {
            continue;
        }
        let this_matches = wildcard_match(&pattern.to_ascii_lowercase(), &alias);
        if negated && this_matches {
            return false;
        }
        if !negated {
            positive = true;
            matched |= this_matches;
        }
    }
    if positive { matched } else { true }
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let value: Vec<char> = value.chars().collect();
    let mut states = vec![false; value.len() + 1];
    states[0] = true;
    for token in pattern {
        let mut next = vec![false; value.len() + 1];
        match token {
            '*' => {
                let mut reachable = false;
                for index in 0..=value.len() {
                    reachable |= states[index];
                    next[index] = reachable;
                }
            }
            '?' => {
                next[1..(value.len() + 1)].copy_from_slice(&states[..value.len()]);
            }
            literal => {
                for index in 0..value.len() {
                    if states[index] && value[index] == literal {
                        next[index + 1] = true;
                    }
                }
            }
        }
        states = next;
    }
    states[value.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_resolves_minimal_host_blocks() -> Result<(), Box<dyn std::error::Error>> {
        let config = SshConfig::parse(
            "Host *\n  User deploy\n  IdentityFile ~/.ssh/id_ed25519\n\nHost staging\n  HostName node-04.internal\n  Port 2222\n  ProxyCommand connect-proxy %h %p\n",
        )?;
        let resolved = config.resolve("staging");
        assert_eq!(resolved.alias, "staging");
        assert_eq!(resolved.host, "node-04.internal");
        assert_eq!(resolved.user.as_deref(), Some("deploy"));
        assert_eq!(resolved.port, 2222);
        assert_eq!(resolved.identity_files, vec!["~/.ssh/id_ed25519"]);
        assert_eq!(
            resolved.proxy_command.as_deref(),
            Some("connect-proxy %h %p")
        );
        Ok(())
    }

    #[test]
    fn first_matching_value_wins_and_identity_files_accumulate()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = SshConfig::parse(
            "Host *\n  User global\n  IdentityFile global.key\nHost db-*\n  User local\n  IdentityFile local.key\n",
        )?;
        let resolved = config.resolve("db-one");
        assert_eq!(resolved.user.as_deref(), Some("global"));
        assert_eq!(resolved.identity_files, vec!["global.key"]);
        Ok(())
    }

    #[test]
    fn proxy_jump_is_a_field_level_unsupported_error() -> Result<(), Box<dyn std::error::Error>> {
        let error = match SshConfig::parse("Host prod\n  ProxyJump bastion\n") {
            Err(error) => error,
            Ok(_) => return Err("ProxyJump unexpectedly parsed".into()),
        };
        assert_eq!(error.field(), Some("ProxyJump"));
        assert_eq!(error.line(), Some(2));
        assert!(error.to_string().contains("unsupported"));
        Ok(())
    }

    #[test]
    fn match_is_a_field_level_unsupported_error() -> Result<(), Box<dyn std::error::Error>> {
        let error = match SshConfig::parse("Match exec true\n  User root\n") {
            Err(error) => error,
            Ok(_) => return Err("Match unexpectedly parsed".into()),
        };
        assert_eq!(error.field(), Some("Match"));
        assert_eq!(error.line(), Some(1));
        Ok(())
    }

    #[test]
    fn quoted_values_and_inline_comments_are_supported() -> Result<(), Box<dyn std::error::Error>> {
        let config = SshConfig::parse(
            "Host 'build host' # alias\n  HostName \"builder.internal\"\n  User 'build user'\n",
        )?;
        let resolved = config.resolve("build host");
        assert_eq!(resolved.host, "builder.internal");
        assert_eq!(resolved.user.as_deref(), Some("build user"));
        Ok(())
    }

    #[test]
    fn negated_and_wildcard_patterns_are_honored() -> Result<(), Box<dyn std::error::Error>> {
        let config = SshConfig::parse("Host * !ignored\n  User operator\n")?;
        assert_eq!(config.resolve("server").user.as_deref(), Some("operator"));
        assert_eq!(config.resolve("ignored").user, None);
        Ok(())
    }

    #[test]
    fn invalid_ports_are_rejected_with_their_field() -> Result<(), Box<dyn std::error::Error>> {
        let error = match SshConfig::parse("Host server\n  Port 0\n") {
            Err(error) => error,
            Ok(_) => return Err("invalid port unexpectedly parsed".into()),
        };
        assert_eq!(error.field(), Some("Port"));
        assert_eq!(error.line(), Some(2));
        Ok(())
    }

    #[test]
    fn unknown_directives_are_ignored_without_environment_access()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = SshConfig::parse("Host server\n  SendEnv LANG\n  User admin\n")?;
        assert_eq!(config.resolve("server").user.as_deref(), Some("admin"));
        Ok(())
    }
}
