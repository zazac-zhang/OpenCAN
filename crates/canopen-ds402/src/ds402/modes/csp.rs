//! Cyclic Sync Position (CSP) mode — CiA 402 mode 8.

use super::{ModeActual, ModeConfig, ModeTarget, OperationModeHandler};
use opencan_canopen_core::sdo::SdoClient;
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::CanOpenError;
use opencan_canopen_core::od::OdValue;

pub struct CyclicSyncPosition;

impl OperationModeHandler for CyclicSyncPosition {
    fn mode_value(&self) -> i8 {
        8
    }

    async fn configure(
        &self,
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
        _config: &ModeConfig,
    ) -> Result<(), CanOpenError> {
        // CSP mode only needs the mode of operation to be set
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
                    "CSP mode requires Position target".to_string(),
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
