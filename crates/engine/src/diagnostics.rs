use std::{
    collections::VecDeque,
    env, fmt, io,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
    },
};

use tracing::{Event, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter,
    layer::{Context, Layer},
    prelude::*,
};

const MAX_MESSAGES: usize = 256;
const DEFAULT_FILTER: &str = "info";

/// One event made available to a future in-app message log.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogMessage {
    level: String,
    target: String,
    message: String,
}

impl LogMessage {
    fn new(level: &str, target: &str, message: String) -> Self {
        Self {
            level: level.to_owned(),
            target: target.to_owned(),
            message,
        }
    }

    /// Returns the event level, such as `INFO` or `WARN`.
    pub fn level(&self) -> &str {
        &self.level
    }

    /// Returns the event target/module.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Returns the already-redacted event message and fields.
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Default)]
struct DiagnosticsState {
    messages: Mutex<VecDeque<LogMessage>>,
    subscribers: Mutex<Vec<SyncSender<LogMessage>>>,
}

/// Bounded diagnostics storage and subscription channel for the app shell.
///
/// The engine does not depend on GPUI or an app-specific message type. E10-S3
/// can consume [`Diagnostics::subscribe`] on its own UI channel, while
/// [`Diagnostics::snapshot`] supports initial population of that view.
#[derive(Clone, Default)]
pub struct Diagnostics {
    state: Arc<DiagnosticsState>,
}

impl Diagnostics {
    /// Creates an empty diagnostics stream.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the most recent messages, oldest first.
    pub fn snapshot(&self) -> Vec<LogMessage> {
        let messages = match self.state.messages.lock() {
            Ok(messages) => messages,
            Err(poisoned) => poisoned.into_inner(),
        };
        messages.iter().cloned().collect()
    }

    /// Subscribes to future messages.
    ///
    /// The channel is bounded so a UI that stops polling cannot grow the
    /// engine's memory without limit. Messages are dropped for a full receiver;
    /// the in-memory snapshot remains available in either case.
    pub fn subscribe(&self) -> Receiver<LogMessage> {
        let (sender, receiver) = sync_channel(64);
        let mut subscribers = match self.state.subscribers.lock() {
            Ok(subscribers) => subscribers,
            Err(poisoned) => poisoned.into_inner(),
        };
        subscribers.push(sender);
        receiver
    }

    fn record(&self, message: LogMessage) {
        {
            let mut messages = match self.state.messages.lock() {
                Ok(messages) => messages,
                Err(poisoned) => poisoned.into_inner(),
            };
            if messages.len() >= MAX_MESSAGES {
                messages.pop_front();
            }
            messages.push_back(message.clone());
        }

        let mut subscribers = match self.state.subscribers.lock() {
            Ok(subscribers) => subscribers,
            Err(poisoned) => poisoned.into_inner(),
        };
        subscribers.retain(|sender| match sender.try_send(message.clone()) {
            Ok(()) | Err(TrySendError::Full(_)) => true,
            Err(TrySendError::Disconnected(_)) => false,
        });
    }
}

/// Owns the rolling file worker and the diagnostics stream installed by
/// [`init_logging`]. Keep this guard alive for the process lifetime so buffered
/// records are flushed when the app exits.
pub struct LoggingGuard {
    file_guard: WorkerGuard,
    diagnostics: Diagnostics,
    log_directory: PathBuf,
}

impl LoggingGuard {
    /// Returns the diagnostics stream attached to this logging setup.
    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics.clone()
    }

    /// Returns the directory containing the daily rolling log files.
    pub fn log_directory(&self) -> &Path {
        &self.log_directory
    }
}

impl fmt::Debug for LoggingGuard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoggingGuard")
            .field("diagnostics", &self.diagnostics.snapshot().len())
            .field("log_directory", &self.log_directory)
            .finish_non_exhaustive()
    }
}

impl Drop for LoggingGuard {
    fn drop(&mut self) {
        // Referencing the guard makes the ownership contract explicit. The
        // worker itself flushes when this field is dropped after this method.
        let _ = &self.file_guard;
    }
}

/// Errors raised while preparing process-wide logging.
#[derive(Debug)]
pub enum LoggingError {
    /// The app-data directory could not be created.
    CreateLogDirectory { path: PathBuf, source: io::Error },
    /// Another caller has already installed a global subscriber.
    AlreadyInitialized,
    /// The `XSABER_LOG` directive was invalid.
    InvalidFilter(String),
}

