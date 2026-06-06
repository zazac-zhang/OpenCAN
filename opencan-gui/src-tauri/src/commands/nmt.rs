//! NMT commands: scan nodes, send NMT commands.

use crate::state::{AppState, NodeInfo, SharedState};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NmtCommandParams {
    pub node_id: u8,
    pub command: String,
}

#[tauri::command]
pub async fn scan_nodes(
    state: tauri::State<'_, SharedState>,
    timeout_ms: u32,
) -> Result<Vec<u8>, String> {
    // TODO: implement actual node scanning via CanopenStack
    // For now, return empty list
    let _timeout = timeout_ms;
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    Ok(guard.nodes.keys().copied().collect())
}

#[tauri::command]
pub async fn nmt_command(
    state: tauri::State<'_, SharedState>,
    params: NmtCommandParams,
) -> Result<(), String> {
    // TODO: implement actual NMT command via CanopenStack
    let _ = params;
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    Ok(())
}
