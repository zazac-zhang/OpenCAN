//! # opencan-can-traits
//!
//! CAN bus trait abstraction and hardware backends for OpenCAN.
//!
//! This crate provides:
//! - [`CanBus`] trait for physical CAN bus I/O (trait-object safe)
//! - [`CanBusFactory`] trait for dynamic backend creation
//! - CAN frame types (Classic and FD)
//! - Hardware backend implementations (feature-gated)
//!
//! ## Backends
//!
//! | Feature     | Backend    | Platform |
//! |-------------|------------|----------|
//! | `socketcan` | SocketCAN  | Linux    |
//! | `kvaser`    | Kvaser     | Win/Linux (FFI) |
//! | `pcan`      | PCAN       | Win/Linux (FFI) |
//! | `zlg`       | ZLG        | Win/Linux (FFI) |

pub mod error;

pub mod monitor;

pub mod recovery;

#[cfg(feature = "socketcan")]
pub mod socketcan;

#[cfg(feature = "kvaser")]
pub mod kvaser;

#[cfg(feature = "pcan")]
pub mod pcan;

#[cfg(feature = "zlg")]
pub mod zlg;

use std::future::Future;

/// CAN bus trait — physical layer interface.
///
/// **Trait-object safe**: all methods can be called via `Box<dyn CanBus>`.
/// Construction is handled by [`CanBusFactory`], not this trait.
pub trait CanBus: Send + Sync + 'static {
    /// Send a CAN frame.
    fn send(&self, frame: &CanFrame) -> Result<(), error::CanError>;

    /// Receive a CAN frame (async).
    fn recv(&self) -> impl Future<Output = Result<CanFrame, error::CanError>> + Send;

    /// Get current bus state.
    fn state(&self) -> CanState;

    /// Set bus bitrate (if supported).
    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), error::CanError>;
}

/// Factory for creating CAN bus instances dynamically.
///
/// Each hardware backend implements this trait.
/// GUI uses `Box<dyn CanBusFactory>` to support runtime backend selection.
pub trait CanBusFactory: Send + Sync + 'static {
    /// Open a CAN bus channel.
    fn open(
        &self,
        channel: &str,
        config: &CanConfig,
    ) -> Result<Box<dyn CanBusDyn>, error::CanError>;

    /// Backend name (e.g. "SocketCAN", "Kvaser", "PCAN").
    fn name(&self) -> &str;

    /// List available channels.
    fn available_channels(&self) -> Vec<String>;
}

/// Dynamic dispatch version of CanBus.
///
/// This is the trait object that GUI layer uses.
/// Blanket impl provided for all `T: CanBus`.
pub trait CanBusDyn: Send + Sync {
    fn send(&self, frame: &CanFrame) -> Result<(), error::CanError>;
    fn recv(
        &self,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<CanFrame, error::CanError>> + Send + '_>>;
    fn state(&self) -> CanState;
    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), error::CanError>;
}

/// Blanket impl: any `T: CanBus` can be used as `CanBusDyn`.
impl<T: CanBus> CanBusDyn for T {
    fn send(&self, frame: &CanFrame) -> Result<(), error::CanError> {
        CanBus::send(self, frame)
    }

    fn recv(
        &self,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<CanFrame, error::CanError>> + Send + '_>>
    {
        Box::pin(CanBus::recv(self))
    }

    fn state(&self) -> CanState {
        CanBus::state(self)
    }

    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), error::CanError> {
        CanBus::set_bitrate(self, bitrate)
    }
}

/// CAN bus state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanState {
    /// Bus is active and operational.
    Active,
    /// Bus is in warning state (error counters elevated).
    Warning,
    /// Bus is in error passive state.
    ErrorPassive,
    /// Bus is off (error counters exceeded limits).
    BusOff,
    /// Not connected.
    NotConnected,
}

/// CAN frame — supports both Classic (2.0) and FD.
#[derive(Debug, Clone, PartialEq)]
pub enum CanFrame {
    /// CAN 2.0 classic frame (max 8 bytes).
    Classic(ClassicFrame),
    /// CAN FD frame (max 64 bytes).
    Fd(FdFrame),
}

impl CanFrame {
    pub fn id(&self) -> CanId {
        match self {
            Self::Classic(f) => f.id,
            Self::Fd(f) => f.id,
        }
    }

    pub fn data(&self) -> &[u8] {
        match self {
            Self::Classic(f) => &f.data[..f.len as usize],
            Self::Fd(f) => &f.data,
        }
    }

    pub fn timestamp_us(&self) -> Option<u64> {
        match self {
            Self::Classic(f) => f.timestamp_us,
            Self::Fd(f) => f.timestamp_us,
        }
    }

    /// Check if this is a CAN FD frame.
    pub fn is_fd(&self) -> bool {
        matches!(self, Self::Fd(_))
    }

    /// Create a new classic CAN frame.
    pub fn new_classic(id: CanId, data: &[u8]) -> Self {
        Self::Classic(ClassicFrame::new(id, data))
    }

    /// Create a new CAN FD frame.
    pub fn new_fd(id: CanId, data: &[u8]) -> Self {
        Self::Fd(FdFrame::new(id, data))
    }

