//! Connection commands: connect, disconnect, list backends.

use crate::state::{AppState, BackendDescriptor, BackendInfo, SharedState};
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
    state: tauri::State<'_, SharedState>,
    params: ConnectParams,
) -> Result<BackendInfo, String> {
    let mut guard = state.write().await;
    guard.connected = true;
    guard.backend_info = Some(BackendInfo {
        backend_type: params.backend_type.clone(),
        channel: params.channel.clone(),
        bitrate: params.bitrate,
        node_id: params.node_id,
    });
    Ok(guard.backend_info.clone().unwrap())
}

#[tauri::command]
pub async fn disconnect(state: tauri::State<'_, SharedState>) -> Result<(), String> {
    let mut guard = state.write().await;
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

    // Other backends are stubs
    for (name, btype) in &[("Kvaser", "kvaser"), ("PCAN", "pcan"), ("ZLG", "zlg")] {
        backends.push(BackendDescriptor {
            name: name.to_string(),
            backend_type: btype.to_string(),
            available: false,
        });
    }

    Ok(backends)
}
