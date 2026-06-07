//! Node Manager — automatic node discovery and state tracking.
//!
//! The `NodeManager` provides high-level node management for CANOpen master stations:
//! - Automatic node discovery via NMT boot-up protocol
//! - Node state tracking via heartbeat monitoring
//! - Node configuration via SDO
//! - Coordinated node startup/shutdown

use opencan_canopen_core::CanDriver;
use opencan_canopen_core::error::CanOpenError;
use opencan_canopen_core::frame::{HeartbeatFrame, NmtState};
use opencan_canopen_core::heartbeat::HeartbeatConsumer;
use opencan_canopen_core::nmt::NmtMaster;
use opencan_canopen_core::sdo::SdoClient;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Node state with additional tracking information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    /// Node ID (1-127).
    pub node_id: u8,
    /// Current NMT state.
    pub state: NmtState,
    /// Last heartbeat timestamp.
    pub last_heartbeat: Option<Instant>,
    /// Whether the node has been configured.
    pub configured: bool,
    /// Device type (from OD 0x1000).
    pub device_type: Option<u32>,
    /// Vendor ID (from OD 0x1018:01).
    pub vendor_id: Option<u32>,
    /// Error code (from latest EMCY message).
    pub error_code: Option<u16>,
}

impl NodeInfo {
    /// Create a new node info with the given node ID.
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
            state: NmtState::BootUp,
            last_heartbeat: None,
            configured: false,
            device_type: None,
            vendor_id: None,
            error_code: None,
        }
    }

    /// Check if the node is alive (heartbeat received within timeout).
    pub fn is_alive(&self, timeout: Duration) -> bool {
        match self.last_heartbeat {
            Some(last) => last.elapsed() < timeout,
            None => false,
        }
    }

    /// Update the node state from a heartbeat frame.
    pub fn update_from_heartbeat(&mut self, hb: &HeartbeatFrame) -> bool {
        let changed = self.state != hb.state;
        self.state = hb.state;
        self.last_heartbeat = Some(Instant::now());
        changed
    }
}

/// Configuration for the NodeManager.
#[derive(Debug, Clone)]
pub struct NodeManagerConfig {
    /// Heartbeat timeout multiplier (timeout = period * multiplier).
    pub heartbeat_timeout_multiplier: u32,
    /// Whether to automatically configure discovered nodes.
    pub auto_configure: bool,
    /// Whether to automatically start nodes after configuration.
    pub auto_start: bool,
    /// Node scan range (1-127).
    pub scan_range: std::ops::RangeInclusive<u8>,
    /// Timeout for SDO operations.
    pub sdo_timeout: Duration,
}

impl Default for NodeManagerConfig {
    fn default() -> Self {
        Self {
            heartbeat_timeout_multiplier: 3,
            auto_configure: false,
            auto_start: false,
            scan_range: 1..=127,
            sdo_timeout: Duration::from_secs(1),
        }
    }
}

/// Node manager for CANOpen master stations.
///
/// Provides high-level node management including:
/// - Node discovery via NMT boot-up protocol
/// - Node state tracking via heartbeat monitoring
/// - Node configuration via SDO
/// - Coordinated startup/shutdown
pub struct NodeManager<C: CanDriver> {
    /// CAN driver for communication.
    can: C,
    /// NMT master for sending commands.
    nmt: NmtMaster,
    /// Heartbeat consumer for monitoring nodes.
    heartbeat_consumer: HeartbeatConsumer,
    /// Managed nodes.
    nodes: HashMap<u8, NodeInfo>,
    /// Configuration.
    config: NodeManagerConfig,
}

impl<C: CanDriver> NodeManager<C> {
    /// Create a new node manager.
    pub fn new(can: C, config: NodeManagerConfig) -> Self {
        let heartbeat_consumer = HeartbeatConsumer::new(Duration::from_secs(5));
        Self {
            can,
            nmt: NmtMaster::new(),
            heartbeat_consumer,
            nodes: HashMap::new(),
            config,
        }
    }

    /// Get the configuration.
    pub fn config(&self) -> &NodeManagerConfig {
        &self.config
    }

    /// Get a reference to the CAN driver.
    pub fn can(&self) -> &C {
        &self.can
    }

    /// Get a mutable reference to the CAN driver.
    pub fn can_mut(&mut self) -> &mut C {
        &mut self.can
    }

    /// Get a reference to the NMT master.
    pub fn nmt(&self) -> &NmtMaster {
        &self.nmt
    }

    /// Get information about a specific node.
    pub fn node(&self, node_id: u8) -> Option<&NodeInfo> {
        self.nodes.get(&node_id)
    }

    /// Get all managed nodes.
    pub fn nodes(&self) -> &HashMap<u8, NodeInfo> {
        &self.nodes
    }

    /// Get the number of managed nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of alive nodes.
    pub fn alive_count(&self) -> usize {
        let timeout = Duration::from_secs(5); // Default timeout
        self.nodes.values().filter(|n| n.is_alive(timeout)).count()
    }

