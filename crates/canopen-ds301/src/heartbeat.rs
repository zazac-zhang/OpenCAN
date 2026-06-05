//! Heartbeat consumer implementation.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use opencan_canopen_core::frame::{HeartbeatFrame, NmtState};

/// Heartbeat consumer — monitors heartbeat messages from remote nodes.
pub struct HeartbeatConsumer {
    /// Expected heartbeat period per node.
    periods: HashMap<u8, Duration>,
    /// Last heartbeat timestamp per node.
    last_heartbeat: HashMap<u8, Instant>,
    /// Current state per node.
    states: HashMap<u8, NmtState>,
    /// Default timeout multiplier (if no period configured).
    default_timeout: Duration,
}

impl HeartbeatConsumer {
    pub fn new(default_timeout: Duration) -> Self {
        Self {
            periods: HashMap::new(),
            last_heartbeat: HashMap::new(),
            states: HashMap::new(),
            default_timeout,
        }
    }

    /// Set expected heartbeat period for a node.
    pub fn set_period(&mut self, node_id: u8, period: Duration) {
        self.periods.insert(node_id, period);
    }

    /// Update state with a received heartbeat.
    /// Returns true if the state changed.
    pub fn update(&mut self, hb: &HeartbeatFrame) -> bool {
        let now = Instant::now();
        let old_state = self.states.get(&hb.node_id).copied();
        self.last_heartbeat.insert(hb.node_id, now);
        self.states.insert(hb.node_id, hb.state);
        old_state != Some(hb.state)
    }

    /// Check if a node is alive (heartbeat received within timeout).
    pub fn is_alive(&self, node_id: u8) -> bool {
        let last = match self.last_heartbeat.get(&node_id) {
            Some(ts) => ts,
            None => return false,
        };
        let timeout = self.periods.get(&node_id)
            .copied()
            .unwrap_or(self.default_timeout);
        last.elapsed() < timeout * 3 // Allow 3x the period
    }

    /// Check for timed-out nodes.
    /// Returns list of (node_id, elapsed_since_last) for timed-out nodes.
    pub fn check_timeouts(&self) -> Vec<(u8, Duration)> {
        let now = Instant::now();
        self.last_heartbeat.iter()
            .filter_map(|(&node_id, &last)| {
                let timeout = self.periods.get(&node_id)
                    .copied()
                    .unwrap_or(self.default_timeout);
                let elapsed = now.duration_since(last);
                if elapsed > timeout * 3 {
                    Some((node_id, elapsed))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the current state of a node.
    pub fn state(&self, node_id: u8) -> Option<NmtState> {
        self.states.get(&node_id).copied()
    }

    /// Get all monitored node IDs.
    pub fn nodes(&self) -> Vec<u8> {
        self.states.keys().copied().collect()
    }
}
