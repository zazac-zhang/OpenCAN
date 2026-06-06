//! NMT commands: scan nodes, send NMT commands.

use crate::state::{NodeInfo, SharedStack, SharedState};
use opencan_canopen_core::error::CanOpenError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NmtCommandParams {
    pub node_id: u8,
    pub command: String,
}

#[tauri::command]
pub async fn scan_nodes(
    app_state: tauri::State<'_, SharedState>,
    stack_state: tauri::State<'_, SharedStack>,
    _timeout_ms: u32,
) -> Result<Vec<u8>, String> {
    // Check connection
    {
        let guard = app_state.lock().await;
        if !guard.connected {
            return Err("Not connected".to_string());
        }
    }

    // Run scan
    let node_ids = {
        let mut guard = stack_state.lock().await;
        guard.scan_nodes().await.map_err(|e| format!("{:?}", e))?
    };

    // Update node registry
    {
        let mut guard = app_state.lock().await;
        for &id in &node_ids {
            guard.nodes.insert(
                id,
                NodeInfo {
                    node_id: id,
                    nmt_state: "PreOperational".to_string(),
                    device_type: None,
                    vendor_id: None,
                    product_name: None,
                },
            );
        }
    }

    Ok(node_ids)
}

#[tauri::command]
pub async fn nmt_command(
    app_state: tauri::State<'_, SharedState>,
    stack_state: tauri::State<'_, SharedStack>,
    params: NmtCommandParams,
) -> Result<(), String> {
    let guard = app_state.lock().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    drop(guard);

    let result = {
        let mut guard = stack_state.lock().await;
        match params.command.as_str() {
            "start" => guard.nmt_start(params.node_id),
            "stop" => guard.nmt_stop(params.node_id),
            "reset" => guard.nmt_reset(params.node_id),
            "reset_comm" => guard.nmt_reset_communication(params.node_id),
            _ => Err(CanOpenError::Protocol(format!(
                "Unknown NMT command: {}",
                params.command
            ))),
        }
    };

    result.map_err(|e| format!("{:?}", e))?;

    // Update node state
    if params.node_id > 0 {
        let mut guard = app_state.lock().await;
        if let Some(node) = guard.nodes.get_mut(&params.node_id) {
            node.nmt_state = match params.command.as_str() {
                "start" => "Operational".to_string(),
                "stop" => "Stopped".to_string(),
                "reset" | "reset_comm" => "PreOperational".to_string(),
                _ => node.nmt_state.clone(),
            };
        }
    }

    Ok(())
}
