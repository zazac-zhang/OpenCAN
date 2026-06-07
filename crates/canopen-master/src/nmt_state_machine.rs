//! NMT State Machine — complete NMT state tracking and transition management.
//!
//! The `NmtStateMachine` provides:
//! - Complete NMT state machine tracking per node
//! - State transition validation
//! - Transition history recording
//! - Coordinated state management across multiple nodes

use opencan_canopen_core::frame::{HeartbeatFrame, NmtState};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// NMT state transition command (from DS301 Table 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NmtCommand {
    /// Start remote node (enter Operational).
    Start,
    /// Stop remote node (enter Stopped).
    Stop,
    /// Enter Pre-Operational.
    EnterPreOperational,
    /// Reset node.
    ResetNode,
    /// Reset communication.
    ResetCommunication,
}

impl NmtCommand {
    /// Get the NMT command specifier value (for encoding).
    pub fn command_specifier(&self) -> u8 {
        match self {
            Self::Start => 0x01,
            Self::Stop => 0x02,
            Self::EnterPreOperational => 0x80,
            Self::ResetNode => 0x81,
            Self::ResetCommunication => 0x82,
        }
    }

    /// Get the target state for this command.
    pub fn target_state(&self) -> NmtState {
        match self {
            Self::Start => NmtState::Operational,
            Self::Stop => NmtState::Stopped,
            Self::EnterPreOperational => NmtState::PreOperational,
            Self::ResetNode => NmtState::BootUp,
            Self::ResetCommunication => NmtState::BootUp,
        }
    }
}

/// State transition record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransition {
    /// Timestamp of the transition.
    pub timestamp: Instant,
    /// Previous state.
    pub from_state: NmtState,
    /// New state.
    pub to_state: NmtState,
    /// Source of the transition (command, heartbeat, boot-up).
    pub source: TransitionSource,
}

/// Source of a state transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionSource {
    /// Transition caused by NMT command.
    Command(NmtCommand),
    /// Transition detected via heartbeat.
    Heartbeat,
    /// Boot-up message received.
    BootUp,
    /// Initial state (no transition yet).
    Initial,
}

/// Node state machine state.
#[derive(Debug, Clone)]
struct NodeStateMachine {
    /// Current state.
    current_state: NmtState,
    /// Transition history.
    history: Vec<StateTransition>,
    /// Maximum history entries (0 = unlimited).
    max_history: usize,
    /// Last heartbeat timestamp.
    last_heartbeat: Option<Instant>,
    /// Whether the node is responsive.
    responsive: bool,
}

impl NodeStateMachine {
    fn new() -> Self {
        Self {
            current_state: NmtState::BootUp,
            history: Vec::new(),
            max_history: 100,
            last_heartbeat: None,
            responsive: false,
        }
    }

    /// Record a state transition.
    fn record_transition(&mut self, to_state: NmtState, source: TransitionSource) {
        let transition = StateTransition {
            timestamp: Instant::now(),
            from_state: self.current_state,
            to_state,
            source,
        };

        self.current_state = to_state;

        // Trim history if needed
        if self.max_history > 0 && self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(transition);
    }

    /// Check if a transition is valid according to DS301.
    fn is_valid_transition(&self, command: NmtCommand) -> bool {
        // DS301 state transition rules:
        // - From any state, ResetNode and ResetCommunication are always valid
        // - Start: any state -> Operational
        // - Stop: any state -> Stopped
        // - EnterPreOperational: any state -> PreOperational
        match command {
            NmtCommand::ResetNode | NmtCommand::ResetCommunication => true,
            NmtCommand::Start => self.current_state != NmtState::Operational,
            NmtCommand::Stop => self.current_state != NmtState::Stopped,
            NmtCommand::EnterPreOperational => self.current_state != NmtState::PreOperational,
        }
    }
}

