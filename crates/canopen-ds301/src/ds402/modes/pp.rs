//! Profile Position (PP) mode — CiA 402 mode 1.

use super::{ModeActual, ModeTarget, OperationModeHandler};
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::CanOpenError;
use opencan_canopen_core::od::OdValue;
use crate::SdoClient;

/// Profile Position mode handler.
pub struct ProfilePosition;

impl OperationModeHandler for ProfilePosition {
    fn mode_value(&self) -> i8 {
        1
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
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
        target: &ModeTarget,
    ) -> Result<(), CanOpenError> {
        let pos = match target {
            ModeTarget::Position(p) => *p,
            _ => {
                return Err(CanOpenError::Protocol(
                    "PP mode requires Position target".to_string(),
                ));
            }
        };
        sdo.download(node_id, 0x607A, 0, &OdValue::Integer32(pos))
            .await
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
                "Expected i32 for actual position, got {:?}",
                other
            ))),
        }
    }
}
