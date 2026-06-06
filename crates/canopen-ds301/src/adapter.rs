//! CanDriverAdapter — bridges CanBus (physical) to CanDriver (protocol).
//!
//! This adapter converts between the physical CAN frame format (CanFrame)
//! and the CANOpen protocol frame format (CanOpenFrame).

use opencan_can_traits::{CanBus, CanFrame, CanId};
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::error::CanOpenError;
use opencan_canopen_core::frame::CanOpenFrame;

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
        CanFrame::Classic(opencan_can_traits::ClassicFrame::new(
            CanId::Standard(frame.cob_id),
            &frame.data,
        ))
    }

    fn can_to_canopen(frame: &CanFrame) -> Result<CanOpenFrame, CanOpenError> {
        match frame {
            CanFrame::Classic(f) => {
                let cob_id = match f.id {
                    CanId::Standard(id) => id,
                    CanId::Extended(_) => {
                        return Err(CanOpenError::Protocol(
                            "CANOpen does not use extended frames".to_string(),
                        ));
                    }
                };
                let mut data = [0u8; 8];
                let len = f.len.min(8) as usize;
                data[..len].copy_from_slice(&f.data[..len]);
                Ok(CanOpenFrame::new(cob_id, data))
            }
            CanFrame::Fd(_) => Err(CanOpenError::Protocol(
                "CANOpen does not use CAN FD frames".to_string(),
            )),
        }
    }
}

impl<B: CanBus> CanDriver for CanDriverAdapter<B> {
    fn send(&mut self, frame: &CanOpenFrame) -> Result<(), CanOpenError> {
        let can_frame = Self::canopen_to_can(frame);
        self.bus.send(&can_frame).map_err(|e| {
            CanOpenError::Can(opencan_canopen_core::error::CanError::Io(e.to_string()))
        })
    }

    async fn recv(&mut self) -> Result<CanOpenFrame, CanOpenError> {
        let can_frame = self.bus.recv().await.map_err(|e| {
            CanOpenError::Can(opencan_canopen_core::error::CanError::Io(e.to_string()))
        })?;
        Self::can_to_canopen(&can_frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencan_can_traits::error::CanError;
    use opencan_can_traits::{CanBitrate, CanState};
    use std::future::Future;

    // Mock CanBus for testing
    struct MockBus;

    impl opencan_can_traits::CanBus for MockBus {
        fn send(&self, _frame: &CanFrame) -> Result<(), CanError> {
            Ok(())
        }

        fn recv(&self) -> impl Future<Output = Result<CanFrame, CanError>> + Send {
            async { Err(CanError::Io("not implemented".into())) }
        }

        fn state(&self) -> CanState {
            CanState::Active
        }

        fn set_bitrate(&self, _bitrate: CanBitrate) -> Result<(), CanError> {
            Ok(())
        }
    }

    #[test]
    fn test_canopen_to_can_roundtrip() {
        let original = CanOpenFrame::new(0x583, [0x43, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00]);
        let can_frame = CanDriverAdapter::<MockBus>::canopen_to_can(&original);
        let restored = CanDriverAdapter::<MockBus>::can_to_canopen(&can_frame).unwrap();

        assert_eq!(restored.cob_id, original.cob_id);
        assert_eq!(restored.data, original.data);
    }

    #[test]
    fn test_canopen_to_can_standard_id() {
        let frame = CanOpenFrame::new(0x180, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
        let can_frame = CanDriverAdapter::<MockBus>::canopen_to_can(&frame);

        match &can_frame {
            CanFrame::Classic(f) => {
                assert_eq!(f.id, CanId::Standard(0x180));
                assert_eq!(
                    f.data[..f.len as usize],
                    [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
                );
            }
            _ => panic!("Expected Classic frame"),
        }
    }

    #[test]
    fn test_can_to_canopen_rejects_extended() {
        let can_frame = CanFrame::Classic(opencan_can_traits::ClassicFrame {
            id: CanId::Extended(0x12345678),
            data: [0u8; 8],
            len: 8,
            timestamp_us: None,
        });
        let result = CanDriverAdapter::<MockBus>::can_to_canopen(&can_frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_can_to_canopen_rejects_fd() {
        let can_frame = CanFrame::Fd(opencan_can_traits::FdFrame {
            id: CanId::Standard(0x123),
            data: vec![0u8; 64],
            flags: opencan_can_traits::FdFlags::default(),
            timestamp_us: None,
        });
        let result = CanDriverAdapter::<MockBus>::can_to_canopen(&can_frame);
        assert!(result.is_err());
    }
}
