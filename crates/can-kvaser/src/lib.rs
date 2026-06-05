//! # opencan-can-kvaser
//!
//! Kvaser CANlib backend for OpenCAN.
//!
//! **Requires**: Kvaser CANlib SDK installed on the system.
//! - Windows: canlib32.dll
//! - Linux: Kvaser driver + canlib

use opencan_can_traits::{CanBusDyn, CanBusFactory, CanConfig, error::CanError};

/// Factory for creating Kvaser CAN bus instances.
pub struct KvaserFactory;

impl CanBusFactory for KvaserFactory {
    fn open(&self, _channel: &str, _config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
        // TODO: Implement using can-hal-kvaser or direct FFI to CANlib
        Err(CanError::Unsupported("Kvaser backend not yet implemented".to_string()))
    }

    fn name(&self) -> &str {
        "Kvaser"
    }

    fn available_channels(&self) -> Vec<String> {
        // TODO: Enumerate Kvaser channels via CANlib
        Vec::new()
    }
}
