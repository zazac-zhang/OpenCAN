//! PDO (Process Data Object) handling.
//!
//! PDOs are used for real-time data exchange in CANOpen.
//! TPDOs are transmitted by a node, RPDOs are received.
//!
//! Re-exports core PDO types for backward compatibility.

use opencan_canopen_core::frame::CanOpenFrame;
pub use opencan_canopen_core::pdo::{
    PdoDirection, PdoMapping, TransmissionType, pdo_comm_index, pdo_map_index, validate_mapping,
};

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

/// Parse a CAN frame as a PDO with timestamp.
pub fn parse_pdo(frame: &CanOpenFrame, timestamp_ms: u64) -> Option<PdoData> {
    let core_pdo = opencan_canopen_core::pdo::parse_pdo(frame)?;

    Some(PdoData {
        cob_id: core_pdo.cob_id,
        node_id: core_pdo.node_id,
        pdo_number: core_pdo.pdo_number,
        direction: core_pdo.direction,
        data: core_pdo.data,
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
        pdo_comm_index(pdo_number, direction).unwrap_or(0)
    }

    /// Get the OD index for this PDO's mapping parameter.
    pub fn map_index(&self, pdo_number: u8, direction: PdoDirection) -> u16 {
        pdo_map_index(pdo_number, direction).unwrap_or(0)
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
        assert_eq!(pdo.timestamp_ms, 1000);
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
