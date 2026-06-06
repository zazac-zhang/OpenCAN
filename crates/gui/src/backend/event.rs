//! Backend events sent from async task to GUI.

/// Backend event sent from async task to GUI.
#[derive(Debug, Clone)]
pub enum BackendEvent {
    // === Connection ===
    /// Connected successfully.
    Connected(String),
    /// Disconnected.
    Disconnected,
    /// Error occurred.
    Error(String),

    // === Node Scanning ===
    /// Node scan result.
    ScanResult(Vec<u8>),

    // === CAN Frames ===
    /// CAN frame received (for logging).
    FrameReceived {
        cob_id: u16,
        data: [u8; 8],
        dlc: u8,
        timestamp_ms: u64,
    },

    // === SDO ===
    /// SDO result.
    SdoResult {
        node_id: u8,
        index: u16,
        subindex: u8,
        result: Result<Vec<u8>, String>,
    },

    // === NMT ===
    /// NMT state changed.
    NmtStateChanged {
        node_id: u8,
        state: String,
    },

    // === Heartbeat ===
    /// Heartbeat state changed.
    HeartbeatChanged {
        node_id: u8,
        alive: bool,
        timestamp_ms: u64,
    },

    // === EMCY ===
    /// Emergency error received.
    EmcyReceived {
        node_id: u8,
        error_code: u16,
        error_register: u8,
        data: [u8; 5],
        timestamp_ms: u64,
    },

    // === SYNC ===
    /// SYNC received.
    SyncReceived {
        timestamp_ms: u64,
    },

    // === DS402 ===
    /// DS402 state result.
    Ds402StateResult {
        node_id: u8,
        state: String,
        status_word: u16,
    },
    /// DS402 position result.
    Ds402PositionResult {
        node_id: u8,
        position: i32,
    },
    /// DS402 velocity result.
    Ds402VelocityResult {
        node_id: u8,
        velocity: i32,
    },
    /// DS402 torque result.
    Ds402TorqueResult {
        node_id: u8,
        torque: i16,
    },

    // === Bus Statistics ===
    /// Bus statistics update.
    BusStatsUpdate {
        bus_load: f32,
        frame_rate: u32,
        tx_errors: u32,
        rx_errors: u32,
    },
    /// Error frame received.
    ErrorFrameReceived {
        timestamp_ms: u64,
        error_type: u8,
        tec: u8,
        rec: u8,
    },
}

impl BackendEvent {
    /// Get a short description of this event.
    pub fn description(&self) -> String {
        match self {
            Self::Connected(info) => format!("Connected: {}", info),
            Self::Disconnected => "Disconnected".to_string(),
            Self::Error(e) => format!("Error: {}", e),
            Self::ScanResult(nodes) => format!("Found {} nodes", nodes.len()),
            Self::FrameReceived { cob_id, .. } => format!("Frame {:03X}", cob_id),
            Self::SdoResult { index, subindex, result, .. } => {
                if result.is_ok() {
                    format!("SDO {:04X}:{:02X} OK", index, subindex)
                } else {
                    format!("SDO {:04X}:{:02X} Error", index, subindex)
                }
            }
            Self::NmtStateChanged { node_id, state } => {
                format!("Node {} → {}", node_id, state)
            }
            Self::HeartbeatChanged { node_id, alive, .. } => {
                format!("Node {} heartbeat {}", node_id, if *alive { "alive" } else { "lost" })
            }
            Self::EmcyReceived { node_id, error_code, .. } => {
                format!("EMCY Node {} 0x{:04X}", node_id, error_code)
            }
            Self::SyncReceived { .. } => "SYNC received".to_string(),
            Self::Ds402StateResult { node_id, state, .. } => {
                format!("DS402 Node {} → {}", node_id, state)
            }
            Self::Ds402PositionResult { node_id, position } => {
                format!("DS402 Node {} pos: {}", node_id, position)
            }
            Self::Ds402VelocityResult { node_id, velocity } => {
                format!("DS402 Node {} vel: {}", node_id, velocity)
            }
            Self::Ds402TorqueResult { node_id, torque } => {
                format!("DS402 Node {} torque: {}", node_id, torque)
            }
            Self::BusStatsUpdate { bus_load, frame_rate, .. } => {
                format!("Bus: {:.1}% load, {} fps", bus_load, frame_rate)
            }
            Self::ErrorFrameReceived { error_type, .. } => {
                format!("Error frame: type {}", error_type)
            }
        }
    }
}
