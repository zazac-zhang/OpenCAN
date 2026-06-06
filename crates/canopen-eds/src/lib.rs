//! # opencan-canopen-eds
//!
//! EDS (Electronic Data Sheet) file parser for CANOpen.

pub mod builder;
pub mod model;
pub mod parser;

pub use model::EdsFile;
