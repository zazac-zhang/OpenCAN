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

/// Heartbeat producer — manages periodic heartbeat transmission for the local node.
///
/// Does not own the CAN driver. Instead, the caller checks `should_send()`
/// and uses the stack's `send_heartbeat()` when it's time.
pub struct HeartbeatProducer {
    period: Duration,
    last_sent: Option<Instant>,
    state: NmtState,
}

impl HeartbeatProducer {
    /// Create a new heartbeat producer with the given period.
    pub fn new(period: Duration) -> Self {
        Self {
            period,
            last_sent: None,
            state: NmtState::PreOperational,
        }
    }

    /// Set the current NMT state to include in heartbeat frames.
    pub fn set_state(&mut self, state: NmtState) {
        self.state = state;
    }

    /// Get the current NMT state.
    pub fn state(&self) -> NmtState {
        self.state
    }

    /// Check if it's time to send a heartbeat.
    pub fn should_send(&self) -> bool {
        match self.last_sent {
            Some(last) => last.elapsed() >= self.period,
            None => true,
        }
    }

    /// Mark that a heartbeat was sent (call after successful send).
    pub fn mark_sent(&mut self) {
        self.last_sent = Some(Instant::now());
    }

    /// Get the heartbeat period.
    pub fn period(&self) -> Duration {
        self.period
    }

    /// Set a new heartbeat period.
    pub fn set_period(&mut self, period: Duration) {
        self.period = period;
    }
}

/// SYNC producer — manages periodic SYNC frame transmission.
///
/// SYNC frames (COB-ID 0x080) are used to synchronize PDO transmissions.
/// The producer optionally includes a counter byte (data[0]) for
/// identifying missed SYNCs.
pub struct SyncProducer {
    period: Duration,
    last_sent: Option<Instant>,
    counter: u8,
    counter_enabled: bool,
}

impl SyncProducer {
    /// Create a new SYNC producer with the given period.
    pub fn new(period: Duration) -> Self {
        Self {
            period,
            last_sent: None,
            counter: 0,
            counter_enabled: false,
        }
    }

    /// Enable or disable the SYNC counter (data[0] byte).
    pub fn set_counter_enabled(&mut self, enabled: bool) {
        self.counter_enabled = enabled;
    }

    /// Check if it's time to send a SYNC.
    pub fn should_send(&self) -> bool {
        match self.last_sent {
            Some(last) => last.elapsed() >= self.period,
            None => true,
        }
    }

    /// Build a SYNC frame and mark it as sent.
    pub fn build_frame(&mut self) -> opencan_canopen_core::frame::CanOpenFrame {
        let mut data = [0u8; 8];
        if self.counter_enabled {
            data[0] = self.counter;
            self.counter = self.counter.wrapping_add(1);
        }
        self.last_sent = Some(Instant::now());
        opencan_canopen_core::frame::CanOpenFrame::new(0x080, data)
    }

    /// Get the SYNC period.
    pub fn period(&self) -> Duration {
        self.period
    }

    /// Set a new SYNC period.
    pub fn set_period(&mut self, period: Duration) {
        self.period = period;
    }

    /// Get the current counter value.
    pub fn counter(&self) -> u8 {
        self.counter
    }

    /// Reset the counter to 0.
    pub fn reset_counter(&mut self) {
        self.counter = 0;
    }
}
