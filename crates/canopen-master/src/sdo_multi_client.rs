//! SDO Multi-Client Manager — manages multiple SDO client sessions.
//!
//! This module provides multi-client SDO support, allowing multiple
//! concurrent SDO sessions with different nodes.

use opencan_canopen_core::sdo::SdoClient;
use opencan_canopen_core::CanDriver;
use std::collections::HashMap;
use std::time::Duration;

/// SDO client session information.
#[derive(Debug, Clone)]
pub struct SdoClientSession {
    /// Target node ID.
    pub node_id: u8,
    /// Session timeout.
    pub timeout: Duration,
    /// Whether the session is active.
    pub active: bool,
    /// Last activity timestamp.
    pub last_activity: Option<std::time::Instant>,
    /// Total operations performed.
    pub operation_count: u64,
}

/// SDO multi-client manager.
///
/// Manages multiple SDO client sessions for concurrent communication
/// with different CANOpen nodes.
pub struct SdoMultiClient<C: CanDriver> {
    /// SDO clients per node.
    clients: HashMap<u8, SdoClient<C>>,
    /// Session information.
    sessions: HashMap<u8, SdoClientSession>,
    /// Default timeout for new sessions.
    default_timeout: Duration,
    /// Maximum concurrent sessions.
    max_sessions: usize,
}

impl<C: CanDriver> SdoMultiClient<C> {
    /// Create a new multi-client manager.
    pub fn new(default_timeout: Duration) -> Self {
        Self {
            clients: HashMap::new(),
            sessions: HashMap::new(),
            default_timeout,
            max_sessions: 8,
        }
    }

    /// Create with custom settings.
    pub fn with_settings(default_timeout: Duration, max_sessions: usize) -> Self {
        Self {
            clients: HashMap::new(),
            sessions: HashMap::new(),
            default_timeout,
            max_sessions,
        }
    }

    /// Add a new SDO client for a node.
    pub fn add_client(&mut self, node_id: u8, can: C) -> Result<(), SdoMultiClientError> {
        if self.clients.len() >= self.max_sessions {
            return Err(SdoMultiClientError::MaxSessionsReached(self.max_sessions));
        }

        if self.clients.contains_key(&node_id) {
            return Err(SdoMultiClientError::SessionAlreadyExists(node_id));
        }

        let client = SdoClient::new(can, self.default_timeout);
        self.clients.insert(node_id, client);
        self.sessions.insert(
            node_id,
            SdoClientSession {
                node_id,
                timeout: self.default_timeout,
                active: true,
                last_activity: None,
                operation_count: 0,
            },
        );

        Ok(())
    }

    /// Remove an SDO client for a node.
    pub fn remove_client(&mut self, node_id: u8) {
        self.clients.remove(&node_id);
        self.sessions.remove(&node_id);
    }

    /// Get a reference to an SDO client.
    pub fn client(&self, node_id: u8) -> Option<&SdoClient<C>> {
        self.clients.get(&node_id)
    }

    /// Get a mutable reference to an SDO client.
    pub fn client_mut(&mut self, node_id: u8) -> Option<&mut SdoClient<C>> {
        self.clients.get_mut(&node_id)
    }

    /// Get session information for a node.
    pub fn session(&self, node_id: u8) -> Option<&SdoClientSession> {
        self.sessions.get(&node_id)
    }

