//! EDS commands: load and parse EDS files.

use crate::state::{EdsObjectInfo, SharedState};
use opencan_canopen_core::od::ObjectType;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EdsInfo {
    pub product_name: String,
    pub vendor_id: u32,
    pub product_code: u32,
    pub revision_number: u32,
    pub baud_rate: u32,
}

#[allow(dead_code)]
fn object_type_to_string(ot: &ObjectType) -> String {
    format!("{}", ot)
}

#[tauri::command]
pub async fn load_eds_file(
    app_state: tauri::State<'_, SharedState>,
    path: String,
) -> Result<EdsInfo, String> {
    #[cfg(feature = "eds")]
    {
        use opencan_canopen_core::eds::builder::build_od;
        use opencan_canopen_core::eds::parser::parse_eds;
        use std::sync::Arc;

        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let eds = parse_eds(&content).map_err(|e| format!("EDS parse error: {:?}", e))?;

        let product_name = eds
            .device_info
            .product_name
            .as_deref()
            .unwrap_or("Unknown")
            .to_string();
        let vendor_id = eds.device_info.vendor_number.unwrap_or(0);
        let product_code = eds.device_info.product_code.unwrap_or(0);
        let revision_number = eds.device_info.revision_number.unwrap_or(0);
        let baud_rate = eds.device_info.baud_rate.unwrap_or(0);

        // Build OD from EDS and store in AppState
        let od = build_od(&eds);

        // Convert EDS entries to frontend-displayable info
        let mut eds_objects: Vec<EdsObjectInfo> = Vec::new();
        let mut eds_sub_entries: Vec<EdsObjectInfo> = Vec::new();

        for (index, entry) in &eds.entries {
            eds_objects.push(EdsObjectInfo {
                index: *index,
                subindex: 0,
                name: entry.parameter_name.clone(),
                object_type: object_type_to_string(&entry.object_type),
                data_type: entry.data_type,
                access_type: entry.access_type.clone(),
                default_value: entry.default_value.clone(),
            });
        }

        for ((index, subindex), sub_entry) in &eds.sub_entries {
            eds_sub_entries.push(EdsObjectInfo {
                index: *index,
                subindex: *subindex,
                name: sub_entry.parameter_name.clone(),
                object_type: "VAR".to_string(),
                data_type: sub_entry.data_type,
                access_type: sub_entry.access_type.clone(),
                default_value: sub_entry.default_value.clone(),
            });
        }

        {
            let mut guard = app_state.lock().await;
            guard.eds_objects = eds_objects;
            guard.eds_sub_entries = eds_sub_entries;
            // Store OD for later use (can be queried via get_od_entries)
            guard.object_dictionary = Some(Arc::new(od));
        }

        Ok(EdsInfo {
            product_name,
            vendor_id,
            product_code,
            revision_number,
            baud_rate,
        })
    }

    #[cfg(not(feature = "eds"))]
    {
        let _ = path;
        let _ = app_state;
        Err("EDS feature not enabled. Build with --features eds".to_string())
    }
}

/// Get Object Dictionary entries from loaded EDS.
#[tauri::command]
pub async fn get_od_entries(
    app_state: tauri::State<'_, SharedState>,
) -> Result<Vec<EdsObjectInfo>, String> {
    let guard = app_state.lock().await;
    let mut entries = guard.eds_objects.clone();
    // Append sub-entries
    entries.extend(guard.eds_sub_entries.clone());
    // Sort by index then subindex
    entries.sort_by(|a, b| a.index.cmp(&b.index).then(a.subindex.cmp(&b.subindex)));
    Ok(entries)
}
