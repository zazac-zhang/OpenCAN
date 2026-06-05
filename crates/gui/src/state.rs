//! Application state types.

use std::collections::BTreeMap;

/// Application state.
#[derive(Debug, Default)]
pub struct App {
    pub current_view: View,
    pub selected_node: Option<u8>,
    pub nodes: Vec<NodeState>,
    pub status_message: String,
    pub sdo_index: String,
    pub sdo_subindex: String,
    pub sdo_value: String,
    pub can_log: Vec<LogEntry>,
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
    pub product_name: Option<String>,
    pub od_cache: BTreeMap<(u16, u8), String>,
}

impl NodeState {
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
            nmt_state: NmtState::Unknown,
            device_type: None,
            product_name: None,
            od_cache: BTreeMap::new(),
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
    SwitchView(View),
    NodeSelected(u8),
    NmtStartNode(u8),
    NmtStopNode(u8),
    NmtResetNode(u8),
    SdoIndexChanged(String),
    SdoSubindexChanged(String),
    SdoValueChanged(String),
    SdoRead,
    SdoWrite,
    Ds402Enable(u8),
    Ds402FaultReset(u8),
}
