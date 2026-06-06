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
    /// Homing method (0x6098). Device-specific.
    pub homing_method: Option<i8>,
    /// Homing speed fast (0x6099:1) in position units/s.
    pub homing_speed_fast: Option<u32>,
    /// Homing speed slow (0x6099:2) in position units/s.
    pub homing_speed_slow: Option<u32>,
    /// Homing acceleration (0x609A) in position units/s².
    pub homing_acceleration: Option<u32>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SdoClient;
    use opencan_canopen_core::testing::MockCanDriver;
    use std::time::Duration;

    fn make_sdo() -> SdoClient<MockCanDriver> {
        SdoClient::new(MockCanDriver::new(), Duration::from_secs(1))
    }

    /// Enqueue a download confirmation for a specific OD index/subindex.
    fn enqueue_download_confirm(mock: &mut MockCanDriver, index: u16, subindex: u8) {
        let mut data = [0x60u8, 0, 0, 0, 0, 0, 0, 0]; // cs=3 (download confirmed)
        data[1..3].copy_from_slice(&index.to_le_bytes());
        data[3] = subindex;
        mock.enqueue(opencan_canopen_core::frame::CanOpenFrame::new(0x583, data));
    }

    #[tokio::test]
    async fn test_pp_configure_full() {
        let mut sdo = make_sdo();
        let mock = sdo.can_mut();
        enqueue_download_confirm(mock, 0x6060, 0); // mode
        enqueue_download_confirm(mock, 0x6081, 0); // profile velocity
        enqueue_download_confirm(mock, 0x6083, 0); // profile acceleration
        enqueue_download_confirm(mock, 0x6084, 0); // profile deceleration
        enqueue_download_confirm(mock, 0x8500, 0); // quick stop deceleration
        enqueue_download_confirm(mock, 0x607F, 0); // max profile velocity

        let config = ModeConfig {
            profile_velocity: Some(1000),
            profile_acceleration: Some(5000),
            profile_deceleration: Some(5000),
            quick_stop_deceleration: Some(10000),
            max_profile_velocity: Some(2000),
            ..Default::default()
        };

        let mode = super::pp::ProfilePosition;
        mode.configure(&mut sdo, 3, &config).await.unwrap();

        let tx = sdo.can().tx_log();
        assert_eq!(tx.len(), 6);
        // First frame: mode of operation = 1
        assert_eq!(tx[0].data[1..4], [0x60, 0x60, 0x00]); // index 0x6060
        assert_eq!(tx[0].data[4], 0x01); // mode = 1 (PP)
    }

    #[tokio::test]
    async fn test_pp_configure_mode_only() {
        let mut sdo = make_sdo();
        enqueue_download_confirm(sdo.can_mut(), 0x6060, 0);

        let config = ModeConfig::default();
        let mode = super::pp::ProfilePosition;
        mode.configure(&mut sdo, 3, &config).await.unwrap();

        assert_eq!(sdo.can().tx_log().len(), 1);
    }

    #[tokio::test]
    async fn test_pv_configure() {
        let mut sdo = make_sdo();
        let mock = sdo.can_mut();
        enqueue_download_confirm(mock, 0x6060, 0); // mode
        enqueue_download_confirm(mock, 0x6083, 0); // acc
        enqueue_download_confirm(mock, 0x6084, 0); // dec
        enqueue_download_confirm(mock, 0x8500, 0); // qsd

        let config = ModeConfig {
            profile_acceleration: Some(3000),
            profile_deceleration: Some(3000),
            quick_stop_deceleration: Some(8000),
            ..Default::default()
        };

        let mode = super::pv::ProfileVelocity;
        mode.configure(&mut sdo, 3, &config).await.unwrap();
        assert_eq!(sdo.can().tx_log().len(), 4);
    }

    #[tokio::test]
    async fn test_pt_configure() {
        let mut sdo = make_sdo();
        let mock = sdo.can_mut();
        enqueue_download_confirm(mock, 0x6060, 0); // mode
        enqueue_download_confirm(mock, 0x6087, 0); // torque slope
        enqueue_download_confirm(mock, 0x6072, 0); // max torque

        let config = ModeConfig {
            torque_slope: Some(100),
            max_torque: Some(500),
            ..Default::default()
        };

        let mode = super::pt::ProfileTorque;
        mode.configure(&mut sdo, 3, &config).await.unwrap();
        assert_eq!(sdo.can().tx_log().len(), 3);
    }

    #[tokio::test]
    async fn test_homing_configure() {
        let mut sdo = make_sdo();
        let mock = sdo.can_mut();
        enqueue_download_confirm(mock, 0x6060, 0); // mode
        enqueue_download_confirm(mock, 0x6098, 0); // homing method
        enqueue_download_confirm(mock, 0x6099, 1); // homing speed fast
        enqueue_download_confirm(mock, 0x6099, 2); // homing speed slow
        enqueue_download_confirm(mock, 0x609A, 0); // homing acceleration

        let config = ModeConfig {
            homing_method: Some(17),
            homing_speed_fast: Some(500),
            homing_speed_slow: Some(100),
            homing_acceleration: Some(2000),
            ..Default::default()
        };

        let mode = super::homing::HomingMode;
        mode.configure(&mut sdo, 3, &config).await.unwrap();
        assert_eq!(sdo.can().tx_log().len(), 5);
    }

    #[tokio::test]
    async fn test_csp_configure_mode_only() {
        let mut sdo = make_sdo();
        enqueue_download_confirm(sdo.can_mut(), 0x6060, 0);

        let config = ModeConfig::default();
        let mode = super::csp::CyclicSyncPosition;
        mode.configure(&mut sdo, 3, &config).await.unwrap();
        assert_eq!(sdo.can().tx_log().len(), 1);
    }

    #[tokio::test]
    async fn test_cst_configure_mode_only() {
        let mut sdo = make_sdo();
        enqueue_download_confirm(sdo.can_mut(), 0x6060, 0);

        let config = ModeConfig::default();
        let mode = super::cst::CyclicSyncTorque;
        mode.configure(&mut sdo, 3, &config).await.unwrap();
        assert_eq!(sdo.can().tx_log().len(), 1);
    }

    #[tokio::test]
    async fn test_csv_configure_mode_only() {
        let mut sdo = make_sdo();
        enqueue_download_confirm(sdo.can_mut(), 0x6060, 0);

        let config = ModeConfig::default();
        let mode = super::csv::CyclicSyncVelocity;
        mode.configure(&mut sdo, 3, &config).await.unwrap();
        assert_eq!(sdo.can().tx_log().len(), 1);
    }
}
