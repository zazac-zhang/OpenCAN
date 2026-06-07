//! CANOpen frame types and encoding/decoding.
//!
//! CANOpen protocol uses CAN 2.0 standard frames with 8-byte data.
//! Each frame has a COB-ID (Communication Object Identifier) that encodes
//! the Function Code and Node ID.

use std::time::Instant;

/// CANOpen protocol frame — fixed 8 bytes data (CAN 2.0).
#[derive(Debug, Clone, PartialEq)]
pub struct CanOpenFrame {
    pub cob_id: u16,
    pub data: [u8; 8],
    pub timestamp: Option<Instant>,
}

impl CanOpenFrame {
    pub fn new(cob_id: u16, data: [u8; 8]) -> Self {
        Self {
            cob_id,
            data,
            timestamp: None,
        }
    }

    pub fn with_timestamp(mut self, ts: Instant) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// Get the raw COB-ID (11-bit).
    pub fn raw_cob_id(&self) -> u16 {
        self.cob_id & 0x7FF
    }
}

/// COB-ID structure: encodes Function Code + Node ID.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CobId {
    pub function: FunctionCode,
    pub node_id: u8,
}

impl CobId {
    pub fn new(function: FunctionCode, node_id: u8) -> Self {
        Self { function, node_id }
    }

    /// Encode to raw COB-ID (11-bit).
    pub fn to_u16(&self) -> u16 {
        (self.function as u16) + self.node_id as u16
    }

    /// Decode from raw COB-ID.
    pub fn from_u16(cob_id: u16) -> Option<Self> {
        let node_id = (cob_id & 0x7F) as u8;
        let fc_raw = cob_id & 0x780;
        let function = FunctionCode::from_u16(fc_raw)?;
        Some(Self { function, node_id })
    }
}

/// CANOpen Function Codes.
///
/// Note: Sync (0x080) and Emergency (0x080) share the same Function Code.
/// They are distinguished by Node ID:
/// - Sync uses Node ID = 0 (broadcast)
/// - Emergency uses Node ID = the sending node's ID
///
/// Since they share the same FC value, we represent them as a single variant
/// and use helper methods to distinguish them based on context (node_id).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
#[non_exhaustive]
pub enum FunctionCode {
    Nmt = 0x000,
    SyncOrEmergency = 0x080, // Sync if node_id=0, Emergency otherwise
    Timestamp = 0x100,
    Tpdo1 = 0x180,
    Rpdo1 = 0x200,
    Tpdo2 = 0x280,
    Rpdo2 = 0x300,
    Tpdo3 = 0x380,
    Rpdo3 = 0x400,
    Tpdo4 = 0x480,
    Rpdo4 = 0x500,
    SdoServer = 0x580, // SDO response (server → client)
    SdoClient = 0x600, // SDO request (client → server)
    Heartbeat = 0x700,
}

impl FunctionCode {
    /// Check if this is a Sync frame (node_id = 0).
    pub fn is_sync(&self, node_id: u8) -> bool {
        *self == Self::SyncOrEmergency && node_id == 0
    }

    /// Check if this is an Emergency frame (node_id != 0).
    pub fn is_emergency(&self, node_id: u8) -> bool {
        *self == Self::SyncOrEmergency && node_id != 0
    }
}

impl FunctionCode {
    /// Convert from raw function code value.
    pub fn from_u16(val: u16) -> Option<Self> {
        match val {
            0x000 => Some(Self::Nmt),
            0x080 => Some(Self::SyncOrEmergency),
            0x100 => Some(Self::Timestamp),
            0x180 => Some(Self::Tpdo1),
            0x200 => Some(Self::Rpdo1),
            0x280 => Some(Self::Tpdo2),
            0x300 => Some(Self::Rpdo2),
            0x380 => Some(Self::Tpdo3),
            0x400 => Some(Self::Rpdo3),
            0x480 => Some(Self::Tpdo4),
            0x500 => Some(Self::Rpdo4),
            0x580 => Some(Self::SdoServer),
            0x600 => Some(Self::SdoClient),
            0x700 => Some(Self::Heartbeat),
            _ => None,
        }
    }

    /// Get the base COB-ID for this function code.
    pub fn base(&self) -> u16 {
        *self as u16
    }
}

// === NMT Frame ===

/// NMT Command Specifiers (DS301 Table 14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NmtCommandSpecifier {
    EnterOperational = 0x01,
    EnterStopped = 0x02,
    EnterPreOperational = 0x80,
    ResetNode = 0x81,
    ResetCommunication = 0x82,
}

impl NmtCommandSpecifier {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x01 => Some(Self::EnterOperational),
            0x02 => Some(Self::EnterStopped),
            0x80 => Some(Self::EnterPreOperational),
            0x81 => Some(Self::ResetNode),
            0x82 => Some(Self::ResetCommunication),
            _ => None,
        }
    }
}

impl std::fmt::Display for NmtCommandSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnterOperational => write!(f, "Start"),
            Self::EnterStopped => write!(f, "Stop"),
            Self::EnterPreOperational => write!(f, "PreOp"),
            Self::ResetNode => write!(f, "ResetNode"),
            Self::ResetCommunication => write!(f, "ResetComm"),
        }
    }
}

/// NMT Command frame.
#[derive(Debug, Clone, PartialEq)]
pub struct NmtCommand {
    pub command: NmtCommandSpecifier,
    pub node_id: u8, // 0 = broadcast to all nodes
}

impl NmtCommand {
    pub fn encode(&self) -> CanOpenFrame {
        CanOpenFrame::new(0x000, [self.command as u8, self.node_id, 0, 0, 0, 0, 0, 0])
    }

    pub fn decode(frame: &CanOpenFrame) -> Option<Self> {
        if frame.cob_id != 0 {
            return None;
        }
        let command = NmtCommandSpecifier::from_u8(frame.data[0])?;
        Some(Self {
            command,
            node_id: frame.data[1],
        })
    }
}

// === NMT State ===

/// NMT states (DS301 Figure 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum NmtState {
    BootUp = 0x00,
    Stopped = 0x04,
    Operational = 0x05,
    PreOperational = 0x7F,
}

