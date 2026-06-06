//! Application state types.

mod tab;
mod node;
mod connection;
mod log;
mod bus;
mod ds402;
mod protocol;
mod message;
mod sdo;

/// Drag target for panel resizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragTarget {
    NodePanel,
    DetailPanel,
}

// Re-export all types
pub use tab::{Tab, PrimaryTab};
pub use node::{
    NodeState, NmtState, OdEntry,
    Ds402Mode,
};
pub use connection::{CanBackend, ConnectionDialog};
pub use log::{LogEntry, LogFilter, LogRecorder, Direction};
pub use bus::{BusStatistics, ErrorFrame, ErrorType};
pub use ds402::Ds402PanelState;
pub use protocol::{EmcyEntry, HeartbeatStatus, SyncStatus};
pub use sdo::SdoDataType;
pub use message::Message;

/// Application state.
#[derive(Debug)]
pub struct App {
    // View state
    pub current_tab: Tab,
    pub selected_node: Option<u8>,
    pub detail_collapsed: bool,

    // Node state
    pub nodes: Vec<NodeState>,

    // Connection state
    pub connected: bool,
    pub connection_dialog: ConnectionDialog,
    pub backend: Option<crate::backend::Backend>,

    // CAN log state
    pub can_log: Vec<LogEntry>,
    pub log_filter: LogFilter,
    pub log_recorder: LogRecorder,
    pub selected_frame: Option<LogEntry>,
    pub paused: bool,

    // Protocol-specific state
    pub pdo_log: Vec<LogEntry>,
    pub emcy_log: Vec<EmcyEntry>,
    pub heartbeat_status: Vec<HeartbeatStatus>,
    pub sync_status: SyncStatus,
    pub bus_stats: BusStatistics,
    pub error_frames: Vec<ErrorFrame>,

    // SDO state
    pub sdo_index: String,
    pub sdo_subindex: String,
    pub sdo_value: String,
    pub sdo_data_type: SdoDataType,
    pub sdo_history: Vec<SdoHistoryEntry>,

    // DS402 state
    pub ds402_state: Ds402PanelState,
    pub trend_chart: crate::views::canopen::trend_chart::TrendChartState,

    // UI state
    pub status_message: String,
    pub toolbar_bitrate: u32,

    // Panel layout state
    pub node_panel_width: u16,
    pub detail_panel_width: u16,
    pub is_dragging: bool,
    pub drag_target: Option<DragTarget>,
}

/// SDO history entry.
#[derive(Debug, Clone)]
pub struct SdoHistoryEntry {
    pub timestamp_ms: u64,
    pub node_id: u8,
    pub index: u16,
    pub subindex: u8,
    pub is_read: bool,
    pub value: String,
    pub success: bool,
    pub error: Option<String>,
}

impl SdoHistoryEntry {
    pub fn success(node_id: u8, index: u16, subindex: u8, value: String, is_read: bool) -> Self {
        Self {
            timestamp_ms: 0,
            node_id,
            index,
            subindex,
            is_read,
            value,
            success: true,
            error: None,
        }
    }

    pub fn failure(node_id: u8, index: u16, subindex: u8, error: String, is_read: bool) -> Self {
        Self {
            timestamp_ms: 0,
            node_id,
            index,
            subindex,
            is_read,
            value: String::new(),
            success: false,
            error: Some(error),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            // View state
            current_tab: Tab::default(),
            selected_node: None,
            detail_collapsed: false,

            // Node state
            nodes: Vec::new(),

            // Connection state
            connected: false,
            connection_dialog: ConnectionDialog::default(),
            backend: None,

            // CAN log state
            can_log: Vec::new(),
            log_filter: LogFilter::default(),
            log_recorder: LogRecorder::default(),
            selected_frame: None,
            paused: false,

            // Protocol-specific state
            pdo_log: Vec::new(),
            emcy_log: Vec::new(),
            heartbeat_status: Vec::new(),
            sync_status: SyncStatus::default(),
            bus_stats: BusStatistics::default(),
            error_frames: Vec::new(),

            // SDO state
            sdo_index: String::new(),
            sdo_subindex: "0".to_string(),
            sdo_value: String::new(),
            sdo_data_type: SdoDataType::Unsigned32,
            sdo_history: Vec::new(),

            // DS402 state
            ds402_state: Ds402PanelState::default(),
            trend_chart: crate::views::canopen::trend_chart::TrendChartState::default(),

            // UI state
            status_message: "Ready — Connect to start".to_string(),
            toolbar_bitrate: 500000,

            // Panel layout state
            node_panel_width: 200,
            detail_panel_width: 200,
            is_dragging: false,
            drag_target: None,
        }
    }
}

impl App {
    /// Get node by ID.
    pub fn get_node(&self, node_id: u8) -> Option<&NodeState> {
        self.nodes.iter().find(|n| n.node_id == node_id)
    }

    /// Get mutable node by ID.
    pub fn get_node_mut(&mut self, node_id: u8) -> Option<&mut NodeState> {
        self.nodes.iter_mut().find(|n| n.node_id == node_id)
    }

    /// Get selected node.
    pub fn selected_node(&self) -> Option<&NodeState> {
        self.selected_node.and_then(|id| self.get_node(id))
    }

    /// Get mutable selected node.
    pub fn selected_node_mut(&mut self) -> Option<&mut NodeState> {
        self.selected_node.and_then(move |id| self.get_node_mut(id))
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.connected && self.backend.is_some()
    }

    /// Get connection summary.
    pub fn connection_summary(&self) -> String {
        if self.connected {
            "Connected".to_string()
        } else {
            "Disconnected".to_string()
        }
    }
}
