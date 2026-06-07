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
//!
//! # CAN FD Support
//!
//! This backend supports both CAN 2.0 Classic and CAN FD frames.
//! When `fd: true` is set in `CanConfig`, the socket is opened as a CAN FD socket
//! which can handle both frame types.

#[cfg(target_os = "linux")]
mod linux {
    use crate::{
        CanBitrate, CanBus, CanBusDyn, CanBusFactory, CanConfig, CanFrame, CanId, CanState,
        ClassicFrame, FdFlags, FdFrame, error::CanError,
    };
    use std::future::Future;
    use tokio::sync::Mutex;

    /// SocketCAN bus implementation.
    ///
    /// Uses `tokio::sync::Mutex` to allow `&self` access for both send and recv.
    /// Supports both CAN 2.0 Classic and CAN FD frames.
    pub struct SocketCanBus {
        socket: Mutex<CanSocketVariant>,
        interface: String,
        fd_enabled: bool,
    }

    /// 内部 socket 变体，支持 Classic 和 FD
    enum CanSocketVariant {
        Classic(socketcan::tokio::CanSocket),
        Fd(socketcan::tokio::CanFdSocket),
    }

    impl SocketCanBus {
        /// Open a SocketCAN interface (e.g. "can0", "vcan0").
        ///
        /// If `fd` is true in config, opens as CAN FD socket.
        pub fn open(interface: &str, config: &CanConfig) -> Result<Self, CanError> {
            let socket = if config.fd {
                let fd_socket = socketcan::tokio::CanFdSocket::open(interface).map_err(|e| {
                    CanError::Io(format!("Failed to open FD socket {}: {}", interface, e))
                })?;
                CanSocketVariant::Fd(fd_socket)
            } else {
                let classic_socket = socketcan::tokio::CanSocket::open(interface)
                    .map_err(|e| CanError::Io(format!("Failed to open {}: {}", interface, e)))?;
                CanSocketVariant::Classic(classic_socket)
            };

            Ok(Self {
                socket: Mutex::new(socket),
                interface: interface.to_string(),
                fd_enabled: config.fd,
            })
        }

        /// Get the interface name.
        pub fn interface(&self) -> &str {
            &self.interface
        }

        /// Check if CAN FD is enabled.
        pub fn is_fd_enabled(&self) -> bool {
            self.fd_enabled
        }

        /// Convert a CanFrame to a socketcan CanAnyFrame.
        fn to_socketcan_any_frame(frame: &CanFrame) -> Result<socketcan::CanAnyFrame, CanError> {
            match frame {
                CanFrame::Classic(f) => {
                    let id = Self::to_socketcan_id(f.id)?;
                    let data =
                        socketcan::CanData::new(&f.data[..f.len as usize]).ok_or_else(|| {
                            CanError::InvalidConfig("Invalid data length".to_string())
                        })?;
                    let classic_frame = socketcan::CanFrame::new(id, &data)
                        .ok_or_else(|| CanError::Io("Failed to create CAN frame".to_string()))?;
                    Ok(socketcan::CanAnyFrame::Classic(classic_frame))
                }
                CanFrame::Fd(f) => {
                    let id = Self::to_socketcan_id(f.id)?;
                    // CAN FD 数据长度必须是有效值: 0,1,2,3,4,5,6,7,8,12,16,20,24,32,48,64
                    let fd_data = &f.data;
                    let fd_flags = Self::to_socketcan_fd_flags(f.flags);
                    let fd_frame = socketcan::CanFdFrame::with_flags(id, fd_data, fd_flags)
                        .ok_or_else(|| {
                            CanError::InvalidConfig(format!(
                                "Invalid CAN FD frame: data length {}",
                                fd_data.len()
                            ))
                        })?;
                    Ok(socketcan::CanAnyFrame::Fd(fd_frame))
                }
            }
        }

        /// Convert CanId to socketcan Id.
        fn to_socketcan_id(id: CanId) -> Result<socketcan::Id, CanError> {
            match id {
                CanId::Standard(id) => socketcan::StandardId::new(id)
                    .ok_or_else(|| CanError::InvalidConfig(format!("Invalid CAN ID: {}", id))),
                CanId::Extended(id) => socketcan::ExtendedId::new(id).ok_or_else(|| {
                    CanError::InvalidConfig(format!("Invalid extended CAN ID: {}", id))
                }),
            }
        }

