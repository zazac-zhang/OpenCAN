//! SDO (Service Data Object) protocol implementation.
//!
//! Groups the SDO client, server, and abort code definitions.

pub mod abort;
pub mod client;
pub mod enhanced_server;
pub mod recovery;
pub mod server;

// Re-export primary types at the module root for convenience.
pub use abort::abort_reason;
pub use client::{SdoClient, sdo_abort_reason};
pub use enhanced_server::{AccessPolicy, EnhancedSdoServer, SdoServerEvent, SdoServerStats};
pub use recovery::{SdoErrorClass, SdoErrorRecovery, SdoRetryConfig, SdoRetryState, classify_error};
pub use server::SdoServer;
