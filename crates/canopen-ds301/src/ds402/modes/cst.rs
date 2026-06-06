//! Cyclic Sync Torque (CST) mode — CiA 402 mode 10.

use super::{ModeActual, ModeConfig, ModeTarget, OperationModeHandler};
use crate::SdoClient;
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::CanOpenError;
use opencan_canopen_core::od::OdValue;

pub struct CyclicSyncTorque;

impl OperationModeHandler for CyclicSyncTorque {
    fn mode_value(&self) -> i8 {
        10
    }

    async fn configure(
        &self,
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
        _config: &ModeConfig,
    ) -> Result<(), CanOpenError> {
        // CST mode only needs the mode of operation to be set
        sdo.download(node_id, 0x6060, 0, &OdValue::Integer8(self.mode_value()))
            .await
    }

    async fn set_target(
        &self,
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
        target: &ModeTarget,
    ) -> Result<(), CanOpenError> {
        let tq = match target {
            ModeTarget::Torque(t) => *t,
            _ => {
                return Err(CanOpenError::Protocol(
                    "CST mode requires Torque target".to_string(),
                ));
            }
        };
        sdo.download(node_id, 0x6071, 0, &OdValue::Integer16(tq))
            .await
    }

    async fn read_actual(
        &self,
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
    ) -> Result<ModeActual, CanOpenError> {
        match sdo.upload(node_id, 0x6077, 0).await? {
            OdValue::Integer16(v) => Ok(ModeActual::Torque(v)),
            other => Err(CanOpenError::Protocol(format!(
                "Expected i16 for actual torque, got {:?}",
                other
            ))),
        }
    }
}
