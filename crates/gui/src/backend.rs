//! Backend integration — connects GUI to CAN/CANOpen protocol stack.
//!
//! The backend runs a tokio task that holds a CanopenStack and processes
//! commands from the GUI, sending back events.

use opencan_canopen_core::CanDriver;
use opencan_canopen_core::od::{DataType, OdValue};
use opencan_canopen_ds301::CanopenStack;
use tokio::sync::{mpsc, oneshot};

/// Backend command sent from GUI to async task.
#[derive(Debug)]
#[allow(dead_code)]
pub enum BackendCommand {
    /// SDO upload (read remote OD)
    SdoUpload {
        node_id: u8,
        index: u16,
        subindex: u8,
        respond: oneshot::Sender<Result<Vec<u8>, String>>,
    },
    /// SDO download (write remote OD)
    SdoDownload {
        node_id: u8,
        index: u16,
        subindex: u8,
        value: Vec<u8>,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// NMT command
    NmtStart(u8),
    NmtStop(u8),
    NmtReset(u8),
    /// Scan for nodes
    ScanNodes {
        respond: oneshot::Sender<Vec<u8>>,
    },
    /// DS402 enable
    Ds402Enable {
        node_id: u8,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// DS402 fault reset
    Ds402FaultReset {
        node_id: u8,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// DS402 read state
    Ds402ReadState {
        node_id: u8,
        respond: oneshot::Sender<Result<u16, String>>,
    },
    /// DS402 set target position
    Ds402SetPosition {
        node_id: u8,
        position: i32,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// DS402 set target velocity
    Ds402SetVelocity {
        node_id: u8,
        velocity: i32,
        respond: oneshot::Sender<Result<(), String>>,
    },
    /// Read actual position
    Ds402ReadPosition {
        node_id: u8,
        respond: oneshot::Sender<Result<i32, String>>,
    },
    /// Read actual velocity
    Ds402ReadVelocity {
        node_id: u8,
        respond: oneshot::Sender<Result<i32, String>>,
    },
    /// Disconnect
    Disconnect,
}

/// Backend event sent from async task to GUI.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BackendEvent {
    /// SDO result
    SdoResult {
        node_id: u8,
        index: u16,
        subindex: u8,
        result: Result<Vec<u8>, String>,
    },
    /// Node scan result
    ScanResult(Vec<u8>),
    /// CAN frame received (for logging)
    FrameReceived {
        cob_id: u16,
        data: [u8; 8],
        timestamp_ms: u64,
    },
    /// Heartbeat state change
    HeartbeatChanged {
        node_id: u8,
        alive: bool,
    },
    /// NMT state change
    NmtStateChanged {
        node_id: u8,
        state: String,
    },
    /// DS402 state
    Ds402StateResult {
        node_id: u8,
        state: String,
        status_word: u16,
    },
    /// DS402 position
    Ds402PositionResult {
        node_id: u8,
        position: i32,
    },
    /// DS402 velocity
    Ds402VelocityResult {
        node_id: u8,
        velocity: i32,
    },
    /// Connection status
    Connected(String),
    Disconnected,
    Error(String),
}

/// Backend manages the async CAN/CANOpen protocol stack.
pub struct Backend {
    cmd_tx: mpsc::Sender<BackendCommand>,
    evt_rx: mpsc::Receiver<BackendEvent>,
}

impl std::fmt::Debug for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Backend").finish()
    }
}

impl Backend {
    /// Create a new backend with a mock CAN driver (for testing/development).
    pub fn new_mock() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel(64);
        let (evt_tx, evt_rx) = mpsc::channel(256);

        tokio::spawn(mock_backend_task(cmd_rx, evt_tx));

        Self { cmd_tx, evt_rx }
    }

    /// Create a new backend with a real CAN driver.
    #[allow(dead_code)]
    pub fn new_with_driver<D: CanDriver + 'static>(driver: D, node_id: u8) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel(64);
        let (evt_tx, evt_rx) = mpsc::channel(256);

        tokio::spawn(real_backend_task(driver, node_id, cmd_rx, evt_tx));

        Self { cmd_tx, evt_rx }
    }

    /// Send a command to the backend (non-blocking).
    pub fn send(&self, cmd: BackendCommand) {
        // We use try_send for non-blocking behavior from GUI thread
        let _ = self.cmd_tx.try_send(cmd);
    }

    /// Try to receive an event (non-blocking).
    pub fn try_recv(&mut self) -> Option<BackendEvent> {
        self.evt_rx.try_recv().ok()
    }
}

