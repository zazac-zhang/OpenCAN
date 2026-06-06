//! # opencan-canopen-core
//!
//! Core traits and types for the CANOpen protocol stack.

pub mod concrete_od;
pub mod error;
pub mod frame;
pub mod od;

#[cfg(feature = "testing")]
pub mod testing;

pub use error::CanOpenError;
pub use frame::{CanOpenFrame, CobId, FunctionCode};
pub use od::{AccessType, CanDriver, DataType, EntryInfo, ObjectDictionary, ObjectType, OdValue};
