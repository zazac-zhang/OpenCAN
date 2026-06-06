//! PDO (Process Data Object) types.
//!
//! PDOs are the primary mechanism for real-time data exchange in CANOpen.
//! This module contains the core PDO types used across the protocol stack.
//!
//! ## PDO Addressing
//!
//! | Function   | COB-ID Range  | Direction           |
//! |------------|---------------|---------------------|
//! | TPDO 1     | 0x180 - 0x1FF | Transmit from node  |
//! | RPDO 1     | 0x200 - 0x27F | Receive by node     |
//! | TPDO 2     | 0x280 - 0x2FF | Transmit from node  |
//! | RPDO 2     | 0x300 - 0x37F | Receive by node     |
//! | TPDO 3     | 0x380 - 0x3FF | Transmit from node  |
//! | RPDO 3     | 0x400 - 0x47F | Receive by node     |
//! | TPDO 4     | 0x480 - 0x4FF | Transmit from node  |
//! | RPDO 4     | 0x500 - 0x57F | Receive by node     |

use crate::frame::CanOpenFrame;
use crate::od::{DataType, OdValue};

/// PDO direction (DS301 Section 7.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PdoDirection {
    /// TPDO — transmitted by the node (producer → consumer).
    Tpdo,
    /// RPDO — received by the node (consumer ← producer).
    Rpdo,
}

impl PdoDirection {
    /// Get the opposite direction.
    pub fn reverse(&self) -> Self {
        match self {
            Self::Tpdo => Self::Rpdo,
            Self::Rpdo => Self::Tpdo,
        }
    }
}

/// PDO transmission type (DS301 Table 54).
///
/// Controls when a PDO is transmitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TransmissionType {
    /// Sync: acyclic — only transmitted on request via RTR.
    SyncAcyclic = 0,
    /// Sync: every SYNC — transmitted on every SYNC frame.
    SyncEvery = 1,
    /// Sync: every Nth SYNC (N = 2..240).
    SyncN(u8),
    /// RTR only (synchronous).
    RtrSync = 252,
    /// RTR only (asynchronous).
    RtrAsync = 253,
    /// Event-driven (manufacturer-specific).
    EventManufacturer = 254,
    /// Event-driven (device profile).
    EventProfile = 255,
}

impl TransmissionType {
    /// Parse from raw u8 value.
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::SyncAcyclic),
            1 => Some(Self::SyncEvery),
            2..=240 => Some(Self::SyncN(val)),
            252 => Some(Self::RtrSync),
            253 => Some(Self::RtrAsync),
            254 => Some(Self::EventManufacturer),
            255 => Some(Self::EventProfile),
            _ => None,
        }
    }

    /// Convert to raw u8 value.
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::SyncAcyclic => 0,
            Self::SyncEvery => 1,
            Self::SyncN(n) => *n,
            Self::RtrSync => 252,
            Self::RtrAsync => 253,
            Self::EventManufacturer => 254,
            Self::EventProfile => 255,
        }
    }

    /// Check if this transmission type is synchronous.
    pub fn is_sync(&self) -> bool {
        matches!(self, Self::SyncAcyclic | Self::SyncEvery | Self::SyncN(_))
    }

    /// Check if this transmission type is event-driven.
    pub fn is_event_driven(&self) -> bool {
        matches!(self, Self::EventManufacturer | Self::EventProfile)
    }
}

/// PDO mapping entry (Index + Subindex + Bit Length).
///
/// Each mapped object in a PDO is described by a 32-bit value:
/// - bits 31..16: Index (16-bit)
/// - bits 15..8:  Subindex (8-bit)
/// - bits 7..0:   Bit length (8-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdoMapping {
    /// OD index of the mapped object.
    pub index: u16,
    /// OD subindex of the mapped object.
    pub subindex: u8,
    /// Number of bits to map (1, 2, 4, 8, 16, 32, or 64).
    pub bit_length: u8,
}

