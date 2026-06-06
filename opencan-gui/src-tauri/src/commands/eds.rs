//! EDS commands: load and parse EDS files.

use crate::state::SharedState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EdsInfo {
    pub product_name: String,
    pub vendor_id: u32,
    pub product_code: u32,
    pub revision_number: u32,
    pub baud_rate: u32,
}

#[tauri::command]
pub async fn load_eds_file(
    state: tauri::State<'_, SharedState>,
    path: String,
) -> Result<EdsInfo, String> {
    let _ = path;
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    // TODO: implement via canopen-core EDS parser
    #[cfg(feature = "eds")]
    {
        // Parse EDS file
    }
    #[cfg(not(feature = "eds"))]
    {
        return Err("EDS feature not enabled".to_string());
    }
    Ok(EdsInfo {
        product_name: String::new(),
        vendor_id: 0,
        product_code: 0,
        revision_number: 0,
        baud_rate: 0,
    })
}
