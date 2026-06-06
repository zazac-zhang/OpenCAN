//! Application state types.

use std::collections::BTreeMap;

/// Application state.
#[derive(Debug)]
pub struct App {
    pub current_tab: Tab,
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
    /// Connection dialog state.
    pub connection_dialog: ConnectionDialog,
    /// CAN log filter state.
    pub log_filter: LogFilter,
    /// PDO monitor entries (separate from can_log for dedicated display).
    pub pdo_log: Vec<LogEntry>,
    /// Selected frame for detail view.
    pub selected_frame: Option<LogEntry>,
    /// Bus statistics.
    pub bus_stats: BusStatistics,
    /// Error frames.
    pub error_frames: Vec<ErrorFrame>,
    /// EMCY log.
    pub emcy_log: Vec<EmcyEntry>,
    /// Heartbeat status.
    pub heartbeat_status: Vec<HeartbeatStatus>,
    /// Sync status.
    pub sync_status: SyncStatus,
    /// Log recording state.
    pub recording: bool,
    /// Toolbar state.
    pub toolbar: ToolbarState,
    /// Detail panel collapsed state.
    pub detail_collapsed: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_tab: Tab::default(),
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
            connection_dialog: ConnectionDialog::default(),
            log_filter: LogFilter::default(),
            pdo_log: Vec::new(),
            selected_frame: None,
            bus_stats: BusStatistics::default(),
            error_frames: Vec::new(),
            emcy_log: Vec::new(),
            heartbeat_status: Vec::new(),
            sync_status: SyncStatus::default(),
            recording: false,
            toolbar: ToolbarState::default(),
            detail_collapsed: false,
        }
    }
}

/// Primary tab (protocol layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrimaryTab {
    #[default]
    CanBus,
    CanOpen,
}

/// Tab routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    // CAN Bus tabs
    FrameMonitor,
    BusStatistics,
    ErrorFrames,
    // CANOpen tabs
    NetworkManagement,
    SdoClient,
    PdoMonitor,
    Ds402Control,
    EmcyLog,
    HeartbeatMonitor,
    SyncManagement,
}

impl Default for Tab {
    fn default() -> Self {
        Self::FrameMonitor
    }
}

impl Tab {
    /// Get the primary tab for this secondary tab.
    pub fn primary(&self) -> PrimaryTab {
        match self {
            Self::FrameMonitor | Self::BusStatistics | Self::ErrorFrames => PrimaryTab::CanBus,
            Self::NetworkManagement | Self::SdoClient | Self::PdoMonitor | Self::Ds402Control
            | Self::EmcyLog | Self::HeartbeatMonitor | Self::SyncManagement => PrimaryTab::CanOpen,
        }
    }

    /// Get all tabs for a primary tab.
    pub fn for_primary(primary: PrimaryTab) -> &'static [Tab] {
        match primary {
            PrimaryTab::CanBus => &[Self::FrameMonitor, Self::BusStatistics, Self::ErrorFrames],
            PrimaryTab::CanOpen => &[
                Self::NetworkManagement,
                Self::SdoClient,
                Self::PdoMonitor,
                Self::Ds402Control,
                Self::EmcyLog,
                Self::HeartbeatMonitor,
                Self::SyncManagement,
            ],
        }
    }

    /// Get display name for this tab.
    pub fn name(&self) -> &'static str {
        match self {
            Self::FrameMonitor => "帧监控",
            Self::BusStatistics => "总线统计",
            Self::ErrorFrames => "错误帧",
            Self::NetworkManagement => "网络管理",
            Self::SdoClient => "SDO 客户端",
            Self::PdoMonitor => "PDO 监控",
            Self::Ds402Control => "DS402 控制",
            Self::EmcyLog => "EMCY 日志",
            Self::HeartbeatMonitor => "心跳监控",
            Self::SyncManagement => "同步管理",
        }
    }
}

/// Node state.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NodeState {
    pub node_id: u8,
    pub nmt_state: NmtState,
    pub device_type: Option<u32>,
    pub vendor_id: Option<u32>,
    pub product_name: Option<String>,
    pub od_cache: BTreeMap<(u16, u8), OdEntry>,
    pub ds402: Ds402NodeState,
    pub heartbeat_period: Option<u32>,
    pub last_heartbeat: Option<u64>,
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
            heartbeat_period: None,
            last_heartbeat: None,
        }
    }
}

/// OD cache entry.
#[derive(Debug, Clone)]
pub struct OdEntry {
    pub value: String,
    pub data_type: Option<String>,
    pub name: Option<String>,
}

impl OdEntry {
    pub fn new(value: String) -> Self {
        Self {
            value,
            data_type: None,
            name: None,
        }
    }

