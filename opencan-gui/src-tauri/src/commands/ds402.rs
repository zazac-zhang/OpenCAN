//! DS402 commands: enable, fault reset, mode set, target set.

use crate::state::SharedStack;
use opencan_canopen_core::od::OdValue;

#[tauri::command]
pub async fn ds402_enable(
    stack_state: tauri::State<'_, SharedStack>,
    node_id: u8,
) -> Result<(), String> {
    // DS402 enable sequence: Shutdown → Switch On → Enable Operation
    // Write control word transitions via SDO to 0x6040
    let mut guard = stack_state.lock().await;

    // Step 1: Shutdown (0x0006)
    guard
        .sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x0006))
        .await
        .map_err(|e| format!("{:?}", e))?;

    // Step 2: Switch On (0x0007)
    guard
        .sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x0007))
        .await
        .map_err(|e| format!("{:?}", e))?;

    // Step 3: Enable Operation (0x000F)
    guard
        .sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x000F))
        .await
        .map_err(|e| format!("{:?}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn ds402_fault_reset(
    stack_state: tauri::State<'_, SharedStack>,
    node_id: u8,
) -> Result<(), String> {
    // Fault reset: write 0x0080 then 0x0000 to control word
    let mut guard = stack_state.lock().await;

    guard
        .sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x0080))
        .await
        .map_err(|e| format!("{:?}", e))?;

    guard
        .sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x0000))
        .await
        .map_err(|e| format!("{:?}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn ds402_set_mode(
    stack_state: tauri::State<'_, SharedStack>,
    node_id: u8,
    mode: u8,
) -> Result<(), String> {
    let mut guard = stack_state.lock().await;
    guard
        .sdo_download(node_id, 0x6060, 0, &OdValue::Integer8(mode as i8))
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn ds402_set_target(
    stack_state: tauri::State<'_, SharedStack>,
    node_id: u8,
    mode: u8,
    target: f64,
) -> Result<(), String> {
    let mut guard = stack_state.lock().await;

    match mode {
        1 => {
            // Profile Position: write to 0x607A (Target Position)
            guard
                .sdo_download(node_id, 0x607A, 0, &OdValue::Integer32(target as i32))
                .await
                .map_err(|e| format!("{:?}", e))?;
        }
        3 => {
            // Profile Velocity: write to 0x60FF (Target Velocity)
            guard
                .sdo_download(node_id, 0x60FF, 0, &OdValue::Integer32(target as i32))
                .await
                .map_err(|e| format!("{:?}", e))?;
        }
        6 => {
            // Homing: write to 0x607C (Home Offset)
            guard
                .sdo_download(node_id, 0x607C, 0, &OdValue::Integer32(target as i32))
                .await
                .map_err(|e| format!("{:?}", e))?;
        }
        _ => {
            return Err(format!("Unsupported DS402 mode: {}", mode));
        }
    }

    Ok(())
}
