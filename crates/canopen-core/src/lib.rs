//! # opencan-canopen-core
//!
//! Core traits and types for the CANOpen protocol stack.
//!
//! This crate provides:
//! - Frame types and encoding/decoding
//! - Object Dictionary trait and implementation
//! - DS301 protocol implementation (NMT, SDO, PDO, Heartbeat, EMCY)

// === Base types (leaf modules — no sub-module dependencies) ===
pub mod concrete_od;
pub mod error;
pub mod frame;
pub mod node_id;
pub mod od;

// === DS301 protocol layer (hierarchical) ===
pub mod protocol;
// Re-export protocol submodules at crate root for ergonomic access.
pub use protocol::{emcy, heartbeat, nmt, pdo, sdo};

// === Stack & testing ===
pub mod stack;
pub mod testing;

#[cfg(feature = "eds")]
pub mod eds;

// === Top-level convenience re-exports ===
pub use concrete_od::{ConcreteOd, OdBuilder, OdEntry};
pub use error::{CanError, CanOpenError, Ds402State, OdError};
pub use frame::{
    CanOpenFrame, CobId, EmergencyFrame, FrameClass, FunctionCode, HeartbeatFrame, NmtCommand,
    NmtCommandSpecifier, NmtState, PdoFrame, SdoData, SdoRequest, SdoResponse, SdoResponseData,
    SyncFrame, TimestampFrame, classify_frame,
};
pub use node_id::{InvalidNodeId, NodeId};
pub use od::{AccessType, CanDriver, DataType, EntryInfo, ObjectDictionary, ObjectType, OdValue};
pub use pdo::{
    PdoData, PdoDirection, PdoError, PdoMapping, TransmissionType, pack_pdo, parse_pdo,
    pdo_comm_index, pdo_map_index, unpack_pdo,
};