/// Configuration for NMT state machine tracking.
#[derive(Debug, Clone)]
pub struct NmtStateMachineConfig {
    /// Maximum history entries per node (0 = unlimited).
    pub max_history_per_node: usize,
    /// Heartbeat timeout for responsiveness check.
    pub heartbeat_timeout: Duration,
    /// Whether to validate state transitions.
    pub validate_transitions: bool,
}

impl Default for NmtStateMachineConfig {
    fn default() -> Self {
        Self {
            max_history_per_node: 100,
            heartbeat_timeout: Duration::from_secs(5),
            validate_transitions: true,
        }
    }
}

/// NMT state machine manager for tracking multiple nodes.
///
/// Provides complete NMT state tracking with:
/// - State transition validation
/// - Transition history
/// - Coordinated state management
pub struct NmtStateMachine {
    /// Node state machines.
    nodes: HashMap<u8, NodeStateMachine>,
    /// Configuration.
    config: NmtStateMachineConfig,
    /// Pending commands to send.
    pending_commands: Vec<(u8, NmtCommand)>,
}

impl NmtStateMachine {
    /// Create a new NMT state machine manager.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            config: NmtStateMachineConfig::default(),
            pending_commands: Vec::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: NmtStateMachineConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            config,
            pending_commands: Vec::new(),
        }
    }

    /// Add a node to track.
    pub fn add_node(&mut self, node_id: u8) {
        self.nodes.insert(node_id, NodeStateMachine::new());
    }

    /// Remove a node from tracking.
    pub fn remove_node(&mut self, node_id: u8) {
        self.nodes.remove(&node_id);
    }

    /// Get the number of tracked nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the current state of a node.
    pub fn node_state(&self, node_id: u8) -> Option<NmtState> {
        self.nodes.get(&node_id).map(|n| n.current_state)
    }

    /// Check if a node is responsive (heartbeat received recently).
    pub fn is_responsive(&self, node_id: u8) -> bool {
        self.nodes.get(&node_id).map_or(false, |n| n.responsive)
    }

    /// Get all nodes in a specific state.
    pub fn nodes_in_state(&self, state: NmtState) -> Vec<u8> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.current_state == state)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get all operational nodes.
    pub fn operational_nodes(&self) -> Vec<u8> {
        self.nodes_in_state(NmtState::Operational)
    }

    /// Get transition history for a node.
    pub fn history(&self, node_id: u8) -> Option<&[StateTransition]> {
        self.nodes.get(&node_id).map(|n| n.history.as_slice())
    }

    /// Get the last transition for a node.
    pub fn last_transition(&self, node_id: u8) -> Option<&StateTransition> {
        self.nodes.get(&node_id).and_then(|n| n.history.last())
    }

    /// Request a state transition for a node.
    ///
    /// Returns true if the transition is valid (or validation is disabled).
    /// The command is queued for sending.
    pub fn request_transition(&mut self, node_id: u8, command: NmtCommand) -> bool {
        let node = self.nodes.entry(node_id).or_insert_with(NodeStateMachine::new);

        if self.config.validate_transitions && !node.is_valid_transition(command) {
            return false;
        }

        let target = command.target_state();
        node.record_transition(target, TransitionSource::Command(command));
        self.pending_commands.push((node_id, command));

        true
    }

    /// Process a received heartbeat frame.
    ///
    /// Updates the node state and responsiveness.
    pub fn process_heartbeat(&mut self, hb: &HeartbeatFrame) {
        let node = self.nodes.entry(hb.node_id).or_insert_with(NodeStateMachine::new);

        let old_state = node.current_state;
        node.last_heartbeat = Some(Instant::now());
        node.responsive = true;

        if old_state != hb.state {
            node.record_transition(hb.state, TransitionSource::Heartbeat);
        }
    }

    /// Process a boot-up message.
    pub fn process_boot_up(&mut self, node_id: u8) {
        let node = self.nodes.entry(node_id).or_insert_with(NodeStateMachine::new);

        node.record_transition(NmtState::BootUp, TransitionSource::BootUp);
        node.last_heartbeat = Some(Instant::now());
        node.responsive = true;
    }

    /// Update responsiveness based on heartbeat timeout.
    pub fn update_responsiveness(&mut self) {
        let timeout = self.config.heartbeat_timeout;
        for node in self.nodes.values_mut() {
            if let Some(last) = node.last_heartbeat {
                node.responsive = last.elapsed() < timeout;
            } else {
                node.responsive = false;
            }
        }
    }

    /// Drain pending commands.
    pub fn drain_commands(&mut self) -> Vec<(u8, NmtCommand)> {
        std::mem::take(&mut self.pending_commands)
    }

    /// Request all nodes to enter Operational.
    pub fn start_all(&mut self) {
        let node_ids: Vec<u8> = self.nodes.keys().copied().collect();
        for node_id in node_ids {
            self.request_transition(node_id, NmtCommand::Start);
        }
    }

    /// Request all nodes to enter Stopped.
    pub fn stop_all(&mut self) {
        let node_ids: Vec<u8> = self.nodes.keys().copied().collect();
        for node_id in node_ids {
            self.request_transition(node_id, NmtCommand::Stop);
        }
    }

    /// Request all nodes to enter Pre-Operational.
    pub fn enter_pre_operational_all(&mut self) {
        let node_ids: Vec<u8> = self.nodes.keys().copied().collect();
        for node_id in node_ids {
            self.request_transition(node_id, NmtCommand::EnterPreOperational);
        }
    }

    /// Request all nodes to reset.
    pub fn reset_all(&mut self) {
        let node_ids: Vec<u8> = self.nodes.keys().copied().collect();
        for node_id in node_ids {
            self.request_transition(node_id, NmtCommand::ResetNode);
        }
    }

    /// Get a summary of all node states.
    pub fn summary(&self) -> NmtSummary {
        let total = self.nodes.len();
        let mut state_counts = HashMap::new();
        let mut responsive_count = 0;

        for node in self.nodes.values() {
            *state_counts.entry(node.current_state).or_insert(0) += 1;
            if node.responsive {
                responsive_count += 1;
            }
        }

        NmtSummary {
            total,
            responsive: responsive_count,
            state_counts,
        }
    }
}

