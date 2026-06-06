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

/// PDO mapping entry (index:16 + subindex:8 + bit_length:8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdoMappingEntry {
    pub index: u16,
    pub subindex: u8,
    pub bit_length: u8,
}

impl PdoMappingEntry {
    /// Encode as a 32-bit value for OD storage.
    pub fn to_u32(&self) -> u32 {
        ((self.index as u32) << 16) | ((self.subindex as u32) << 8) | (self.bit_length as u32)
    }

    /// Decode from a 32-bit OD value.
    pub fn from_u32(val: u32) -> Self {
        Self {
            index: ((val >> 16) & 0xFFFF) as u16,
            subindex: ((val >> 8) & 0xFF) as u8,
            bit_length: (val & 0xFF) as u8,
        }
    }
}

/// Get the OD index for PDO communication parameters.
pub fn pdo_comm_index(pdo_number: u8, direction: PdoDirection) -> Option<u16> {
    match (pdo_number, direction) {
        (1..=4, PdoDirection::Rpdo) => Some(0x1400 + (pdo_number - 1) as u16),
        (1..=4, PdoDirection::Tpdo) => Some(0x1800 + (pdo_number - 1) as u16),
        _ => None,
    }
}

/// Get the OD index for PDO mapping parameters.
pub fn pdo_map_index(pdo_number: u8, direction: PdoDirection) -> Option<u16> {
    match (pdo_number, direction) {
        (1..=4, PdoDirection::Rpdo) => Some(0x1600 + (pdo_number - 1) as u16),
        (1..=4, PdoDirection::Tpdo) => Some(0x1A00 + (pdo_number - 1) as u16),
        _ => None,
    }
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
    ) -> Result<Vec<PdoMappingEntry>, CanOpenError> {
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
            entries.push(PdoMappingEntry::from_u32(val));
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
        mappings: &[PdoMappingEntry],
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

/// Validate PDO mapping entries (GUI layer helper).
///
/// Returns Ok(()) if valid, Err(description) if invalid.
pub fn validate_mapping(mappings: &[PdoMappingEntry]) -> Result<(), String> {
    if mappings.is_empty() {
        return Ok(());
    }

    // Total bit length must be ≤ 64
    let total_bits: u16 = mappings.iter().map(|m| m.bit_length as u16).sum();
    if total_bits > 64 {
        return Err(format!(
            "Total bit length {} exceeds maximum 64 bits",
            total_bits
        ));
    }

    // Each entry's bit length should be 1, 2, 4, 8, 16, 32, or 64
    for (i, entry) in mappings.iter().enumerate() {
        if entry.bit_length == 0 {
            return Err(format!("Mapping entry {} has zero bit length", i));
        }
        if !matches!(entry.bit_length, 1 | 2 | 4 | 8 | 16 | 32 | 64) {
            return Err(format!(
                "Mapping entry {} has invalid bit length {} (must be 1, 2, 4, 8, 16, 32, or 64)",
                i, entry.bit_length
            ));
        }
    }

    Ok(())
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
        let entry = PdoMappingEntry {
            index: 0x6041,
            subindex: 0,
            bit_length: 16,
        };
        let val = entry.to_u32();
        assert_eq!(val, 0x60410010);
        let decoded = PdoMappingEntry::from_u32(val);
        assert_eq!(decoded, entry);
    }

    #[test]
    fn test_validate_mapping_valid() {
        let mappings = vec![
            PdoMappingEntry {
                index: 0x6041,
                subindex: 0,
                bit_length: 16,
            },
            PdoMappingEntry {
                index: 0x6064,
                subindex: 0,
                bit_length: 32,
            },
        ];
        assert!(validate_mapping(&mappings).is_ok());
    }

    #[test]
    fn test_validate_mapping_too_large() {
        let mappings = vec![
            PdoMappingEntry {
                index: 0x6041,
                subindex: 0,
                bit_length: 32,
            },
            PdoMappingEntry {
                index: 0x6064,
                subindex: 0,
                bit_length: 32,
            },
            PdoMappingEntry {
                index: 0x606C,
                subindex: 0,
                bit_length: 8,
            },
        ];
        assert!(validate_mapping(&mappings).is_err());
    }

    #[test]
    fn test_validate_mapping_invalid_bit_length() {
        let mappings = vec![PdoMappingEntry {
            index: 0x6041,
            subindex: 0,
            bit_length: 3,
        }];
        assert!(validate_mapping(&mappings).is_err());
    }

    #[test]
    fn test_validate_mapping_zero_length() {
        let mappings = vec![PdoMappingEntry {
            index: 0x6041,
            subindex: 0,
            bit_length: 0,
        }];
        assert!(validate_mapping(&mappings).is_err());
    }
}
