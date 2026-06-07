//! DS301 protocol communication objects.
//!
//! Groups CANOpen DS301 protocol implementations by communication object type,
//! matching the standard's functional grouping:
//!
//! - **NMT** — Network Management (COB-ID 0x000)
//! - **SDO** — Service Data Object (COB-ID 0x580/0x600 + NodeID)
//! - **PDO** — Process Data Object (COB-ID 0x140–0x5FF + NodeID)
//! - **Heartbeat** — Node heartbeat (COB-ID 0x700 + NodeID)
//! - **SYNC** — Synchronisation (COB-ID 0x080) — currently lives in `heartbeat`
//! - **EMCY** — Emergency (COB-ID 0x080 + NodeID)

pub mod emcy;
pub mod heartbeat;
pub mod nmt;
pub mod pdo;
pub mod sdo;