    pub fn with_type(value: String, data_type: String) -> Self {
        Self {
            value,
            data_type: Some(data_type),
            name: None,
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
#[allow(dead_code)]
pub struct Ds402NodeState {
    pub state: String,
    pub status_word: u16,
    pub actual_position: i32,
    pub actual_velocity: i32,
    pub actual_torque: i16,
    pub position_history: Vec<i32>,
    pub velocity_history: Vec<i32>,
    pub torque_history: Vec<i16>,
}

/// DS402 panel state (UI inputs).
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Ds402PanelState {
    pub target_position: String,
    pub target_velocity: String,
    pub target_torque: String,
    pub selected_mode: Ds402Mode,
}

/// DS402 operation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ds402Mode {
    #[default]
    ProfilePosition,
    InterpolatedPosition,
    CyclicSyncPosition,
    CyclicSyncVelocity,
    CyclicSyncTorque,
}

impl Ds402Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ProfilePosition => "PP (Profile Position)",
            Self::InterpolatedPosition => "IP (Interpolated Position)",
            Self::CyclicSyncPosition => "CSP (Cyclic Sync Position)",
            Self::CyclicSyncVelocity => "CSV (Cyclic Sync Velocity)",
            Self::CyclicSyncTorque => "CST (Cyclic Sync Torque)",
        }
    }

    pub fn all() -> &'static [Ds402Mode] {
        &[
            Self::ProfilePosition,
            Self::InterpolatedPosition,
            Self::CyclicSyncPosition,
            Self::CyclicSyncVelocity,
            Self::CyclicSyncTorque,
        ]
    }
}

/// Log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp_ms: u64,
    pub cob_id: u16,
    pub data: [u8; 8],
    pub description: String,
}

/// CAN log filter state.
#[derive(Debug, Clone)]
pub struct LogFilter {
    pub text: String,
    pub show_nmt: bool,
    pub show_sdo: bool,
    pub show_pdo: bool,
    pub show_heartbeat: bool,
    pub show_emcy: bool,
    pub show_other: bool,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            text: String::new(),
            show_nmt: true,
            show_sdo: true,
            show_pdo: true,
            show_heartbeat: true,
            show_emcy: true,
            show_other: true,
        }
    }
}

impl LogFilter {
    /// Check if a log entry passes this filter.
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if !self.text.is_empty() {
            let query = self.text.to_lowercase();
            let hex_data = entry.data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            let haystack = format!("{:03X} {} {}", entry.cob_id, hex_data, entry.description).to_lowercase();
            if !haystack.contains(&query) {
                return false;
            }
        }
        let cob = entry.cob_id;
        match cob {
            0x000 => self.show_nmt,
            0x080 => self.show_other,
            0x081..=0x0FF => self.show_emcy,
            0x100..=0x17F => self.show_other,
            0x180..=0x57F => self.show_pdo,
            0x580..=0x67F => self.show_sdo,
            0x700..=0x77F => self.show_heartbeat,
            _ => self.show_other,
        }
    }
}

/// Bus statistics.
#[derive(Debug, Clone)]
pub struct BusStatistics {
    pub bus_load_percent: f32,
    pub frame_rate: u32,
    pub tx_errors: u32,
    pub rx_errors: u32,
    pub bitrate: u32,
    pub bus_state: BusState,
    pub frame_count: u64,
}

impl Default for BusStatistics {
    fn default() -> Self {
        Self {
            bus_load_percent: 0.0,
            frame_rate: 0,
            tx_errors: 0,
            rx_errors: 0,
            bitrate: 500000,
            bus_state: BusState::Unknown,
            frame_count: 0,
        }
    }
}

/// CAN bus state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusState {
    Unknown,
    Active,
    Warning,
    Passive,
    BusOff,
}

impl BusState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Active => "Active",
            Self::Warning => "Warning",
            Self::Passive => "Passive",
            Self::BusOff => "Bus Off",
        }
    }
}

/// Error frame.
#[derive(Debug, Clone)]
pub struct ErrorFrame {
    pub timestamp_ms: u64,
    pub error_type: ErrorType,
    pub error_flag: u8,
    pub tec: u8,
    pub rec: u8,
}

/// CAN error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    BitError,
    StuffError,
    CrcError,
    FormError,
    AckError,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BitError => "Bit Error",
            Self::StuffError => "Stuff Error",
            Self::CrcError => "CRC Error",
            Self::FormError => "Form Error",
            Self::AckError => "ACK Error",
        }
    }
}

/// EMCY log entry.
#[derive(Debug, Clone)]
pub struct EmcyEntry {
    pub timestamp_ms: u64,
    pub node_id: u8,
    pub error_code: u16,
    pub error_register: u8,
    pub data: [u8; 5],
}

