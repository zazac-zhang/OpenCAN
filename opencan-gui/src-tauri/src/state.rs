//! Global application state shared across Tauri commands and event loops.

use crate::channels::Channels;
use opencan_canopen_ds301::stack::{CanEvent, CanopenStack};
use opencan_canopen_core::frame::CanOpenFrame;
use opencan_canopen_core::testing::MockCanDriver;
use opencan_canopen_core::CanDriver;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Node information discovered on the CANOpen network.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeInfo {
    pub node_id: u8,
    pub nmt_state: String,
    pub device_type: Option<u32>,
    pub vendor_id: Option<u32>,
    pub product_name: Option<String>,
}

/// Backend connection information.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackendInfo {
    pub backend_type: String,
    pub channel: String,
    pub bitrate: u32,
    pub node_id: u8,
}

/// Descriptor for available CAN backends.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackendDescriptor {
    pub name: String,
    pub backend_type: String,
    pub available: bool,
}

/// Type alias for the concrete stack using MockCanDriver.
pub type AppStack = CanopenStack<MockCanDriver>;

/// Shared stack reference: Arc<Mutex<CanopenStack<MockCanDriver>>>
pub type SharedStack = Arc<Mutex<AppStack>>;

/// Shared application state accessible from Tauri commands.
pub struct AppState {
    pub connected: bool,
    pub backend_info: Option<BackendInfo>,
    pub nodes: HashMap<u8, NodeInfo>,
}

pub type SharedState = Arc<Mutex<AppState>>;

/// Handle for the backend event loop task.
pub struct BackendHandle {
    pub _task: tokio::task::JoinHandle<()>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connected: false,
            backend_info: None,
            nodes: HashMap::new(),
        }
    }
}

/// Start the backend event loop that processes CAN frames and emits events.
/// Returns a handle to keep the task alive.
pub fn start_backend_loop(
    stack: SharedStack,
    channels: Channels,
) -> BackendHandle {
    let task = tokio::spawn(async move {
        use std::time::Duration;

        loop {
            // Acquire lock, process one frame or poll periodic tasks
            {
                let mut guard = stack.lock().await;

                // Try to receive a frame (non-blocking)
                // MockCanDriver has pre-queued frames; real hardware would block here.
                if let Ok(frame) = guard.can_mut().recv().await {
                    emit_raw_frame(&channels, &frame);

                    let events = guard.process(&frame);
                    for event in events {
                        emit_event(&channels, event);
                    }
                }

                // Periodic: check timeouts via a dummy frame
                let dummy = CanOpenFrame::new(0x080, [0u8; 8]);
                let events = guard.process(&dummy);
                for event in events {
                    emit_event(&channels, event);
                }

                // Poll SYNC production
                let _ = guard.poll_sync();
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    BackendHandle { _task: task }
}

fn emit_raw_frame(channels: &Channels, frame: &CanOpenFrame) {
    channels.emit_frame(crate::channels::CanFrameEvent {
        cob_id: frame.cob_id,
        data: frame.data.to_vec(),
        dlc: 8,
        timestamp_ms: current_millis(),
        direction: "rx".to_string(),
    });
}

fn emit_event(channels: &Channels, event: CanEvent) {
    let now = current_millis();
    match event {
        CanEvent::HeartbeatChanged { node_id, alive } => {
            channels.emit_heartbeat(crate::channels::HeartbeatEvent {
                node_id,
                state: if alive { "alive" } else { "lost" }.to_string(),
                timestamp_ms: now,
            });
        }
        CanEvent::HeartbeatTimeout { node_id } => {
            channels.emit_heartbeat(crate::channels::HeartbeatEvent {
                node_id,
                state: "timeout".to_string(),
                timestamp_ms: now,
            });
        }
        CanEvent::Emergency {
            node_id,
            error_code,
        } => {
            channels.emit_emcy(crate::channels::EmcyEvent {
                node_id,
                error_code,
                error_register: 0,
                data: [0u8; 5],
                timestamp_ms: now,
            });
        }
        CanEvent::PdoReceived { pdo } => {
            let cob_id = pdo.cob_id;
            let node_id = if cob_id >= 0x180 && cob_id <= 0x57F {
                ((cob_id - 0x180) / 0x100 + 1) as u8
            } else {
                0
            };
            channels.emit_pdo(crate::channels::PdoEvent {
                node_id,
                pdo_type: if cob_id < 0x200 {
                    "tpdo"
                } else {
                    "rpdo"
                }
                .to_string(),
                cob_id,
                data: pdo.data.to_vec(),
                timestamp_ms: now,
            });
        }
        CanEvent::SdoComplete { .. } => {
            // Handled synchronously by command calls
        }
        CanEvent::SyncTriggered { .. } => {}
        CanEvent::SyncReceived { counter } => {
            channels.emit_log(crate::channels::LogEvent {
                level: "info".to_string(),
                message: format!("SYNC received, counter: {}", counter),
                timestamp_ms: now,
            });
        }
        CanEvent::TimestampReceived { ms_of_day, days } => {
            channels.emit_log(crate::channels::LogEvent {
                level: "info".to_string(),
                message: format!("TIME_STAMP: {}ms, day {}", ms_of_day, days),
                timestamp_ms: now,
            });
        }
    }
}

fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
