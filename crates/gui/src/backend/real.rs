//! Real backend task that uses CanopenStack for actual protocol operations.

use tokio::sync::mpsc;
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::od::{DataType, OdValue};
use opencan_canopen_ds301::CanopenStack;

use super::command::BackendCommand;
use super::event::BackendEvent;

/// Real backend task that uses CanopenStack for actual protocol operations.
pub async fn real_backend_task<C: CanDriver>(
    driver: C,
    node_id: u8,
    mut cmd_rx: mpsc::Receiver<BackendCommand>,
    evt_tx: mpsc::Sender<BackendEvent>,
) {
    let mut stack = CanopenStack::new(driver, node_id);

    let _ = evt_tx.send(BackendEvent::Connected("Real CAN".to_string())).await;

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            BackendCommand::Disconnect => {
                let _ = evt_tx.send(BackendEvent::Disconnected).await;
                break;
            }

            // === Node Scanning ===
            BackendCommand::ScanNodes { respond } => {
                match stack.scan_nodes().await {
                    Ok(nodes) => {
                        let _ = respond.send(Ok(nodes.clone()));
                        let _ = evt_tx.send(BackendEvent::ScanResult(nodes)).await;
                    }
                    Err(e) => {
                        let _ = respond.send(Err(e.to_string()));
                        let _ = evt_tx.send(BackendEvent::Error(e.to_string())).await;
                    }
                }
            }

            // === NMT ===
            BackendCommand::NmtStart(id) => {
                if let Err(e) = stack.nmt_start(id) {
                    let _ = evt_tx.send(BackendEvent::Error(e.to_string())).await;
                } else {
                    let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                        node_id: id, state: "Operational".to_string()
                    }).await;
                }
            }
            BackendCommand::NmtStop(id) => {
                if let Err(e) = stack.nmt_stop(id) {
                    let _ = evt_tx.send(BackendEvent::Error(e.to_string())).await;
                } else {
                    let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                        node_id: id, state: "Stopped".to_string()
                    }).await;
                }
            }
            BackendCommand::NmtReset(id) => {
                if let Err(e) = stack.nmt_reset(id) {
                    let _ = evt_tx.send(BackendEvent::Error(e.to_string())).await;
                } else {
                    let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                        node_id: id, state: "PreOperational".to_string()
                    }).await;
                }
            }
            BackendCommand::NmtResetComm(id) => {
                if let Err(e) = stack.nmt_reset(id) {
                    let _ = evt_tx.send(BackendEvent::Error(e.to_string())).await;
                } else {
                    let _ = evt_tx.send(BackendEvent::NmtStateChanged {
                        node_id: id, state: "PreOperational".to_string()
                    }).await;
                }
            }

            // === SDO ===
            BackendCommand::SdoUpload { node_id, index, subindex, respond } => {
                let result = stack.sdo_upload(node_id, index, subindex, DataType::Unsigned32).await;
                let response = match result {
                    Ok(val) => Ok(val.to_bytes()),
                    Err(e) => Err(e.to_string()),
                };
                let _ = respond.send(response.clone());
                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index, subindex, result: response,
                }).await;
            }

            BackendCommand::SdoDownload { node_id, index, subindex, value, respond } => {
                let od_value = OdValue::Domain(value);
                let result = stack.sdo_download(node_id, index, subindex, &od_value).await;
                let response = result.map_err(|e| e.to_string());
                let _ = respond.send(response.clone());
                let _ = evt_tx.send(BackendEvent::SdoResult {
                    node_id, index, subindex,
                    result: response.map(|()| vec![]),
                }).await;
            }

            // === DS402 ===
            BackendCommand::Ds402Enable { node_id, respond } => {
                // DS402 enable sequence: Shutdown → SwitchOn → EnableOperation
                let result = async {
                    stack.sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x0006)).await?;
                    stack.sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x0007)).await?;
                    stack.sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x000F)).await?;
                    Ok::<(), opencan_canopen_core::CanOpenError>(())
                }.await;

                let response = result.map_err(|e| e.to_string());
                let _ = respond.send(response.clone());

                if response.is_ok() {
                    let _ = evt_tx.send(BackendEvent::Ds402StateResult {
                        node_id, state: "OperationEnabled".to_string(), status_word: 0x0027,
                    }).await;
                }
            }

            BackendCommand::Ds402FaultReset { node_id, respond } => {
                let result = stack.sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(0x0080)).await;
                let response = result.map_err(|e| e.to_string());
                let _ = respond.send(response.clone());

                if response.is_ok() {
                    let _ = evt_tx.send(BackendEvent::Ds402StateResult {
                        node_id, state: "SwitchOnDisabled".to_string(), status_word: 0x0040,
                    }).await;
                }
            }

            BackendCommand::Ds402ReadState { node_id, respond } => {
                let result = stack.sdo_upload(node_id, 0x6041, 0, DataType::Unsigned16).await;
                match result {
                    Ok(OdValue::Unsigned16(word)) => {
                        let state = opencan_canopen_ds402::Ds402State::from_status_word(word);
                        let state_str = format!("{:?}", state);
                        let _ = respond.send(Ok(word));
                        let _ = evt_tx.send(BackendEvent::Ds402StateResult {
                            node_id, state: state_str, status_word: word,
                        }).await;
                    }
                    Ok(other) => {
                        let _ = respond.send(Err(format!("Unexpected type: {:?}", other)));
                    }
                    Err(e) => {
                        let _ = respond.send(Err(e.to_string()));
                    }
                }
            }

            BackendCommand::Ds402WriteControl { node_id, control_word, respond } => {
                let result = stack.sdo_download(node_id, 0x6040, 0, &OdValue::Unsigned16(control_word)).await;
                let _ = respond.send(result.map_err(|e| e.to_string()));
            }

            BackendCommand::Ds402SetPosition { node_id, position, respond } => {
                let result = stack.sdo_download(node_id, 0x607A, 0, &OdValue::Integer32(position)).await;
                let _ = respond.send(result.map_err(|e| e.to_string()));
            }

            BackendCommand::Ds402SetVelocity { node_id, velocity, respond } => {
                let result = stack.sdo_download(node_id, 0x60FF, 0, &OdValue::Integer32(velocity)).await;
                let _ = respond.send(result.map_err(|e| e.to_string()));
            }

            BackendCommand::Ds402SetTorque { node_id, torque, respond } => {
                let result = stack.sdo_download(node_id, 0x6071, 0, &OdValue::Integer16(torque)).await;
                let _ = respond.send(result.map_err(|e| e.to_string()));
            }

            BackendCommand::Ds402ReadPosition { node_id, respond } => {
                let result = stack.sdo_upload(node_id, 0x6064, 0, DataType::Integer32).await;
                match result {
                    Ok(OdValue::Integer32(pos)) => {
                        let _ = respond.send(Ok(pos));
                        let _ = evt_tx.send(BackendEvent::Ds402PositionResult { node_id, position: pos }).await;
                    }
                    Ok(OdValue::Unsigned32(pos)) => {
                        let pos = pos as i32;
                        let _ = respond.send(Ok(pos));
                        let _ = evt_tx.send(BackendEvent::Ds402PositionResult { node_id, position: pos }).await;
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
                let result = stack.sdo_upload(node_id, 0x606C, 0, DataType::Integer32).await;
                match result {
                    Ok(OdValue::Integer32(vel)) => {
                        let _ = respond.send(Ok(vel));
                        let _ = evt_tx.send(BackendEvent::Ds402VelocityResult { node_id, velocity: vel }).await;
                    }
                    Ok(OdValue::Unsigned32(vel)) => {
                        let vel = vel as i32;
                        let _ = respond.send(Ok(vel));
                        let _ = evt_tx.send(BackendEvent::Ds402VelocityResult { node_id, velocity: vel }).await;
                    }
                    Ok(other) => {
                        let _ = respond.send(Err(format!("Unexpected type: {:?}", other)));
                    }
                    Err(e) => {
                        let _ = respond.send(Err(e.to_string()));
                    }
                }
            }

            BackendCommand::Ds402ReadTorque { node_id, respond } => {
                let result = stack.sdo_upload(node_id, 0x6077, 0, DataType::Integer16).await;
                match result {
                    Ok(OdValue::Integer16(torque)) => {
                        let _ = respond.send(Ok(torque));
                        let _ = evt_tx.send(BackendEvent::Ds402TorqueResult { node_id, torque }).await;
                    }
                    Ok(other) => {
                        let _ = respond.send(Err(format!("Unexpected type: {:?}", other)));
                    }
                    Err(e) => {
                        let _ = respond.send(Err(e.to_string()));
                    }
                }
            }

            BackendCommand::Ds402SetMode { node_id, mode, respond } => {
                let result = stack.sdo_download(node_id, 0x6060, 0, &OdValue::Integer8(mode)).await;
                let _ = respond.send(result.map_err(|e| e.to_string()));
            }

            // === PDO ===
            BackendCommand::ReadPdoMapping { node_id, pdo_index, respond } => {
                // Read PDO mapping (0x1A00 for TPDO1, 0x1600 for RPDO1, etc.)
                let index = match pdo_index {
                    1 => 0x1A00u16,
                    2 => 0x1A01,
                    3 => 0x1A02,
                    4 => 0x1A03,
                    _ => {
                        let _ = respond.send(Err("Invalid PDO index".to_string()));
                        continue;
                    }
                };

                // Read number of mapped objects
                let count_result = stack.sdo_upload(node_id, index, 0, DataType::Unsigned8).await;
                match count_result {
                    Ok(OdValue::Unsigned8(count)) => {
                        let mut mapping = Vec::new();
                        for sub in 1..=count {
                            if let Ok(OdValue::Unsigned32(entry)) = stack.sdo_upload(node_id, index, sub, DataType::Unsigned32).await {
                                let obj_index = (entry >> 16) as u16;
                                let obj_subindex = ((entry >> 8) & 0xFF) as u8;
                                let bit_length = (entry & 0xFF) as u8;
                                mapping.push((obj_index, obj_subindex, bit_length));
                            }
                        }
                        let _ = respond.send(Ok(mapping));
                    }
                    Ok(_) => {
                        let _ = respond.send(Err("Unexpected type for PDO mapping count".to_string()));
                    }
                    Err(e) => {
                        let _ = respond.send(Err(e.to_string()));
                    }
                }
            }
        }
    }
}
