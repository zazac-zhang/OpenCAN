//! DS402 device control.
//!
//! High-level API for controlling CANOpen motion control devices.

use super::state_machine::{ControlWord, Ds402State, OperationMode};
use opencan_canopen_core::{CanDriver, CanOpenError, OdValue};
use crate::SdoClient;

/// DS402 motion control device.
///
/// Wraps an SDO client to provide high-level motion control operations.
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
            OdValue::Unsigned32(v) => v as u16, // Handle 4-byte response
            other => {
                return Err(CanOpenError::Protocol(format!(
                    "Invalid status word type: {:?}",
                    other
                )));
            }
        };
        Ok(Ds402State::from_status_word(word))
    }

    /// Write control word (0x6040).
    async fn write_control_word(&mut self, word: u16) -> Result<(), CanOpenError> {
        self.sdo
            .download(self.node_id, 0x6040, 0, &OdValue::Unsigned16(word))
            .await
    }

    /// Execute state transition.
    pub async fn transition(&mut self, target: Ds402State) -> Result<(), CanOpenError> {
        let current = self.state().await?;
        let cmd = current.transition_to(target).ok_or_else(|| {
            CanOpenError::Protocol(format!("Invalid transition: {:?} -> {:?}", current, target))
        })?;
        self.write_control_word(cmd).await
    }

    /// Shutdown → ReadyToSwitchOn.
    pub async fn shutdown(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(ControlWord::SHUTDOWN).await
    }

    /// Switch On → SwitchedOn.
    pub async fn switch_on(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(ControlWord::SWITCH_ON).await
    }

    /// Enable Operation → OperationEnabled.
    pub async fn enable_operation(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(ControlWord::ENABLE_OPERATION).await
    }

    /// Disable Voltage → SwitchOnDisabled.
    pub async fn disable_voltage(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(ControlWord::DISABLE_VOLTAGE).await
    }

    /// Quick Stop → QuickStopActive.
    pub async fn quick_stop(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(ControlWord::QUICK_STOP).await
    }

    /// Fault Reset → SwitchOnDisabled (from Fault state).
    pub async fn fault_reset(&mut self) -> Result<(), CanOpenError> {
        self.write_control_word(ControlWord::FAULT_RESET).await
    }

    /// Convenience: full enable sequence (Shutdown → SwitchOn → EnableOperation).
    pub async fn enable(&mut self) -> Result<(), CanOpenError> {
        self.shutdown().await?;
        self.switch_on().await?;
        self.enable_operation().await
    }

    /// Set operation mode (0x6060).
    pub async fn set_mode(&mut self, mode: OperationMode) -> Result<(), CanOpenError> {
        self.sdo
            .download(self.node_id, 0x6060, 0, &OdValue::Integer8(mode as i8))
            .await
    }

    /// Read operation mode (0x6061).
    pub async fn mode(&mut self) -> Result<OperationMode, CanOpenError> {
        let value = self.sdo.upload(self.node_id, 0x6061, 0).await?;
        let mode_val: i8 = match value {
            OdValue::Integer8(v) => v,
            other => {
                return Err(CanOpenError::Protocol(format!(
                    "Invalid mode type: {:?}",
                    other
                )));
            }
        };
        OperationMode::from_i8(mode_val)
            .ok_or_else(|| CanOpenError::Protocol(format!("Unknown operation mode: {}", mode_val)))
    }

    // === Position mode ===

    /// Set target position (0x607A).
    pub async fn set_target_position(&mut self, pos: i32) -> Result<(), CanOpenError> {
        self.sdo
            .download(self.node_id, 0x607A, 0, &OdValue::Integer32(pos))
            .await
    }

    /// Read actual position (0x6064).
    pub async fn actual_position(&mut self) -> Result<i32, CanOpenError> {
        match self.sdo.upload(self.node_id, 0x6064, 0).await? {
            OdValue::Integer32(v) => Ok(v),
            OdValue::Unsigned32(v) => Ok(v as i32),
            other => Err(CanOpenError::Protocol(format!(
                "Invalid position: {:?}",
                other
            ))),
        }
    }

    // === Velocity mode ===

    /// Set target velocity (0x60FF).
    pub async fn set_target_velocity(&mut self, vel: i32) -> Result<(), CanOpenError> {
        self.sdo
            .download(self.node_id, 0x60FF, 0, &OdValue::Integer32(vel))
            .await
    }

    /// Read actual velocity (0x606C).
    pub async fn actual_velocity(&mut self) -> Result<i32, CanOpenError> {
        match self.sdo.upload(self.node_id, 0x606C, 0).await? {
            OdValue::Integer32(v) => Ok(v),
            OdValue::Unsigned32(v) => Ok(v as i32),
            other => Err(CanOpenError::Protocol(format!(
                "Invalid velocity: {:?}",
                other
            ))),
        }
    }

    // === Torque mode ===

    /// Set target torque (0x6071).
    pub async fn set_target_torque(&mut self, tq: i16) -> Result<(), CanOpenError> {
        self.sdo
            .download(self.node_id, 0x6071, 0, &OdValue::Integer16(tq))
            .await
    }

    /// Read actual torque (0x6077).
    pub async fn actual_torque(&mut self) -> Result<i16, CanOpenError> {
        match self.sdo.upload(self.node_id, 0x6077, 0).await? {
            OdValue::Integer16(v) => Ok(v),
            other => Err(CanOpenError::Protocol(format!(
                "Invalid torque: {:?}",
                other
            ))),
        }
    }

    /// Get a reference to the underlying SDO client.
    pub fn sdo(&self) -> &SdoClient<C> {
        &self.sdo
    }

    /// Get a mutable reference to the underlying SDO client.
    pub fn sdo_mut(&mut self) -> &mut SdoClient<C> {
        &mut self.sdo
    }
}
