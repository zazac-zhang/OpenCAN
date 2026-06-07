//! Heartbeat Monitor — enhanced heartbeat monitoring with timeout handling.
//!
//! The `HeartbeatMonitor` provides advanced heartbeat monitoring features:
//! - Configurable timeout per node
//! - State change notifications
//! - Statistics tracking
//! - Automatic recovery detection

use opencan_canopen_core::frame::{HeartbeatFrame, NmtState};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Heartbeat event types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeartbeatEvent {
    /// Node state changed.
    StateChanged {
        node_id: u8,
        old_state: NmtState,
        new_state: NmtState,
    },
    /// Node timed out (no heartbeat received within timeout).
    Timeout {
        node_id: u8,
        elapsed: Duration,
    },
    /// Node recovered after timeout.
    Recovered {
        node_id: u8,
        state: NmtState,
    },
    /// New node discovered.
    Discovered {
        node_id: u8,
        state: NmtState,
    },
}

/// Node heartbeat statistics.
#[derive(Debug, Clone, Default)]
pub struct HeartbeatStats {
    /// Total heartbeats received.
    pub received_count: u64,
    /// Number of state changes.
    pub state_change_count: u64,
    /// Number of timeouts detected.
    pub timeout_count: u64,
    /// Last heartbeat timestamp.
    pub last_heartbeat: Option<Instant>,
    /// Last state change timestamp.
    pub last_state_change: Option<Instant>,
    /// Minimum heartbeat interval observed.
    pub min_interval: Option<Duration>,
    /// Maximum heartbeat interval observed.
    pub max_interval: Option<Duration>,
    /// Average heartbeat interval.
    pub avg_interval: Option<Duration>,
}

impl HeartbeatStats {
    /// Update statistics with a new heartbeat.
    fn update(&mut self, now: Instant) {
        if let Some(last) = self.last_heartbeat {
            let interval = now.duration_since(last);
            self.min_interval = Some(match self.min_interval {
                Some(min) => min.min(interval),
                None => interval,
            });
            self.max_interval = Some(match self.max_interval {
                Some(max) => max.max(interval),
                None => interval,
            });
            // Simple moving average
            let count = self.received_count as f64;
            let current_avg = self.avg_interval.unwrap_or(Duration::ZERO).as_secs_f64();
            let new_avg = (current_avg * count + interval.as_secs_f64()) / (count + 1.0);
            self.avg_interval = Some(Duration::from_secs_f64(new_avg));
        }
        self.last_heartbeat = Some(now);
        self.received_count += 1;
    }

    /// Record a state change.
    fn record_state_change(&mut self, now: Instant) {
        self.state_change_count += 1;
        self.last_state_change = Some(now);
    }

    /// Record a timeout.
    fn record_timeout(&mut self) {
        self.timeout_count += 1;
    }
}

/// Configuration for a monitored node.
#[derive(Debug, Clone)]
pub struct NodeMonitorConfig {
    /// Expected heartbeat period.
    pub period: Duration,
    /// Timeout multiplier (timeout = period * multiplier).
    pub timeout_multiplier: u32,
    /// Whether to track statistics.
    pub track_stats: bool,
}

impl Default for NodeMonitorConfig {
    fn default() -> Self {
        Self {
            period: Duration::from_secs(1),
            timeout_multiplier: 3,
            track_stats: true,
        }
    }
}

/// Monitored node state.
#[derive(Debug, Clone)]
struct MonitoredNode {
    /// Current NMT state.
    state: NmtState,
    /// Last heartbeat timestamp.
    last_heartbeat: Option<Instant>,
    /// Monitor configuration.
    config: NodeMonitorConfig,
    /// Statistics.
    stats: HeartbeatStats,
    /// Whether the node is currently timed out.
    timed_out: bool,
}

impl MonitoredNode {
    fn new(config: NodeMonitorConfig) -> Self {
        Self {
            state: NmtState::BootUp,
            last_heartbeat: None,
            config,
            stats: HeartbeatStats::default(),
            timed_out: false,
        }
    }

    /// Get the timeout duration.
    fn timeout(&self) -> Duration {
        self.config.period * self.config.timeout_multiplier
    }

    /// Check if the node is alive.
    fn is_alive(&self) -> bool {
        match self.last_heartbeat {
            Some(last) => last.elapsed() < self.timeout(),
            None => false,
        }
    }
}

/// Enhanced heartbeat monitor with event tracking and statistics.
///
/// Provides advanced heartbeat monitoring features beyond the basic
/// `HeartbeatConsumer` in canopen-core.
pub struct HeartbeatMonitor {
    /// Monitored nodes.
    nodes: HashMap<u8, MonitoredNode>,
    /// Event queue.
    events: Vec<HeartbeatEvent>,
    /// Default configuration for new nodes.
    default_config: NodeMonitorConfig,
    /// Whether to auto-discover new nodes from heartbeats.
    auto_discover: bool,
}

