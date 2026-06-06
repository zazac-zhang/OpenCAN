//! PDO Configuration Manager — reads/writes PDO config via SDO.
//!
//! PDO configuration is stored in the Object Dictionary at well-known indices:
//! - Communication parameters: 0x1400-0x1403 (RPDO), 0x1800-0x1803 (TPDO)
//! - Mapping parameters: 0x1600-0x1603 (RPDO), 0x1A00-0x1A03 (TPDO)

use crate::heartbeat::PdoDirection;
use crate::sdo::SdoClient;
use opencan_canopen_core::CanDriver;
use opencan_canopen_core::CanOpenError;
use opencan_canopen_core::od::OdValue;
use opencan_canopen_core::pdo::{PdoMapping, pdo_comm_index, pdo_map_index};

// Re-export core PDO utilities for downstream consumers
pub use opencan_canopen_core::pdo::validate_mapping;

/// PDO communication parameters (0x1400/0x1800 + pdo_number - 1).
#[derive(Debug, Clone)]
pub struct PdoCommParams {
    /// COB-ID (bit 31 = valid flag, bits 10:0 = CAN ID)
    pub cob_id: u32,
    /// Transmission type (0=acyclic, 1=every SYNC, 2-240=every N SYNC, 254/255=event-driven)
    pub transmission_type: u8,
    /// Inhibit time in 100μs units (0 = not used)
    pub inhibit_time: u16,
    /// Event timer in ms (0 = not used)
    pub event_timer: u16,
}

/// PDO configuration manager — reads/writes PDO config via SDO.
pub struct PdoConfigManager<'a, C: CanDriver> {
    sdo: &'a mut SdoClient<C>,
}

impl<'a, C: CanDriver> PdoConfigManager<'a, C> {
    /// Create a new PDO config manager wrapping an SDO client.
    pub fn new(sdo: &'a mut SdoClient<C>) -> Self {
        Self { sdo }
    }

    /// Read PDO communication parameters for a remote node.
    pub async fn read_comm_params(
        &mut self,
        node_id: u8,
        pdo_number: u8,
        direction: PdoDirection,
    ) -> Result<PdoCommParams, CanOpenError> {
        let base = pdo_comm_index(pdo_number, direction)
            .ok_or_else(|| CanOpenError::Protocol(format!("Invalid PDO number: {}", pdo_number)))?;

        let cob_id = match self.sdo.upload(node_id, base, 0).await? {
            OdValue::Unsigned32(v) => v,
            other => {
                return Err(CanOpenError::Protocol(format!(
                    "Expected u32 for COB-ID, got {:?}",
                    other
                )));
            }
        };

        let transmission_type = match self.sdo.upload(node_id, base, 1).await? {
            OdValue::Unsigned8(v) => v,
            other => {
                return Err(CanOpenError::Protocol(format!(
                    "Expected u8 for transmission type, got {:?}",
                    other
                )));
            }
        };

        // Sub-index 2: Inhibit Time (optional, may not exist)
        let inhibit_time = match self.sdo.upload(node_id, base, 2).await {
            Ok(OdValue::Unsigned16(v)) => v,
            _ => 0,
        };

        // Sub-index 5: Event Timer (optional, may not exist)
        let event_timer = match self.sdo.upload(node_id, base, 5).await {
            Ok(OdValue::Unsigned16(v)) => v,
            _ => 0,
        };

        Ok(PdoCommParams {
            cob_id,
            transmission_type,
            inhibit_time,
            event_timer,
        })
    }

    /// Read PDO mapping for a remote node.
    pub async fn read_mapping(
        &mut self,
        node_id: u8,
        pdo_number: u8,
        direction: PdoDirection,
    ) -> Result<Vec<PdoMapping>, CanOpenError> {
        let base = pdo_map_index(pdo_number, direction)
            .ok_or_else(|| CanOpenError::Protocol(format!("Invalid PDO number: {}", pdo_number)))?;

        // Sub-index 0: Number of mapped objects
        let count = match self.sdo.upload(node_id, base, 0).await? {
            OdValue::Unsigned8(v) => v as usize,
            other => {
                return Err(CanOpenError::Protocol(format!(
                    "Expected u8 for mapping count, got {:?}",
                    other
                )));
            }
        };

        let mut entries = Vec::with_capacity(count);
        for i in 1..=count {
            let val = match self.sdo.upload(node_id, base, i as u8).await? {
                OdValue::Unsigned32(v) => v,
                other => {
                    return Err(CanOpenError::Protocol(format!(
                        "Expected u32 for mapping entry, got {:?}",
                        other
                    )));
                }
            };
            entries.push(PdoMapping::from_u32(val));
        }

        Ok(entries)
    }

