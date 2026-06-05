//! CANOpen stack — main loop and event processing.

use opencan_canopen_core::CanDriver;
use opencan_canopen_core::frame::{CanOpenFrame, FrameClass, classify_frame, PdoFrame};

use crate::heartbeat::HeartbeatConsumer;
use crate::emcy::EmergencyHandler;
use crate::nmt::NmtMaster;

/// CANOpen protocol events emitted by the stack.
#[derive(Debug, Clone)]
pub enum CanEvent {
    HeartbeatChanged { node_id: u8, alive: bool },
    HeartbeatTimeout { node_id: u8 },
    Emergency { node_id: u8, error_code: u16 },
    PdoReceived { pdo: PdoFrame },
    SdoComplete { node_id: u8, result: Result<Vec<u8>, String> },
}

/// Main CANOpen protocol stack.
pub struct CanopenStack<C: CanDriver> {
    can: C,
    node_id: u8,
    heartbeat: HeartbeatConsumer,
    emergency: EmergencyHandler,
    nmt: NmtMaster,
}

impl<C: CanDriver> CanopenStack<C> {
    pub fn new(can: C, node_id: u8) -> Self {
        Self {
            can,
            node_id,
            heartbeat: HeartbeatConsumer::new(std::time::Duration::from_secs(1)),
            emergency: EmergencyHandler::new(1000),
            nmt: NmtMaster::new(),
        }
    }

    /// Process one CAN frame — call this in a loop.
    pub fn process(&mut self, frame: &CanOpenFrame) -> Vec<CanEvent> {
        let mut events = Vec::new();

        match classify_frame(frame) {
            FrameClass::Heartbeat(hb) => {
                let changed = self.heartbeat.update(&hb);
                if changed {
                    events.push(CanEvent::HeartbeatChanged {
                        node_id: hb.node_id,
                        alive: self.heartbeat.is_alive(hb.node_id),
                    });
                }
            }
            FrameClass::Emergency(emcy) => {
                self.emergency.record(&emcy);
                events.push(CanEvent::Emergency {
                    node_id: emcy.node_id,
                    error_code: emcy.error_code,
                });
            }
            FrameClass::Pdo(pdo) => {
                events.push(CanEvent::PdoReceived { pdo });
            }
            FrameClass::Nmt(_) => {
                // NMT commands from other masters — ignore for now
            }
            FrameClass::SdoResponse(_) => {
                // SDO responses are handled by SdoClient directly
            }
            FrameClass::Sync => {
                // TODO: Handle SYNC
            }
            FrameClass::Timestamp => {
                // TODO: Handle TIME
            }
            FrameClass::Unknown => {
                // Unknown frame — log if needed
            }
        }

        // Check for heartbeat timeouts
        for (node_id, _elapsed) in self.heartbeat.check_timeouts() {
            events.push(CanEvent::HeartbeatTimeout { node_id });
        }

        events
    }

    /// Get a reference to the NMT master.
    pub fn nmt(&self) -> &NmtMaster {
        &self.nmt
    }

    /// Get a mutable reference to the NMT master.
    pub fn nmt_mut(&mut self) -> &mut NmtMaster {
        &mut self.nmt
    }

    /// Get a reference to the heartbeat consumer.
    pub fn heartbeat(&self) -> &HeartbeatConsumer {
        &self.heartbeat
    }

    /// Get a reference to the emergency handler.
    pub fn emergency(&self) -> &EmergencyHandler {
        &self.emergency
    }

    /// Get the node ID of this stack.
    pub fn node_id(&self) -> u8 {
        self.node_id
    }

    /// Get a reference to the CAN driver.
    pub fn can(&self) -> &C {
        &self.can
    }

    /// Get a mutable reference to the CAN driver.
    pub fn can_mut(&mut self) -> &mut C {
        &mut self.can
    }
}
