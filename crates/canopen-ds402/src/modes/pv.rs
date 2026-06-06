//! Profile Velocity (PV) mode — CiA 402 mode 3.

use super::{ModeActual, ModeTarget, OperationModeHandler};
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::CanOpenError;
use opencan_canopen_core::od::OdValue;
use opencan_canopen_ds301::SdoClient;

pub struct ProfileVelocity;

impl OperationModeHandler for ProfileVelocity {
    fn mode_value(&self) -> i8 {
        3
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
        let vel = match target {
            ModeTarget::Velocity(v) => *v,
            _ => {
                return Err(CanOpenError::Protocol(
                    "PV mode requires Velocity target".to_string(),
                ));
            }
        };
        sdo.download(node_id, 0x60FF, 0, &OdValue::Integer32(vel))
            .await
    }

    async fn read_actual(
        &self,
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
    ) -> Result<ModeActual, CanOpenError> {
        match sdo.upload(node_id, 0x606C, 0).await? {
            OdValue::Integer32(v) => Ok(ModeActual::Velocity(v)),
            OdValue::Unsigned32(v) => Ok(ModeActual::Velocity(v as i32)),
            other => Err(CanOpenError::Protocol(format!(
                "Expected i32 for actual velocity, got {:?}",
                other
            ))),
        }
    }
}
