//! SYNC commands: start/stop SYNC producer.

use crate::state::SharedState;

#[tauri::command]
pub async fn start_sync(
    state: tauri::State<'_, SharedState>,
    period_us: u32,
) -> Result<(), String> {
    let _ = period_us;
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    // TODO: implement via CanopenStack::enable_sync_production
    Ok(())
}

#[tauri::command]
pub async fn stop_sync(state: tauri::State<'_, SharedState>) -> Result<(), String> {
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    // TODO: implement via CanopenStack::disable_sync_production
    Ok(())
}
