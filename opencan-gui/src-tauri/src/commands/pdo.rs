//! PDO commands: read PDO mapping.

use crate::state::SharedState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PdoMapping {
    pub cob_id: u16,
    pub entries: Vec<PdoMappingEntry>,
}

#[derive(Debug, Serialize)]
pub struct PdoMappingEntry {
    pub index: u16,
    pub subindex: u8,
    pub bit_length: u8,
}

#[tauri::command]
pub async fn read_pdo_mapping(
    state: tauri::State<'_, SharedState>,
    node_id: u8,
    pdo_index: u8,
) -> Result<PdoMapping, String> {
    let _ = (node_id, pdo_index);
    let guard = state.read().await;
    if !guard.connected {
        return Err("Not connected".to_string());
    }
    // TODO: implement via SDO read of PDO mapping objects (0x1A00+, 0x1600+)
    Ok(PdoMapping {
        cob_id: 0,
        entries: vec![],
    })
}
