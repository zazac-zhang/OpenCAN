//! EDS commands: load and parse EDS files.

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
pub async fn load_eds_file(path: String) -> Result<EdsInfo, String> {
    #[cfg(feature = "eds")]
    {
        use opencan_canopen_core::eds::parser::parse_eds;
        use opencan_canopen_core::eds::builder::build_od;

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

        // Build OD from EDS (available for later use)
        let _od = build_od(&eds);

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
        Err("EDS feature not enabled. Build with --features eds".to_string())
    }
}
