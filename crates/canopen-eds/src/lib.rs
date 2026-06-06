//! # opencan-canopen-eds
//!
//! EDS (Electronic Data Sheet) file parser for CANOpen.

pub mod parser;
pub mod model;
pub mod builder;

pub use model::EdsFile;