impl HeartbeatMonitor {
    /// Create a new heartbeat monitor.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            events: Vec::new(),
            default_config: NodeMonitorConfig::default(),
            auto_discover: true,
        }
    }

    /// Create a new heartbeat monitor with custom default config.
    pub fn with_config(default_config: NodeMonitorConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            events: Vec::new(),
            default_config,
            auto_discover: true,
        }
    }

    /// Set whether to auto-discover new nodes.
    pub fn set_auto_discover(&mut self, auto_discover: bool) {
        self.auto_discover = auto_discover;
    }

    /// Add a node to monitor with default configuration.
    pub fn add_node(&mut self, node_id: u8) {
        self.nodes.insert(
            node_id,
            MonitoredNode::new(self.default_config.clone()),
        );
    }

    /// Add a node to monitor with custom configuration.
    pub fn add_node_with_config(&mut self, node_id: u8, config: NodeMonitorConfig) {
        self.nodes.insert(node_id, MonitoredNode::new(config));
    }

    /// Remove a node from monitoring.
    pub fn remove_node(&mut self, node_id: u8) {
        self.nodes.remove(&node_id);
    }

    /// Get the number of monitored nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the current state of a node.
    pub fn node_state(&self, node_id: u8) -> Option<NmtState> {
        self.nodes.get(&node_id).map(|n| n.state)
    }

    /// Check if a node is alive.
    pub fn is_alive(&self, node_id: u8) -> bool {
        self.nodes.get(&node_id).map_or(false, |n| n.is_alive())
    }

    /// Get all alive node IDs.
    pub fn alive_nodes(&self) -> Vec<u8> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.is_alive())
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get all timed-out node IDs.
    pub fn timed_out_nodes(&self) -> Vec<u8> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.timed_out)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get statistics for a node.
    pub fn stats(&self, node_id: u8) -> Option<&HeartbeatStats> {
        self.nodes.get(&node_id).map(|n| &n.stats)
    }

    /// Process a received heartbeat frame.
    ///
    /// Returns a list of events generated by this heartbeat.
    pub fn process_heartbeat(&mut self, hb: &HeartbeatFrame) -> Vec<HeartbeatEvent> {
        let now = Instant::now();
        let mut events = Vec::new();

        // Get or create the node
        let node = if let Some(node) = self.nodes.get_mut(&hb.node_id) {
            node
        } else if self.auto_discover {
            // Auto-discover new node
            events.push(HeartbeatEvent::Discovered {
                node_id: hb.node_id,
                state: hb.state,
            });
            self.nodes
                .entry(hb.node_id)
                .or_insert_with(|| MonitoredNode::new(self.default_config.clone()))
        } else {
            return events;
        };

        // Update statistics
        if node.config.track_stats {
            node.stats.update(now);
        }

        // Check for recovery from timeout
        if node.timed_out {
            node.timed_out = false;
            events.push(HeartbeatEvent::Recovered {
                node_id: hb.node_id,
                state: hb.state,
            });
        }

        // Check for state change
        if node.state != hb.state {
            if node.config.track_stats {
                node.stats.record_state_change(now);
            }
            events.push(HeartbeatEvent::StateChanged {
                node_id: hb.node_id,
                old_state: node.state,
                new_state: hb.state,
            });
            node.state = hb.state;
        }

        // Update last heartbeat timestamp
        node.last_heartbeat = Some(now);

        events
    }

    /// Check for timed-out nodes.
    ///
    /// Returns a list of timeout events.
    pub fn check_timeouts(&mut self) -> Vec<HeartbeatEvent> {
        let mut events = Vec::new();

        for (&node_id, node) in self.nodes.iter_mut() {
            if !node.timed_out && !node.is_alive() {
                node.timed_out = true;
                if node.config.track_stats {
                    node.stats.record_timeout();
                }
                let elapsed = node
                    .last_heartbeat
                    .map(|last| last.elapsed())
                    .unwrap_or(Duration::MAX);
                events.push(HeartbeatEvent::Timeout {
                    node_id,
                    elapsed,
                });
            }
        }

        events
    }

    /// Drain all pending events.
    pub fn drain_events(&mut self) -> Vec<HeartbeatEvent> {
        std::mem::take(&mut self.events)
    }

    /// Get a summary of the monitor state.
    pub fn summary(&self) -> MonitorSummary {
        let total = self.nodes.len();
        let alive = self.nodes.values().filter(|n| n.is_alive()).count();
        let timed_out = self.nodes.values().filter(|n| n.timed_out).count();

        let state_counts = self.nodes.values().fold(HashMap::new(), |mut acc, node| {
            *acc.entry(node.state).or_insert(0) += 1;
            acc
        });

        MonitorSummary {
            total,
            alive,
            timed_out,
            state_counts,
        }
    }
}

impl Default for HeartbeatMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of heartbeat monitor state.
#[derive(Debug, Clone)]
pub struct MonitorSummary {
    /// Total number of monitored nodes.
    pub total: usize,
    /// Number of alive nodes.
    pub alive: usize,
    /// Number of timed-out nodes.
    pub timed_out: usize,
    /// Count of nodes in each state.
    pub state_counts: HashMap<NmtState, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_add_remove() {
        let mut monitor = HeartbeatMonitor::new();
        assert_eq!(monitor.node_count(), 0);

        monitor.add_node(1);
        monitor.add_node(2);
        assert_eq!(monitor.node_count(), 2);

        monitor.remove_node(1);
        assert_eq!(monitor.node_count(), 1);
    }

