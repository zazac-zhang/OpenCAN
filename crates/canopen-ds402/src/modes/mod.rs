//! DS402 operation mode handlers.
//!
//! Each mode implements the [`OperationModeHandler`] trait for configuration,
//! target setting, and actual value reading.

pub mod csp;
pub mod cst;
pub mod csv;
pub mod homing;
pub mod pp;
pub mod pt;
pub mod pv;

use opencan_canopen_core::CanDriver;
use opencan_canopen_core::CanOpenError;
use opencan_canopen_ds301::SdoClient;

/// Target value for an operation mode.
#[derive(Debug, Clone)]
pub enum ModeTarget {
    Position(i32),
    Velocity(i32),
    Torque(i16),
}

/// Actual value from an operation mode.
#[derive(Debug, Clone)]
pub enum ModeActual {
    Position(i32),
    Velocity(i32),
    Torque(i16),
}

/// Trait for DS402 operation mode handlers.
#[allow(async_fn_in_trait)]
pub trait OperationModeHandler {
    /// The operation mode identifier.
    fn mode_value(&self) -> i8;

    /// Configure the mode (write OD parameters if needed).
    async fn configure(
        &self,
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
    ) -> Result<(), CanOpenError> {
        let _ = (sdo, node_id);
        Ok(())
    }

    /// Set target value.
    async fn set_target(
        &self,
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
        target: &ModeTarget,
    ) -> Result<(), CanOpenError>;

    /// Read actual value.
    async fn read_actual(
        &self,
        sdo: &mut SdoClient<impl CanDriver>,
        node_id: u8,
    ) -> Result<ModeActual, CanOpenError>;
}