impl NmtState {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(Self::BootUp),
            0x04 => Some(Self::Stopped),
            0x05 => Some(Self::Operational),
            0x7F => Some(Self::PreOperational),
            _ => None,
        }
    }
}

// === Heartbeat Frame ===

/// Heartbeat frame (COB-ID = 0x700 + node_id).
#[derive(Debug, Clone, PartialEq)]
pub struct HeartbeatFrame {
    pub node_id: u8,
    pub state: NmtState,
}

impl HeartbeatFrame {
    pub fn encode(&self) -> CanOpenFrame {
        CanOpenFrame::new(
            0x700 + self.node_id as u16,
            [self.state as u8, 0, 0, 0, 0, 0, 0, 0],
        )
    }

    pub fn decode(frame: &CanOpenFrame) -> Option<Self> {
        if frame.cob_id < 0x700 || frame.cob_id > 0x77F {
            return None;
        }
        let node_id = (frame.cob_id - 0x700) as u8;
        let state = NmtState::from_u8(frame.data[0])?;
        Some(Self { node_id, state })
    }
}

// === Emergency Frame ===

/// Emergency frame (COB-ID = 0x080 + node_id).
#[derive(Debug, Clone, PartialEq)]
pub struct EmergencyFrame {
    pub node_id: u8,
    pub error_code: u16,
    pub error_register: u8,
    pub data: [u8; 5],
}

impl EmergencyFrame {
    pub fn encode(&self) -> CanOpenFrame {
        let mut data = [0u8; 8];
        data[0..2].copy_from_slice(&self.error_code.to_le_bytes());
        data[2] = self.error_register;
        data[3..8].copy_from_slice(&self.data);
        CanOpenFrame::new(0x080 + self.node_id as u16, data)
    }

    pub fn decode(frame: &CanOpenFrame) -> Option<Self> {
        if frame.cob_id < 0x080 || frame.cob_id > 0x0FF {
            return None;
        }
        let node_id = (frame.cob_id - 0x080) as u8;
        let error_code = u16::from_le_bytes([frame.data[0], frame.data[1]]);
        let error_register = frame.data[2];
        let mut data = [0u8; 5];
        data.copy_from_slice(&frame.data[3..8]);
        Some(Self {
            node_id,
            error_code,
            error_register,
            data,
        })
    }
}

// === Sync Frame ===

/// SYNC frame (COB-ID = 0x080).
///
/// SYNC frames are used to synchronize PDO transmissions.
/// An optional counter byte (data[0]) can be used to detect missed SYNCs.
/// Counter values 1-240 are valid per DS301.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncFrame {
    /// Optional counter value (1-240). None means no counter.
    pub counter: Option<u8>,
}

impl SyncFrame {
    /// Create a SYNC frame without a counter.
    pub fn new() -> Self {
        Self { counter: None }
    }

    /// Create a SYNC frame with a counter value.
    pub fn with_counter(counter: u8) -> Self {
        Self {
            counter: Some(counter),
        }
    }

    /// Encode as a CanOpenFrame.
    pub fn encode(&self) -> CanOpenFrame {
        let mut data = [0u8; 8];
        if let Some(c) = self.counter {
            data[0] = c;
        }
        CanOpenFrame::new(0x080, data)
    }

    /// Decode from a CanOpenFrame.
    /// Returns None if the frame is not a SYNC (COB-ID != 0x080).
    pub fn decode(frame: &CanOpenFrame) -> Option<Self> {
        if frame.cob_id != 0x080 {
            return None;
        }
        // Per DS301: data[0] = 0 means no counter, 1-240 is counter value
        let counter = if frame.data[0] == 0 {
            None
        } else {
            Some(frame.data[0])
        };
        Some(Self { counter })
    }
}

impl Default for SyncFrame {
    fn default() -> Self {
        Self::new()
    }
}

// === Timestamp Frame ===

/// TIME_STAMP frame (COB-ID = 0x100).
///
/// Transmits the CANOpen TIME_OF_DAY object as a 6-byte payload:
/// - bytes[0..4]: milliseconds since midnight (u32 LE)
/// - bytes[4..6]: days since Jan 1, 1984 (u16 LE)
///
/// DS301 Section 7.2.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimestampFrame {
    /// Milliseconds since midnight (0..=86399999).
    pub ms_of_day: u32,
    /// Days since January 1, 1984.
    pub days: u16,
}

impl TimestampFrame {
    /// Create a new timestamp frame.
    pub fn new(ms_of_day: u32, days: u16) -> Self {
        Self { ms_of_day, days }
    }

    /// Encode as a CanOpenFrame.
    pub fn encode(&self) -> CanOpenFrame {
        let mut data = [0u8; 8];
        data[0..4].copy_from_slice(&self.ms_of_day.to_le_bytes());
        data[4..6].copy_from_slice(&self.days.to_le_bytes());
        CanOpenFrame::new(0x100, data)
    }

    /// Decode from a CanOpenFrame.
    /// Returns None if the frame is not a TIME_STAMP (COB-ID != 0x100).
    pub fn decode(frame: &CanOpenFrame) -> Option<Self> {
        if frame.cob_id != 0x100 {
            return None;
        }
        let ms_of_day =
            u32::from_le_bytes([frame.data[0], frame.data[1], frame.data[2], frame.data[3]]);
        let days = u16::from_le_bytes([frame.data[4], frame.data[5]]);
        Some(Self { ms_of_day, days })
    }

    /// Convert to total milliseconds since Jan 1, 1984 00:00:00.
    pub fn to_total_ms(&self) -> u64 {
        (self.days as u64) * 86_400_000 + self.ms_of_day as u64
    }
}

// === SDO Frame ===

/// SDO command specifier (DS301 Table 33-34).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SdoCommandSpecifier {
    DownloadInitiated,
    DownloadSegment,
    UploadInitiated,
    UploadSegment,
    Abort,
}

/// SDO request frame.
#[derive(Debug, Clone, PartialEq)]
pub struct SdoRequest {
    pub node_id: u8,
    pub index: u16,
    pub subindex: u8,
    pub data: SdoData,
}

