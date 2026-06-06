//! Backend commands sent from GUI to async task.

use tokio::sync::oneshot;

/// Backend command sent from GUI to async task.
#[derive(Debug)]
pub enum BackendCommand {
    // === Connection ===
    /// Disconnect from backend.
    Disconnect,

    // === Node Scanning ===
    /// Scan for nodes on the bus.
    ScanNodes {
        respond: oneshot::Sender<Result<Vec<u8>, String>>,
    },

    // === NMT ===
    /// Start a node (or all nodes if node_id = 0).
    NmtStart(u8),
    /// Stop a node (or all nodes if node_id = 0).
    NmtStop(u8),
    /// Reset a node (or all nodes if node_id = 0).
    NmtReset(u8),
    /// Reset communication for a node.
    NmtResetComm(u8),

    // === SDO ===
    /// SDO upload (read remote OD).
    SdoUpload {
        node_id: u8,
        index: u16,
        subindex: u8,
        respond: oneshot::Sender<Result<Vec<u8>, String>>,
    },
    /// SDO download (write remote OD).
    SdoDownload {
        node_id: u8,
        index: u16,
        subindex: u8,
        value: Vec<u8>,
        respond: oneshot::Sender<Result<(), String>>,
    },

    // === DS402 ===
    /// Enable DS402 operation.
    Ds402Enable {
        node_id: u8,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// Reset DS402 fault.
    Ds402FaultReset {
        node_id: u8,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// Read DS402 state (status word).
    Ds402ReadState {
        node_id: u8,
        respond: oneshot::Sender<Result<u16, String>>,
    },
    /// Write control word.
    Ds402WriteControl {
        node_id: u8,
        control_word: u16,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// Set target position.
    Ds402SetPosition {
        node_id: u8,
        position: i32,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// Set target velocity.
    Ds402SetVelocity {
        node_id: u8,
        velocity: i32,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// Set target torque.
    Ds402SetTorque {
        node_id: u8,
        torque: i16,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// Read actual position.
    Ds402ReadPosition {
        node_id: u8,
        respond: oneshot::Sender<Result<i32, String>>,
    },
    /// Read actual velocity.
    Ds402ReadVelocity {
        node_id: u8,
        respond: oneshot::Sender<Result<i32, String>>,
    },
    /// Read actual torque.
    Ds402ReadTorque {
        node_id: u8,
        respond: oneshot::Sender<Result<i16, String>>,
    },
    /// Set operation mode.
    Ds402SetMode {
        node_id: u8,
        mode: i8,
        respond: oneshot::Sender<Result<(), String>>,
    },

    // === PDO ===
    /// Read PDO mapping.
    ReadPdoMapping {
        node_id: u8,
        pdo_index: u8,
        respond: oneshot::Sender<Result<Vec<(u16, u8, u8)>, String>>,
    },
}