impl fmt::Display for LoggingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateLogDirectory { path, source } => {
                write!(
                    formatter,
                    "cannot create log directory {}: {source}",
                    path.display()
                )
            }
            Self::AlreadyInitialized => formatter.write_str("logging is already initialized"),
            Self::InvalidFilter(filter) => {
                write!(formatter, "invalid XSABER_LOG filter {filter:?}")
            }
        }
    }
}

impl std::error::Error for LoggingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreateLogDirectory { source, .. } => Some(source),
            Self::AlreadyInitialized | Self::InvalidFilter(_) => None,
        }
    }
}

/// Installs a global tracing subscriber with a daily rolling file.
///
/// Logs are written below `<app_data_dir>/logs/xsaber.log.*`. The
/// `XSABER_LOG` environment variable accepts normal `tracing-subscriber`
/// directives, so `XSABER_LOG=debug` raises verbosity without a rebuild.
pub fn init_logging(app_data_dir: impl AsRef<Path>) -> Result<LoggingGuard, LoggingError> {
    let log_directory = app_data_dir.as_ref().join("logs");
    std::fs::create_dir_all(&log_directory).map_err(|source| LoggingError::CreateLogDirectory {
        path: log_directory.clone(),
        source,
    })?;

    let filter = configured_filter()?;
    let appender = tracing_appender::rolling::daily(&log_directory, "xsaber.log");
    let (non_blocking, file_guard) = tracing_appender::non_blocking(appender);
    let diagnostics = Diagnostics::new();
    let diagnostics_layer = DiagnosticsLayer {
        diagnostics: diagnostics.clone(),
    };
    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking);
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(diagnostics_layer);

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| LoggingError::AlreadyInitialized)?;

    Ok(LoggingGuard {
        file_guard,
        diagnostics,
        log_directory,
    })
}

fn configured_filter() -> Result<EnvFilter, LoggingError> {
    configured_filter_value(env::var("XSABER_LOG").ok().as_deref())
}

fn configured_filter_value(value: Option<&str>) -> Result<EnvFilter, LoggingError> {
    let directive = match value {
        Some(value) if !value.trim().is_empty() => value.to_owned(),
        _ => DEFAULT_FILTER.to_owned(),
    };
    EnvFilter::try_new(directive.clone()).map_err(|_| LoggingError::InvalidFilter(directive))
}

struct DiagnosticsLayer {
    diagnostics: Diagnostics,
}

impl<S> Layer<S> for DiagnosticsLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let message = match visitor.message {
            Some(message) if visitor.fields.is_empty() => message,
            Some(message) => format!("{message} ({})", visitor.fields.join(", ")),
            None => visitor.fields.join(", "),
        };
        if message.is_empty() {
            return;
        }
        self.diagnostics.record(LogMessage::new(
            event.metadata().level().as_str(),
            event.metadata().target(),
            message,
        ));
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: Option<String>,
    fields: Vec<String>,
}

