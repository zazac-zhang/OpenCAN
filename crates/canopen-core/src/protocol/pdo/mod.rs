//! PDO (Process Data Object) protocol implementation.
//!
//! Groups PDO type definitions and PDO configuration management.

pub mod config;
pub mod dynamic;
pub mod event;
pub mod sync;
pub mod types;

// Re-export primary types at the module root.
pub use config::PdoCommParams;
pub use dynamic::{DynamicPdoMapper, MappingHistoryEntry, PdoMappingError, PdoTemplate};
pub use event::{PdoEvent, PdoEventHandler, PdoEventHandlerConfig, PdoSubscription};
pub use sync::{Ds402SyncConfigs, PdoBuffer, PdoSyncConfig, SyncEvent, SyncPdoProcessor, SyncPdoResult};
pub use types::*;
