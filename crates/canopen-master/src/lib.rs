//! # opencan-canopen-master
//!
//! CANOpen master station enhancements.
//!
//! This crate provides advanced features for CANOpen master stations:
//! - [`CanDriverAdapter`] — bridges physical CAN bus to protocol layer
//! - [`NodeManager`] — automatic node discovery, state tracking, configuration
//! - [`HeartbeatMonitor`] — enhanced heartbeat monitoring with timeout handling
//! - [`NmtStateMachine`] — complete NMT state tracking and transition management
//! - [`EmergencyHandler`] — EMCY message processing and error tracking
//!
//! ## Architecture
//!
//! ```text
//! canopen-master (this crate)
//!   ├── CanDriverAdapter — bridges CanBus → CanDriver
//!   ├── NodeManager — auto-discovery, state tracking
//!   ├── HeartbeatMonitor — enhanced heartbeat monitoring
//!   ├── NmtStateMachine — NMT state tracking and transitions
//!   ├── EmergencyHandler — EMCY message processing
//!   └── NetworkDiagnostics — bus stats, error tracking (planned)
//!
//! canopen-core
//!   ├── DS301 protocol (NMT, SDO, PDO, Heartbeat, EMCY)
//!   └── Object Dictionary
//!
//! canopen-ds402
//!   └── DS402 motion control profile
//! ```

pub mod adapter;
pub mod emergency_handler;
pub mod heartbeat_monitor;
pub mod nmt_state_machine;
pub mod node_manager;
pub mod sdo_multi_client;

pub use adapter::CanDriverAdapter;
pub use emergency_handler::{EmergencyEvent, EmergencyHandler, EmergencyHandlerConfig, EmergencyHandlerEvent, EmergencyStats, EmergencySummary, ErrorCode, ErrorRegister};
pub use heartbeat_monitor::{HeartbeatEvent, HeartbeatMonitor, HeartbeatStats, MonitorSummary, NodeMonitorConfig};
pub use nmt_state_machine::{NmtCommand, NmtStateMachine, NmtStateMachineConfig, NmtSummary, StateTransition, TransitionSource};
pub use node_manager::{NodeInfo, NodeManager, NodeManagerConfig, NodeSummary};
pub use sdo_multi_client::{SdoClientSession, SdoMultiClient, SdoMultiClientError};