    /// Write PDO mapping to a remote node.
    ///
    /// The caller must ensure the PDO is disabled (bit 31 of COB-ID set) before writing.
    pub async fn write_mapping(
        &mut self,
        node_id: u8,
        pdo_number: u8,
        direction: PdoDirection,
        mappings: &[PdoMapping],
    ) -> Result<(), CanOpenError> {
        let base = pdo_map_index(pdo_number, direction)
            .ok_or_else(|| CanOpenError::Protocol(format!("Invalid PDO number: {}", pdo_number)))?;

        // Write number of mapped objects (0 to clear)
        self.sdo
            .download(node_id, base, 0, &OdValue::Unsigned8(0))
            .await?;

        // Write each mapping entry
        for (i, entry) in mappings.iter().enumerate() {
            self.sdo
                .download(
                    node_id,
                    base,
                    (i + 1) as u8,
                    &OdValue::Unsigned32(entry.to_u32()),
                )
                .await?;
        }

        // Write the actual count
        self.sdo
            .download(node_id, base, 0, &OdValue::Unsigned8(mappings.len() as u8))
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdo_comm_index() {
        assert_eq!(pdo_comm_index(1, PdoDirection::Rpdo), Some(0x1400));
        assert_eq!(pdo_comm_index(1, PdoDirection::Tpdo), Some(0x1800));
        assert_eq!(pdo_comm_index(4, PdoDirection::Rpdo), Some(0x1403));
        assert_eq!(pdo_comm_index(4, PdoDirection::Tpdo), Some(0x1803));
        assert_eq!(pdo_comm_index(5, PdoDirection::Rpdo), None);
        assert_eq!(pdo_comm_index(0, PdoDirection::Tpdo), None);
    }

    #[test]
    fn test_pdo_map_index() {
        assert_eq!(pdo_map_index(1, PdoDirection::Rpdo), Some(0x1600));
        assert_eq!(pdo_map_index(1, PdoDirection::Tpdo), Some(0x1A00));
        assert_eq!(pdo_map_index(4, PdoDirection::Rpdo), Some(0x1603));
        assert_eq!(pdo_map_index(4, PdoDirection::Tpdo), Some(0x1A03));
    }

    #[test]
    fn test_mapping_entry_roundtrip() {
        let entry = PdoMapping::new(0x6041, 0, 16);
        let val = entry.to_u32();
        assert_eq!(val, 0x60410010);
        let decoded = PdoMapping::from_u32(val);
        assert_eq!(decoded, entry);
    }

    #[test]
    fn test_validate_mapping_valid() {
        let mappings = vec![
            PdoMapping::new(0x6041, 0, 16),
            PdoMapping::new(0x6064, 0, 32),
        ];
        assert!(validate_mapping(&mappings).is_ok());
    }

    #[test]
    fn test_validate_mapping_too_large() {
        let mappings = vec![
            PdoMapping::new(0x6041, 0, 32),
            PdoMapping::new(0x6064, 0, 32),
            PdoMapping::new(0x606C, 0, 8),
        ];
        assert!(validate_mapping(&mappings).is_err());
    }

    #[test]
    fn test_validate_mapping_invalid_bit_length() {
        let mappings = vec![PdoMapping::new(0x6041, 0, 3)];
        assert!(validate_mapping(&mappings).is_err());
    }

    #[test]
    fn test_validate_mapping_zero_length() {
        let mappings = vec![PdoMapping::new(0x6041, 0, 0)];
        assert!(validate_mapping(&mappings).is_err());
    }
}
