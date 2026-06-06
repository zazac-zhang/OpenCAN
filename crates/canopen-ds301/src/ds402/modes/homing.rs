//! Homing mode — CiA 402 mode 6.

use super::{ModeActual, ModeTarget, OperationModeHandler};
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
    ) -> Result<(), CanOpenError> {
        sdo.download(node_id, 0x6060, 0, &OdValue::Integer8(self.mode_value()))
            .await
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
