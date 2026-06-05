//! # opencan-canopen-core
//!
//! Core traits and types for the CANOpen protocol stack.
//!
//! This crate provides:
//! - [`CanDriver`] trait for CAN bus I/O abstraction
//! - [`ObjectDictionary`] trait for OD access
//! - CANOpen frame types and encoding/decoding
//! - Error types for the protocol stack

pub mod error;
pub mod frame;
pub mod od;

pub use error::CanOpenError;
pub use frame::{CanOpenFrame, CobId, FunctionCode};
pub use od::{AccessType, CanDriver, DataType, EntryInfo, ObjectType, OdValue, ObjectDictionary};