        /// Convert our FdFlags to socketcan FdFlags.
        fn to_socketcan_fd_flags(flags: FdFlags) -> socketcan::FdFlags {
            let mut fd_flags = socketcan::FdFlags::empty();
            if flags.brs {
                fd_flags |= socketcan::FdFlags::BRS;
            }
            if flags.esi {
                fd_flags |= socketcan::FdFlags::ESI;
            }
            fd_flags
        }

        /// Convert a socketcan CanAnyFrame to our CanFrame.
        fn from_socketcan_any_frame(frame: &socketcan::CanAnyFrame) -> CanFrame {
            match frame {
                socketcan::CanAnyFrame::Classic(f) => {
                    let id = Self::from_socketcan_id(f.id());
                    let data = f.data();
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
                socketcan::CanAnyFrame::Fd(f) => {
                    let id = Self::from_socketcan_id(f.id());
                    let data = f.data().to_vec();
                    let flags = Self::from_socketcan_fd_flags(f.flags());

                    CanFrame::Fd(FdFrame {
                        id,
                        data,
                        flags,
                        timestamp_us: None,
                    })
                }
            }
        }

        /// Convert socketcan Id to our CanId.
        fn from_socketcan_id(id: socketcan::Id) -> CanId {
            if id.is_extended() {
                CanId::Extended(id.as_raw())
            } else {
                CanId::Standard(id.as_raw() as u16)
            }
        }

        /// Convert socketcan FdFlags to our FdFlags.
        fn from_socketcan_fd_flags(flags: socketcan::FdFlags) -> FdFlags {
            FdFlags {
                brs: flags.contains(socketcan::FdFlags::BRS),
                esi: flags.contains(socketcan::FdFlags::ESI),
            }
        }
    }

    impl CanBus for SocketCanBus {
        fn send(&self, frame: &CanFrame) -> Result<(), CanError> {
            let can_frame = Self::to_socketcan_any_frame(frame)?;

            // Use block_on for synchronous send.
            // This works when called from non-async context (e.g. GUI thread).
            // If called from within tokio runtime, it uses the current handle.
            let rt = tokio::runtime::Handle::current();

            match &*self.socket.blocking_lock() {
                CanSocketVariant::Classic(socket) => {
                    if let socketcan::CanAnyFrame::Classic(f) = &can_frame {
                        rt.block_on(socket.write_frame(f.clone()))
                            .map_err(|e| CanError::Io(format!("SocketCAN send error: {}", e)))?;
                    } else {
                        return Err(CanError::Unsupported(
                            "Cannot send FD frame on classic socket".to_string(),
                        ));
                    }
                }
                CanSocketVariant::Fd(socket) => {
                    rt.block_on(socket.write_frame(&can_frame))
                        .map_err(|e| CanError::Io(format!("SocketCAN send error: {}", e)))?;
                }
            }
            Ok(())
        }

        fn recv(&self) -> impl Future<Output = Result<CanFrame, CanError>> + Send {
            async move {
                let mut socket = self.socket.lock().await;
                let frame =
                    match &mut *socket {
                        CanSocketVariant::Classic(socket) => {
                            let frame = socket.read_frame().await.map_err(|e| {
                                CanError::Io(format!("SocketCAN recv error: {}", e))
                            })?;
                            Self::from_socketcan_any_frame(&socketcan::CanAnyFrame::Classic(frame))
                        }
                        CanSocketVariant::Fd(socket) => {
                            let frame = socket.read_frame().await.map_err(|e| {
                                CanError::Io(format!("SocketCAN recv error: {}", e))
                            })?;
                            Self::from_socketcan_any_frame(&frame)
                        }
                    };
                Ok(frame)
            }
        }

        fn state(&self) -> CanState {
            // SocketCAN 状态可以通过 /sys/class/net/<iface>/operstate 查询
            // 可能的值: up, down, testing, dormant, notpresent, lowerlayerdown
            let operstate_path = format!("/sys/class/net/{}/operstate", self.interface);
            if let Ok(state_str) = std::fs::read_to_string(&operstate_path) {
                match state_str.trim() {
                    "up" => CanState::Active,
                    "down" => CanState::NotConnected,
                    "testing" => CanState::Warning,
                    "dormant" => CanState::Warning,
                    _ => CanState::Active,
                }
            } else {
                CanState::NotConnected
            }
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
        fn open(&self, channel: &str, config: &CanConfig) -> Result<Box<dyn CanBusDyn>, CanError> {
            Ok(Box::new(SocketCanBus::open(channel, config)?))
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