impl Default for NmtStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of NMT state machine state.
#[derive(Debug, Clone)]
pub struct NmtSummary {
    /// Total number of tracked nodes.
    pub total: usize,
    /// Number of responsive nodes.
    pub responsive: usize,
    /// Count of nodes in each state.
    pub state_counts: HashMap<NmtState, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_add_remove() {
        let mut sm = NmtStateMachine::new();
        assert_eq!(sm.node_count(), 0);

        sm.add_node(1);
        sm.add_node(2);
        assert_eq!(sm.node_count(), 2);

        sm.remove_node(1);
        assert_eq!(sm.node_count(), 1);
    }

    #[test]
    fn test_state_machine_initial_state() {
        let mut sm = NmtStateMachine::new();
        sm.add_node(1);

        assert_eq!(sm.node_state(1), Some(NmtState::BootUp));
        assert!(!sm.is_responsive(1));
    }

    #[test]
    fn test_state_machine_heartbeat() {
        let mut sm = NmtStateMachine::new();
        sm.add_node(1);

        let hb = HeartbeatFrame {
            node_id: 1,
            state: NmtState::Operational,
        };
        sm.process_heartbeat(&hb);

        assert_eq!(sm.node_state(1), Some(NmtState::Operational));
        assert!(sm.is_responsive(1));

        // Check history
        let history = sm.history(1).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].from_state, NmtState::BootUp);
        assert_eq!(history[0].to_state, NmtState::Operational);
        assert_eq!(history[0].source, TransitionSource::Heartbeat);
    }

    #[test]
    fn test_state_machine_command() {
        let mut sm = NmtStateMachine::new();
        sm.add_node(1);

        // Request start
        let valid = sm.request_transition(1, NmtCommand::Start);
        assert!(valid);

        // Check state changed
        assert_eq!(sm.node_state(1), Some(NmtState::Operational));

        // Check pending command
        let commands = sm.drain_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], (1, NmtCommand::Start));
    }

    #[test]
    fn test_state_machine_invalid_transition() {
        let mut sm = NmtStateMachine::new();
        sm.add_node(1);

        // Start -> Start is invalid (already Operational)
        sm.request_transition(1, NmtCommand::Start);
        let valid = sm.request_transition(1, NmtCommand::Start);
        assert!(!valid);
    }

    #[test]
    fn test_state_machine_boot_up() {
        let mut sm = NmtStateMachine::new();
        sm.add_node(1);

        // Move to Operational first
        sm.request_transition(1, NmtCommand::Start);
        assert_eq!(sm.node_state(1), Some(NmtState::Operational));

        // Process boot-up (resets to BootUp)
        sm.process_boot_up(1);
        assert_eq!(sm.node_state(1), Some(NmtState::BootUp));
    }

    #[test]
    fn test_state_machine_auto_discover() {
        let mut sm = NmtStateMachine::new();
        assert_eq!(sm.node_count(), 0);

        // Process heartbeat for unknown node (auto-creates)
        let hb = HeartbeatFrame {
            node_id: 5,
            state: NmtState::PreOperational,
        };
        sm.process_heartbeat(&hb);

        assert_eq!(sm.node_count(), 1);
        assert_eq!(sm.node_state(5), Some(NmtState::PreOperational));
    }

    #[test]
    fn test_state_machine_nodes_in_state() {
        let mut sm = NmtStateMachine::new();
        sm.add_node(1);
        sm.add_node(2);
        sm.add_node(3);

        // Set different states
        sm.request_transition(1, NmtCommand::Start);
        sm.request_transition(2, NmtCommand::EnterPreOperational);
        // Node 3 stays in BootUp

        let operational = sm.operational_nodes();
        assert_eq!(operational.len(), 1);
        assert!(operational.contains(&1));

        let pre_op = sm.nodes_in_state(NmtState::PreOperational);
        assert_eq!(pre_op.len(), 1);
        assert!(pre_op.contains(&2));

        let boot_up = sm.nodes_in_state(NmtState::BootUp);
        assert_eq!(boot_up.len(), 1);
        assert!(boot_up.contains(&3));
    }

    #[test]
    fn test_state_machine_summary() {
        let mut sm = NmtStateMachine::new();
        sm.add_node(1);
        sm.add_node(2);
        sm.add_node(3);

        // Set states
        sm.request_transition(1, NmtCommand::Start);
        let hb = HeartbeatFrame {
            node_id: 2,
            state: NmtState::Operational,
        };
        sm.process_heartbeat(&hb);

        let summary = sm.summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.responsive, 1); // Only node 2 has heartbeat
        assert_eq!(summary.state_counts.get(&NmtState::Operational), Some(&2));
        assert_eq!(summary.state_counts.get(&NmtState::BootUp), Some(&1));
    }

    #[test]
    fn test_nmt_command_target_state() {
        assert_eq!(NmtCommand::Start.target_state(), NmtState::Operational);
        assert_eq!(NmtCommand::Stop.target_state(), NmtState::Stopped);
        assert_eq!(
            NmtCommand::EnterPreOperational.target_state(),
            NmtState::PreOperational
        );
        assert_eq!(NmtCommand::ResetNode.target_state(), NmtState::BootUp);
        assert_eq!(
            NmtCommand::ResetCommunication.target_state(),
            NmtState::BootUp
        );
    }

    #[test]
    fn test_nmt_command_specifier() {
        assert_eq!(NmtCommand::Start.command_specifier(), 0x01);
        assert_eq!(NmtCommand::Stop.command_specifier(), 0x02);
        assert_eq!(NmtCommand::EnterPreOperational.command_specifier(), 0x80);
        assert_eq!(NmtCommand::ResetNode.command_specifier(), 0x81);
        assert_eq!(NmtCommand::ResetCommunication.command_specifier(), 0x82);
    }
}
