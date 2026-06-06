//! SDO commands: upload (read) and download (write).

use crate::state::SharedState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SdoUploadParams {
    pub node_id: u8,
    pub index: u16,
    pub subindex: u8,
    pub data_type: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SdoResult {
    pub node_id: u8,
    pub index: u16,
    pub subindex: u8,
    pub data: Vec<u8>,
    pub data_type: String,
}

#[derive(Debug, Deserialize)]
pub struct SdoDownloadParams {
    pub node_id: u8,
    pub index: u16,
    pub subindex: u8,
    pub data: Vec<u8>,
}

#[tauri::command]
pub async fn sdo_upload(
    state: tauri::State<'_, SharedState>,
    params: SdoUploadParams,
) -> Result<SdoResult, String> {
    // TODO: implement via CanopenStack::sdo_upload
    let _ = params;
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    Ok(SdoResult {
        node_id: params.node_id,
        index: params.index,
        subindex: params.subindex,
        data: vec![],
        data_type: params.data_type,
    })
}

#[tauri::command]
pub async fn sdo_download(
    state: tauri::State<'_, SharedState>,
    params: SdoDownloadParams,
) -> Result<(), String> {
    // TODO: implement via CanopenStack::sdo_download
    let _ = params;
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    Ok(())
}
