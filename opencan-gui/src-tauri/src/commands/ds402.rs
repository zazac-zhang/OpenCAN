//! DS402 commands: enable, fault reset, mode set, target set.

use crate::state::SharedState;
use serde::Deserialize;

#[tauri::command]
pub async fn ds402_enable(
    state: tauri::State<'_, SharedState>,
    node_id: u8,
) -> Result<(), String> {
    let _ = node_id;
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    // TODO: implement via CanopenStack DS402 enable sequence
    Ok(())
}

#[tauri::command]
pub async fn ds402_fault_reset(
    state: tauri::State<'_, SharedState>,
    node_id: u8,
) -> Result<(), String> {
    let _ = node_id;
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    // TODO: implement via CanopenStack DS402 fault reset
    Ok(())
}

#[tauri::command]
pub async fn ds402_set_mode(
    state: tauri::State<'_, SharedState>,
    node_id: u8,
    mode: u8,
) -> Result<(), String> {
    let _ = (node_id, mode);
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    // TODO: implement via CanopenStack DS402 mode set
    Ok(())
}

#[tauri::command]
pub async fn ds402_set_target(
    state: tauri::State<'_, SharedState>,
    node_id: u8,
    mode: u8,
    target: f64,
) -> Result<(), String> {
    let _ = (node_id, mode, target);
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    // TODO: implement via CanopenStack DS402 target set
    Ok(())
}
