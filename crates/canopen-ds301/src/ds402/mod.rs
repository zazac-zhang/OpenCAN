//! DS402 motion control profile (CiA 402).
//!
//! This module implements the CiA 402 state machine and device control API
//! for CANOpen motion control devices.

pub mod control;
pub mod modes;
pub mod state_machine;

pub use control::Ds402Device;
pub use state_machine::Ds402State;