    /// Create a new CAN FD frame with flags.
    pub fn new_fd_with_flags(id: CanId, data: &[u8], flags: FdFlags) -> Self {
        Self::Fd(FdFrame::with_flags(id, data, flags))
    }
}

/// CAN 2.0 classic frame.
///
/// Uses fixed `[u8; 8]` + `len` instead of `Vec<u8>` to avoid heap allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassicFrame {
    pub id: CanId,
    pub data: [u8; 8],
    pub len: u8,
    /// Timestamp in microseconds (from hardware or software clock).
    /// Format is backend-dependent. None if not available.
    pub timestamp_us: Option<u64>,
}

impl ClassicFrame {
    pub fn new(id: CanId, data: &[u8]) -> Self {
        let len = data.len().min(8) as u8;
        let mut buf = [0u8; 8];
        buf[..len as usize].copy_from_slice(&data[..len as usize]);
        Self {
            id,
            data: buf,
            len,
            timestamp_us: None,
        }
    }

    pub fn with_timestamp(mut self, ts_us: u64) -> Self {
        self.timestamp_us = Some(ts_us);
        self
    }
}

/// CAN FD frame.
#[derive(Debug, Clone, PartialEq)]
pub struct FdFrame {
    pub id: CanId,
    pub data: Vec<u8>,
    pub flags: FdFlags,
    pub timestamp_us: Option<u64>,
}

impl FdFrame {
    /// Create a new CAN FD frame.
    pub fn new(id: CanId, data: &[u8]) -> Self {
        Self {
            id,
            data: data.to_vec(),
            flags: FdFlags::default(),
            timestamp_us: None,
        }
    }

    /// Create a new CAN FD frame with flags.
    pub fn with_flags(id: CanId, data: &[u8], flags: FdFlags) -> Self {
        Self {
            id,
            data: data.to_vec(),
            flags,
            timestamp_us: None,
        }
    }

    /// Set timestamp.
    pub fn with_timestamp(mut self, ts_us: u64) -> Self {
        self.timestamp_us = Some(ts_us);
        self
    }
}

/// CAN FD flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FdFlags {
    /// Bit Rate Switch — use higher data rate for data phase.
    pub brs: bool,
    /// Error State Indicator — sender is error active.
    pub esi: bool,
}

impl FdFlags {
    /// Create new FD flags.
    pub fn new(brs: bool, esi: bool) -> Self {
        Self { brs, esi }
    }
}

/// CAN Identifier — standard (11-bit) or extended (29-bit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanId {
    /// Standard 11-bit identifier (0x000 - 0x7FF).
    Standard(u16),
    /// Extended 29-bit identifier (0x00000000 - 0x1FFFFFFF).
    Extended(u32),
}

impl CanId {
    pub fn raw(&self) -> u32 {
        match self {
            Self::Standard(id) => *id as u32,
            Self::Extended(id) => *id,
        }
    }

    pub fn is_extended(&self) -> bool {
        matches!(self, Self::Extended(_))
    }
}

/// CAN bus bitrate configuration.
#[derive(Debug, Clone, Copy)]
pub struct CanBitrate {
    /// Nominal (arbitration) bitrate, e.g. 500_000 for 500 kbit/s.
    pub nominal: u32,
    /// Data bitrate for CAN FD, e.g. 2_000_000 for 2 Mbit/s.
    pub data: Option<u32>,
}

impl CanBitrate {
    pub fn new(nominal: u32) -> Self {
        Self {
            nominal,
            data: None,
        }
    }

    pub fn fd(nominal: u32, data: u32) -> Self {
        Self {
            nominal,
            data: Some(data),
        }
    }
}

/// Configuration for opening a CAN bus.
#[derive(Debug, Clone)]
pub struct CanConfig {
    pub bitrate: CanBitrate,
    pub listen_only: bool,
    pub fd: bool,
}

impl Default for CanConfig {
    fn default() -> Self {
        Self {
            bitrate: CanBitrate::new(500_000),
            listen_only: false,
            fd: false,
        }
    }
}

/// 设备热插拔事件
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceEvent {
    /// 设备已连接
    Connected {
        /// 后端名称 (如 "SocketCAN", "Kvaser")
        backend: String,
        /// 通道名称
        channel: String,
    },
    /// 设备已断开
    Disconnected {
        /// 后端名称
        backend: String,
        /// 通道名称
        channel: String,
    },
}

/// 设备监控器 trait
///
/// 用于监听 CAN 设备的连接和断开事件。
/// GUI 可以使用此 trait 来动态更新可用设备列表。
pub trait DeviceMonitor: Send + Sync + 'static {
    /// 启动设备监控
    fn start(&mut self) -> Result<(), error::CanError>;

    /// 停止设备监控
    fn stop(&mut self) -> Result<(), error::CanError>;

    /// 检查是否有新的设备事件
    fn poll_event(&mut self) -> Option<DeviceEvent>;

    /// 获取当前所有可用设备
    fn available_devices(&self) -> Vec<DeviceInfo>;
}

/// 设备信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// 后端名称
    pub backend: String,
    /// 通道名称
    pub channel: String,
    /// 设备描述
    pub description: String,
    /// 是否在线
    pub online: bool,
}
