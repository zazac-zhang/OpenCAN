//! # opencan-canopen-ds402
//!
//! CANOpen DS402 motion control profile.

pub mod control;
pub mod modes;
pub mod state_machine;

pub use control::Ds402Device;
pub use state_machine::Ds402State;
