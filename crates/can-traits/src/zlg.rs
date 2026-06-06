//! # opencan-can-zlg
//!
//! ZLG (致远电子) CAN backend for OpenCAN.
//!
//! **Requires**: ZLG CAN driver installed on the system.
//! - Windows: ControlCAN.dll
//! - Linux: libcontrolcan.so

use crate::{CanBusDyn, CanBusFactory, CanConfig, error::CanError};

/// Factory for creating ZLG CAN bus instances.
pub struct ZlgFactory;

impl CanBusFactory for ZlgFactory {
    fn open(&self, _channel: &str, _config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
        // TODO: Implement using zlgcan crate or direct FFI
        Err(CanError::Unsupported(
            "ZLG backend not yet implemented".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "ZLG"
    }

    fn available_channels(&self) -> Vec<String> {
        // TODO: Enumerate ZLG channels
        Vec::new()
    }
}