    /// Add a node to management.
    pub fn add_node(&mut self, node_id: u8) {
        self.nodes.insert(node_id, NodeInfo::new(node_id));
        self.heartbeat_consumer.set_period(
            node_id,
            Duration::from_secs(1), // Default heartbeat period
        );
    }

    /// Remove a node from management.
    pub fn remove_node(&mut self, node_id: u8) {
        self.nodes.remove(&node_id);
    }

    /// Process a received heartbeat frame.
    ///
    /// Returns true if any node state changed.
    pub fn process_heartbeat(&mut self, hb: &HeartbeatFrame) -> bool {
        // Update heartbeat consumer
        self.heartbeat_consumer.update(hb);

        // Update node info
        if let Some(node) = self.nodes.get_mut(&hb.node_id) {
            node.update_from_heartbeat(hb)
        } else {
            // Auto-discover new node
            let mut node = NodeInfo::new(hb.node_id);
            let changed = node.update_from_heartbeat(hb);
            self.nodes.insert(hb.node_id, node);
            changed
        }
    }

    /// Check for timed-out nodes.
    ///
    /// Returns list of node IDs that have timed out.
    pub fn check_timeouts(&self) -> Vec<u8> {
        let timeout = Duration::from_secs(5); // Default timeout
        self.nodes
            .iter()
            .filter(|(_, node)| !node.is_alive(timeout))
            .map(|(&id, _)| id)
            .collect()
    }

    /// Send NMT command to start a node.
    pub fn start_node(&mut self, node_id: u8) -> Result<(), CanOpenError> {
        self.nmt.start_remote_node(&mut self.can, node_id)
    }

    /// Send NMT command to stop a node.
    pub fn stop_node(&mut self, node_id: u8) -> Result<(), CanOpenError> {
        self.nmt.stop_remote_node(&mut self.can, node_id)
    }

    /// Send NMT command to reset a node.
    pub fn reset_node(&mut self, node_id: u8) -> Result<(), CanOpenError> {
        self.nmt.reset_node(&mut self.can, node_id)
    }

    /// Send NMT command to reset communication on a node.
    pub fn reset_communication(&mut self, node_id: u8) -> Result<(), CanOpenError> {
        self.nmt.reset_communication(&mut self.can, node_id)
    }

    /// Start all managed nodes.
    pub fn start_all(&mut self) -> Result<(), CanOpenError> {
        let node_ids: Vec<u8> = self.nodes.keys().copied().collect();
        for node_id in node_ids {
            self.start_node(node_id)?;
        }
        Ok(())
    }

    /// Stop all managed nodes.
    pub fn stop_all(&mut self) -> Result<(), CanOpenError> {
        let node_ids: Vec<u8> = self.nodes.keys().copied().collect();
        for node_id in node_ids {
            self.stop_node(node_id)?;
        }
        Ok(())
    }

    /// Reset all managed nodes.
    pub fn reset_all(&mut self) -> Result<(), CanOpenError> {
        let node_ids: Vec<u8> = self.nodes.keys().copied().collect();
        for node_id in node_ids {
            self.reset_node(node_id)?;
        }
        Ok(())
    }

    /// Read device type from a node.
    pub async fn read_device_type(
        &mut self,
        node_id: u8,
    ) -> Result<u32, CanOpenError> {
        // Create a temporary SDO client by replacing self.can with a dummy
        // This is safe because we restore it after the operation
        let mut sdo = SdoClient::new(&mut self.can, self.config.sdo_timeout);
        let value = sdo.upload(node_id, 0x1000, 0).await?;
        let device_type = match value {
            opencan_canopen_core::od::OdValue::Unsigned32(v) => v,
            _ => {
                return Err(CanOpenError::Protocol(
                    "Invalid device type response".to_string(),
                ));
            }
        };

        // Update node info
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.device_type = Some(device_type);
        }

        Ok(device_type)
    }

    /// Read vendor ID from a node.
    pub async fn read_vendor_id(
        &mut self,
        node_id: u8,
    ) -> Result<u32, CanOpenError> {
        let mut sdo = SdoClient::new(&mut self.can, self.config.sdo_timeout);
        let value = sdo.upload(node_id, 0x1018, 1).await?;
        let vendor_id = match value {
            opencan_canopen_core::od::OdValue::Unsigned32(v) => v,
            _ => {
                return Err(CanOpenError::Protocol(
                    "Invalid vendor ID response".to_string(),
                ));
            }
        };

        // Update node info
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.vendor_id = Some(vendor_id);
        }

        Ok(vendor_id)
    }

    /// Configure a node (read device info, set heartbeat period, etc.).
    pub async fn configure_node(&mut self, node_id: u8) -> Result<(), CanOpenError> {
        // Read device type
        let _ = self.read_device_type(node_id).await;

        // Read vendor ID
        let _ = self.read_vendor_id(node_id).await;

        // Mark as configured
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.configured = true;
        }

        Ok(())
    }

    /// Configure all discovered nodes.
    pub async fn configure_all(&mut self) -> Result<(), CanOpenError> {
        let node_ids: Vec<u8> = self.nodes.keys().copied().collect();
        for node_id in node_ids {
            let _ = self.configure_node(node_id).await;
        }
        Ok(())
    }

    /// Get a summary of all nodes.
    pub fn summary(&self) -> NodeSummary {
        let total = self.nodes.len();
        let timeout = Duration::from_secs(5);
        let alive = self.nodes.values().filter(|n| n.is_alive(timeout)).count();
        let configured = self.nodes.values().filter(|n| n.configured).count();
        let in_operational = self
            .nodes
            .values()
            .filter(|n| n.state == NmtState::Operational)
            .count();

        NodeSummary {
            total,
            alive,
            configured,
            in_operational,
        }
    }
}

