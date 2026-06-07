//! Connection commands: connect, disconnect, list backends, send frames.

use crate::state::{BackendDescriptor, BackendInfo, SharedStack, SharedState};
use opencan_canopen_core::frame::CanOpenFrame;
use opencan_canopen_core::testing::MockCanDriver;
use opencan_canopen_ds301::stack::CanopenStack;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConnectParams {
    pub backend_type: String,
    pub channel: String,
    pub bitrate: u32,
    pub node_id: u8,
}

#[tauri::command]
pub async fn connect_backend(
    app_state: tauri::State<'_, SharedState>,
    stack_state: tauri::State<'_, SharedStack>,
    params: ConnectParams,
) -> Result<BackendInfo, String> {
    // Reset the stack with a fresh mock driver
    let new_stack = CanopenStack::new(MockCanDriver::new(), params.node_id);
    {
        let mut guard = stack_state.lock().await;
        *guard = new_stack;
    }

    let info = BackendInfo {
        backend_type: params.backend_type.clone(),
        channel: params.channel.clone(),
        bitrate: params.bitrate,
        node_id: params.node_id,
    };

    {
        let mut guard = app_state.lock().await;
        guard.connected = true;
        guard.backend_info = Some(info.clone());
    }

    Ok(info)
}

#[tauri::command]
pub async fn disconnect(
    app_state: tauri::State<'_, SharedState>,
) -> Result<(), String> {
    let mut guard = app_state.lock().await;
    guard.connected = false;
    guard.backend_info = None;
    guard.nodes.clear();
    Ok(())
}

#[tauri::command]
pub async fn get_backends() -> Result<Vec<BackendDescriptor>, String> {
    let mut backends = Vec::new();

    backends.push(BackendDescriptor {
        name: "Mock".to_string(),
        backend_type: "mock".to_string(),
        available: true,
    });

    #[cfg(feature = "socketcan")]
    backends.push(BackendDescriptor {
        name: "SocketCAN".to_string(),
        backend_type: "socketcan".to_string(),
        available: true,
    });

    #[cfg(not(feature = "socketcan"))]
    backends.push(BackendDescriptor {
        name: "SocketCAN".to_string(),
        backend_type: "socketcan".to_string(),
        available: false,
    });

    for (name, btype) in &[("Kvaser", "kvaser"), ("PCAN", "pcan"), ("ZLG", "zlg")] {
        backends.push(BackendDescriptor {
            name: name.to_string(),
            backend_type: btype.to_string(),
            available: false,
        });
    }

    Ok(backends)
}

/// Send a raw CANOpen frame on the bus
#[tauri::command]
pub async fn send_frame(
    app_state: tauri::State<'_, SharedState>,
    stack_state: tauri::State<'_, SharedStack>,
    cob_id: u16,
    data: Vec<u8>,
) -> Result<(), String> {
    let guard = app_state.lock().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    drop(guard);

    let mut frame_data = [0u8; 8];
    let len = data.len().min(8);
    frame_data[..len].copy_from_slice(&data[..len]);

    let frame = CanOpenFrame::new(cob_id, frame_data);

    let mut guard = stack_state.lock().await;
    guard.send_frame(frame).map_err(|e| format!("{:?}", e))
}
