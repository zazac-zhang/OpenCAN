//! Mock backend task for testing without real hardware.

use tokio::sync::mpsc;
use super::command::BackendCommand;
use super::event::BackendEvent;

/// Mock backend task for testing without real hardware.
pub async fn mock_backend_task(
    mut cmd_rx: mpsc::Receiver<BackendCommand>,
    evt_tx: mpsc::Sender<BackendEvent>,
) {
    let _ = evt_tx.send(BackendEvent::Connected("Mock CAN".to_string())).await;

    // Simulated node list
    let known_nodes = vec![1u8, 2, 3, 5, 10];

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            BackendCommand::Disconnect => {
                let _ = evt_tx.send(BackendEvent::Disconnected).await;
                break;
            }

            BackendCommand::ScanNodes { respond } => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let _ = respond.send(Ok(known_nodes.clone()));
                let _ = evt_tx.send(BackendEvent::ScanResult(known_nodes.clone())).await;
            }

            // === NMT ===
            BackendCommand::NmtStart(id) => {
                if id == 0 {
                    // Broadcast
                    for &node_id in &known_nodes {
                        let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                            node_id,
                            state: "Operational".to_string(),
                        }).await;
                    }
                } else {
                    let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                        node_id: id,
                        state: "Operational".to_string(),
                    }).await;
                }
            }
            BackendCommand::NmtStop(id) => {
                if id == 0 {
                    for &node_id in &known_nodes {
                        let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                            node_id,
                            state: "Stopped".to_string(),
                        }).await;
                    }
                } else {
                    let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                        node_id: id,
                        state: "Stopped".to_string(),
                    }).await;
                }
            }
            BackendCommand::NmtReset(id) => {
                if id == 0 {
                    for &node_id in &known_nodes {
                        let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                            node_id,
                            state: "PreOperational".to_string(),
                        }).await;
                    }
                } else {
                    let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                        node_id: id,
                        state: "PreOperational".to_string(),
                    }).await;
                }
            }
            BackendCommand::NmtResetComm(id) => {
                let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                    node_id: id,
                    state: "PreOperational".to_string(),
                }).await;
            }

            // === SDO ===
            BackendCommand::SdoUpload { node_id, index, subindex, respond } => {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                let result: Result<Vec<u8>, String> = match (index, subindex) {
                    (0x1000, 0) => Ok(vec![0x92, 0x01, 0x02, 0x00]), // Device Type
                    (0x1001, 0) => Ok(vec![0x00]),                    // Error Register
                    (0x1018, 1) => Ok(vec![0x78, 0x56, 0x34, 0x12]), // Vendor ID
                    (0x1018, 2) => Ok(vec![0x01, 0x00, 0x00, 0x00]), // Product Code
                    (0x6041, 0) => Ok(vec![0x27, 0x00]),             // Status Word (OpEnabled)
                    (0x6064, 0) => Ok(vec![0xD2, 0x04, 0x00, 0x00]), // Actual Position (1234)
                    (0x606C, 0) => Ok(vec![0x2C, 0x01, 0x00, 0x00]), // Actual Velocity (300)
                    (0x6077, 0) => Ok(vec![0x64, 0x00]),             // Actual Torque (100)
                    (0x6060, 0) => Ok(vec![0x08]),                   // Modes of Operation (CSP)
                    (0x1005, 0) => Ok(vec![0x80, 0x00, 0x00, 0x00]), // SYNC COB-ID
                    (0x1017, 0) => Ok(vec![0xE8, 0x03]),             // Heartbeat Producer (1000ms)
                    _ => Err(format!("Object {:04X}:{:02X} does not exist", index, subindex)),
                };

                let _ = respond.send(result.clone());
                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index, subindex, result,
                }).await;
            }

            BackendCommand::SdoDownload { node_id, index, subindex, value: _, respond } => {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                let result: Result<(), String> = Ok(());
                let _ = respond.send(result);
                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index, subindex, result: Ok(vec![]),
                }).await;
            }

            // === DS402 ===
            BackendCommand::Ds402Enable { node_id: id, respond } => {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                let _ = respond.send(Ok(()));
                let _ = evt_tx.send(BackendEvent::Ds402StateResult {
                    node_id: id, state: "OperationEnabled".to_string(), status_word: 0x0027,
                }).await;
            }

            BackendCommand::Ds402FaultReset { node_id: id, respond } => {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                let _ = respond.send(Ok(()));
                let _ = evt_tx.send(BackendEvent::Ds402StateResult {
                    node_id: id, state: "SwitchOnDisabled".to_string(), status_word: 0x0040,
                }).await;
            }

            BackendCommand::Ds402ReadState { node_id: id, respond } => {
                let _ = respond.send(Ok(0x0027));
                let _ = evt_tx.send(BackendEvent::Ds402StateResult {
                    node_id: id, state: "OperationEnabled".to_string(), status_word: 0x0027,
                }).await;
            }

            BackendCommand::Ds402WriteControl { node_id: _, control_word: _, respond } => {
                let _ = respond.send(Ok(()));
            }

            BackendCommand::Ds402SetPosition { node_id, position: _, respond } => {
                let _ = respond.send(Ok(()));
                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index: 0x607A, subindex: 0, result: Ok(vec![]),
                }).await;
            }

            BackendCommand::Ds402SetVelocity { node_id, velocity: _, respond } => {
                let _ = respond.send(Ok(()));
                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index: 0x60FF, subindex: 0, result: Ok(vec![]),
                }).await;
            }

            BackendCommand::Ds402SetTorque { node_id, torque: _, respond } => {
                let _ = respond.send(Ok(()));
                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index: 0x6071, subindex: 0, result: Ok(vec![]),
                }).await;
            }

            BackendCommand::Ds402ReadPosition { node_id: id, respond } => {
                let _ = respond.send(Ok(12345));
                let _ = evt_tx.send(BackendEvent::Ds402PositionResult {
                    node_id: id, position: 12345,
                }).await;
            }

            BackendCommand::Ds402ReadVelocity { node_id: id, respond } => {
                let _ = respond.send(Ok(500));
                let _ = evt_tx.send(BackendEvent::Ds402VelocityResult {
                    node_id: id, velocity: 500,
                }).await;
            }

            BackendCommand::Ds402ReadTorque { node_id: id, respond } => {
                let _ = respond.send(Ok(100));
                let _ = evt_tx.send(BackendEvent::Ds402TorqueResult {
                    node_id: id, torque: 100,
                }).await;
            }

            BackendCommand::Ds402SetMode { node_id: _, mode: _, respond } => {
                let _ = respond.send(Ok(()));
            }

            // === PDO ===
            BackendCommand::ReadPdoMapping { node_id: _, pdo_index, respond } => {
                // Return mock PDO mapping
                let mapping = match pdo_index {
                    1 => vec![(0x6041, 0x00, 0x10)], // Status Word (16 bit)
                    2 => vec![(0x6064, 0x00, 0x20)], // Actual Position (32 bit)
                    3 => vec![(0x606C, 0x00, 0x20)], // Actual Velocity (32 bit)
                    _ => vec![],
                };
                let _ = respond.send(Ok(mapping));
            }
        }
    }
}
