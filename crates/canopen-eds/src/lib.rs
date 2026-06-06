//! # opencan-canopen-eds
//!
//! EDS (Electronic Data Sheet) file parser for CANOpen.
//!
//! **This crate is now a thin re-export of `opencan_canopen_core::eds`.**
//! New code should use `opencan_canopen_core` with the `eds` feature directly:
//!
//! ```toml
//! [dependencies]
//! opencan-canopen-core = { version = "...", features = ["eds"] }
//! ```

// Re-export everything from canopen-core's eds module
pub use opencan_canopen_core::eds::*;