/// SDO data payload.
#[derive(Debug, Clone, PartialEq)]
pub enum SdoData {
    /// Expedited download (≤ 4 bytes).
    Expedited { data: [u8; 4], size: Option<u8> },
    /// Initiated segmented download.
    SegmentedInitiated { size: u32 },
    /// Segment download.
    Segment {
        toggle: bool,
        last: bool,
        data: [u8; 7],
        size: Option<u8>,
    },
    /// Upload request (no data).
    UploadRequest,
    /// Abort.
    Abort { code: u32 },
    /// Initiate block upload request (client → server, cs=5).
    BlockUploadRequest {
        /// Enable CRC in block transfer.
        crc_enabled: bool,
    },
    /// Start block upload (client → server, cs=6).
    BlockUploadStart,
    /// Block segment (cs=0, sequence number in data[0]).
    BlockSegment {
        /// Sequence number (1-127).
        seq: u8,
        /// Segment data (up to 7 bytes).
        data: [u8; 7],
    },
    /// End block upload/download (cs=1 or cs=2).
    BlockEnd {
        /// Number of unused bytes in last segment.
        n: u8,
        /// CRC (if used).
        crc: Option<u16>,
    },
    /// Initiate block download request (client → server, cs=6).
    BlockDownloadRequest {
        /// Enable CRC in block transfer.
        crc_enabled: bool,
        /// Total data size (if known).
        size: Option<u32>,
    },
}

impl SdoRequest {
    /// Encode as SDO client request (COB-ID = 0x600 + node_id).
    pub fn encode(&self) -> CanOpenFrame {
        let mut data = [0u8; 8];

        match &self.data {
            SdoData::Expedited { data: d, size } => {
                let mut cmd: u8 = 0x20; // initiate download
                cmd |= 0x02; // expedited
                if let Some(s) = size {
                    cmd |= 0x01; // size indicated
                    cmd |= (4 - s) << 2;
                }
                data[0] = cmd;
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
                data[4..8].copy_from_slice(d);
            }
            SdoData::UploadRequest => {
                data[0] = 0x40; // initiate upload
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
            }
            SdoData::Segment {
                toggle,
                last,
                data: d,
                size,
            } => {
                let mut cmd: u8 = 0x00;
                if *toggle {
                    cmd |= 0x10;
                }
                if *last {
                    cmd |= 0x01;
                }
                if let Some(s) = size {
                    let n = 7 - s;
                    cmd |= (n & 0x07) << 1;
                }
                data[0] = cmd;
                data[1..8].copy_from_slice(d);
            }
            SdoData::Abort { code } => {
                data[0] = 0x80; // abort
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
                data[4..8].copy_from_slice(&code.to_le_bytes());
            }
            SdoData::SegmentedInitiated { size } => {
                let mut cmd: u8 = 0x21; // initiate segmented download
                cmd |= 0x01; // size indicated
                data[0] = cmd;
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
                data[4..8].copy_from_slice(&size.to_le_bytes());
            }
            SdoData::BlockUploadRequest { crc_enabled } => {
                let mut cmd: u8 = 0xA0; // initiate block upload (cs=5)
                if *crc_enabled {
                    cmd |= 0x04; // CRC enabled
                }
                data[0] = cmd;
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
            }
            SdoData::BlockUploadStart => {
                data[0] = 0xC0; // start block upload (cs=6)
            }
            SdoData::BlockSegment { seq, data: d } => {
                data[0] = *seq & 0x7F; // sequence number (1-127)
                data[1..8].copy_from_slice(d);
            }
            SdoData::BlockEnd { n, crc } => {
                let mut cmd: u8 = 0xC1; // end block (cs=1)
                if crc.is_some() {
                    cmd |= 0x04; // CRC used
                }
                cmd |= (n & 0x07) << 1;
                data[0] = cmd;
                if let Some(crc_val) = crc {
                    data[1..3].copy_from_slice(&crc_val.to_le_bytes());
                }
            }
            SdoData::BlockDownloadRequest { crc_enabled, size } => {
                let mut cmd: u8 = 0xC0; // initiate block download (cs=6)
                if *crc_enabled {
                    cmd |= 0x04; // CRC enabled
                }
                if let Some(s) = size {
                    cmd |= 0x02; // size indicated
                    data[0] = cmd;
                    data[1..3].copy_from_slice(&self.index.to_le_bytes());
                    data[3] = self.subindex;
                    data[4..8].copy_from_slice(&s.to_le_bytes());
                } else {
                    data[0] = cmd;
                    data[1..3].copy_from_slice(&self.index.to_le_bytes());
                    data[3] = self.subindex;
                }
            }
        }

        CanOpenFrame::new(0x600 + self.node_id as u16, data)
    }