impl PdoMapping {
    /// Create a new mapping entry.
    pub fn new(index: u16, subindex: u8, bit_length: u8) -> Self {
        Self {
            index,
            subindex,
            bit_length,
        }
    }

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

    /// Get the byte length (rounded up).
    pub fn byte_length(&self) -> u8 {
        self.bit_length.div_ceil(8)
    }
}

/// Parsed PDO data with addressing information.
#[derive(Debug, Clone, PartialEq)]
pub struct PdoData {
    /// COB-ID of the PDO frame.
    pub cob_id: u16,
    /// Node ID of the PDO producer/consumer.
    pub node_id: u8,
    /// PDO number (1-4).
    pub pdo_number: u8,
    /// Direction (TPDO or RPDO).
    pub direction: PdoDirection,
    /// Raw data bytes (up to 8 bytes for classic CAN).
    pub data: [u8; 8],
}

/// Parse a CANOpen frame as a PDO.
///
/// Returns `None` if the frame's COB-ID is not in the PDO range (0x180..=0x57F).
pub fn parse_pdo(frame: &CanOpenFrame) -> Option<PdoData> {
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

    Some(PdoData {
        cob_id: id,
        node_id,
        pdo_number,
        direction,
        data: frame.data,
    })
}

/// Get the OD index for PDO communication parameters.
///
/// - RPDO: 0x1400 + (pdo_number - 1)
/// - TPDO: 0x1800 + (pdo_number - 1)
pub fn pdo_comm_index(pdo_number: u8, direction: PdoDirection) -> Option<u16> {
    match (pdo_number, direction) {
        (1..=4, PdoDirection::Rpdo) => Some(0x1400 + (pdo_number - 1) as u16),
        (1..=4, PdoDirection::Tpdo) => Some(0x1800 + (pdo_number - 1) as u16),
        _ => None,
    }
}

/// Get the OD index for PDO mapping parameters.
///
/// - RPDO: 0x1600 + (pdo_number - 1)
/// - TPDO: 0x1A00 + (pdo_number - 1)
pub fn pdo_map_index(pdo_number: u8, direction: PdoDirection) -> Option<u16> {
    match (pdo_number, direction) {
        (1..=4, PdoDirection::Rpdo) => Some(0x1600 + (pdo_number - 1) as u16),
        (1..=4, PdoDirection::Tpdo) => Some(0x1A00 + (pdo_number - 1) as u16),
        _ => None,
    }
}

/// Validate PDO mapping entries.
///
/// Returns Ok(()) if valid, Err(description) if invalid.
pub fn validate_mapping(mappings: &[PdoMapping]) -> Result<(), String> {
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

// === PDO Packing/Unpacking ===

/// Errors that can occur during PDO packing/unpacking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdoError {
    /// Mappings and values/types arrays have different lengths.
    LengthMismatch { mappings: usize, values: usize },
    /// Value bytes too short for the mapping's bit length.
    ValueTooShort { index: u16, subindex: u8 },
    /// Total mapped data exceeds 64 bits.
    DataExceeds64Bits { total_bits: u16 },
    /// Non-byte-aligned mappings not yet supported.
    NonByteAligned { index: u16, subindex: u8, bit_offset: u16 },
    /// Failed to decode a value from PDO data.
    DecodeFailed { index: u16, subindex: u8 },
}

impl std::fmt::Display for PdoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LengthMismatch { mappings, values } => {
                write!(f, "PDO length mismatch: {} mappings, {} values", mappings, values)
            }
            Self::ValueTooShort { index, subindex } => {
                write!(f, "value too short for mapping at {:04X}:{:02X}", index, subindex)
            }
            Self::DataExceeds64Bits { total_bits } => {
                write!(f, "PDO data exceeds 64 bits ({} bits)", total_bits)
            }
            Self::NonByteAligned { index, subindex, bit_offset } => {
                write!(f, "non-byte-aligned mapping at {:04X}:{:02X} (bit offset {})", index, subindex, bit_offset)
            }
            Self::DecodeFailed { index, subindex } => {
                write!(f, "failed to decode PDO value at {:04X}:{:02X}", index, subindex)
            }
        }
    }
}

