//! Tauri event channels for continuous data streams.
//!
//! Channels batch and emit events to the frontend via Tauri's event system.

use tauri::{AppHandle, Emitter};

/// CAN frame event emitted to frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CanFrameEvent {
    pub cob_id: u16,
    pub data: Vec<u8>,
    pub dlc: u8,
    pub timestamp_ms: u64,
    pub direction: String, // "tx" | "rx"
}

/// PDO event emitted to frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PdoEvent {
    pub node_id: u8,
    pub pdo_type: String, // "tpdo" | "rpdo"
    pub cob_id: u16,
    pub data: Vec<u8>,
    pub timestamp_ms: u64,
}

/// Log event emitted to frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEvent {
    pub level: String,
    pub message: String,
    pub timestamp_ms: u64,
}

/// EMCY event emitted to frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EmcyEvent {
    pub node_id: u8,
    pub error_code: u16,
    pub error_register: u8,
    pub data: [u8; 5],
    pub timestamp_ms: u64,
}

/// Heartbeat event emitted to frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HeartbeatEvent {
    pub node_id: u8,
    pub state: String,
    pub timestamp_ms: u64,
}

/// DS402 state event emitted to frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Ds402StateEvent {
    pub node_id: u8,
    pub state: String,
    pub status_word: u16,
    pub actual_position: i32,
    pub actual_velocity: i32,
    pub actual_torque: i16,
}

/// Bus statistics event emitted to frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BusStatsEvent {
    pub bus_load: f64,
    pub frame_rate: u32,
    pub tx_errors: u64,
    pub rx_errors: u64,
}

/// Error frame event emitted to frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorFrameEvent {
    pub timestamp_ms: u64,
    pub error_type: String, // "Bus Off" | "Error Passive" | "Warning"
    pub tec: u32,           // Transmit Error Counter
    pub rec: u32,           // Receive Error Counter
}

/// Channel manager for emitting events to the frontend.
#[derive(Clone)]
pub struct Channels {
    pub app_handle: AppHandle,
}

impl Channels {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub fn emit_frame(&self, event: CanFrameEvent) {
        let _ = self.app_handle.emit("frame_stream", event);
    }

    pub fn emit_frame_batch(&self, events: Vec<CanFrameEvent>) {
        let _ = self.app_handle.emit("frame_stream_batch", events);
    }

    pub fn emit_pdo(&self, event: PdoEvent) {
        let _ = self.app_handle.emit("pdo_stream", event);
    }

    pub fn emit_pdo_batch(&self, events: Vec<PdoEvent>) {
        let _ = self.app_handle.emit("pdo_stream_batch", events);
    }

    pub fn emit_log(&self, event: LogEvent) {
        let _ = self.app_handle.emit("log_stream", event);
    }

    pub fn emit_emcy(&self, event: EmcyEvent) {
        let _ = self.app_handle.emit("emcy_stream", event);
    }

    pub fn emit_heartbeat(&self, event: HeartbeatEvent) {
        let _ = self.app_handle.emit("heartbeat_stream", event);
    }

    pub fn emit_ds402_state(&self, event: Ds402StateEvent) {
        let _ = self.app_handle.emit("ds402_state_stream", event);
    }

    pub fn emit_bus_stats(&self, event: BusStatsEvent) {
        let _ = self.app_handle.emit("bus_stats_stream", event);
    }

    pub fn emit_error_frame(&self, event: ErrorFrameEvent) {
        let _ = self.app_handle.emit("error_frame_stream", event);
    }
}