    #[test]
    fn test_monitor_process_heartbeat() {
        let mut monitor = HeartbeatMonitor::new();
        monitor.add_node(1);

        let hb = HeartbeatFrame {
            node_id: 1,
            state: NmtState::Operational,
        };

        let events = monitor.process_heartbeat(&hb);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            HeartbeatEvent::StateChanged {
                node_id: 1,
                old_state: NmtState::BootUp,
                new_state: NmtState::Operational,
            }
        );

        assert_eq!(monitor.node_state(1), Some(NmtState::Operational));
        assert!(monitor.is_alive(1));
    }

    #[test]
    fn test_monitor_auto_discover() {
        let mut monitor = HeartbeatMonitor::new();
        assert_eq!(monitor.node_count(), 0);

        let hb = HeartbeatFrame {
            node_id: 5,
            state: NmtState::PreOperational,
        };

        let events = monitor.process_heartbeat(&hb);
        // Should generate Discovered and StateChanged events
        assert!(events.len() >= 1);
        assert!(events.iter().any(|e| matches!(e, HeartbeatEvent::Discovered { node_id: 5, state: NmtState::PreOperational })));

        assert_eq!(monitor.node_count(), 1);
        assert_eq!(monitor.node_state(5), Some(NmtState::PreOperational));
    }

    #[test]
    fn test_monitor_no_auto_discover() {
        let mut monitor = HeartbeatMonitor::new();
        monitor.set_auto_discover(false);

        let hb = HeartbeatFrame {
            node_id: 5,
            state: NmtState::PreOperational,
        };

        let events = monitor.process_heartbeat(&hb);
        assert_eq!(events.len(), 0);
        assert_eq!(monitor.node_count(), 0);
    }

    #[test]
    fn test_monitor_timeout() {
        let mut monitor = HeartbeatMonitor::new();
        let config = NodeMonitorConfig {
            period: Duration::from_millis(100),
            timeout_multiplier: 2,
            track_stats: true,
        };
        monitor.add_node_with_config(1, config);

        // No heartbeat yet - should not be alive
        assert!(!monitor.is_alive(1));

        // Send heartbeat
        let hb = HeartbeatFrame {
            node_id: 1,
            state: NmtState::Operational,
        };
        monitor.process_heartbeat(&hb);
        assert!(monitor.is_alive(1));

        // Check for timeouts (should be none)
        let events = monitor.check_timeouts();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_monitor_stats() {
        let mut monitor = HeartbeatMonitor::new();
        monitor.add_node(1);

        // Send multiple heartbeats
        for _ in 0..5 {
            let hb = HeartbeatFrame {
                node_id: 1,
                state: NmtState::Operational,
            };
            monitor.process_heartbeat(&hb);
        }

        let stats = monitor.stats(1).unwrap();
        assert_eq!(stats.received_count, 5);
        assert_eq!(stats.state_change_count, 1); // BootUp -> Operational
    }

    #[test]
    fn test_monitor_summary() {
        let mut monitor = HeartbeatMonitor::new();
        monitor.add_node(1);
        monitor.add_node(2);
        monitor.add_node(3);

        // Set node 1 to Operational
        let hb1 = HeartbeatFrame {
            node_id: 1,
            state: NmtState::Operational,
        };
        monitor.process_heartbeat(&hb1);

        // Set node 2 to PreOperational
        let hb2 = HeartbeatFrame {
            node_id: 2,
            state: NmtState::PreOperational,
        };
        monitor.process_heartbeat(&hb2);

        let summary = monitor.summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.alive, 2);
        assert_eq!(summary.timed_out, 0);
        assert_eq!(
            summary.state_counts.get(&NmtState::Operational),
            Some(&1)
        );
        assert_eq!(
            summary.state_counts.get(&NmtState::PreOperational),
            Some(&1)
        );
        assert_eq!(summary.state_counts.get(&NmtState::BootUp), Some(&1));
    }

    #[test]
    fn test_heartbeat_stats_interval() {
        let mut stats = HeartbeatStats::default();
        let now = Instant::now();

        // First heartbeat
        stats.update(now);
        assert_eq!(stats.received_count, 1);
        assert!(stats.min_interval.is_none());

        // Second heartbeat after 100ms
        stats.update(now + Duration::from_millis(100));
        assert_eq!(stats.received_count, 2);
        assert_eq!(stats.min_interval, Some(Duration::from_millis(100)));
        assert_eq!(stats.max_interval, Some(Duration::from_millis(100)));

        // Third heartbeat after 200ms
        stats.update(now + Duration::from_millis(300));
        assert_eq!(stats.received_count, 3);
        assert_eq!(stats.min_interval, Some(Duration::from_millis(100)));
        assert_eq!(stats.max_interval, Some(Duration::from_millis(200)));
    }
}
