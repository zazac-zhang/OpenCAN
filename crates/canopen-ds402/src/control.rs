//! DS402 device control.

use opencan_canopen_core::{CanDriver, CanOpenError, OdValue};
use opencan_canopen_ds301::SdoClient;
use super::state_machine::{Ds402State, OperationMode};

/// DS402 motion control device.
pub struct Ds402Device<C: CanDriver> {
    sdo: SdoClient<C>,
    node_id: u8,
}

impl<C: CanDriver> Ds402Device<C> {
    pub fn new(sdo: SdoClient<C>, node_id: u8) -> Self {
        Self { sdo, node_id }
    }

    /// Read the current state from status word (0x6041).
    pub async fn state(&mut self) -> Result<Ds402State, CanOpenError> {
        let value = self.sdo.upload(self.node_id, 0x6041, 0).await?;
        let word: u16 = match value {
            OdValue::Unsigned16(v) => v,
            _ => return Err(CanOpenError::Protocol("Invalid status word type".to_string())),
        };
        Ok(Ds402State::from_status_word(word))
    }

    /// Execute state transition by writing control word (0x6040).
    async fn write_control_word(&mut self, word: u16) -> Result<(), CanOpenError> {
        self.sdo.download(self.node_id, 0x6040, 0, &OdValue::Unsigned16(word)).await
    }

    /// Shutdown → ReadyToSwitchOn.
    pub async fn shutdown(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(0x0006).await
    }

    /// Switch On → SwitchedOn.
    pub async fn switch_on(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(0x0007).await
    }

    /// Enable Operation → OperationEnabled.
    pub async fn enable_operation(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(0x000F).await
    }

    /// Disable Voltage → SwitchOnDisabled.
    pub async fn disable_voltage(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(0x0000).await
    }

    /// Quick Stop → QuickStopActive.
    pub async fn quick_stop(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(0x0002).await
    }

    /// Fault Reset → SwitchOnDisabled (from Fault state).
    pub async fn fault_reset(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(0x0080).await
    }

    /// Convenience: full enable sequence (Shutdown → SwitchOn → EnableOperation).
    pub async fn enable(&mut self) -> Result<(), CanOpenError> {
        self.shutdown().await?;
        self.switch_on().await?;
        self.enable_operation().await
    }

    /// Set operation mode (0x6060).
    pub async fn set_mode(&mut self, mode: OperationMode) -> Result<(), CanOpenError> {
        self.sdo.download(self.node_id, 0x6060, 0, &OdValue::Integer8(mode as i8)).await
    }

    /// Read operation mode (0x6061).
    pub async fn mode(&mut self) -> Result<OperationMode, CanOpenError> {
        let value = self.sdo.upload(self.node_id, 0x6061, 0).await?;
        let mode_val: i8 = match value {
            OdValue::Integer8(v) => v,
            _ => return Err(CanOpenError::Protocol("Invalid mode type".to_string())),
        };
        OperationMode::from_i8(mode_val)
            .ok_or_else(|| CanOpenError::Protocol(format!("Unknown operation mode: {}", mode_val)))
    }

    /// Set target position (0x607A) — for Profile Position / CSP modes.
    pub async fn set_target_position(&mut self, pos: i32) -> Result<(), CanOpenError> {
        self.sdo.download(self.node_id, 0x607A, 0, &OdValue::Integer32(pos)).await
    }

    /// Read actual position (0x6064).
    pub async fn actual_position(&mut self) -> Result<i32, CanOpenError> {
        let value = self.sdo.upload(self.node_id, 0x6064, 0).await?;
        match value {
            OdValue::Integer32(v) => Ok(v),
            _ => Err(CanOpenError::Protocol("Invalid position type".to_string())),
        }
    }

    /// Set target velocity (0x60FF) — for Profile Velocity / CSV modes.
    pub async fn set_target_velocity(&mut self, vel: i32) -> Result<(), CanOpenError> {
        self.sdo.download(self.node_id, 0x60FF, 0, &OdValue::Integer32(vel)).await
    }

    /// Read actual velocity (0x606C).
    pub async fn actual_velocity(&mut self) -> Result<i32, CanOpenError> {
        let value = self.sdo.upload(self.node_id, 0x606C, 0).await?;
        match value {
            OdValue::Integer32(v) => Ok(v),
            _ => Err(CanOpenError::Protocol("Invalid velocity type".to_string())),
        }
    }

    /// Set target torque (0x6071) — for Profile Torque / CST modes.
    pub async fn set_target_torque(&mut self, tq: i16) -> Result<(), CanOpenError> {
        self.sdo.download(self.node_id, 0x6071, 0, &OdValue::Integer16(tq)).await
    }

    /// Read actual torque (0x6077).
    pub async fn actual_torque(&mut self) -> Result<i16, CanOpenError> {
        let value = self.sdo.upload(self.node_id, 0x6077, 0).await?;
        match value {
            OdValue::Integer16(v) => Ok(v),
            _ => Err(CanOpenError::Protocol("Invalid torque type".to_string())),
        }
    }
}
