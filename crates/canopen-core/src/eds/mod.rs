//! EDS (Electronic Data Sheet) file parser for CANOpen.
//!
//! EDS files are INI-format files describing CANOpen device object dictionaries.
//! This module provides parsing and OD construction from EDS files.

pub mod builder;
pub mod model;
pub mod parser;

pub use model::EdsFile;
pub use parser::parse_eds;
