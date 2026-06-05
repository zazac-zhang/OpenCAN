//! # opencan-can-traits
//!
//! Unified CAN bus trait abstraction for OpenCAN.
//!
//! This crate provides:
//! - [`CanBus`] trait for physical CAN bus I/O
//! - [`CanBusFactory`] trait for dynamic backend creation
//! - [`CanBusDyn`] trait for trait-object based usage in GUI
//! - CAN frame types (Classic and FD)

pub mod error;

use std::time::Instant;

/// CAN bus trait — physical layer interface.
///
/// Each hardware backend implements this trait.
/// For trait-object based usage in GUI, see [`CanBusDyn`].
pub trait CanBus: Send + Sync + 'static {
    fn send(&self, frame: &CanFrame) -> Result<(), error::CanError>;
    fn recv(&self) -> Result<CanFrame, error::CanError>;
    fn state(&self) -> CanState;
    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), error::CanError>;
}

/// Dynamic dispatch version of CanBus for GUI layer.
pub trait CanBusDyn: Send + Sync {
    fn send(&self, frame: &CanFrame) -> Result<(), error::CanError>;
    fn recv(&self) -> Result<CanFrame, error::CanError>;
    fn state(&self) -> CanState;
    fn set_bitrate(&self, bitrate: CanBitrate) -> Result<(), error::CanError>;
}

/// Factory for creating CAN bus instances dynamically.
pub trait CanBusFactory: Send + Sync {
    fn open(&self, channel: &str, config: &CanConfig) -> Result<Box<dyn CanBusDyn>, error::CanError>;
    fn name(&self) -> &str;
    fn available_channels(&self) -> Vec<String>;
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
            Self::Classic(f) => &f.data,
            Self::Fd(f) => &f.data,
        }
    }

    pub fn timestamp(&self) -> Option<Instant> {
        match self {
            Self::Classic(f) => f.timestamp,
            Self::Fd(f) => f.timestamp,
        }
    }
}

/// CAN 2.0 classic frame.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassicFrame {
    pub id: CanId,
    pub data: Vec<u8>,        // max 8 bytes
    pub timestamp: Option<Instant>,
}

impl ClassicFrame {
    pub fn new(id: CanId, data: Vec<u8>) -> Self {
        Self { id, data, timestamp: None }
    }

    pub fn with_timestamp(mut self, ts: Instant) -> Self {
        self.timestamp = Some(ts);
        self
    }
}

/// CAN FD frame.
#[derive(Debug, Clone, PartialEq)]
pub struct FdFrame {
    pub id: CanId,
    pub data: Vec<u8>,        // max 64 bytes
    pub flags: FdFlags,
    pub timestamp: Option<Instant>,
}

/// CAN FD flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FdFlags {
    /// Bit Rate Switch — use higher data rate for data phase.
    pub brs: bool,
    /// Error State Indicator — sender is error active.
    pub esi: bool,
}

impl Default for FdFlags {
    fn default() -> Self {
        Self { brs: false, esi: false }
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
        Self { nominal, data: None }
    }

    pub fn fd(nominal: u32, data: u32) -> Self {
        Self { nominal, data: Some(data) }
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
