//! Homing mode — CiA 402 mode 6.

use super::{ModeActual, ModeConfig, ModeTarget, OperationModeHandler};
use crate::SdoClient;
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::CanOpenError;
use opencan_canopen_core::od::OdValue;

pub struct HomingMode;

impl OperationModeHandler for HomingMode {
    fn mode_value(&self) -> i8 {
        6
    }

    async fn configure(
        &self,
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
        config: &ModeConfig,
    ) -> Result<(), CanOpenError> {
        // Set mode of operation
        sdo.download(node_id, 0x6060, 0, &OdValue::Integer8(self.mode_value()))
            .await?;

        // Set homing method (0x6098)
        if let Some(method) = config.homing_method {
            sdo.download(node_id, 0x6098, 0, &OdValue::Integer8(method))
                .await?;
        }

        // Set homing speed fast (0x6099:1)
        if let Some(speed) = config.homing_speed_fast {
            sdo.download(node_id, 0x6099, 1, &OdValue::Unsigned32(speed))
                .await?;
        }

        // Set homing speed slow (0x6099:2)
        if let Some(speed) = config.homing_speed_slow {
            sdo.download(node_id, 0x6099, 2, &OdValue::Unsigned32(speed))
                .await?;
        }

        // Set homing acceleration (0x609A)
        if let Some(acc) = config.homing_acceleration {
            sdo.download(node_id, 0x609A, 0, &OdValue::Unsigned32(acc))
                .await?;
        }

        Ok(())
    }

    async fn set_target(
        &self,
        _sdo: &mut SdoClient<impl CanDriver>,
        _node_id: u8,
        _target: &ModeTarget,
    ) -> Result<(), CanOpenError> {
        // Homing doesn't use a target position — it uses homing method (0x6098)
        // The homing is triggered via control word bit 4
        Ok(())
    }

    async fn read_actual(
        &self,
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
    ) -> Result<ModeActual, CanOpenError> {
        match sdo.upload(node_id, 0x6064, 0).await? {
            OdValue::Integer32(v) => Ok(ModeActual::Position(v)),
            OdValue::Unsigned32(v) => Ok(ModeActual::Position(v as i32)),
            other => Err(CanOpenError::Protocol(format!(
                "Expected i32 for position, got {:?}",
                other
            ))),
        }
    }
}
