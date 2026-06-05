//! Backend integration — connects GUI to CAN/CANOpen protocol stack.

use tokio::sync::mpsc;
use opencan_canopen_core::CanDriver;

/// Backend command sent from GUI to async task.
#[derive(Debug, Clone)]
pub enum BackendCommand {
    /// SDO upload (read remote OD)
    SdoUpload { node_id: u8, index: u16, subindex: u8 },
    /// SDO download (write remote OD)
    SdoDownload { node_id: u8, index: u16, subindex: u8, value: Vec<u8> },
    /// NMT command
    NmtStart(u8),
    NmtStop(u8),
    NmtReset(u8),
    /// Scan for nodes
    ScanNodes,
    /// DS402 enable
    Ds402Enable(u8),
    /// DS402 fault reset
    Ds402FaultReset(u8),
    /// DS402 read state
    Ds402ReadState(u8),
    /// DS402 set target position
    Ds402SetPosition { node_id: u8, position: i32 },
    /// DS402 set target velocity
    Ds402SetVelocity { node_id: u8, velocity: i32 },
    /// Read actual position
    Ds402ReadPosition(u8),
    /// Read actual velocity
    Ds402ReadVelocity(u8),
    /// Disconnect
    Disconnect,
}

/// Backend event sent from async task to GUI.
#[derive(Debug, Clone)]
pub enum BackendEvent {
    /// SDO result
    SdoResult { node_id: u8, index: u16, subindex: u8, result: Result<Vec<u8>, String> },
    /// Node scan result
    ScanResult(Vec<u8>),
    /// CAN frame received (for logging)
    FrameReceived { cob_id: u16, data: [u8; 8], timestamp_ms: u64 },
    /// Heartbeat state change
    HeartbeatChanged { node_id: u8, alive: bool },
    /// NMT state change
    NmtStateChanged { node_id: u8, state: String },
    /// DS402 state
    Ds402StateResult { node_id: u8, state: String, status_word: u16 },
    /// DS402 position
    Ds402PositionResult { node_id: u8, position: i32 },
    /// DS402 velocity
    Ds402VelocityResult { node_id: u8, velocity: i32 },
    /// Connection status
    Connected(String),
    Disconnected,
    Error(String),
}

/// Backend manages the async CAN/CANOpen protocol stack.
#[derive(Debug)]
pub struct Backend {
    cmd_tx: mpsc::Sender<BackendCommand>,
    evt_rx: mpsc::Receiver<BackendEvent>,
}

impl Backend {
    /// Create a new backend with a mock CAN driver (for testing).
    pub fn new_mock() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel(64);
        let (evt_tx, evt_rx) = mpsc::channel(256);

        tokio::spawn(mock_backend_task(cmd_rx, evt_tx));

        Self { cmd_tx, evt_rx }
    }

    /// Send a command to the backend.
    pub async fn send(&self, cmd: BackendCommand) {
        let _ = self.cmd_tx.send(cmd).await;
    }

    /// Try to receive an event (non-blocking).
    pub fn try_recv(&mut self) -> Option<BackendEvent> {
        self.evt_rx.try_recv().ok()
    }

    /// Receive an event (blocking).
    pub async fn recv(&mut self) -> Option<BackendEvent> {
        self.evt_rx.recv().await
    }
}

/// Mock backend task for testing without real hardware.
async fn mock_backend_task(
    mut cmd_rx: mpsc::Receiver<BackendCommand>,
    evt_tx: mpsc::Sender<BackendEvent>,
) {
    

    let _ = evt_tx.send(BackendEvent::Connected("Mock CAN".to_string())).await;

    // Simulated node list
    let known_nodes = vec![1u8, 2, 3, 5, 10];

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            BackendCommand::SdoUpload { node_id, index, subindex } => {
                // Simulate SDO read
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                let result = match (index, subindex) {
                    (0x1000, 0) => Ok(vec![0x92, 0x01, 0x02, 0x00]), // Device Type
                    (0x1001, 0) => Ok(vec![0x00]),                    // Error Register
                    (0x1018, 1) => Ok(vec![0x78, 0x56, 0x34, 0x12]), // Vendor ID
                    (0x6041, 0) => Ok(vec![0x27, 0x00]),             // Status Word (OpEnabled)
                    (0x6064, 0) => Ok(vec![0xD2, 0x04, 0x00, 0x00]), // Actual Position (1234)
                    (0x606C, 0) => Ok(vec![0x2C, 0x01, 0x00, 0x00]), // Actual Velocity (300)
                    _ => Err("Object does not exist".to_string()),
                };

                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index, subindex, result,
                }).await;
            }
            BackendCommand::SdoDownload { node_id, index, subindex, value: _ } => {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index, subindex, result: Ok(vec![]),
                }).await;
            }
            BackendCommand::ScanNodes => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let _ = evt_tx.send(BackendEvent::ScanResult(known_nodes.clone())).await;
            }
            BackendCommand::NmtStart(id) => {
                let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                    node_id: id, state: "Operational".to_string()
                }).await;
            }
            BackendCommand::NmtStop(id) => {
                let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                    node_id: id, state: "Stopped".to_string()
                }).await;
            }
            BackendCommand::NmtReset(id) => {
                let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                    node_id: id, state: "PreOperational".to_string()
                }).await;
            }
            BackendCommand::Ds402Enable(id) => {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                let _ = evt_tx.send(BackendEvent::Ds402StateResult {
                    node_id: id, state: "OperationEnabled".to_string(), status_word: 0x0027,
                }).await;
            }
            BackendCommand::Ds402FaultReset(id) => {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                let _ = evt_tx.send(BackendEvent::Ds402StateResult {
                    node_id: id, state: "SwitchOnDisabled".to_string(), status_word: 0x0040,
                }).await;
            }
            BackendCommand::Ds402ReadState(id) => {
                let _ = evt_tx.send(BackendEvent::Ds402StateResult {
                    node_id: id, state: "OperationEnabled".to_string(), status_word: 0x0027,
                }).await;
            }
            BackendCommand::Ds402SetPosition { node_id, position: _ } => {
                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index: 0x607A, subindex: 0, result: Ok(vec![]),
                }).await;
            }
            BackendCommand::Ds402SetVelocity { node_id, velocity: _ } => {
                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index: 0x60FF, subindex: 0, result: Ok(vec![]),
                }).await;
            }
            BackendCommand::Ds402ReadPosition(id) => {
                let _ = evt_tx.send(BackendEvent::Ds402PositionResult {
                    node_id: id, position: 12345,
                }).await;
            }
            BackendCommand::Ds402ReadVelocity(id) => {
                let _ = evt_tx.send(BackendEvent::Ds402VelocityResult {
                    node_id: id, velocity: 500,
                }).await;
            }
            BackendCommand::Disconnect => {
                let _ = evt_tx.send(BackendEvent::Disconnected).await;
                break;
            }
        }
    }
}
