//! Error types for CAN bus operations.

use thiserror::Error;

/// CAN bus layer errors.
#[derive(Error, Debug, Clone)]
pub enum CanError {
    #[error("bus off")]
    BusOff,

    #[error("bus error: {0}")]
    BusError(String),

    #[error("interface not found: {0}")]
    InterfaceNotFound(String),

    #[error("timeout")]
    Timeout,

    #[error("IO error: {0}")]
    Io(String),

    #[error("not connected")]
    NotConnected,

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),
}
