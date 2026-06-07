//! # opencan-canopen-ds402
//!
//! CANOpen DS402 motion control profile implementation.
//!
//! This crate provides DS402 motion control device abstraction:
//! - [`Ds402Device`] — high-level motion control API
//! - Operation mode handlers (CSP, CSV, CST, PP, PV, PT, Homing)
//! - State machine management
//!
//! ## Architecture
//!
//! ```text
//! canopen-ds402 (this crate)
//!   ├── Ds402Device — motion control API
//!   ├── state_machine — DS402 state transitions
//!   └── modes/ — operation mode handlers
//!
//! canopen-core
//!   ├── DS301 protocol (NMT, SDO, PDO, Heartbeat, EMCY)
//!   └── Object Dictionary
//! ```

pub mod ds402;

pub use ds402::control::Ds402Device;
pub use ds402::state_machine::{ControlWord, Ds402State, OperationMode, StatusWord};