impl std::error::Error for PdoError {}

/// Pack OD values into a PDO data frame according to mappings.
///
/// All mappings must be byte-aligned (bit_length must be a multiple of 8).
/// Returns an 8-byte array ready for CAN transmission.
///
/// # Example
/// ```
/// use opencan_canopen_core::pdo::{PdoMapping, pack_pdo};
/// use opencan_canopen_core::od::OdValue;
///
/// let mappings = vec![
///     PdoMapping::new(0x6041, 0, 16),  // Status Word (2 bytes)
///     PdoMapping::new(0x6064, 0, 32),  // Position (4 bytes)
/// ];
/// let values = vec![
///     OdValue::Unsigned16(0x0027),
///     OdValue::Unsigned32(12345),
/// ];
/// let data = pack_pdo(&mappings, &values).unwrap();
/// assert_eq!(&data[0..2], &0x0027u16.to_le_bytes());
/// assert_eq!(&data[2..6], &12345u32.to_le_bytes());
/// ```
pub fn pack_pdo(mappings: &[PdoMapping], values: &[OdValue]) -> Result<[u8; 8], PdoError> {
    if mappings.len() != values.len() {
        return Err(PdoError::LengthMismatch {
            mappings: mappings.len(),
            values: values.len(),
        });
    }

    let mut data = [0u8; 8];
    let mut bit_offset: u16 = 0;

    for (mapping, value) in mappings.iter().zip(values) {
        // Only byte-aligned packing is supported
        if !bit_offset.is_multiple_of(8) || !mapping.bit_length.is_multiple_of(8) {
            return Err(PdoError::NonByteAligned {
                index: mapping.index,
                subindex: mapping.subindex,
                bit_offset,
            });
        }

        if bit_offset + mapping.bit_length as u16 > 64 {
            return Err(PdoError::DataExceeds64Bits {
                total_bits: bit_offset + mapping.bit_length as u16,
            });
        }

        let bytes = value.to_bytes();
        let byte_len = (mapping.bit_length / 8) as usize;
        if bytes.len() < byte_len {
            return Err(PdoError::ValueTooShort {
                index: mapping.index,
                subindex: mapping.subindex,
            });
        }

        let byte_start = (bit_offset / 8) as usize;
        data[byte_start..byte_start + byte_len].copy_from_slice(&bytes[..byte_len]);
        bit_offset += mapping.bit_length as u16;
    }

    Ok(data)
}

