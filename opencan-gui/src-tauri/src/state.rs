//! Global application state shared across Tauri commands and event loops.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Node information discovered on the CANOpen network.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeInfo {
    pub node_id: u8,
    pub nmt_state: String,
    pub device_type: Option<u32>,
    pub vendor_id: Option<u32>,
    pub product_name: Option<String>,
}

/// Backend connection information.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackendInfo {
    pub backend_type: String,
    pub channel: String,
    pub bitrate: u32,
    pub node_id: u8,
}

/// Descriptor for available CAN backends.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackendDescriptor {
    pub name: String,
    pub backend_type: String,
    pub available: bool,
}

/// Shared application state accessible from Tauri commands.
pub struct AppState {
    pub connected: bool,
    pub backend_info: Option<BackendInfo>,
    pub nodes: HashMap<u8, NodeInfo>,
    // TODO: stack integration - will hold CanopenStack when connected
    // TODO: recording integration - will hold SessionRecorder when active
}

pub type SharedState = Arc<RwLock<AppState>>;

impl AppState {
    pub fn new() -> Self {
        Self {
            connected: false,
            backend_info: None,
            nodes: HashMap::new(),
        }
    }
}
