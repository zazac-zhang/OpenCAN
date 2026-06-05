//! # opencan-can-socketcan
//!
//! Linux SocketCAN backend for OpenCAN.
//!
//! Uses the `socketcan` crate to access Linux SocketCAN interfaces.
//! Only available on Linux.

#[cfg(target_os = "linux")]
mod linux {
    use std::future::Future;
    use opencan_can_traits::{
        CanBus, CanBusDyn, CanBusFactory, CanBitrate, CanConfig, CanFrame, CanId,
        CanState, error::CanError,
    };

    /// SocketCAN bus implementation.
    pub struct SocketCanBus {
        socket: socketcan::tokio::CanSocket,
        interface: String,
    }

    impl SocketCanBus {
        pub fn open(interface: &str) -> Result<Self, CanError> {
            let socket = socketcan::tokio::CanSocket::open(interface)
                .map_err(|e| CanError::Io(format!("Failed to open {}: {}", interface, e)))?;
            Ok(Self { socket, interface: interface.to_string() })
        }
    }

    impl CanBus for SocketCanBus {
        fn send(&self, frame: &CanFrame) -> Result<(), CanError> {
            let can_frame = match frame {
                CanFrame::Classic(f) => {
                    let id = match f.id {
                        CanId::Standard(id) => socketcan::StandardId::new(id)
                            .ok_or_else(|| CanError::InvalidConfig(format!("Invalid CAN ID: {}", id)))?,
                        CanId::Extended(_) => {
                            return Err(CanError::Unsupported("Extended ID not yet supported".to_string()));
                        }
                    };
                    let data = socketcan::CanData::new(&f.data[..f.len as usize])
                        .ok_or_else(|| CanError::InvalidConfig("Invalid data length".to_string()))?;
                    socketcan::CanFrame::new(id, &data)
                        .ok_or_else(|| CanError::Io("Failed to create frame".to_string()))?
                }
                CanFrame::Fd(_) => {
                    return Err(CanError::Unsupported("CAN FD not yet supported".to_string()));
                }
            };

            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.socket.write_frame(can_frame))
                .map_err(|e| CanError::Io(e.to_string()))?;
            Ok(())
        }

        fn recv(&self) -> impl Future<Output = Result<CanFrame, CanError>> + Send {
            // Simplified: block_on in a spawn_blocking
            async move {
                Err(CanError::Io("Async recv implementation pending".to_string()))
            }
        }

        fn state(&self) -> CanState { CanState::Active }
        fn set_bitrate(&self, _bitrate: CanBitrate) -> Result<(), CanError> {
            Err(CanError::Unsupported("Bitrate must be set at interface level".to_string()))
        }
    }

    /// Factory for SocketCAN.
    pub struct SocketCanFactory;

    impl CanBusFactory for SocketCanFactory {
        fn open(&self, channel: &str, _config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
            Ok(Box::new(SocketCanBus::open(channel)?))
        }
        fn name(&self) -> &str { "SocketCAN" }
        fn available_channels(&self) -> Vec<String> {
            let mut channels = Vec::new();
            if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let type_path = format!("/sys/class/net/{}/type", name);
                    if let Ok(type_str) = std::fs::read_to_string(&type_path) {
                        if type_str.trim() == "280" { channels.push(name); }
                    }
                }
            }
            channels
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::{SocketCanBus, SocketCanFactory};