/// Real backend task that uses CanopenStack for actual protocol operations.
#[allow(dead_code)]
async fn real_backend_task<C: CanDriver>(
    driver: C,
    node_id: u8,
    mut cmd_rx: mpsc::Receiver<BackendCommand>,
    evt_tx: mpsc::Sender<BackendEvent>,
) {
    let mut stack = CanopenStack::new(driver, node_id);

    let _ = evt_tx
        .send(BackendEvent::Connected("Real CAN".to_string()))
        .await;

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            BackendCommand::SdoUpload {
                node_id,
                index,
                subindex,
                respond,
            } => {
                let result = stack
                    .sdo_upload(node_id, index, subindex, DataType::Unsigned32)
                    .await;
                let response = match result {
                    Ok(val) => Ok(val.to_bytes()),
                    Err(e) => Err(e.to_string()),
                };
                let _ = respond.send(response.clone());
                let _ = evt_tx
                    .send(BackendEvent::SdoResult {
                        node_id,
                        index,
                        subindex,
                        result: response,
                    })
                    .await;
            }
            BackendCommand::SdoDownload {
                node_id,
                index,
                subindex,
                value,
                respond,
            } => {
                let od_value = OdValue::Domain(value);
                let result = stack
                    .sdo_download(node_id, index, subindex, &od_value)
                    .await;
                let response = result.map_err(|e| e.to_string());
                let _ = respond.send(response.clone());
                let _ = evt_tx
                    .send(BackendEvent::SdoResult {
                        node_id,
                        index,
                        subindex,
                        result: response.map(|()| vec![]),
                    })
                    .await;
            }
            BackendCommand::NmtStart(id) => {
                if let Err(e) = stack.nmt_start(id) {
                    let _ = evt_tx.send(BackendEvent::Error(e.to_string())).await;
                } else {
                    let _ = evt_tx
                        .send(BackendEvent::NmtStateChanged {
                            node_id: id,
                            state: "Operational".to_string(),
                        })
                        .await;
                }
            }
            BackendCommand::NmtStop(id) => {
                if let Err(e) = stack.nmt_stop(id) {
                    let _ = evt_tx.send(BackendEvent::Error(e.to_string())).await;
                } else {
                    let _ = evt_tx
                        .send(BackendEvent::NmtStateChanged {
                            node_id: id,
                            state: "Stopped".to_string(),
                        })
                        .await;
                }
            }
            BackendCommand::NmtReset(id) => {
                if let Err(e) = stack.nmt_reset(id) {
                    let _ = evt_tx.send(BackendEvent::Error(e.to_string())).await;
                } else {
                    let _ = evt_tx
                        .send(BackendEvent::NmtStateChanged {
                            node_id: id,
                            state: "PreOperational".to_string(),
                        })
                        .await;
                }
            }
            BackendCommand::ScanNodes { respond } => match stack.scan_nodes().await {
                Ok(nodes) => {
                    let _ = respond.send(nodes.clone());
                    let _ = evt_tx.send(BackendEvent::ScanResult(nodes)).await;
                }
                Err(e) => {
                    let _ = respond.send(vec![]);
                    let _ = evt_tx.send(BackendEvent::Error(e.to_string())).await;
                }
            },
            BackendCommand::Ds402Enable { node_id, respond } => {
                // DS402 enable sequence: Shutdown → SwitchOn → EnableOperation
                // Control word 0x6040, Status word 0x6041
                let result = async {
                    // Shutdown (0x0006)
                    stack
                        .sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x0006))
                        .await?;
                    // Switch On (0x0007)
                    stack
                        .sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x0007))
                        .await?;
                    // Enable Operation (0x000F)
                    stack
                        .sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x000F))
                        .await?;
                    Ok::<(), opencan_canopen_core::CanOpenError>(())
                }
                .await;

                let response = result.map_err(|e| e.to_string());
                let _ = respond.send(response.clone());

                if response.is_ok() {
                    let _ = evt_tx
                        .send(BackendEvent::Ds402StateResult {
                            node_id,
                            state: "OperationEnabled".to_string(),
                            status_word: 0x0027,
                        })
                        .await;
                }
            }
            BackendCommand::Ds402FaultReset { node_id, respond } => {
                let result = stack
                    .sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x0080))
                    .await;
                let response = result.map_err(|e| e.to_string());
                let _ = respond.send(response.clone());

                if response.is_ok() {
                    let _ = evt_tx
                        .send(BackendEvent::Ds402StateResult {
                            node_id,
                            state: "SwitchOnDisabled".to_string(),
                            status_word: 0x0040,
                        })
                        .await;
                }
            }
            BackendCommand::Ds402ReadState { node_id, respond } => {
                let result = stack
                    .sdo_upload(node_id, 0x6041, 0, DataType::Unsigned16)
                    .await;
                match result {
                    Ok(OdValue::Unsigned16(word)) => {
                        let state = opencan_canopen_ds402::Ds402State::from_status_word(word);
                        let state_str = format!("{:?}", state);
                        let _ = respond.send(Ok(word));
                        let _ = evt_tx
                            .send(BackendEvent::Ds402StateResult {
                                node_id,
                                state: state_str,
                                status_word: word,
                            })
                            .await;
                    }
                    Ok(other) => {
                        let _ = respond.send(Err(format!("Unexpected type: {:?}", other)));
                    }
                    Err(e) => {
                        let _ = respond.send(Err(e.to_string()));
                    }
                }
            }
            BackendCommand::Ds402SetPosition {
                node_id,
                position,
                respond,
            } => {
                let result = stack
                    .sdo_download(node_id, 0x607A, 0, &OdValue::Integer32(position))
                    .await;
                let _ = respond.send(result.map_err(|e| e.to_string()));
            }
            BackendCommand::Ds402SetVelocity {
                node_id,
                velocity,
                respond,
            } => {
                let result = stack
                    .sdo_download(node_id, 0x60FF, 0, &OdValue::Integer32(velocity))
                    .await;
                let _ = respond.send(result.map_err(|e| e.to_string()));
            }
            BackendCommand::Ds402ReadPosition { node_id, respond } => {
                let result = stack
                    .sdo_upload(node_id, 0x6064, 0, DataType::Integer32)
                    .await;
                match result {
                    Ok(OdValue::Integer32(pos)) => {
                        let _ = respond.send(Ok(pos));
                        let _ = evt_tx
                            .send(BackendEvent::Ds402PositionResult {
                                node_id,
                                position: pos,
                            })
                            .await;
                    }
                    Ok(OdValue::Unsigned32(pos)) => {
                        let pos = pos as i32;
                        let _ = respond.send(Ok(pos));
                        let _ = evt_tx
                            .send(BackendEvent::Ds402PositionResult {
                                node_id,
                                position: pos,
                            })
                            .await;
                    }
                    Ok(other) => {
                        let _ = respond.send(Err(format!("Unexpected type: {:?}", other)));
                    }
                    Err(e) => {
                        let _ = respond.send(Err(e.to_string()));
                    }
                }
            }
            BackendCommand::Ds402ReadVelocity { node_id, respond } => {
                let result = stack
                    .sdo_upload(node_id, 0x606C, 0, DataType::Integer32)
                    .await;
                match result {
                    Ok(OdValue::Integer32(vel)) => {
                        let _ = respond.send(Ok(vel));
                        let _ = evt_tx
                            .send(BackendEvent::Ds402VelocityResult {
                                node_id,
                                velocity: vel,
                            })
                            .await;
                    }
                    Ok(OdValue::Unsigned32(vel)) => {
                        let vel = vel as i32;
                        let _ = respond.send(Ok(vel));
                        let _ = evt_tx
                            .send(BackendEvent::Ds402VelocityResult {
                                node_id,
                                velocity: vel,
                            })
                            .await;
                    }
                    Ok(other) => {
                        let _ = respond.send(Err(format!("Unexpected type: {:?}", other)));
                    }
                    Err(e) => {
                        let _ = respond.send(Err(e.to_string()));
                    }
                }
            }
            BackendCommand::Disconnect => {
                let _ = evt_tx.send(BackendEvent::Disconnected).await;
                break;
            }
        }
    }
}

