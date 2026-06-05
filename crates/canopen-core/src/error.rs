//! Error types for CANOpen protocol stack.

use std::time::Duration;

use thiserror::Error;

/// Main error type for CANOpen operations.
#[derive(Error, Debug)]
pub enum CanOpenError {
    #[error("CAN bus error: {0}")]
    Can(#[from] CanError),

    #[error("SDO abort: {code:#06x} - {reason}")]
    SdoAbort { code: u32, reason: &'static str },

    #[error("SDO timeout after {0:?}")]
    SdoTimeout(Duration),

    #[error("Object dictionary error: {0}")]
    Od(#[from] OdError),

    #[error("DS402 state transition invalid: {from:?} -> {to:?}")]
    Ds402InvalidTransition { from: Ds402State, to: Ds402State },

    #[error("DS402 fault: code={code:#06x}, register={register:#04x}")]
    Ds402Fault { code: u16, register: u8 },

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Timeout")]
    Timeout,
}

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
}

/// Object Dictionary errors.
#[derive(Error, Debug, Clone)]
pub enum OdError {
    #[error("index {index:#06x} not found")]
    IndexNotFound { index: u16 },

    #[error("subindex {index:#06x}:{subindex} not found")]
    SubindexNotFound { index: u16, subindex: u8 },

    #[error("access denied: {access:?} on {index:#06x}:{subindex}")]
    AccessDenied { index: u16, subindex: u8, access: AccessType },

    #[error("type mismatch: expected {expected:?}, got {actual:?}")]
    TypeMismatch { expected: DataType, actual: DataType },
}

/// DS402 states (placeholder - full implementation in canopen-ds402).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ds402State {
    NotReadyToSwitchOn,
    SwitchOnDisabled,
    ReadyToSwitchOn,
    SwitchedOn,
    OperationEnabled,
    QuickStopActive,
    FaultReactionActive,
    Fault,
}

use crate::od::{AccessType, DataType};
