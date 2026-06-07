//! Global application state shared across Tauri commands and event loops.

use crate::channels::Channels;
use opencan_canopen_ds301::stack::{CanEvent, CanopenStack};
use opencan_canopen_core::frame::CanOpenFrame;
use opencan_canopen_core::testing::MockCanDriver;
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::od::ObjectDictionary;
#[cfg(feature = "eds")]
use opencan_canopen_core::eds::model::{EdsEntry, EdsSubEntry};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::AtomicBool;
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
    /// Recording file handle (writer)
    pub recording_file: Option<Arc<Mutex<BufWriter<File>>>>,
    /// Playback cancellation flag
    pub playback_running: Arc<AtomicBool>,
    /// Built Object Dictionary from loaded EDS
#[allow(dead_code)]
    pub object_dictionary: Option<Arc<dyn ObjectDictionary + Send + Sync>>,
    /// EDS objects for frontend display
    pub eds_objects: Vec<EdsObjectInfo>,
    /// EDS sub-entries for frontend display
    pub eds_sub_entries: Vec<EdsObjectInfo>,
    /// Bus stats tracking
    pub frame_count_window: u32,
    pub error_frame_count: u64,
    pub tx_errors: u64,
    pub rx_errors: u64,
    pub last_stats_emit: std::time::Instant,
    /// DS402 state cache per node
    pub ds402_states: HashMap<u8, Ds402StateCache>,
}

/// DS402 state cached for periodic emission.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Ds402StateCache {
    pub status_word: u16,
    pub actual_position: i32,
    pub actual_velocity: i32,
    pub actual_torque: i16,
    pub mode_of_operation: i8,
}

/// Simplified EDS entry info for frontend display.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EdsObjectInfo {
    pub index: u16,
    pub subindex: u8,
    pub name: String,
    pub object_type: String,
    pub data_type: Option<u16>,
    pub access_type: Option<String>,
    pub default_value: Option<String>,
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
            recording_file: None,
            playback_running: Arc::new(AtomicBool::new(false)),
            object_dictionary: None,
            eds_objects: Vec::new(),
            eds_sub_entries: Vec::new(),
            frame_count_window: 0,
            error_frame_count: 0,
            tx_errors: 0,
            rx_errors: 0,
            last_stats_emit: std::time::Instant::now(),
            ds402_states: HashMap::new(),
        }
    }
}

/// Start the backend event loop that processes CAN frames and emits events.
/// Returns a handle to keep the task alive.
pub fn start_backend_loop(
    stack: SharedStack,
    channels: Channels,
    app_state: SharedState,
) -> BackendHandle {
    let task = tokio::spawn(async move {
        use std::time::Duration;

        let mut ds402_poll_counter: u64 = 0;

        loop {
            // Acquire lock, process one frame or poll periodic tasks
            {
                let mut guard = stack.lock().await;

                // Try to receive a frame (non-blocking)
                // MockCanDriver has pre-queued frames; real hardware would block here.
                if let Ok(frame) = guard.can_mut().recv().await {
                    emit_raw_frame(&channels, &frame);
                    // Track for recording
                    emit_recording_frame(&app_state, &frame).await;
                    // Track for bus stats
                    {
                        let mut app = app_state.lock().await;
                        app.frame_count_window += 1;
                        // Detect error frames by COB-ID range (0x000-0x07F are special, errors often use 0x07F-like)
                        if frame.cob_id >= 0x070 && frame.cob_id <= 0x07F {
                            app.error_frame_count += 1;
                            emit_error_frame_from_frame(&channels, &frame, current_millis());
                        }
                    }

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

            // Periodic bus stats emission (every 1000ms)
            {
                let mut app = app_state.lock().await;
                let now = std::time::Instant::now();
                if now.duration_since(app.last_stats_emit) >= Duration::from_secs(1) {
                    let frame_rate = app.frame_count_window;
                    // Estimate bus load: assume 115 bits per frame at 500kbps
                    let bus_load = (frame_rate as f64 * 115.0 / 500_000.0 * 100.0).min(100.0);
                    channels.emit_bus_stats(crate::channels::BusStatsEvent {
                        bus_load,
                        frame_rate,
                        tx_errors: app.tx_errors,
                        rx_errors: app.rx_errors,
                    });
                    app.frame_count_window = 0;
                    app.last_stats_emit = now;
                }
            }

            // Periodic DS402 state emission (every 500ms) — synthetic for MockCanDriver
            ds402_poll_counter += 1;
            if ds402_poll_counter % 50 == 0 {
                emit_synthetic_ds402_state(&channels, &app_state).await;
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

/// Write a frame to the active recording file.
async fn emit_recording_frame(app_state: &SharedState, frame: &CanOpenFrame) {
    let guard = app_state.lock().await;
    if let Some(ref writer) = guard.recording_file {
        let record = serde_json::json!({
            "type": "frame",
            "cob_id": frame.cob_id,
            "data": frame.data.to_vec(),
            "timestamp_ms": current_millis(),
        });
        let mut w = writer.lock().await;
        let _ = writeln!(w, "{}", record);
        let _ = w.flush();
    }
}

/// Emit an error frame event when a frame looks like an error frame.
fn emit_error_frame_from_frame(channels: &Channels, frame: &CanOpenFrame, now: u64) {
    // Classify error type based on COB-ID and data patterns
    let error_type = if frame.cob_id == 0x07F {
        "Bus Off"
    } else if frame.data.iter().any(|&b| b > 127) {
        "Error Passive"
    } else {
        "Warning"
    };

    // Estimate TEC/REC from frame data if available
    let tec = frame.data.first().copied().unwrap_or(0) as u32;
    let rec = frame.data.get(1).copied().unwrap_or(0) as u32;

    channels.emit_error_frame(crate::channels::ErrorFrameEvent {
        timestamp_ms: now,
        error_type: error_type.to_string(),
        tec,
        rec,
    });
}

/// Emit synthetic DS402 state for demonstration (MockCanDriver doesn't produce real DS402 data).
async fn emit_synthetic_ds402_state(channels: &Channels, app_state: &SharedState) {
    let guard = app_state.lock().await;
    // If we have DS402 states from previous commands, emit them with slight variation
    for (&node_id, cache) in &guard.ds402_states {
        channels.emit_ds402_state(crate::channels::Ds402StateEvent {
            node_id,
            state: ds402_state_from_status_word(cache.status_word).to_string(),
            status_word: cache.status_word,
            actual_position: cache.actual_position,
            actual_velocity: cache.actual_velocity,
            actual_torque: cache.actual_torque,
        });
    }
    // If no DS402 states tracked yet, emit a demo state for node 1
    if guard.ds402_states.is_empty() && guard.connected {
        channels.emit_ds402_state(crate::channels::Ds402StateEvent {
            node_id: 1,
            state: "Operation Enabled".to_string(),
            status_word: 0x0237, // Ready to Switch On + Switched On + Operation Enabled
            actual_position: 0,
            actual_velocity: 0,
            actual_torque: 0,
        });
    }
}

/// Decode a DS402 state string from StatusWord value.
fn ds402_state_from_status_word(sw: u16) -> &'static str {
    if sw & (1 << 3) != 0 {
        "Fault"
    } else if sw & (1 << 2) != 0 {
        "Operation Enabled"
    } else if sw & (1 << 1) != 0 {
        "Switched On"
    } else if sw & (1 << 0) != 0 {
        "Ready to Switch On"
    } else if sw & (1 << 6) != 0 {
        "Switch On Disabled"
    } else {
        "Not Ready to Switch On"
    }
}