impl EmcyEntry {
    pub fn error_description(&self) -> String {
        match self.error_code {
            0x0000 => "Error Reset / No Error".to_string(),
            0x1000 => "Generic Error".to_string(),
            0x2000 => "Current Error".to_string(),
            0x2100 => "Current, device input side".to_string(),
            0x2200 => "Current inside the device".to_string(),
            0x2300 => "Current, device output side".to_string(),
            0x3000 => "Voltage Error".to_string(),
            0x3100 => "Mains Voltage Error".to_string(),
            0x3200 => "Voltage inside the device".to_string(),
            0x3300 => "Output Voltage Error".to_string(),
            0x4000 => "Temperature Error".to_string(),
            0x4100 => "Ambient Temperature".to_string(),
            0x4200 => "Device Temperature".to_string(),
            0x5000 => "Hardware Error".to_string(),
            0x6000 => "Software Error".to_string(),
            0x6100 => "Internal Software Error".to_string(),
            0x6200 => "User Software Error".to_string(),
            0x6300 => "Data Set Error".to_string(),
            0x7000 => "Additional Modules Error".to_string(),
            0x8000 => "Monitoring Error".to_string(),
            0x8100 => "Communication Error".to_string(),
            0x8110 => "CAN Overrun".to_string(),
            0x8120 => "CAN Error Passive".to_string(),
            0x8130 => "Life Guard Error / Heartbeat Error".to_string(),
            0x8140 => "Recovered from Bus Off".to_string(),
            0x8200 => "Protocol Error".to_string(),
            0x8210 => "PDO not processed".to_string(),
            0x8220 => "PDO length exceeded".to_string(),
            0x9000 => "External Error".to_string(),
            0xF000 => "Additional Functions Error".to_string(),
            0xFF00 => "Device Specific Error".to_string(),
            _ => format!("Unknown Error (0x{:04X})", self.error_code),
        }
    }
}

/// Heartbeat status.
#[derive(Debug, Clone)]
pub struct HeartbeatStatus {
    pub node_id: u8,
    pub producer_period_ms: Option<u32>,
    pub consumer_config: Option<u32>,
    pub last_heartbeat_ms: Option<u64>,
    pub alive: bool,
}

impl HeartbeatStatus {
    pub fn status_text(&self) -> &'static str {
        if self.alive {
            "Online"
        } else {
            "Offline"
        }
    }
}

/// Sync status.
#[derive(Debug, Clone, Default)]
pub struct SyncStatus {
    pub producer_enabled: bool,
    pub producer_period_us: u32,
    pub consumer_count: u32,
    pub last_sync_ms: Option<u64>,
}

/// Toolbar state.
#[derive(Debug, Clone)]
pub struct ToolbarState {
    pub bitrate: u32,
    pub paused: bool,
}

impl Default for ToolbarState {
    fn default() -> Self {
        Self {
            bitrate: 500000,
            paused: false,
        }
    }
}

/// Available CAN backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanBackend {
    Mock,
    SocketCan,
    Kvaser,
    Pcan,
    Zlg,
}

impl CanBackend {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mock => "Mock (Testing)",
            Self::SocketCan => "SocketCAN (Linux)",
            Self::Kvaser => "Kvaser",
            Self::Pcan => "PCAN",
            Self::Zlg => "ZLG",
        }
    }

    /// All available backends.
    pub fn all() -> &'static [CanBackend] {
        &[Self::Mock, Self::SocketCan, Self::Kvaser, Self::Pcan, Self::Zlg]
    }
}

/// Connection dialog state.
#[derive(Debug)]
pub struct ConnectionDialog {
    pub visible: bool,
    pub selected_backend: CanBackend,
    pub channel: String,
    pub bitrate: String,
    pub node_id: String,
}

impl Default for ConnectionDialog {
    fn default() -> Self {
        Self {
            visible: false,
            selected_backend: CanBackend::Mock,
            channel: "can0".to_string(),
            bitrate: "500000".to_string(),
            node_id: "0".to_string(),
        }
    }
}

/// Messages.
#[derive(Debug, Clone)]
pub enum Message {
    // View
    SwitchTab(Tab),
    SwitchPrimary(PrimaryTab),
    NodeSelected(u8),
    ToggleDetailPanel,

    // Connection
    ConnectMock,
    ShowConnectionDialog,
    HideConnectionDialog,
    ConnectionBackendChanged(CanBackend),
    ConnectionChannelChanged(String),
    ConnectionBitrateChanged(String),
    ConnectionNodeIdChanged(String),
    ConnectionConnect,
    Disconnect,
    ScanNodes,

    // Toolbar
    TogglePause,
    BitrateChanged(u32),
    ClearLog,
    ExportLog,
    ImportLog,

    // NMT
    NmtStartNode(u8),
    NmtStopNode(u8),
    NmtResetNode(u8),
    NmtStartAll,
    NmtStopAll,

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
    Ds402ModeChanged(Ds402Mode),

    // Frame detail
    FrameSelected(LogEntry),

    // CAN log filter
    LogFilterChanged(String),
    LogClear,

    // Tick (polls backend events)
    Tick,
}