/// Mock backend task for testing without real hardware.
async fn mock_backend_task(
    mut cmd_rx: mpsc::Receiver<BackendCommand>,
    evt_tx: mpsc::Sender<BackendEvent>,
) {
    let _ = evt_tx
        .send(BackendEvent::Connected("Mock CAN".to_string()))
        .await;

    // Simulated node list
    let known_nodes = vec![1u8, 2, 3, 5, 10];

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            BackendCommand::SdoUpload {
                node_id,
                index,
                subindex,
                respond,
            } => {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                let result: Result<Vec<u8>, String> = match (index, subindex) {
                    (0x1000, 0) => Ok(vec![0x92, 0x01, 0x02, 0x00]), // Device Type
                    (0x1001, 0) => Ok(vec![0x00]),                   // Error Register
                    (0x1018, 1) => Ok(vec![0x78, 0x56, 0x34, 0x12]), // Vendor ID
                    (0x6041, 0) => Ok(vec![0x27, 0x00]),             // Status Word (OpEnabled)
                    (0x6064, 0) => Ok(vec![0xD2, 0x04, 0x00, 0x00]), // Actual Position (1234)
                    (0x606C, 0) => Ok(vec![0x2C, 0x01, 0x00, 0x00]), // Actual Velocity (300)
                    _ => Err("Object does not exist".to_string()),
                };

                let _ = respond.send(result.clone());
                let _ = evt_tx
                    .send(BackendEvent::SdoResult {
                        node_id,
                        index,
                        subindex,
                        result,
                    })
                    .await;
            }
            BackendCommand::SdoDownload {
                node_id,
                index,
                subindex,
                value: _,
                respond,
            } => {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                let result: Result<(), String> = Ok(());
                let _ = respond.send(result);
                let _ = evt_tx
                    .send(BackendEvent::SdoResult {
                        node_id,
                        index,
                        subindex,
                        result: Ok(vec![]),
                    })
                    .await;
            }
            BackendCommand::ScanNodes { respond } => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let _ = respond.send(known_nodes.clone());
                let _ = evt_tx
                    .send(BackendEvent::ScanResult(known_nodes.clone()))
                    .await;
            }
            BackendCommand::NmtStart(id) => {
                let _ = evt_tx
                    .send(BackendEvent::NmtStateChanged {
                        node_id: id,
                        state: "Operational".to_string(),
                    })
                    .await;
            }
            BackendCommand::NmtStop(id) => {
                let _ = evt_tx
                    .send(BackendEvent::NmtStateChanged {
                        node_id: id,
                        state: "Stopped".to_string(),
                    })
                    .await;
            }
            BackendCommand::NmtReset(id) => {
                let _ = evt_tx
                    .send(BackendEvent::NmtStateChanged {
                        node_id: id,
                        state: "PreOperational".to_string(),
                    })
                    .await;
            }
            BackendCommand::Ds402Enable {
                node_id: id,
                respond,
            } => {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                let _ = respond.send(Ok(()));
                let _ = evt_tx
                    .send(BackendEvent::Ds402StateResult {
                        node_id: id,
                        state: "OperationEnabled".to_string(),
                        status_word: 0x0027,
                    })
                    .await;
            }
            BackendCommand::Ds402FaultReset {
                node_id: id,
                respond,
            } => {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                let _ = respond.send(Ok(()));
                let _ = evt_tx
                    .send(BackendEvent::Ds402StateResult {
                        node_id: id,
                        state: "SwitchOnDisabled".to_string(),
                        status_word: 0x0040,
                    })
                    .await;
            }
            BackendCommand::Ds402ReadState {
                node_id: id,
                respond,
            } => {
                let _ = respond.send(Ok(0x0027));
                let _ = evt_tx
                    .send(BackendEvent::Ds402StateResult {
                        node_id: id,
                        state: "OperationEnabled".to_string(),
                        status_word: 0x0027,
                    })
                    .await;
            }
            BackendCommand::Ds402SetPosition {
                node_id,
                position: _,
                respond,
            } => {
                let _ = respond.send(Ok(()));
                let _ = evt_tx
                    .send(BackendEvent::SdoResult {
                        node_id,
                        index: 0x607A,
                        subindex: 0,
                        result: Ok(vec![]),
                    })
                    .await;
            }
            BackendCommand::Ds402SetVelocity {
                node_id,
                velocity: _,
                respond,
            } => {
                let _ = respond.send(Ok(()));
                let _ = evt_tx
                    .send(BackendEvent::SdoResult {
                        node_id,
                        index: 0x60FF,
                        subindex: 0,
                        result: Ok(vec![]),
                    })
                    .await;
            }
            BackendCommand::Ds402ReadPosition {
                node_id: id,
                respond,
            } => {
                let _ = respond.send(Ok(12345));
                let _ = evt_tx
                    .send(BackendEvent::Ds402PositionResult {
                        node_id: id,
                        position: 12345,
                    })
                    .await;
            }
            BackendCommand::Ds402ReadVelocity {
                node_id: id,
                respond,
            } => {
                let _ = respond.send(Ok(500));
                let _ = evt_tx
                    .send(BackendEvent::Ds402VelocityResult {
                        node_id: id,
                        velocity: 500,
                    })
                    .await;
            }
            BackendCommand::Disconnect => {
                let _ = evt_tx.send(BackendEvent::Disconnected).await;
                break;
            }
        }
    }
}
