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

pub use error::{CanOpenError, CanError, OdError, Ds402State};
pub use concrete_od::{ConcreteOd, OdBuilder, OdEntry};
pub use frame::{
    CanOpenFrame, CobId, FunctionCode, SyncFrame, TimestampFrame,
    FrameClass, classify_frame,
    SdoRequest, SdoResponse, SdoData, SdoResponseData,
    NmtState, NmtCommandSpecifier, NmtCommand,
    HeartbeatFrame, EmergencyFrame, PdoFrame,
};
pub use node_id::{NodeId, InvalidNodeId};
pub use od::{AccessType, CanDriver, DataType, EntryInfo, ObjectDictionary, ObjectType, OdValue};
pub use pdo::{PdoDirection, PdoMapping, TransmissionType, PdoData, parse_pdo, pdo_comm_index, pdo_map_index, pack_pdo, unpack_pdo, PdoError};