impl MessageVisitor {
    fn add_field(&mut self, field: &tracing::field::Field, value: impl fmt::Display) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.push(format!("{}={value}", field.name()));
        }
    }
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        self.add_field(field, format_args!("{value:?}"));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.add_field(field, value);
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.add_field(field, value);
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.add_field(field, value);
    }

    fn record_i128(&mut self, field: &tracing::field::Field, value: i128) {
        self.add_field(field, value);
    }

    fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
        self.add_field(field, value);
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.add_field(field, value);
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.add_field(field, value);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{
            Arc, Mutex,
            atomic::{AtomicU64, Ordering},
        },
        time::{Duration, Instant},
    };

    use tracing_subscriber::layer::Context;

    use super::*;
    use crate::Secret;

    struct SpanCapture {
        values: Arc<Mutex<Vec<String>>>,
    }

    impl<S> Layer<S> for SpanCapture
    where
        S: Subscriber,
    {
        fn on_new_span(
            &self,
            attributes: &tracing::span::Attributes<'_>,
            _id: &tracing::span::Id,
            _context: Context<'_, S>,
        ) {
            let mut visitor = MessageVisitor::default();
            attributes.record(&mut visitor);
            let mut values = match self.values.lock() {
                Ok(values) => values,
                Err(poisoned) => poisoned.into_inner(),
            };
            values.extend(visitor.fields);
        }
    }

    #[test]
    fn every_secret_kind_is_redacted_in_span_fields() {
        let values = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::registry().with(SpanCapture {
            values: values.clone(),
        });

        tracing::subscriber::with_default(subscriber, || {
            let password = Secret::password("correct horse battery staple");
            let passphrase = Secret::passphrase("a very private phrase");
            let key_material = Secret::key_material("-----BEGIN PRIVATE KEY-----");
            let span = tracing::info_span!(
                "credentials",
                password = ?password,
                passphrase = ?passphrase,
                key_material = ?key_material,
            );
            let _entered = span.enter();
        });

        let values = match values.lock() {
            Ok(values) => values,
            Err(poisoned) => poisoned.into_inner(),
        };
        let rendered = values.join(" ");
        assert!(rendered.contains("password=<redacted>"));
        assert!(rendered.contains("passphrase=<redacted>"));
        assert!(rendered.contains("key_material=<redacted>"));
        assert!(!rendered.contains("correct horse battery staple"));
        assert!(!rendered.contains("a very private phrase"));
        assert!(!rendered.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn diagnostics_retains_a_bounded_snapshot_and_streams_messages() {
        let diagnostics = Diagnostics::new();
        let receiver = diagnostics.subscribe();
        diagnostics.record(LogMessage::new("INFO", "test", "connected".to_owned()));

        let received = receiver.recv().ok();
        assert_eq!(
            received.as_ref().map(LogMessage::message),
            Some("connected")
        );
        assert_eq!(diagnostics.snapshot().len(), 1);
    }

    #[test]
    fn xsaber_log_debug_raises_verbosity_without_mutating_process_environment()
    -> Result<(), LoggingError> {
        let default_diagnostics = Diagnostics::new();
        let debug_diagnostics = Diagnostics::new();

        let default_subscriber = tracing_subscriber::registry()
            .with(configured_filter_value(None)?)
            .with(DiagnosticsLayer {
                diagnostics: default_diagnostics.clone(),
            });
        tracing::subscriber::with_default(default_subscriber, || {
            tracing::debug!(target: "xsaber::filter_test", "debug hidden");
            tracing::info!(target: "xsaber::filter_test", "info visible");
        });

        let debug_subscriber = tracing_subscriber::registry()
            .with(configured_filter_value(Some("debug"))?)
            .with(DiagnosticsLayer {
                diagnostics: debug_diagnostics.clone(),
            });
        tracing::subscriber::with_default(debug_subscriber, || {
            tracing::debug!(target: "xsaber::filter_test", "debug visible");
            tracing::info!(target: "xsaber::filter_test", "info visible");
        });

        let default_messages = default_diagnostics.snapshot();
        assert!(
            default_messages
                .iter()
                .all(|message| !message.message().contains("debug hidden"))
        );
        assert!(
            default_messages
                .iter()
                .any(|message| message.message().contains("info visible"))
        );

        let debug_messages = debug_diagnostics.snapshot();
        assert!(
            debug_messages
                .iter()
                .any(|message| message.message().contains("debug visible"))
        );
        assert!(
            debug_messages
                .iter()
                .any(|message| message.message().contains("info visible"))
        );
        Ok(())
    }

    #[test]
    fn rolling_logs_are_created_under_app_data_logs() -> Result<(), Box<dyn std::error::Error>> {
        let app_data = unique_test_directory("rolling-logs")?;
        let guard = init_logging(&app_data)?;
        tracing::info!(target: "xsaber::file_test", "rolling file probe");

        // Dropping the non-blocking guard flushes records before the directory
        // is inspected. The short retry also accommodates filesystem latency.
        drop(guard);
        let log_directory = app_data.join("logs");
        let deadline = Instant::now() + Duration::from_secs(2);
        let log_file = loop {
            let entries = fs::read_dir(&log_directory)?;
            if let Some(entry) = entries.flatten().find(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("xsaber.log.")
            }) {
                break entry.path();
            }
            assert!(
                Instant::now() < deadline,
                "rolling log file was not created"
            );
            std::thread::yield_now();
        };

        let contents = fs::read_to_string(&log_file)?;
        assert!(contents.contains("rolling file probe"));
        fs::remove_dir_all(app_data)?;
        Ok(())
    }

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    fn unique_test_directory(label: &str) -> std::io::Result<std::path::PathBuf> {
        let path = std::env::temp_dir().join(format!(
            "xsaber-{label}-{}-{}",
            std::process::id(),
            NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path)?;
        Ok(path)
    }
}