    /// Decode from SDO client request frame (COB-ID = 0x600 + node_id).
    pub fn decode(frame: &CanOpenFrame) -> Option<Self> {
        if frame.cob_id < 0x600 || frame.cob_id > 0x67F {
            return None;
        }
        let node_id = (frame.cob_id - 0x600) as u8;
        let cmd = frame.data[0];
        let index = u16::from_le_bytes([frame.data[1], frame.data[2]]);
        let subindex = frame.data[3];

        let data = if cmd == 0x80 {
            // Abort
            let code =
                u32::from_le_bytes([frame.data[4], frame.data[5], frame.data[6], frame.data[7]]);
            SdoData::Abort { code }
        } else if cmd & 0xE0 == 0x20 {
            // Initiate download (cs=1)
            let expedited = cmd & 0x02 != 0;
            let size_indicated = cmd & 0x01 != 0;

            if expedited {
                let size = if size_indicated {
                    Some(4 - ((cmd >> 2) & 0x03))
                } else {
                    None
                };
                let mut d = [0u8; 4];
                d.copy_from_slice(&frame.data[4..8]);
                SdoData::Expedited { data: d, size }
            } else if size_indicated {
                // Segmented download initiated
                let size = u32::from_le_bytes([
                    frame.data[4],
                    frame.data[5],
                    frame.data[6],
                    frame.data[7],
                ]);
                SdoData::SegmentedInitiated { size }
            } else {
                SdoData::SegmentedInitiated { size: 0 }
            }
        } else if cmd & 0xE0 == 0x40 {
            // Initiate upload (cs=2)
            SdoData::UploadRequest
        } else if cmd & 0xE0 == 0x00 {
            // Segment (cs=0 — used for both upload segment request and download segment)
            let toggle = cmd & 0x10 != 0;
            let last = cmd & 0x01 != 0;
            let size = if cmd & 0x0E != 0 {
                Some(7 - ((cmd >> 1) & 0x07))
            } else {
                None
            };
            let mut d = [0u8; 7];
            d.copy_from_slice(&frame.data[1..8]);
            SdoData::Segment {
                toggle,
                last,
                data: d,
                size,
            }
        } else if cmd & 0xE0 == 0xA0 {
            // Block upload request (cs=5)
            let crc_enabled = cmd & 0x04 != 0;
            SdoData::BlockUploadRequest { crc_enabled }
        } else if cmd == 0xC0 {
            // Start block upload (cs=6)
            SdoData::BlockUploadStart
        } else if cmd & 0xE0 == 0xC0 {
            // Block download request (cs=6) or block end (cs=1)
            let cs = (cmd >> 5) & 0x07;
            match cs {
                6 => {
                    // Initiate block download
                    let crc_enabled = cmd & 0x04 != 0;
                    let size_indicated = cmd & 0x02 != 0;
                    let size = if size_indicated {
                        Some(u32::from_le_bytes([
                            frame.data[4],
                            frame.data[5],
                            frame.data[6],
                            frame.data[7],
                        ]))
                    } else {
                        None
                    };
                    SdoData::BlockDownloadRequest { crc_enabled, size }
                }
                1 => {
                    // Block end
                    let n = (cmd >> 1) & 0x07;
                    let crc_used = cmd & 0x04 != 0;
                    let crc = if crc_used {
                        Some(u16::from_le_bytes([frame.data[1], frame.data[2]]))
                    } else {
                        None
                    };
                    SdoData::BlockEnd { n, crc }
                }
                _ => return None,
            }
        } else {
            return None;
        };

        Some(Self {
            node_id,
            index,
            subindex,
            data,
        })
    }
}

/// SDO response frame.
#[derive(Debug, Clone, PartialEq)]
pub struct SdoResponse {
    pub node_id: u8,
    pub index: u16,
    pub subindex: u8,
    pub data: SdoResponseData,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SdoResponseData {
    /// Expedited upload response (≤ 4 bytes).
    Expedited { data: [u8; 4], size: Option<u8> },
    /// Segmented upload initiated response.
    SegmentedInitiated { size: u32 },
    /// Upload segment response.
    Segment {
        toggle: bool,
        last: bool,
        data: [u8; 7],
        size: Option<u8>,
    },
    /// Download confirmed.
    DownloadConfirmed,
    /// Abort.
    Abort { code: u32 },
    /// Block upload initiated response (server → client, cs=5).
    BlockUploadInitiated {
        /// Server's proposed block size (1-127).
        block_size: u8,
        /// CRC supported by server.
        crc_supported: bool,
        /// Total data size (if indicated).
        size: Option<u32>,
    },
    /// Block segment (cs=0, sequence number in cmd).
    BlockSegment {
        /// Sequence number (1-127).
        seq: u8,
        /// Segment data (up to 7 bytes).
        data: [u8; 7],
    },
    /// End block upload/download (cs=1 or cs=2).
    BlockEnd {
        /// Number of unused bytes in last segment.
        n: u8,
        /// CRC (if used).
        crc: Option<u16>,
    },
    /// Block download confirmed (server → client, cs=4).
    BlockDownloadConfirmed {
        /// CRC supported by server.
        crc_supported: bool,
    },
}

impl SdoResponse {
    /// Decode from SDO server response frame (COB-ID = 0x580 + node_id).
    pub fn decode(frame: &CanOpenFrame) -> Option<Self> {
        if frame.cob_id < 0x580 || frame.cob_id > 0x5FF {
            return None;
        }
        let node_id = (frame.cob_id - 0x580) as u8;
        let cmd = frame.data[0];
        let index = u16::from_le_bytes([frame.data[1], frame.data[2]]);
        let subindex = frame.data[3];

        let data = if cmd == 0x80 {
            // Could be Abort or Block Download Confirmed (cs=4)
            // Distinguish by checking if abort code is non-zero
            let code =
                u32::from_le_bytes([frame.data[4], frame.data[5], frame.data[6], frame.data[7]]);
            if code != 0 {
                SdoResponseData::Abort { code }
            } else {
                // Block download confirmed
                let crc_supported = cmd & 0x04 != 0;
                SdoResponseData::BlockDownloadConfirmed { crc_supported }
            }
        } else if cmd == 0xA0 || cmd == 0xA4 || cmd == 0xA2 || cmd == 0xA6 {
            // Initiate block upload response (cs=5)
            let crc_supported = cmd & 0x04 != 0;
            let size_indicated = cmd & 0x02 != 0;
            let block_size = frame.data[4];
            let size = if size_indicated {
                Some(u32::from_le_bytes([
                    frame.data[5],
                    frame.data[6],
                    frame.data[7],
                    0,
                ]))
            } else {
                None
            };
            SdoResponseData::BlockUploadInitiated {
                block_size,
                crc_supported,
                size,
            }
        } else if cmd & 0xE0 == 0x40 {
            // Initiate upload response (cs=2)
            let expedited = cmd & 0x02 != 0;
            let size_indicated = cmd & 0x01 != 0;

            if expedited {
                let size = if size_indicated {
                    Some(4 - ((cmd >> 2) & 0x03))
                } else {
                    None
                };
                let mut d = [0u8; 4];
                d.copy_from_slice(&frame.data[4..8]);
                SdoResponseData::Expedited { data: d, size }
            } else if size_indicated {
                let size = u32::from_le_bytes([
                    frame.data[4],
                    frame.data[5],
                    frame.data[6],
                    frame.data[7],
                ]);
                SdoResponseData::SegmentedInitiated { size }
            } else {
                SdoResponseData::SegmentedInitiated { size: 0 }
            }
        } else if cmd & 0xE0 == 0x20 || cmd & 0xE0 == 0x60 {
            // Download confirmed (cs=1 or cs=3)
            SdoResponseData::DownloadConfirmed
        } else if cmd & 0xE0 == 0x00 {
            // Segment download response (cs=0)
            let toggle = cmd & 0x10 != 0;
            let last = cmd & 0x01 != 0;
            let size = if cmd & 0x0E != 0 {
                Some(7 - ((cmd >> 1) & 0x07))
            } else {
                None
            };
            let mut d = [0u8; 7];
            d.copy_from_slice(&frame.data[1..8]);
            SdoResponseData::Segment {
                toggle,
                last,
                data: d,
                size,
            }
        } else if cmd & 0xE0 == 0xA0 || cmd & 0xE0 == 0xC0 {
            // Block end (cs=5 from server or cs=6 from client)
            let n = (cmd >> 1) & 0x07;
            let crc_used = cmd & 0x04 != 0;
            let crc = if crc_used {
                Some(u16::from_le_bytes([frame.data[1], frame.data[2]]))
            } else {
                None
            };
            SdoResponseData::BlockEnd { n, crc }
        } else {
            return None;
        };

        Some(Self {
            node_id,
            index,
            subindex,
            data,
        })
    }

