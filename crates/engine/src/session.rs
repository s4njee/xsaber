//! SSH session state and events shared by the headless engine and UI.
//!
//! This module deliberately contains no connection setup.  The actor that
//! owns a russh handle will use [`SessionState`] as its small, thread-safe
//! state machine and [`SessionEvents`] as its UI-facing event bus.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::{RwLock, broadcast};

/// Defaults for one SSH connection.
///
/// These values are kept in the engine rather than in the shell so a CLI and
/// the desktop app use the same transport policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionConfig {
    /// Enable TCP_NODELAY on the SSH socket.
    pub nodelay: bool,
    /// Initial SSH channel window.
    pub window_size: u32,
    /// Interval between SSH keepalive packets.
    pub keepalive_interval: Duration,
    /// Number of unanswered keepalives before disconnecting.
    pub keepalive_max: usize,
    /// Optional inactivity timeout.  `None` means the session is not
    /// collected merely because it is idle.
    pub inactivity_timeout: Option<Duration>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            nodelay: true,
            window_size: 8 * 1024 * 1024,
            keepalive_interval: Duration::from_secs(30),
            keepalive_max: 3,
            inactivity_timeout: None,
        }
    }
}

impl SessionConfig {
    /// Converts the engine policy into russh's client configuration.
    #[must_use]
    pub fn russh_config(&self) -> russh::client::Config {
        russh::client::Config {
            nodelay: self.nodelay,
            window_size: self.window_size,
            keepalive_interval: Some(self.keepalive_interval),
            keepalive_max: self.keepalive_max,
            inactivity_timeout: self.inactivity_timeout,
            ..russh::client::Config::default()
        }
    }
}

/// A step in the SSH handshake log.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectingStep {
    /// DNS/address resolution completed.
    ResolvedHost { host: String, port: u16 },
    /// The TCP socket was established.
    TcpEstablished,
    /// The peer's SSH identification string was received.
    Server(String),
    /// Key exchange completed with the named algorithm.
    KeyExchange(String),
    /// The host key is being checked against trust policy.
    VerifyingHostKey,
    /// Authentication is in progress.
    Authenticating,
}

/// A host-key prompt emitted while a connection is waiting for a trust
/// decision.  The actual trust policy is supplied by the later host-key
/// story; keeping the event type here lets the actor remain headless.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostKeyPrompt {
    /// Key algorithm, for example `ssh-ed25519`.
    pub algorithm: String,
    /// Display fingerprint, normally in `SHA256:` form.
    pub fingerprint: String,
}

/// Events emitted by one SSH session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    /// A handshake step and the elapsed time since connection attempt start.
    Connecting {
        step: ConnectingStep,
        elapsed: Duration,
    },
    /// The host key needs a user decision.
    HostKey(HostKeyPrompt),
    /// Authentication completed successfully.
    Authenticated,
    /// The transport is unhealthy, but a reconnect supervisor may recover it.
    Degraded,
    /// The session has stopped and will not emit more lifecycle events.
    Disconnected(String),
}

/// A bounded broadcast bus for session events.
#[derive(Clone, Debug)]
pub struct SessionEvents {
    sender: broadcast::Sender<Event>,
}

impl Default for SessionEvents {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionEvents {
    /// Creates a bus with enough capacity for a handshake and normal
    /// lifecycle transitions without allowing a stalled UI to grow memory.
    #[must_use]
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(64);
        Self { sender }
    }

