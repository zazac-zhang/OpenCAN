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
        Self { cob_id, data, timestamp: None }
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
pub enum FunctionCode {
    Nmt             = 0x000,
    SyncOrEmergency = 0x080,  // Sync if node_id=0, Emergency otherwise
    Timestamp       = 0x100,
    Tpdo1           = 0x180,
    Rpdo1           = 0x200,
    Tpdo2           = 0x280,
    Rpdo2           = 0x300,
    Tpdo3           = 0x380,
    Rpdo3           = 0x400,
    Tpdo4           = 0x480,
    Rpdo4           = 0x500,
    SdoServer       = 0x580,  // SDO response (server → client)
    SdoClient       = 0x600,  // SDO request (client → server)
    Heartbeat       = 0x700,
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
    EnterOperational    = 0x01,
    EnterStopped        = 0x02,
    EnterPreOperational = 0x80,
    ResetNode           = 0x81,
    ResetCommunication  = 0x82,
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

/// NMT Command frame.
#[derive(Debug, Clone, PartialEq)]
pub struct NmtCommand {
    pub command: NmtCommandSpecifier,
    pub node_id: u8,  // 0 = broadcast to all nodes
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
        Some(Self { command, node_id: frame.data[1] })
    }
}

// === NMT State ===

/// NMT states (DS301 Figure 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NmtState {
    BootUp          = 0x00,
    Stopped         = 0x04,
    Operational     = 0x05,
    PreOperational  = 0x7F,
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
        CanOpenFrame::new(0x700 + self.node_id as u16, [self.state as u8, 0, 0, 0, 0, 0, 0, 0])
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
        Some(Self { node_id, error_code, error_register, data })
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
    Segment { toggle: bool, last: bool, data: [u8; 7], size: Option<u8> },
    /// Upload request (no data).
    UploadRequest,
    /// Abort.
    Abort { code: u32 },
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
            SdoData::Segment { toggle, last, data: d, size } => {
                let mut cmd: u8 = 0x00;
                if *toggle { cmd |= 0x10; }
                if *last { cmd |= 0x01; }
                if let Some(s) = size {
                    cmd |= 0x0C; // size indicated
                    cmd |= (7 - s) << 4;
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
        }

        CanOpenFrame::new(0x600 + self.node_id as u16, data)
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
    Segment { toggle: bool, last: bool, data: [u8; 7], size: Option<u8> },
    /// Download confirmed.
    DownloadConfirmed,
    /// Abort.
    Abort { code: u32 },
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
            // Abort
            let code = u32::from_le_bytes([frame.data[4], frame.data[5], frame.data[6], frame.data[7]]);
            SdoResponseData::Abort { code }
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
                let size = u32::from_le_bytes([frame.data[4], frame.data[5], frame.data[6], frame.data[7]]);
                SdoResponseData::SegmentedInitiated { size }
            } else {
                SdoResponseData::SegmentedInitiated { size: 0 }
            }
        } else if cmd & 0xE0 == 0x20 {
            // Download confirmed (cs=1)
            SdoResponseData::DownloadConfirmed
        } else if cmd & 0xE0 == 0x00 {
            // Segment download response (cs=0)
            let toggle = cmd & 0x10 != 0;
            let last = cmd & 0x01 != 0;
            let size = if cmd & 0x0E != 0 {
                Some(7 - ((cmd >> 4) & 0x07))
            } else {
                None
            };
            let mut d = [0u8; 7];
            d.copy_from_slice(&frame.data[1..8]);
            SdoResponseData::Segment { toggle, last, data: d, size }
        } else {
            return None;
        };

        Some(Self { node_id, index, subindex, data })
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
            0x180..=0x1FF | 0x200..=0x27F | 0x280..=0x2FF |
            0x300..=0x37F | 0x380..=0x3FF | 0x400..=0x47F |
            0x480..=0x4FF | 0x500..=0x57F => {
                Some(Self {
                    cob_id: frame.cob_id,
                    data: frame.data,
                    timestamp: frame.timestamp,
                })
            }
            _ => None,
        }
    }
}

/// Classify a CANOpen frame by its function.
#[derive(Debug, Clone, PartialEq)]
pub enum FrameClass {
    Nmt(NmtCommand),
    Sync,
    Emergency(EmergencyFrame),
    Timestamp,
    Pdo(PdoFrame),
    SdoResponse(SdoResponse),
    Heartbeat(HeartbeatFrame),
    Unknown,
}

pub fn classify_frame(frame: &CanOpenFrame) -> FrameClass {
    let id = frame.cob_id;
    match id {
        0x000 => {
            NmtCommand::decode(frame)
                .map(FrameClass::Nmt)
                .unwrap_or(FrameClass::Unknown)
        }
        0x080..=0x0FF => {
            if id == 0x080 {
                FrameClass::Sync
            } else {
                EmergencyFrame::decode(frame)
                    .map(FrameClass::Emergency)
                    .unwrap_or(FrameClass::Unknown)
            }
        }
        0x100..=0x17F => FrameClass::Timestamp,
        0x180..=0x57F => {
            if let Some(hb) = HeartbeatFrame::decode(frame) {
                FrameClass::Heartbeat(hb)
            } else if let Some(resp) = SdoResponse::decode(frame) {
                FrameClass::SdoResponse(resp)
            } else if let Some(pdo) = PdoFrame::from_canopen(frame) {
                FrameClass::Pdo(pdo)
            } else {
                FrameClass::Unknown
            }
        }
        0x700..=0x77F => {
            HeartbeatFrame::decode(frame)
                .map(FrameClass::Heartbeat)
                .unwrap_or(FrameClass::Unknown)
        }
        _ => FrameClass::Unknown,
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
        let hb = HeartbeatFrame { node_id: 5, state: NmtState::Operational };
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
        let heartbeat = HeartbeatFrame { node_id: 5, state: NmtState::Operational };
        let frame = heartbeat.encode();
        let class = classify_frame(&frame);
        assert!(matches!(class, FrameClass::Heartbeat(_)));

        let nmt = NmtCommand { command: NmtCommandSpecifier::EnterOperational, node_id: 0 };
        let frame = nmt.encode();
        let class = classify_frame(&frame);
        assert!(matches!(class, FrameClass::Nmt(_)));
    }
}
