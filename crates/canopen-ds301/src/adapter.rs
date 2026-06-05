//! CanDriverAdapter — bridges CanBus (physical) to CanDriver (protocol).
//!
//! This adapter converts between the physical CAN frame format (CanFrame)
//! and the CANOpen protocol frame format (CanOpenFrame).

use opencan_canopen_core::CanDriver;
use opencan_canopen_core::error::CanOpenError;
use opencan_canopen_core::frame::CanOpenFrame;
use opencan_can_traits::{CanBus, CanFrame, CanId};

/// Adapter that wraps a CanBus implementation to provide CanDriver.
pub struct CanDriverAdapter<B: CanBus> {
    bus: B,
}

impl<B: CanBus> CanDriverAdapter<B> {
    pub fn new(bus: B) -> Self {
        Self { bus }
    }

    pub fn bus(&self) -> &B {
        &self.bus
    }

    fn canopen_to_can(frame: &CanOpenFrame) -> CanFrame {
        // CANOpen uses standard 11-bit COB-ID
        CanFrame::Classic(opencan_can_traits::ClassicFrame::new(
            CanId::Standard(frame.cob_id),
            frame.data.to_vec(),
        ))
    }

    fn can_to_canopen(frame: &CanFrame) -> Result<CanOpenFrame, CanOpenError> {
        match frame {
            CanFrame::Classic(f) => {
                let cob_id = match f.id {
                    CanId::Standard(id) => id,
                    CanId::Extended(_) => {
                        return Err(CanOpenError::Protocol(
                            "CANOpen does not use extended frames".to_string()
                        ));
                    }
                };
                let mut data = [0u8; 8];
                let len = f.data.len().min(8);
                data[..len].copy_from_slice(&f.data[..len]);
                Ok(CanOpenFrame::new(cob_id, data).with_timestamp(
                    f.timestamp.unwrap_or_else(std::time::Instant::now)
                ))
            }
            CanFrame::Fd(_) => {
                Err(CanOpenError::Protocol(
                    "CANOpen does not use CAN FD frames".to_string()
                ))
            }
        }
    }
}

impl<B: CanBus> CanDriver for CanDriverAdapter<B> {
    fn send(&mut self, frame: &CanOpenFrame) -> Result<(), CanOpenError> {
        let can_frame = Self::canopen_to_can(frame);
        self.bus.send(&can_frame).map_err(|e| CanOpenError::Can(opencan_canopen_core::error::CanError::Io(e.to_string())))
    }

    fn recv(&mut self) -> Result<CanOpenFrame, CanOpenError> {
        let can_frame = self.bus.recv().map_err(|e| CanOpenError::Can(opencan_canopen_core::error::CanError::Io(e.to_string())))?;
        Self::can_to_canopen(&can_frame)
    }

    async fn recv_async(&mut self) -> Result<CanOpenFrame, CanOpenError> {
        // For now, use blocking recv
        // Real implementation would use tokio::task::spawn_blocking
        // or an async-native CanBus implementation
        let can_frame = self.bus.recv().map_err(|e| CanOpenError::Can(opencan_canopen_core::error::CanError::Io(e.to_string())))?;
        Self::can_to_canopen(&can_frame)
    }
}
