//! Recording commands: start/stop recording, load/playback sessions.

use crate::state::SharedState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RecordingMeta {
    pub path: String,
    pub frame_count: u64,
    pub duration_ms: u64,
    pub start_time: String,
}

#[tauri::command]
pub async fn start_recording(
    state: tauri::State<'_, SharedState>,
    path: String,
) -> Result<(), String> {
    let _ = (state, path);
    // TODO: implement session recorder
    Ok(())
}

#[tauri::command]
pub async fn stop_recording(state: tauri::State<'_, SharedState>) -> Result<(), String> {
    let _ = state;
    // TODO: implement session recorder stop
    Ok(())
}

#[tauri::command]
pub async fn load_recording(path: String) -> Result<RecordingMeta, String> {
    let _ = path;
    // TODO: implement recording file loader
    Ok(RecordingMeta {
        path: String::new(),
        frame_count: 0,
        duration_ms: 0,
        start_time: String::new(),
    })
}

#[tauri::command]
pub async fn start_playback(speed: f64) -> Result<(), String> {
    let _ = speed;
    // TODO: implement playback engine
    Ok(())
}

#[tauri::command]
pub async fn stop_playback() -> Result<(), String> {
    // TODO: implement playback stop
    Ok(())
}
