//! Profile Torque (PT) mode — CiA 402 mode 4.

use super::{ModeActual, ModeConfig, ModeTarget, OperationModeHandler};
use opencan_canopen_core::sdo::SdoClient;
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::CanOpenError;
use opencan_canopen_core::od::OdValue;

pub struct ProfileTorque;

impl OperationModeHandler for ProfileTorque {
    fn mode_value(&self) -> i8 {
        4
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

        // Set torque slope (0x6087)
        if let Some(slope) = config.torque_slope {
            sdo.download(node_id, 0x6087, 0, &OdValue::Unsigned32(slope))
                .await?;
        }

        // Set max torque (0x6072)
        if let Some(max_tq) = config.max_torque {
            sdo.download(node_id, 0x6072, 0, &OdValue::Unsigned16(max_tq))
                .await?;
        }

        Ok(())
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
                    "PT mode requires Torque target".to_string(),
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