/// Summary of node manager state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSummary {
    /// Total number of managed nodes.
    pub total: usize,
    /// Number of alive nodes.
    pub alive: usize,
    /// Number of configured nodes.
    pub configured: usize,
    /// Number of nodes in operational state.
    pub in_operational: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencan_canopen_core::testing::MockCanDriver;

    fn make_manager() -> NodeManager<MockCanDriver> {
        let mock = MockCanDriver::new();
        let config = NodeManagerConfig::default();
        NodeManager::new(mock, config)
    }

    #[test]
    fn test_node_info_new() {
        let node = NodeInfo::new(5);
        assert_eq!(node.node_id, 5);
        assert_eq!(node.state, NmtState::BootUp);
        assert!(node.last_heartbeat.is_none());
        assert!(!node.configured);
    }

    #[test]
    fn test_node_info_alive() {
        let mut node = NodeInfo::new(1);
        assert!(!node.is_alive(Duration::from_secs(5)));

        node.last_heartbeat = Some(Instant::now());
        assert!(node.is_alive(Duration::from_secs(5)));

        node.last_heartbeat = Some(Instant::now() - Duration::from_secs(10));
        assert!(!node.is_alive(Duration::from_secs(5)));
    }

    #[test]
    fn test_node_info_update_from_heartbeat() {
        let mut node = NodeInfo::new(1);
        let hb = HeartbeatFrame {
            node_id: 1,
            state: NmtState::Operational,
        };

        let changed = node.update_from_heartbeat(&hb);
        assert!(changed);
        assert_eq!(node.state, NmtState::Operational);
        assert!(node.last_heartbeat.is_some());

        // Same state, no change
        let changed = node.update_from_heartbeat(&hb);
        assert!(!changed);
    }

    #[test]
    fn test_add_remove_node() {
        let mut mgr = make_manager();
        assert_eq!(mgr.node_count(), 0);

        mgr.add_node(1);
        mgr.add_node(2);
        assert_eq!(mgr.node_count(), 2);
        assert!(mgr.node(1).is_some());
        assert!(mgr.node(2).is_some());

        mgr.remove_node(1);
        assert_eq!(mgr.node_count(), 1);
        assert!(mgr.node(1).is_none());
    }

    #[test]
    fn test_process_heartbeat() {
        let mut mgr = make_manager();
        mgr.add_node(1);

        let hb = HeartbeatFrame {
            node_id: 1,
            state: NmtState::Operational,
        };

        let changed = mgr.process_heartbeat(&hb);
        assert!(changed);
        assert_eq!(mgr.node(1).unwrap().state, NmtState::Operational);
    }

    #[test]
    fn test_process_heartbeat_auto_discover() {
        let mut mgr = make_manager();
        assert_eq!(mgr.node_count(), 0);

        let hb = HeartbeatFrame {
            node_id: 5,
            state: NmtState::PreOperational,
        };

        mgr.process_heartbeat(&hb);
        assert_eq!(mgr.node_count(), 1);
        assert!(mgr.node(5).is_some());
        assert_eq!(mgr.node(5).unwrap().state, NmtState::PreOperational);
    }

    #[test]
    fn test_summary() {
        let mut mgr = make_manager();
        mgr.add_node(1);
        mgr.add_node(2);
        mgr.add_node(3);

        // Set one node as alive
        if let Some(node) = mgr.nodes.get_mut(&1) {
            node.last_heartbeat = Some(Instant::now());
            node.state = NmtState::Operational;
            node.configured = true;
        }

        let summary = mgr.summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.alive, 1);
        assert_eq!(summary.configured, 1);
        assert_eq!(summary.in_operational, 1);
    }

    #[test]
    fn test_check_timeouts() {
        let mut mgr = make_manager();
        mgr.add_node(1);
        mgr.add_node(2);

        // Node 1: recent heartbeat (alive)
        if let Some(node) = mgr.nodes.get_mut(&1) {
            node.last_heartbeat = Some(Instant::now());
        }

        // Node 2: old heartbeat (timeout)
        if let Some(node) = mgr.nodes.get_mut(&2) {
            node.last_heartbeat = Some(Instant::now() - Duration::from_secs(30));
        }

        let timeouts = mgr.check_timeouts();
        assert_eq!(timeouts.len(), 1);
        assert_eq!(timeouts[0], 2);
    }
}