    /// Subscribes to future events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Publishes an event.  A bus with no subscribers is still valid; the
    /// receiver count is intentionally not part of engine state.
    pub fn send(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    /// Returns the number of active subscribers, useful for diagnostics and
    /// tests without exposing the sender itself.
    #[must_use]
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Lifecycle state of a session actor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionState {
    /// The transport handshake or authentication is still in progress.
    Connecting,
    /// Authentication completed and channels may be opened.
    Authenticated,
    /// The transport dropped unexpectedly and may be reconnected.
    Degraded,
    /// The actor has stopped permanently.
    Disconnected { reason: String },
}

/// Thread-safe state shared by a session actor and its consumers.
#[derive(Clone, Debug)]
pub struct SessionState {
    state: Arc<RwLock<ConnectionState>>,
    events: SessionEvents,
    started: Instant,
}

impl SessionState {
    /// Creates a state machine in the connecting state.
    #[must_use]
    pub fn new(events: SessionEvents) -> Self {
        Self {
            state: Arc::new(RwLock::new(ConnectionState::Connecting)),
            events,
            started: Instant::now(),
        }
    }

    /// Returns the current state without exposing the lock.
    pub async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// Returns the elapsed time since this state machine was created.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// Publishes a handshake step with a monotonic elapsed timestamp.
    pub fn connecting(&self, step: ConnectingStep) {
        self.events.send(Event::Connecting {
            step,
            elapsed: self.elapsed(),
        });
    }

    /// Moves the state to authenticated and publishes the matching event.
    pub async fn authenticate(&self) {
        *self.state.write().await = ConnectionState::Authenticated;
        self.events.send(Event::Authenticated);
    }

    /// Marks the connection unhealthy and publishes a degraded event.
    pub async fn degrade(&self) {
        let mut state = self.state.write().await;
        if !matches!(*state, ConnectionState::Disconnected { .. }) {
            *state = ConnectionState::Degraded;
            self.events.send(Event::Degraded);
        }
    }

    /// Permanently stops the session and publishes one terminal event.
    pub async fn disconnect(&self, reason: impl Into<String>) {
        let reason = reason.into();
        let mut state = self.state.write().await;
        if matches!(*state, ConnectionState::Disconnected { .. }) {
            return;
        }
        *state = ConnectionState::Disconnected {
            reason: reason.clone(),
        };
        self.events.send(Event::Disconnected(reason));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConnectingStep, ConnectionState, Event, HostKeyPrompt, SessionConfig, SessionEvents,
        SessionState,
    };
    use std::time::Duration;

    #[test]
    fn defaults_match_the_session_contract() {
        let config = SessionConfig::default();
        assert!(config.nodelay);
        assert_eq!(config.window_size, 8 * 1024 * 1024);
        assert_eq!(config.keepalive_interval, Duration::from_secs(30));
        assert_eq!(config.keepalive_max, 3);
        assert_eq!(config.inactivity_timeout, None);

        let russh = config.russh_config();
        assert!(russh.nodelay);
        assert_eq!(russh.window_size, 8 * 1024 * 1024);
        assert_eq!(russh.keepalive_interval, Some(Duration::from_secs(30)));
        assert_eq!(russh.keepalive_max, 3);
        assert_eq!(russh.inactivity_timeout, None);
    }

    #[tokio::test]
    async fn events_carry_elapsed_connecting_times_and_lifecycle_state() {
        let events = SessionEvents::new();
        let mut receiver = events.subscribe();
        let state = SessionState::new(events);

        state.connecting(ConnectingStep::ResolvedHost {
            host: "example.test".to_owned(),
            port: 22,
        });
        state.authenticate().await;
        state.disconnect("test complete").await;

        let connecting = receiver.recv().await;
        assert!(matches!(connecting, Ok(Event::Connecting { .. })));
        if let Ok(Event::Connecting { elapsed, .. }) = connecting {
            assert!(elapsed <= state.elapsed());
        }
        assert!(receiver.recv().await.is_ok());
        assert!(receiver.recv().await.is_ok());
        assert_eq!(
            state.state().await,
            ConnectionState::Disconnected {
                reason: "test complete".to_owned(),
            }
        );
    }

    #[tokio::test]
    async fn disconnect_is_terminal_and_degrade_does_not_resurrect_it() {
        let events = SessionEvents::new();
        let mut receiver = events.subscribe();
        let state = SessionState::new(events);

        state.disconnect("closed").await;
        state.degrade().await;
        state.disconnect("second reason").await;

        assert_eq!(
            state.state().await,
            ConnectionState::Disconnected {
                reason: "closed".to_owned(),
            }
        );
        assert!(matches!(
            receiver.recv().await,
            Ok(Event::Disconnected(reason)) if reason == "closed"
        ));
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn event_types_are_owned_and_comparable() {
        let event = Event::HostKey(HostKeyPrompt {
            algorithm: "ssh-ed25519".to_owned(),
            fingerprint: "SHA256:example".to_owned(),
        });
        assert_eq!(event.clone(), event);
    }
}
