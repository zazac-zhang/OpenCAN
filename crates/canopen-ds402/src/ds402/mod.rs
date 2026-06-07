//! DS402 motion control profile (CiA 402).
//!
//! This module implements the CiA 402 state machine and device control API
//! for CANOpen motion control devices.

pub mod control;
pub mod error;
pub mod mode_validator;
pub mod modes;
pub mod state_machine;

pub use control::Ds402Device;
pub use error::{Ds402Error, LimitChecker, LimitDirection, PositionLimits, TorqueLimits, VelocityLimits};
pub use mode_validator::{ModeSwitchValidator, ModeTransitionHistory};
pub use state_machine::Ds402State;
