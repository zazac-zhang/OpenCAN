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

use crate::SdoClient;
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::CanOpenError;

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

/// Configuration parameters for DS402 operation modes.
///
/// Not all parameters are used by all modes.
#[derive(Debug, Clone, Default)]
pub struct ModeConfig {
    /// Profile velocity (0x6081) in position units/s.
    pub profile_velocity: Option<u32>,
    /// Profile acceleration (0x6083) in position units/s².
    pub profile_acceleration: Option<u32>,
    /// Profile deceleration (0x6084) in position units/s².
    pub profile_deceleration: Option<u32>,
    /// Quick stop deceleration (0x8500) in position units/s².
    pub quick_stop_deceleration: Option<u32>,
    /// Max profile velocity (0x607F) in position units/s.
    pub max_profile_velocity: Option<u32>,
    /// Max acceleration (0x60C5) in position units/s².
    pub max_acceleration: Option<u32>,
    /// Max deceleration (0x60C6) in position units/s².
    pub max_deceleration: Option<u32>,
    /// Torque slope (0x6087) in 0.1%/s.
    pub torque_slope: Option<u32>,
    /// Max torque (0x6072) in 0.1% of rated torque.
    pub max_torque: Option<u16>,
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
        config: &ModeConfig,
    ) -> Result<(), CanOpenError> {
        let _ = (sdo, node_id, config);
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
