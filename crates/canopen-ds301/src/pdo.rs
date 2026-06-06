//! PDO (Process Data Object) handling.
//!
//! PDOs are used for real-time data exchange in CANOpen.
//! TPDOs are transmitted by a node, RPDOs are received.

use opencan_canopen_core::frame::CanOpenFrame;

/// PDO transmission type (DS301 Table 54).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TransmissionType {
    /// Sync: acyclic
    SyncAcyclic = 0,
    /// Sync: every sync
    SyncEvery = 1,
    /// Sync: every 2nd sync
    Sync2 = 2,
    /// Sync: every 3rd sync
    Sync3 = 3,
    // ... up to 240
    /// RTR only (synchronous)
    RtrSync = 252,
    /// RTR only (asynchronous)
    RtrAsync = 253,
    /// Event-driven (manufacturer-specific)
    EventManufacturer = 254,
    /// Event-driven (device profile)
    EventProfile = 255,
}

impl TransmissionType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::SyncAcyclic),
            1 => Some(Self::SyncEvery),
            2..=240 => Some(Self::SyncEvery), // Simplified
            252 => Some(Self::RtrSync),
            253 => Some(Self::RtrAsync),
            254 => Some(Self::EventManufacturer),
            255 => Some(Self::EventProfile),
            _ => None,
        }
    }
}

/// PDO mapping entry (Index + Subindex + Bit Length).
#[derive(Debug, Clone, Copy)]
pub struct PdoMapping {
    pub index: u16,
    pub subindex: u8,
    pub bit_length: u8,
}

/// PDO frame with decoded mapping.
#[derive(Debug, Clone)]
pub struct PdoData {
    pub cob_id: u16,
    pub node_id: u8,
    pub pdo_number: u8, // 1-4
    pub direction: PdoDirection,
    pub data: [u8; 8],
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdoDirection {
    Tpdo, // Transmit from node
    Rpdo, // Receive by node
}

/// Parse a CAN frame as a PDO.
pub fn parse_pdo(frame: &CanOpenFrame, timestamp_ms: u64) -> Option<PdoData> {
    let id = frame.cob_id;

    let (node_id, pdo_number, direction) = match id {
        0x180..=0x1FF => ((id - 0x180) as u8, 1, PdoDirection::Tpdo),
        0x200..=0x27F => ((id - 0x200) as u8, 1, PdoDirection::Rpdo),
        0x280..=0x2FF => ((id - 0x280) as u8, 2, PdoDirection::Tpdo),
        0x300..=0x37F => ((id - 0x300) as u8, 2, PdoDirection::Rpdo),
        0x380..=0x3FF => ((id - 0x380) as u8, 3, PdoDirection::Tpdo),
        0x400..=0x47F => ((id - 0x400) as u8, 3, PdoDirection::Rpdo),
        0x480..=0x4FF => ((id - 0x480) as u8, 4, PdoDirection::Tpdo),
        0x500..=0x57F => ((id - 0x500) as u8, 4, PdoDirection::Rpdo),
        _ => return None,
    };

    let mut data = [0u8; 8];
    data.copy_from_slice(&frame.data);

    Some(PdoData {
        cob_id: id,
        node_id,
        pdo_number,
        direction,
        data,
        timestamp_ms,
    })
}

/// PDO configuration for a node.
#[derive(Debug, Clone)]
pub struct PdoConfig {
    pub cob_id: u16,
    pub transmission_type: TransmissionType,
    pub mapping: Vec<PdoMapping>,
}

impl PdoConfig {
    /// Get the OD index for this PDO's communication parameter.
    pub fn comm_index(&self, pdo_number: u8, direction: PdoDirection) -> u16 {
        match (pdo_number, direction) {
            (1, PdoDirection::Tpdo) => 0x1800,
            (1, PdoDirection::Rpdo) => 0x1400,
            (2, PdoDirection::Tpdo) => 0x1801,
            (2, PdoDirection::Rpdo) => 0x1401,
            (3, PdoDirection::Tpdo) => 0x1802,
            (3, PdoDirection::Rpdo) => 0x1402,
            (4, PdoDirection::Tpdo) => 0x1803,
            (4, PdoDirection::Rpdo) => 0x1403,
            _ => 0,
        }
    }

    /// Get the OD index for this PDO's mapping parameter.
    pub fn map_index(&self, pdo_number: u8, direction: PdoDirection) -> u16 {
        match (pdo_number, direction) {
            (1, PdoDirection::Tpdo) => 0x1A00,
            (1, PdoDirection::Rpdo) => 0x1600,
            (2, PdoDirection::Tpdo) => 0x1A01,
            (2, PdoDirection::Rpdo) => 0x1601,
            (3, PdoDirection::Tpdo) => 0x1A02,
            (3, PdoDirection::Rpdo) => 0x1602,
            (4, PdoDirection::Tpdo) => 0x1A03,
            (4, PdoDirection::Rpdo) => 0x1603,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencan_canopen_core::frame::CanOpenFrame;

    #[test]
    fn test_parse_tpdo1() {
        let frame = CanOpenFrame::new(0x185, [1, 2, 3, 4, 5, 6, 7, 8]);
        let pdo = parse_pdo(&frame, 1000).unwrap();
        assert_eq!(pdo.node_id, 5);
        assert_eq!(pdo.pdo_number, 1);
        assert_eq!(pdo.direction, PdoDirection::Tpdo);
        assert_eq!(pdo.data, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_parse_rpdo2() {
        let frame = CanOpenFrame::new(0x303, [0xAA; 8]);
        let pdo = parse_pdo(&frame, 2000).unwrap();
        assert_eq!(pdo.node_id, 3);
        assert_eq!(pdo.pdo_number, 2);
        assert_eq!(pdo.direction, PdoDirection::Rpdo);
    }

    #[test]
    fn test_parse_non_pdo() {
        let frame = CanOpenFrame::new(0x603, [0; 8]); // SDO client
        assert!(parse_pdo(&frame, 0).is_none());
    }

    #[test]
    fn test_pdo_config_index() {
        let config = PdoConfig {
            cob_id: 0x180,
            transmission_type: TransmissionType::SyncEvery,
            mapping: vec![],
        };
        assert_eq!(config.comm_index(1, PdoDirection::Tpdo), 0x1800);
        assert_eq!(config.map_index(1, PdoDirection::Rpdo), 0x1600);
    }
}
