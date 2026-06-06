//! # opencan-canopen-core
//!
//! Core traits and types for the CANOpen protocol stack.

pub mod concrete_od;
pub mod error;
pub mod frame;
pub mod node_id;
pub mod od;
pub mod pdo;
pub mod sdo_abort;

#[cfg(feature = "eds")]
pub mod eds;

#[cfg(feature = "testing")]
pub mod testing;

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
