//! SYNC commands: start/stop SYNC producer.

use crate::state::SharedStack;
use std::time::Duration;

#[tauri::command]
pub async fn start_sync(
    stack_state: tauri::State<'_, SharedStack>,
    period_us: u32,
) -> Result<(), String> {
    let mut guard = stack_state.lock().await;
    guard.enable_sync_production(Duration::from_micros(period_us as u64));
    Ok(())
}

#[tauri::command]
pub async fn stop_sync(stack_state: tauri::State<'_, SharedStack>) -> Result<(), String> {
    let mut guard = stack_state.lock().await;
    guard.disable_sync_production();
    Ok(())
}
