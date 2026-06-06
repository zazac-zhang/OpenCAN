//! # opencan-can-socketcan
//!
//! Linux SocketCAN backend for OpenCAN.
//!
//! Uses the `socketcan` crate to access Linux SocketCAN interfaces.
//! Only available on Linux.
//!
//! # Thread Safety
//!
//! `SocketCanBus` wraps the tokio async CAN socket in a `tokio::sync::Mutex`
//! to allow concurrent access from the `CanBus` trait methods which take `&self`.

#[cfg(target_os = "linux")]
mod linux {
    use opencan_can_traits::{
        CanBitrate, CanBus, CanBusDyn, CanBusFactory, CanConfig, CanFrame, CanId, CanState,
        ClassicFrame, error::CanError,
    };
    use std::future::Future;
    use tokio::sync::Mutex;

    /// SocketCAN bus implementation.
    ///
    /// Uses `tokio::sync::Mutex` to allow `&self` access for both send and recv.
    pub struct SocketCanBus {
        socket: Mutex<socketcan::tokio::CanSocket>,
        interface: String,
    }

    impl SocketCanBus {
        /// Open a SocketCAN interface (e.g. "can0", "vcan0").
        pub fn open(interface: &str) -> Result<Self, CanError> {
            let socket = socketcan::tokio::CanSocket::open(interface)
                .map_err(|e| CanError::Io(format!("Failed to open {}: {}", interface, e)))?;
            Ok(Self {
                socket: Mutex::new(socket),
                interface: interface.to_string(),
            })
        }

        /// Get the interface name.
        pub fn interface(&self) -> &str {
            &self.interface
        }

        /// Convert a CanFrame to a socketcan frame.
        fn to_socketcan_frame(frame: &CanFrame) -> Result<socketcan::CanFrame, CanError> {
            match frame {
                CanFrame::Classic(f) => {
                    let id = match f.id {
                        CanId::Standard(id) => socketcan::StandardId::new(id).ok_or_else(|| {
                            CanError::InvalidConfig(format!("Invalid CAN ID: {}", id))
                        })?,
                        CanId::Extended(id) => socketcan::ExtendedId::new(id).ok_or_else(|| {
                            CanError::InvalidConfig(format!("Invalid extended CAN ID: {}", id))
                        })?,
                    };
                    let data =
                        socketcan::CanData::new(&f.data[..f.len as usize]).ok_or_else(|| {
                            CanError::InvalidConfig("Invalid data length".to_string())
                        })?;
                    socketcan::CanFrame::new(id, &data)
                        .ok_or_else(|| CanError::Io("Failed to create CAN frame".to_string()))
                }
                CanFrame::Fd(_) => Err(CanError::Unsupported(
                    "CAN FD not yet supported for SocketCAN".to_string(),
                )),
            }
        }

        /// Convert a socketcan frame to a CanFrame.
        fn from_socketcan_frame(frame: &socketcan::CanFrame) -> CanFrame {
            let id = if frame.is_extended() {
                CanId::Extended(frame.id().as_raw())
            } else {
                CanId::Standard(frame.id().as_raw() as u16)
            };

            let data = frame.data();
            let mut buf = [0u8; 8];
            let len = data.len().min(8);
            buf[..len].copy_from_slice(&data[..len]);

            CanFrame::Classic(ClassicFrame {
                id,
                data: buf,
                len: len as u8,
                timestamp_us: None,
            })
        }
    }

    impl CanBus for SocketCanBus {
        fn send(&self, frame: &CanFrame) -> Result<(), CanError> {
            let can_frame = Self::to_socketcan_frame(frame)?;

            // Use block_on for synchronous send.
            // This works when called from non-async context (e.g. GUI thread).
            // If called from within tokio runtime, it uses the current handle.
            let rt = tokio::runtime::Handle::current();
            let socket = self.socket.blocking_lock();
            rt.block_on(socket.write_frame(can_frame))
                .map_err(|e| CanError::Io(format!("SocketCAN send error: {}", e)))?;
            Ok(())
        }

        fn recv(&self) -> impl Future<Output = Result<CanFrame, CanError>> + Send {
            async move {
                let socket = self.socket.lock().await;
                let frame = socket
                    .read_frame()
                    .await
                    .map_err(|e| CanError::Io(format!("SocketCAN recv error: {}", e)))?;
                Ok(Self::from_socketcan_frame(&frame))
            }
        }

        fn state(&self) -> CanState {
            // SocketCAN doesn't provide easy state query via the socket API.
            // We'd need to check /sys/class/net/<iface>/state for real state.
            // For now, assume active if the socket was opened successfully.
            CanState::Active
        }

        fn set_bitrate(&self, _bitrate: CanBitrate) -> Result<(), CanError> {
            // Bitrate must be set at the interface level (ip link set can0 type can bitrate 500000)
            Err(CanError::Unsupported(
                "SocketCAN bitrate must be set at interface level via 'ip link set'".to_string(),
            ))
        }
    }

    /// Factory for creating SocketCAN bus instances.
    pub struct SocketCanFactory;

    impl CanBusFactory for SocketCanFactory {
        fn open(&self, channel: &str, _config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
            Ok(Box::new(SocketCanBus::open(channel)?))
        }

        fn name(&self) -> &str {
            "SocketCAN"
        }

        fn available_channels(&self) -> Vec<String> {
            let mut channels = Vec::new();
            if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // CAN interfaces have type 280 (ARPHRD_CAN)
                    let type_path = format!("/sys/class/net/{}/type", name);
                    if let Ok(type_str) = std::fs::read_to_string(&type_path) {
                        if type_str.trim() == "280" {
                            channels.push(name);
                        }
                    }
                }
            }
            channels
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::{SocketCanBus, SocketCanFactory};
