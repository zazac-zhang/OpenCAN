//! # opencan-can-pcan
//!
//! Peak PCAN backend for OpenCAN.
//!
//! **Requires**: PCAN-Basic API installed on the system.
//! - Windows: PCANBasic.dll
//! - Linux: libpcanbasic.so

use crate::{CanBusDyn, CanBusFactory, CanConfig, error::CanError};

/// Factory for creating PCAN bus instances.
pub struct PcanFactory;

impl CanBusFactory for PcanFactory {
    fn open(&self, _channel: &str, _config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
        // TODO: Implement using peak-can or pcanbasic crate
        Err(CanError::Unsupported(
            "PCAN backend not yet implemented".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "PCAN"
    }

    fn available_channels(&self) -> Vec<String> {
        // TODO: Enumerate PCAN channels
        Vec::new()
    }
}
