//! PDO commands: read PDO mapping.

use crate::state::{SharedStack, SharedState};
use opencan_canopen_core::od::OdValue;
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
    app_state: tauri::State<'_, SharedState>,
    stack_state: tauri::State<'_, SharedStack>,
    node_id: u8,
    pdo_index: u8,
) -> Result<PdoMapping, String> {
    // Check connection
    {
        let guard = app_state.lock().await;
        if !guard.connected {
            return Err("Not connected".to_string());
        }
    }

    // PDO mapping is at 0x1A00 + (pdo_index - 1) for TPDO
    // and 0x1600 + (pdo_index - 1) for RPDO
    let tpdo_map_index = 0x1A00 + (pdo_index - 1) as u16;

    // First, read the COB-ID from PDO communication parameter (0x1800 + idx, subindex 1)
    let cob_id = {
        let mut guard = stack_state.lock().await;
        guard
            .sdo_upload_u32(node_id, 0x1800 + (pdo_index - 1) as u16, 1)
            .await
    };

    let cob_id = match cob_id {
        Ok(OdValue::Unsigned32(v)) => v as u16,
        Ok(OdValue::Unsigned16(v)) => v,
        Err(e) => return Err(format!("Failed to read COB-ID: {:?}", e)),
        _ => return Err("Invalid COB-ID type".to_string()),
    };

    // Read the number of mapped objects (subindex 0)
    let num_entries = {
        let mut guard = stack_state.lock().await;
        guard
            .sdo_upload_u32(node_id, tpdo_map_index, 0)
            .await
    };

    let num_entries = match num_entries {
        Ok(OdValue::Unsigned32(v)) => v as u8,
        Err(e) => return Err(format!("Failed to read mapping count: {:?}", e)),
        _ => return Err("Invalid mapping count".to_string()),
    };

    // Read each mapped object entry
    let mut entries = Vec::new();
    for i in 1..=num_entries {
        let entry = {
            let mut guard = stack_state.lock().await;
            guard
                .sdo_upload_u32(node_id, tpdo_map_index, i)
                .await
        };

        if let Ok(OdValue::Unsigned32(v)) = entry {
            // Format: bit_length(8) | subindex(8) | index(16)
            let bit_length = ((v >> 24) & 0xFF) as u8;
            let subindex = ((v >> 16) & 0xFF) as u8;
            let index = (v & 0xFFFF) as u16;
            entries.push(PdoMappingEntry {
                index,
                subindex,
                bit_length,
            });
        }
    }

    Ok(PdoMapping { cob_id, entries })
}
