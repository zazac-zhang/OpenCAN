//! Recording commands: start/stop recording, load/playback sessions.

use crate::state::SharedStack;
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::frame::CanOpenFrame;
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct RecordingMeta {
    pub path: String,
    pub frame_count: u64,
    pub duration_ms: u64,
    pub start_time: String,
}

/// Raw frame record for JSON recording format.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FrameRecord {
    cob_id: u16,
    data: Vec<u8>,
    timestamp_ms: u64,
}

#[tauri::command]
pub async fn start_recording(
    stack_state: tauri::State<'_, SharedStack>,
    path: String,
) -> Result<(), String> {
    // Create the recording file
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    // Write initial metadata
    let meta = serde_json::json!({
        "type": "recording_meta",
        "start_time_ms": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    });
    writeln!(file, "{}", meta).map_err(|e| e.to_string())?;

    // Store the file handle somewhere accessible...
    // For simplicity, we'll just confirm the path is writable.
    // A proper implementation would store the writer in AppState.
    let _ = (stack_state, file);

    Ok(())
}

#[tauri::command]
pub async fn stop_recording(stack_state: tauri::State<'_, SharedStack>) -> Result<(), String> {
    let _ = stack_state;
    // In a proper implementation, this would close the recording file handle
    Ok(())
}

#[tauri::command]
pub async fn load_recording(path: String) -> Result<RecordingMeta, String> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut frame_count = 0u64;
    let mut start_time_ms = 0u64;
    let mut end_time_ms = 0u64;

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let record: serde_json::Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;

        if record.get("type").and_then(|t| t.as_str()) == Some("frame") {
            frame_count += 1;
            if let Some(ts) = record.get("timestamp_ms").and_then(|v| v.as_u64()) {
                if start_time_ms == 0 || ts < start_time_ms {
                    start_time_ms = ts;
                }
                if ts > end_time_ms {
                    end_time_ms = ts;
                }
            }
        } else if record.get("type").and_then(|t| t.as_str()) == Some("recording_meta") {
            start_time_ms = record
                .get("start_time_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
        }
    }

    let duration_ms = end_time_ms.saturating_sub(start_time_ms);

    Ok(RecordingMeta {
        path: path.to_string_lossy().to_string(),
        frame_count,
        duration_ms,
        start_time: chrono_lite_format(start_time_ms),
    })
}

#[tauri::command]
pub async fn start_playback(
    stack_state: tauri::State<'_, SharedStack>,
    path: String,
    speed: f64,
) -> Result<(), String> {
    let path = Path::new(&path);
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let mut records: Vec<FrameRecord> = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let record: serde_json::Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;

        if record.get("type").and_then(|t| t.as_str()) == Some("frame") {
            if let Ok(fr) = serde_json::from_value::<FrameRecord>(record) {
                records.push(fr);
            }
        }
    }

    if records.is_empty() {
        return Err("No frames found in recording".to_string());
    }

    // Sort by timestamp
    records.sort_by_key(|r| r.timestamp_ms);
    let base_time = records[0].timestamp_ms;

    // Play back frames
    for record in records {
        let frame = CanOpenFrame::new(record.cob_id, {
            let mut data = [0u8; 8];
            let len = record.data.len().min(8);
            data[..len].copy_from_slice(&record.data[..len]);
            data
        });

        // Send via stack
        {
            let mut guard = stack_state.lock().await;
            let _ = guard.can_mut().send(&frame);
        }

        // Wait according to speed-adjusted timing
        let delay_ms = ((record.timestamp_ms - base_time) as f64 / speed) as u64;
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_playback() -> Result<(), String> {
    // In a proper implementation, this would signal the playback task to stop
    Ok(())
}

fn chrono_lite_format(ms: u64) -> String {
    let secs = ms / 1000;
    let millis = ms % 1000;
    // Simple format without chrono dependency
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, millis)
}