/// Unpack PDO data into OdValues according to mappings and data types.
///
/// All mappings must be byte-aligned. The `types` slice must have the same
/// length as `mappings` and contain the DataType for each mapped object.
///
/// # Example
/// ```
/// use opencan_canopen_core::pdo::{PdoMapping, unpack_pdo};
/// use opencan_canopen_core::od::{OdValue, DataType};
///
/// let mappings = vec![
///     PdoMapping::new(0x6041, 0, 16),
///     PdoMapping::new(0x6064, 0, 32),
/// ];
/// let types = vec![DataType::Unsigned16, DataType::Unsigned32];
/// let mut data = [0u8; 8];
/// data[0..2].copy_from_slice(&0x0027u16.to_le_bytes());
/// data[2..6].copy_from_slice(&12345u32.to_le_bytes());
///
/// let values = unpack_pdo(&mappings, &data, &types).unwrap();
/// assert_eq!(values[0], OdValue::Unsigned16(0x0027));
/// assert_eq!(values[1], OdValue::Unsigned32(12345));
/// ```
pub fn unpack_pdo(
    mappings: &[PdoMapping],
    data: &[u8; 8],
    types: &[DataType],
) -> Result<Vec<OdValue>, PdoError> {
    if mappings.len() != types.len() {
        return Err(PdoError::LengthMismatch {
            mappings: mappings.len(),
            values: types.len(),
        });
    }

    let mut values = Vec::with_capacity(mappings.len());
    let mut bit_offset: u16 = 0;

    for (mapping, data_type) in mappings.iter().zip(types) {
        if !bit_offset.is_multiple_of(8) || !mapping.bit_length.is_multiple_of(8) {
            return Err(PdoError::NonByteAligned {
                index: mapping.index,
                subindex: mapping.subindex,
                bit_offset,
            });
        }

        let byte_start = (bit_offset / 8) as usize;
        let byte_len = (mapping.bit_length / 8) as usize;

        if byte_start + byte_len > 8 {
            return Err(PdoError::DataExceeds64Bits {
                total_bits: bit_offset + mapping.bit_length as u16,
            });
        }

        let slice = &data[byte_start..byte_start + byte_len];
        let value = OdValue::from_bytes(*data_type, slice).ok_or(PdoError::DecodeFailed {
            index: mapping.index,
            subindex: mapping.subindex,
        })?;
        values.push(value);
        bit_offset += mapping.bit_length as u16;
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdo_direction_reverse() {
        assert_eq!(PdoDirection::Tpdo.reverse(), PdoDirection::Rpdo);
        assert_eq!(PdoDirection::Rpdo.reverse(), PdoDirection::Tpdo);
    }

    #[test]
    fn test_transmission_type_roundtrip() {
        let types = [
            TransmissionType::SyncAcyclic,
            TransmissionType::SyncEvery,
            TransmissionType::SyncN(10),
            TransmissionType::RtrSync,
            TransmissionType::RtrAsync,
            TransmissionType::EventManufacturer,
            TransmissionType::EventProfile,
        ];

        for tt in &types {
            let raw = tt.to_u8();
            let decoded = TransmissionType::from_u8(raw).unwrap();
            assert_eq!(*tt, decoded);
        }
    }

    #[test]
    fn test_transmission_type_properties() {
        assert!(TransmissionType::SyncEvery.is_sync());
        assert!(TransmissionType::SyncN(5).is_sync());
        assert!(!TransmissionType::EventProfile.is_sync());

        assert!(TransmissionType::EventManufacturer.is_event_driven());
        assert!(TransmissionType::EventProfile.is_event_driven());
        assert!(!TransmissionType::SyncEvery.is_event_driven());
    }

    #[test]
    fn test_pdo_mapping_roundtrip() {
        let mapping = PdoMapping::new(0x6041, 0, 16);
        let val = mapping.to_u32();
        assert_eq!(val, 0x60410010);

        let decoded = PdoMapping::from_u32(val);
        assert_eq!(decoded, mapping);
    }

    #[test]
    fn test_pdo_mapping_byte_length() {
        assert_eq!(PdoMapping::new(0, 0, 1).byte_length(), 1);
        assert_eq!(PdoMapping::new(0, 0, 8).byte_length(), 1);
        assert_eq!(PdoMapping::new(0, 0, 9).byte_length(), 2);
        assert_eq!(PdoMapping::new(0, 0, 16).byte_length(), 2);
        assert_eq!(PdoMapping::new(0, 0, 32).byte_length(), 4);
    }

    #[test]
    fn test_parse_tpdo1() {
        let frame = CanOpenFrame::new(0x185, [1, 2, 3, 4, 5, 6, 7, 8]);
        let pdo = parse_pdo(&frame).unwrap();
        assert_eq!(pdo.node_id, 5);
        assert_eq!(pdo.pdo_number, 1);
        assert_eq!(pdo.direction, PdoDirection::Tpdo);
        assert_eq!(pdo.data, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_parse_rpdo2() {
        let frame = CanOpenFrame::new(0x303, [0xAA; 8]);
        let pdo = parse_pdo(&frame).unwrap();
        assert_eq!(pdo.node_id, 3);
        assert_eq!(pdo.pdo_number, 2);
        assert_eq!(pdo.direction, PdoDirection::Rpdo);
    }

    #[test]
    fn test_parse_non_pdo() {
        let frame = CanOpenFrame::new(0x603, [0; 8]); // SDO client
        assert!(parse_pdo(&frame).is_none());
    }

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

    #[test]
    fn test_pack_pdo_basic() {
        let mappings = vec![
            PdoMapping::new(0x6041, 0, 16),  // Status Word (2 bytes)
            PdoMapping::new(0x6064, 0, 32),  // Position (4 bytes)
        ];
        let values = vec![
            OdValue::Unsigned16(0x0027),
            OdValue::Unsigned32(12345),
        ];
        let data = pack_pdo(&mappings, &values).unwrap();
        assert_eq!(&data[0..2], &0x0027u16.to_le_bytes());
        assert_eq!(&data[2..6], &12345u32.to_le_bytes());
        assert_eq!(&data[6..8], &[0, 0]); // unused bytes are zero
    }

    #[test]
    fn test_pack_pdo_length_mismatch() {
        let mappings = vec![PdoMapping::new(0x6041, 0, 16)];
        let values: Vec<OdValue> = vec![];
        assert!(matches!(
            pack_pdo(&mappings, &values),
            Err(PdoError::LengthMismatch { .. })
        ));
    }

    #[test]
    fn test_pack_pdo_exceeds_64_bits() {
        let mappings = vec![
            PdoMapping::new(0x6041, 0, 32),
            PdoMapping::new(0x6064, 0, 32),
            PdoMapping::new(0x606C, 0, 8),
        ];
        let values = vec![
            OdValue::Unsigned32(0),
            OdValue::Unsigned32(0),
            OdValue::Unsigned8(0),
        ];
        assert!(matches!(
            pack_pdo(&mappings, &values),
            Err(PdoError::DataExceeds64Bits { .. })
        ));
    }

    #[test]
    fn test_unpack_pdo_basic() {
        let mappings = vec![
            PdoMapping::new(0x6041, 0, 16),
            PdoMapping::new(0x6064, 0, 32),
        ];
        let types = vec![DataType::Unsigned16, DataType::Unsigned32];
        let mut data = [0u8; 8];
        data[0..2].copy_from_slice(&0x0027u16.to_le_bytes());
        data[2..6].copy_from_slice(&12345u32.to_le_bytes());

        let values = unpack_pdo(&mappings, &data, &types).unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], OdValue::Unsigned16(0x0027));
        assert_eq!(values[1], OdValue::Unsigned32(12345));
    }

    #[test]
    fn test_unpack_pdo_roundtrip() {
        // Pack then unpack should give the same values
        let mappings = vec![
            PdoMapping::new(0x6041, 0, 16),
            PdoMapping::new(0x6064, 0, 32),
            PdoMapping::new(0x60FD, 0, 16),
        ];
        let values = vec![
            OdValue::Unsigned16(0x0027),
            OdValue::Unsigned32(12345),
            OdValue::Unsigned16(0xABCD),
        ];
        let data = pack_pdo(&mappings, &values).unwrap();

        let types = vec![DataType::Unsigned16, DataType::Unsigned32, DataType::Unsigned16];
        let unpacked = unpack_pdo(&mappings, &data, &types).unwrap();
        assert_eq!(unpacked, values);
    }

    #[test]
    fn test_unpack_pdo_length_mismatch() {
        let mappings = vec![PdoMapping::new(0x6041, 0, 16)];
        let types: Vec<DataType> = vec![];
        let data = [0u8; 8];
        assert!(matches!(
            unpack_pdo(&mappings, &data, &types),
            Err(PdoError::LengthMismatch { .. })
        ));
    }
}
