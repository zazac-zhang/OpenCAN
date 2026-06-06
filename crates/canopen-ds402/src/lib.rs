//! # opencan-canopen-ds402
//!
//! CANOpen DS402 motion control profile.

pub mod state_machine;
pub mod control;
pub mod modes;

pub use state_machine::Ds402State;
pub use control::Ds402Device;
