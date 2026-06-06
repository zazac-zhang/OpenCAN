//! SDO commands: upload (read) and download (write).

use crate::state::SharedStack;
use opencan_canopen_core::od::{DataType, OdValue};
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

fn parse_data_type(name: &str) -> Result<DataType, String> {
    match name {
        "UNS8" => Ok(DataType::Unsigned8),
        "UNS16" => Ok(DataType::Unsigned16),
        "UNS32" => Ok(DataType::Unsigned32),
        "UNS64" => Ok(DataType::Unsigned64),
        "INT8" => Ok(DataType::Integer8),
        "INT16" => Ok(DataType::Integer16),
        "INT32" => Ok(DataType::Integer32),
        "INT64" => Ok(DataType::Integer64),
        "BOOLEAN" => Ok(DataType::Boolean),
        "REAL32" => Ok(DataType::Real32),
        "REAL64" => Ok(DataType::Real64),
        "VISIBLE_STRING" => Ok(DataType::VisibleString),
        "OCTET_STRING" => Ok(DataType::OctetString),
        "DOMAIN" => Ok(DataType::Domain),
        other => Err(format!("Unknown data type: {}", other)),
    }
}

#[tauri::command]
pub async fn sdo_upload(
    stack_state: tauri::State<'_, SharedStack>,
    params: SdoUploadParams,
) -> Result<SdoResult, String> {
    let data_type = parse_data_type(&params.data_type)?;

    let result = {
        let mut guard = stack_state.lock().await;
        guard
            .sdo_upload(params.node_id, params.index, params.subindex, data_type)
            .await
    };

    match result {
        Ok(value) => Ok(SdoResult {
            node_id: params.node_id,
            index: params.index,
            subindex: params.subindex,
            data: value.to_bytes(),
            data_type: params.data_type,
        }),
        Err(e) => Err(format!("{:?}", e)),
    }
}

#[tauri::command]
pub async fn sdo_download(
    stack_state: tauri::State<'_, SharedStack>,
    params: SdoDownloadParams,
) -> Result<(), String> {
    // Parse bytes into an OdValue based on expected type
    // For now, use Domain (raw bytes) which works for any size
    let value = OdValue::Domain(params.data.clone());

    let result = {
        let mut guard = stack_state.lock().await;
        guard
            .sdo_download(params.node_id, params.index, params.subindex, &value)
            .await
    };

    result.map_err(|e| format!("{:?}", e))
}