    /// Encode as SDO server response frame (COB-ID = 0x580 + node_id).
    pub fn encode(&self) -> CanOpenFrame {
        let mut data = [0u8; 8];

        match &self.data {
            SdoResponseData::Expedited { data: d, size } => {
                let mut cmd: u8 = 0x40; // initiate upload response (cs=2)
                cmd |= 0x02; // expedited
                if let Some(s) = size {
                    cmd |= 0x01; // size indicated
                    cmd |= (4 - s) << 2;
                }
                data[0] = cmd;
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
                data[4..8].copy_from_slice(d);
            }
            SdoResponseData::SegmentedInitiated { size } => {
                let mut cmd: u8 = 0x41; // initiate upload response, not expedited
                if *size > 0 {
                    cmd |= 0x01; // size indicated
                }
                data[0] = cmd;
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
                data[4..8].copy_from_slice(&size.to_le_bytes());
            }
            SdoResponseData::Segment {
                toggle,
                last,
                data: d,
                size,
            } => {
                let mut cmd: u8 = 0x00;
                if *toggle {
                    cmd |= 0x10;
                }
                if *last {
                    cmd |= 0x01;
                }
                // n = number of bytes that do NOT contain data
                if let Some(s) = size {
                    let n = 7 - s;
                    cmd |= (n & 0x07) << 1;
                }
                data[0] = cmd;
                data[1..8].copy_from_slice(d);
            }
            SdoResponseData::DownloadConfirmed => {
                data[0] = 0x20; // download confirmed (cs=1)
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
            }
            SdoResponseData::Abort { code } => {
                data[0] = 0x80;
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
                data[4..8].copy_from_slice(&code.to_le_bytes());
            }
            SdoResponseData::BlockUploadInitiated {
                block_size,
                crc_supported,
                size,
            } => {
                let mut cmd: u8 = 0xA0; // initiate block upload response (cs=5)
                if *crc_supported {
                    cmd |= 0x04;
                }
                if size.is_some() {
                    cmd |= 0x02; // size indicated
                }
                data[0] = cmd;
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
                data[4] = *block_size;
                if let Some(s) = size {
                    data[5..8].copy_from_slice(&s.to_le_bytes()[0..3]);
                }
            }
            SdoResponseData::BlockSegment { seq, data: d } => {
                data[0] = *seq & 0x7F; // sequence number (1-127)
                data[1..8].copy_from_slice(d);
            }
            SdoResponseData::BlockEnd { n, crc } => {
                let mut cmd: u8 = 0xA1; // end block (cs=5)
                if crc.is_some() {
                    cmd |= 0x04; // CRC used
                }
                cmd |= (n & 0x07) << 1;
                data[0] = cmd;
                if let Some(crc_val) = crc {
                    data[1..3].copy_from_slice(&crc_val.to_le_bytes());
                }
            }
            SdoResponseData::BlockDownloadConfirmed { crc_supported } => {
                let mut cmd: u8 = 0x80; // block download confirmed (cs=4)
                if *crc_supported {
                    cmd |= 0x04;
                }
                data[0] = cmd;
                data[1..3].copy_from_slice(&self.index.to_le_bytes());
                data[3] = self.subindex;
            }
        }

        CanOpenFrame::new(0x580 + self.node_id as u16, data)
    }
}

// === PDO Frame ===

/// PDO frame (TPDO/RPDO).
#[derive(Debug, Clone, PartialEq)]
pub struct PdoFrame {
    pub cob_id: u16,
    pub data: [u8; 8],
    pub timestamp: Option<Instant>,
}

impl PdoFrame {
    pub fn from_canopen(frame: &CanOpenFrame) -> Option<Self> {
        // TPDO1-4: 0x180-0x4FF, RPDO1-4: 0x200-0x57F
        match frame.cob_id {
            0x180..=0x57F => Some(Self {
                cob_id: frame.cob_id,
                data: frame.data,
                timestamp: frame.timestamp,
            }),
            _ => None,
        }
    }
}

/// Classify a CANOpen frame by its function.
#[derive(Debug, Clone, PartialEq)]
pub enum FrameClass {
    Nmt(NmtCommand),
    Sync(SyncFrame),
    Emergency(EmergencyFrame),
    TimestampFrame(TimestampFrame),
    Pdo(PdoFrame),
    SdoRequest(SdoRequest),
    SdoResponse(SdoResponse),
    Heartbeat(HeartbeatFrame),
    /// Unrecognized or malformed frame. Carries the raw COB-ID for diagnostics.
    Unknown {
        cob_id: u16,
    },
}

pub fn classify_frame(frame: &CanOpenFrame) -> FrameClass {
    let id = frame.cob_id;
    match id {
        0x000 => NmtCommand::decode(frame)
            .map(FrameClass::Nmt)
            .unwrap_or(FrameClass::Unknown { cob_id: id }),
        0x080 => SyncFrame::decode(frame)
            .map(FrameClass::Sync)
            .unwrap_or(FrameClass::Unknown { cob_id: id }),
        0x081..=0x0FF => EmergencyFrame::decode(frame)
            .map(FrameClass::Emergency)
            .unwrap_or(FrameClass::Unknown { cob_id: id }),
        0x100 => TimestampFrame::decode(frame)
            .map(FrameClass::TimestampFrame)
            .unwrap_or(FrameClass::Unknown { cob_id: id }),
        0x180..=0x57F => PdoFrame::from_canopen(frame)
            .map(FrameClass::Pdo)
            .unwrap_or(FrameClass::Unknown { cob_id: id }),
        0x580..=0x5FF => SdoResponse::decode(frame)
            .map(FrameClass::SdoResponse)
            .unwrap_or(FrameClass::Unknown { cob_id: id }),
        0x600..=0x67F => SdoRequest::decode(frame)
            .map(FrameClass::SdoRequest)
            .unwrap_or(FrameClass::Unknown { cob_id: id }),
        0x700..=0x77F => HeartbeatFrame::decode(frame)
            .map(FrameClass::Heartbeat)
            .unwrap_or(FrameClass::Unknown { cob_id: id }),
        _ => FrameClass::Unknown { cob_id: id },
    }
}

// === Display impls ===

impl std::fmt::Display for FunctionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nmt => write!(f, "NMT"),
            Self::SyncOrEmergency => write!(f, "SYNC/EMCY"),
            Self::Timestamp => write!(f, "TIMESTAMP"),
            Self::Tpdo1 => write!(f, "TPDO1"),
            Self::Rpdo1 => write!(f, "RPDO1"),
            Self::Tpdo2 => write!(f, "TPDO2"),
            Self::Rpdo2 => write!(f, "RPDO2"),
            Self::Tpdo3 => write!(f, "TPDO3"),
            Self::Rpdo3 => write!(f, "RPDO3"),
            Self::Tpdo4 => write!(f, "TPDO4"),
            Self::Rpdo4 => write!(f, "RPDO4"),
            Self::SdoServer => write!(f, "SDO-S"),
            Self::SdoClient => write!(f, "SDO-C"),
            Self::Heartbeat => write!(f, "HB"),
        }
    }
}

