//! Application state types.

use std::collections::BTreeMap;

/// Application state.
#[derive(Debug)]
pub struct App {
    pub current_view: View,
    pub selected_node: Option<u8>,
    pub nodes: Vec<NodeState>,
    pub status_message: String,
    pub sdo_index: String,
    pub sdo_subindex: String,
    pub sdo_value: String,
    pub can_log: Vec<LogEntry>,
    pub backend: Option<crate::backend::Backend>,
    pub ds402_state: Ds402PanelState,
    pub connected: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_view: View::default(),
            selected_node: None,
            nodes: Vec::new(),
            status_message: "Ready — Connect to start".to_string(),
            sdo_index: String::new(),
            sdo_subindex: "0".to_string(),
            sdo_value: String::new(),
            can_log: Vec::new(),
            backend: None,
            ds402_state: Ds402PanelState::default(),
            connected: false,
        }
    }
}

/// View routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    NetworkOverview,
    NodeDetail,
    Ds402,
    PdoMonitor,
    CanLog,
}

/// Node state.
#[derive(Debug, Clone)]
pub struct NodeState {
    pub node_id: u8,
    pub nmt_state: NmtState,
    pub device_type: Option<u32>,
    pub vendor_id: Option<u32>,
    pub product_name: Option<String>,
    pub od_cache: BTreeMap<(u16, u8), String>,
    pub ds402: Ds402NodeState,
}

impl NodeState {
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
            nmt_state: NmtState::Unknown,
            device_type: None,
            vendor_id: None,
            product_name: None,
            od_cache: BTreeMap::new(),
            ds402: Ds402NodeState::default(),
        }
    }
}

/// NMT state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NmtState {
    Unknown,
    BootUp,
    PreOperational,
    Operational,
    Stopped,
}

impl NmtState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::BootUp => "BootUp",
            Self::PreOperational => "Pre-Operational",
            Self::Operational => "Operational",
            Self::Stopped => "Stopped",
        }
    }
}

/// DS402 node state.
#[derive(Debug, Clone, Default)]
pub struct Ds402NodeState {
    pub state: String,
    pub status_word: u16,
    pub actual_position: i32,
    pub actual_velocity: i32,
    pub actual_torque: i16,
}

/// DS402 panel state (UI inputs).
#[derive(Debug, Clone, Default)]
pub struct Ds402PanelState {
    pub target_position: String,
    pub target_velocity: String,
    pub target_torque: String,
}

/// Log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp_ms: u64,
    pub cob_id: u16,
    pub data: [u8; 8],
    pub description: String,
}

/// Messages.
#[derive(Debug, Clone)]
pub enum Message {
    // View
    SwitchView(View),
    NodeSelected(u8),

    // Connection
    ConnectMock,
    Disconnect,
    ScanNodes,

    // NMT
    NmtStartNode(u8),
    NmtStopNode(u8),
    NmtResetNode(u8),

    // SDO
    SdoIndexChanged(String),
    SdoSubindexChanged(String),
    SdoValueChanged(String),
    SdoRead,
    SdoWrite,

    // DS402
    Ds402Enable(u8),
    Ds402FaultReset(u8),
    Ds402ReadState(u8),
    Ds402TargetPositionChanged(String),
    Ds402TargetVelocityChanged(String),
    Ds402SetPosition(u8),
    Ds402SetVelocity(u8),
    Ds402ReadPosition(u8),
    Ds402ReadVelocity(u8),

    // Backend events
    BackendEvent(crate::backend::BackendEvent),

    // Tick
    Tick,
}