    /// Get all active session node IDs.
    pub fn active_sessions(&self) -> Vec<u8> {
        self.sessions
            .iter()
            .filter(|(_, session)| session.active)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get the number of active sessions.
    pub fn session_count(&self) -> usize {
        self.clients.len()
    }

    /// Update session activity.
    fn update_activity(&mut self, node_id: u8) {
        if let Some(session) = self.sessions.get_mut(&node_id) {
            session.last_activity = Some(std::time::Instant::now());
            session.operation_count += 1;
        }
    }

    /// Perform an SDO upload (read from remote node).
    pub async fn upload(
        &mut self,
        node_id: u8,
        index: u16,
        subindex: u8,
    ) -> Result<opencan_canopen_core::od::OdValue, SdoMultiClientError> {
        let client = self.clients.get_mut(&node_id).ok_or({
            SdoMultiClientError::SessionNotFound(node_id)
        })?;

        let result = client.upload(node_id, index, subindex).await;
        self.update_activity(node_id);
        result.map_err(SdoMultiClientError::SdoError)
    }

    /// Perform an SDO download (write to remote node).
    pub async fn download(
        &mut self,
        node_id: u8,
        index: u16,
        subindex: u8,
        value: &opencan_canopen_core::od::OdValue,
    ) -> Result<(), SdoMultiClientError> {
        let client = self.clients.get_mut(&node_id).ok_or({
            SdoMultiClientError::SessionNotFound(node_id)
        })?;

        let result = client.download(node_id, index, subindex, value).await;
        self.update_activity(node_id);
        result.map_err(SdoMultiClientError::SdoError)
    }

    /// Reset a client session.
    pub fn reset_session(&mut self, node_id: u8) {
        if let Some(session) = self.sessions.get_mut(&node_id) {
            session.active = true;
            session.last_activity = None;
            session.operation_count = 0;
        }
    }

    /// Get the default timeout.
    pub fn default_timeout(&self) -> Duration {
        self.default_timeout
    }

    /// Set the timeout for a specific session.
    pub fn set_session_timeout(&mut self, node_id: u8, timeout: Duration) {
        if let Some(session) = self.sessions.get_mut(&node_id) {
            session.timeout = timeout;
        }
    }
}

/// SDO multi-client error types.
#[derive(Debug)]
pub enum SdoMultiClientError {
    /// Session not found for the specified node.
    SessionNotFound(u8),
    /// Session already exists for the specified node.
    SessionAlreadyExists(u8),
    /// Maximum number of sessions reached.
    MaxSessionsReached(usize),
    /// SDO error from the underlying client.
    SdoError(opencan_canopen_core::CanOpenError),
}

impl std::fmt::Display for SdoMultiClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionNotFound(node_id) => {
                write!(f, "SDO session not found for node {}", node_id)
            }
            Self::SessionAlreadyExists(node_id) => {
                write!(f, "SDO session already exists for node {}", node_id)
            }
            Self::MaxSessionsReached(max) => {
                write!(f, "Maximum number of SDO sessions reached: {}", max)
            }
            Self::SdoError(e) => write!(f, "SDO error: {}", e),
        }
    }
}

impl std::error::Error for SdoMultiClientError {}

#[cfg(test)]
mod tests {
    use super::*;
    use opencan_canopen_core::testing::MockCanDriver;

    #[test]
    fn test_multi_client_add_remove() {
        let mut manager = SdoMultiClient::<MockCanDriver>::new(Duration::from_secs(1));

        manager.add_client(1, MockCanDriver::new()).unwrap();
        manager.add_client(2, MockCanDriver::new()).unwrap();

        assert_eq!(manager.session_count(), 2);
        assert!(manager.client(1).is_some());
        assert!(manager.client(2).is_some());

        manager.remove_client(1);
        assert_eq!(manager.session_count(), 1);
        assert!(manager.client(1).is_none());
    }

    #[test]
    fn test_multi_client_max_sessions() {
        let mut manager = SdoMultiClient::<MockCanDriver>::with_settings(
            Duration::from_secs(1),
            2,
        );

        manager.add_client(1, MockCanDriver::new()).unwrap();
        manager.add_client(2, MockCanDriver::new()).unwrap();

        let result = manager.add_client(3, MockCanDriver::new());
        assert!(matches!(result, Err(SdoMultiClientError::MaxSessionsReached(2))));
    }

    #[test]
    fn test_multi_client_duplicate_session() {
        let mut manager = SdoMultiClient::<MockCanDriver>::new(Duration::from_secs(1));

        manager.add_client(1, MockCanDriver::new()).unwrap();

        let result = manager.add_client(1, MockCanDriver::new());
        assert!(matches!(result, Err(SdoMultiClientError::SessionAlreadyExists(1))));
    }

    #[test]
    fn test_multi_client_session_info() {
        let mut manager = SdoMultiClient::<MockCanDriver>::new(Duration::from_secs(1));

        manager.add_client(1, MockCanDriver::new()).unwrap();
        manager.add_client(2, MockCanDriver::new()).unwrap();

        let session = manager.session(1).unwrap();
        assert_eq!(session.node_id, 1);
        assert!(session.active);

        let active = manager.active_sessions();
        assert_eq!(active.len(), 2);
        assert!(active.contains(&1));
        assert!(active.contains(&2));
    }

    #[test]
    fn test_multi_client_error_display() {
        let err = SdoMultiClientError::SessionNotFound(5);
        assert!(!err.to_string().is_empty());

        let err = SdoMultiClientError::SessionAlreadyExists(5);
        assert!(!err.to_string().is_empty());

        let err = SdoMultiClientError::MaxSessionsReached(8);
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_multi_client_settings() {
        let manager = SdoMultiClient::<MockCanDriver>::with_settings(
            Duration::from_secs(2),
            16,
        );

        assert_eq!(manager.default_timeout(), Duration::from_secs(2));
    }
}