impl std::fmt::Display for CobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(node={})", self.function, self.node_id)
    }
}

impl std::fmt::Display for NmtState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BootUp => write!(f, "BootUp"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Operational => write!(f, "Operational"),
            Self::PreOperational => write!(f, "PreOperational"),
        }
    }
}

impl std::fmt::Display for CanOpenFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fc = CobId::from_u16(self.cob_id & 0x7FF);
        let fc_str = fc
            .map(|c| c.to_string())
            .unwrap_or_else(|| format!("0x{:03X}", self.cob_id));
        write!(f, "COB-ID=0x{:03X} ({}) data=[", self.cob_id, fc_str)?;
        for (i, b) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{:02X}", b)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cob_id_encode_decode() {
        let cob = CobId::new(FunctionCode::Tpdo1, 5);
        assert_eq!(cob.to_u16(), 0x185);

        let decoded = CobId::from_u16(0x185).unwrap();
        assert_eq!(decoded.function, FunctionCode::Tpdo1);
        assert_eq!(decoded.node_id, 5);

        // Sync: FC=0x080, node_id=0
        let sync = CobId::new(FunctionCode::SyncOrEmergency, 0);
        assert_eq!(sync.to_u16(), 0x080);
        assert!(sync.function.is_sync(sync.node_id));

        // Emergency: FC=0x080, node_id=5
        let emcy = CobId::new(FunctionCode::SyncOrEmergency, 5);
        assert_eq!(emcy.to_u16(), 0x085);
        assert!(emcy.function.is_emergency(emcy.node_id));
    }

    #[test]
    fn test_nmt_encode_decode() {
        let cmd = NmtCommand {
            command: NmtCommandSpecifier::EnterOperational,
            node_id: 3,
        };
        let frame = cmd.encode();
        assert_eq!(frame.cob_id, 0x000);
        assert_eq!(frame.data[0], 0x01);
        assert_eq!(frame.data[1], 3);

        let decoded = NmtCommand::decode(&frame).unwrap();
        assert_eq!(decoded.command, NmtCommandSpecifier::EnterOperational);
        assert_eq!(decoded.node_id, 3);
    }

    #[test]
    fn test_heartbeat_encode_decode() {
        let hb = HeartbeatFrame {
            node_id: 5,
            state: NmtState::Operational,
        };
        let frame = hb.encode();
        assert_eq!(frame.cob_id, 0x705);
        assert_eq!(frame.data[0], 0x05);

        let decoded = HeartbeatFrame::decode(&frame).unwrap();
        assert_eq!(decoded.node_id, 5);
        assert_eq!(decoded.state, NmtState::Operational);
    }

    #[test]
    fn test_emergency_encode_decode() {
        let emcy = EmergencyFrame {
            node_id: 3,
            error_code: 0x1000,
            error_register: 0x01,
            data: [0, 0, 0, 0, 0],
        };
        let frame = emcy.encode();
        assert_eq!(frame.cob_id, 0x083);

        let decoded = EmergencyFrame::decode(&frame).unwrap();
        assert_eq!(decoded.node_id, 3);
        assert_eq!(decoded.error_code, 0x1000);
        assert_eq!(decoded.error_register, 0x01);
    }

    #[test]
    fn test_sdo_expedited_upload_request() {
        let req = SdoRequest {
            node_id: 3,
            index: 0x1000,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = req.encode();
        assert_eq!(frame.cob_id, 0x603);
        assert_eq!(frame.data[0], 0x40); // initiate upload
        assert_eq!(frame.data[1], 0x00); // index low
        assert_eq!(frame.data[2], 0x10); // index high
        assert_eq!(frame.data[3], 0x00); // subindex
    }

    #[test]
    fn test_sdo_expedited_download() {
        let req = SdoRequest {
            node_id: 3,
            index: 0x6040,
            subindex: 0,
            data: SdoData::Expedited {
                data: [0x06, 0x00, 0, 0], // Control word: enable operation
                size: Some(2),
            },
        };
        let frame = req.encode();
        assert_eq!(frame.cob_id, 0x603);
        // cmd = 0x20 | 0x02 (expedited) | 0x01 (size indicated) | (4-2)<<2 = 0x2B
        assert_eq!(frame.data[0], 0x2B);
        assert_eq!(frame.data[4], 0x06);
        assert_eq!(frame.data[5], 0x00);
    }

    #[test]
    fn test_sdo_expedited_upload_response() {
        let mut frame = CanOpenFrame::new(0x583, [0u8; 8]);
        frame.data[0] = 0x43; // cs=2 (initiate upload response), expedited, 4 bytes
        frame.data[1] = 0x00;
        frame.data[2] = 0x10; // index 0x1000
        frame.data[3] = 0x00; // subindex 0
        frame.data[4] = 0x92;
        frame.data[5] = 0x01;
        frame.data[6] = 0x02;
        frame.data[7] = 0x00;

        let resp = SdoResponse::decode(&frame).unwrap();
        assert_eq!(resp.node_id, 3);
        assert_eq!(resp.index, 0x1000);
        assert_eq!(resp.subindex, 0);
        match resp.data {
            SdoResponseData::Expedited { data, size } => {
                assert_eq!(data, [0x92, 0x01, 0x02, 0x00]);
                assert_eq!(size, Some(4));
            }
            _ => panic!("Expected expedited response"),
        }
    }

    #[test]
    fn test_classify_frame() {
        let heartbeat = HeartbeatFrame {
            node_id: 5,
            state: NmtState::Operational,
        };
        let frame = heartbeat.encode();
        let class = classify_frame(&frame);
        assert!(matches!(class, FrameClass::Heartbeat(_)));

        let nmt = NmtCommand {
            command: NmtCommandSpecifier::EnterOperational,
            node_id: 0,
        };
        let frame = nmt.encode();
        let class = classify_frame(&frame);
        assert!(matches!(class, FrameClass::Nmt(_)));

        // SYNC without counter
        let sync = SyncFrame::new();
        let frame = sync.encode();
        let class = classify_frame(&frame);
        assert!(matches!(class, FrameClass::Sync(_)));

        // SYNC with counter
        let sync = SyncFrame::with_counter(42);
        let frame = sync.encode();
        let class = classify_frame(&frame);
        match &class {
            FrameClass::Sync(s) => assert_eq!(s.counter, Some(42)),
            _ => panic!("Expected Sync"),
        }

        // Timestamp
        let ts = TimestampFrame::new(12345, 100);
        let frame = ts.encode();
        let class = classify_frame(&frame);
        assert!(matches!(class, FrameClass::TimestampFrame(_)));

        // Emergency
        let emcy = EmergencyFrame {
            node_id: 3,
            error_code: 0x1000,
            error_register: 0x01,
            data: [0; 5],
        };
        let frame = emcy.encode();
        let class = classify_frame(&frame);
        assert!(matches!(class, FrameClass::Emergency(_)));

        // SDO client frame should be classified
        let sdo_req = SdoRequest {
            node_id: 3,
            index: 0x1000,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = sdo_req.encode();
        let class = classify_frame(&frame);
        assert!(matches!(class, FrameClass::SdoRequest(_)));

        // Unknown FC (e.g. 0x780) should carry cob_id
        let frame = CanOpenFrame::new(0x780, [0; 8]);
        let class = classify_frame(&frame);
        assert!(matches!(class, FrameClass::Unknown { cob_id: 0x780 }));
    }

    #[test]
    fn test_sync_frame_no_counter() {
        let sync = SyncFrame::new();
        let frame = sync.encode();
        assert_eq!(frame.cob_id, 0x080);
        assert_eq!(frame.data[0], 0); // no counter

        let decoded = SyncFrame::decode(&frame).unwrap();
        assert_eq!(decoded.counter, None);
    }

    #[test]
    fn test_sync_frame_with_counter() {
        let sync = SyncFrame::with_counter(42);
        let frame = sync.encode();
        assert_eq!(frame.cob_id, 0x080);
        assert_eq!(frame.data[0], 42);

        let decoded = SyncFrame::decode(&frame).unwrap();
        assert_eq!(decoded.counter, Some(42));
    }

    #[test]
    fn test_sync_frame_decode_wrong_cob_id() {
        let frame = CanOpenFrame::new(0x081, [0; 8]);
        assert!(SyncFrame::decode(&frame).is_none());
    }

    #[test]
    fn test_timestamp_frame_encode_decode() {
        // 43200000 ms = 12 hours, day 365
        let ts = TimestampFrame::new(43_200_000, 365);
        let frame = ts.encode();
        assert_eq!(frame.cob_id, 0x100);

        let decoded = TimestampFrame::decode(&frame).unwrap();
        assert_eq!(decoded.ms_of_day, 43_200_000);
        assert_eq!(decoded.days, 365);
    }

    #[test]
    fn test_timestamp_frame_total_ms() {
        let ts = TimestampFrame::new(0, 1);
        assert_eq!(ts.to_total_ms(), 86_400_000); // 1 day

        let ts = TimestampFrame::new(1000, 0);
        assert_eq!(ts.to_total_ms(), 1000);
    }

    #[test]
    fn test_timestamp_frame_decode_wrong_cob_id() {
        let frame = CanOpenFrame::new(0x101, [0; 8]);
        assert!(TimestampFrame::decode(&frame).is_none());
    }

    // === SdoResponse segment + abort tests ===

    #[test]
    fn test_sdo_response_decode_abort() {
        let mut frame = CanOpenFrame::new(0x583, [0u8; 8]);
        frame.data[0] = 0x80; // abort
        frame.data[1] = 0x00;
        frame.data[2] = 0x10; // index 0x1000
        frame.data[3] = 0x00; // subindex 0
        // abort code 0x06020000 in little-endian: [0x00, 0x00, 0x02, 0x06]
        frame.data[4] = 0x00;
        frame.data[5] = 0x00;
        frame.data[6] = 0x02;
        frame.data[7] = 0x06;

        let resp = SdoResponse::decode(&frame).unwrap();
        assert_eq!(resp.node_id, 3);
        assert_eq!(resp.index, 0x1000);
        assert_eq!(resp.subindex, 0);
        match resp.data {
            SdoResponseData::Abort { code } => {
                assert_eq!(code, 0x0602_0000);
            }
            _ => panic!("Expected abort response"),
        }
    }

    #[test]
    fn test_sdo_response_decode_segment_full() {
        // Segment response with 7 bytes of data (size not indicated)
        let mut frame = CanOpenFrame::new(0x583, [0u8; 8]);
        frame.data[0] = 0x00; // cs=0, toggle=0, no size indicated, not last
        frame.data[1..8].copy_from_slice(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47]);

        let resp = SdoResponse::decode(&frame).unwrap();
        match resp.data {
            SdoResponseData::Segment {
                toggle,
                last,
                data,
                size,
            } => {
                assert!(!toggle);
                assert!(!last);
                assert_eq!(data, [0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47]);
                assert_eq!(size, None); // size not indicated
            }
            _ => panic!("Expected segment response"),
        }
    }

    #[test]
    fn test_sdo_response_decode_segment_with_size() {
        // Segment response with size indicated (e.g., 3 bytes)
        // Size encoding: n = (cmd >> 1) & 0x07, actual_size = 7 - n
        // For 3 bytes: n = 4, so cmd bits 1-3 = 100, cmd = 0x08
        let mut frame = CanOpenFrame::new(0x583, [0u8; 8]);
        frame.data[0] = 0x08; // cs=0, toggle=0, size indicated (n=4 → 3 bytes), not last
        frame.data[1..8].copy_from_slice(&[0x41, 0x42, 0x43, 0x00, 0x00, 0x00, 0x00]);

        let resp = SdoResponse::decode(&frame).unwrap();
        match resp.data {
            SdoResponseData::Segment {
                toggle,
                last,
                data: _,
                size,
            } => {
                assert!(!toggle);
                assert!(!last);
                assert_eq!(size, Some(3));
            }
            _ => panic!("Expected segment response"),
        }
    }

    #[test]
    fn test_sdo_response_decode_segment_last() {
        // Last segment in transfer
        let mut frame = CanOpenFrame::new(0x583, [0u8; 8]);
        frame.data[0] = 0x01; // cs=0, toggle=0, no size, last=1
        frame.data[1..8].copy_from_slice(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47]);

        let resp = SdoResponse::decode(&frame).unwrap();
        match resp.data {
            SdoResponseData::Segment { toggle, last, .. } => {
                assert!(!toggle);
                assert!(last);
            }
            _ => panic!("Expected segment response"),
        }
    }

    #[test]
    fn test_sdo_response_decode_segment_toggled() {
        // Segment with toggle bit set
        let mut frame = CanOpenFrame::new(0x583, [0u8; 8]);
        frame.data[0] = 0x10; // cs=0, toggle=1, no size, not last
        frame.data[1..8].copy_from_slice(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47]);

        let resp = SdoResponse::decode(&frame).unwrap();
        match resp.data {
            SdoResponseData::Segment { toggle, last, .. } => {
                assert!(toggle);
                assert!(!last);
            }
            _ => panic!("Expected segment response"),
        }
    }

    #[test]
    fn test_sdo_response_decode_segmented_initiated() {
        // Segmented upload initiated with size indicated
        let mut frame = CanOpenFrame::new(0x583, [0u8; 8]);
        frame.data[0] = 0x41; // cs=2, not expedited, size indicated
        frame.data[1] = 0x00;
        frame.data[2] = 0x20; // index 0x2000
        frame.data[3] = 0x00;
        frame.data[4..8].copy_from_slice(&1000u32.to_le_bytes()); // size = 1000

        let resp = SdoResponse::decode(&frame).unwrap();
        match resp.data {
            SdoResponseData::SegmentedInitiated { size } => {
                assert_eq!(size, 1000);
            }
            _ => panic!("Expected segmented initiated response"),
        }
    }

    #[test]
    fn test_sdo_response_decode_download_confirmed() {
        // Download confirmed (cs=1)
        let mut frame = CanOpenFrame::new(0x583, [0u8; 8]);
        frame.data[0] = 0x20; // cs=1 (download confirmed)
        frame.data[1] = 0x40;
        frame.data[2] = 0x60; // index 0x6040
        frame.data[3] = 0x00;

        let resp = SdoResponse::decode(&frame).unwrap();
        assert_eq!(resp.index, 0x6040);
        assert!(matches!(resp.data, SdoResponseData::DownloadConfirmed));
    }

    #[test]
    fn test_sdo_response_decode_wrong_cob_id() {
        // Frame with COB-ID outside SDO response range
        let frame = CanOpenFrame::new(0x180, [0u8; 8]);
        assert!(SdoResponse::decode(&frame).is_none());
    }

    #[test]
    fn test_sdo_response_encode_decode_roundtrip() {
        let original = SdoResponse {
            node_id: 5,
            index: 0x6041,
            subindex: 0,
            data: SdoResponseData::Expedited {
                data: [0x37, 0x02, 0x00, 0x00],
                size: Some(2),
            },
        };
        let encoded = original.encode();
        let decoded = SdoResponse::decode(&encoded).unwrap();

        assert_eq!(decoded.node_id, original.node_id);
        assert_eq!(decoded.index, original.index);
        assert_eq!(decoded.subindex, original.subindex);
        match (&decoded.data, &original.data) {
            (
                SdoResponseData::Expedited { data: d1, size: s1 },
                SdoResponseData::Expedited { data: d2, size: s2 },
            ) => {
                assert_eq!(d1, d2);
                assert_eq!(s1, s2);
            }
            _ => panic!("Type mismatch in roundtrip"),
        }
    }
}
