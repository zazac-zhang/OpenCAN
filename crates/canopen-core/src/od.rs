//! Object Dictionary types and trait.
//!
//! The Object Dictionary (OD) is the core data structure in CANOpen.
//! It organizes all device parameters as a collection of objects,
//! each identified by an Index (16-bit) and Subindex (8-bit).

/// CANOpen data types (DS301 Table 37).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum DataType {
    Boolean       = 0x0001,
    Integer8      = 0x0002,
    Integer16     = 0x0003,
    Integer32     = 0x0004,
    Unsigned8     = 0x0005,
    Unsigned16    = 0x0006,
    Unsigned32    = 0x0007,
    Real32        = 0x0008,
    VisibleString = 0x0009,
    OctetString   = 0x000A,
    UnicodeString = 0x000B,
    TimeOfDay     = 0x000C,
    TimeDifference = 0x000D,
    Domain        = 0x000F,
    Integer24     = 0x0010,
    Real64        = 0x0011,
    Integer40     = 0x0012,
    Integer48     = 0x0013,
    Integer56     = 0x0014,
    Integer64     = 0x0015,
    Unsigned24    = 0x0016,
    Unsigned40    = 0x0017,
    Unsigned48    = 0x0018,
    Unsigned56    = 0x0019,
    Unsigned64    = 0x001A,
}

impl DataType {
    pub fn from_u16(val: u16) -> Option<Self> {
        match val {
            0x0001 => Some(Self::Boolean),
            0x0002 => Some(Self::Integer8),
            0x0003 => Some(Self::Integer16),
            0x0004 => Some(Self::Integer32),
            0x0005 => Some(Self::Unsigned8),
            0x0006 => Some(Self::Unsigned16),
            0x0007 => Some(Self::Unsigned32),
            0x0008 => Some(Self::Real32),
            0x0009 => Some(Self::VisibleString),
            0x000A => Some(Self::OctetString),
            0x000B => Some(Self::UnicodeString),
            0x000C => Some(Self::TimeOfDay),
            0x000D => Some(Self::TimeDifference),
            0x000F => Some(Self::Domain),
            0x0010 => Some(Self::Integer24),
            0x0011 => Some(Self::Real64),
            0x0012 => Some(Self::Integer40),
            0x0013 => Some(Self::Integer48),
            0x0014 => Some(Self::Integer56),
            0x0015 => Some(Self::Integer64),
            0x0016 => Some(Self::Unsigned24),
            0x0017 => Some(Self::Unsigned40),
            0x0018 => Some(Self::Unsigned48),
            0x0019 => Some(Self::Unsigned56),
            0x001A => Some(Self::Unsigned64),
            _ => None,
        }
    }
}

/// Access type for OD entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Constant,
}

/// Object type (DS301).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    /// Simple variable (single subindex 0).
    Var,
    /// Array (multiple subindices, same data type).
    Array,
    /// Record (multiple subindices, potentially different data types).
    Record,
}

/// OD entry metadata.
#[derive(Debug, Clone)]
pub struct EntryInfo {
    pub index: u16,
    pub subindex: u8,
    pub object_type: ObjectType,
    pub data_type: DataType,
    pub access: AccessType,
    pub default_value: Option<OdValue>,
    pub name: String,
}

/// OD value types.
#[derive(Debug, Clone, PartialEq)]
pub enum OdValue {
    Boolean(bool),
    Integer8(i8),
    Integer16(i16),
    Integer32(i32),
    Integer64(i64),
    Unsigned8(u8),
    Unsigned16(u16),
    Unsigned32(u32),
    Unsigned64(u64),
    Real32(f32),
    Real64(f64),
    VisibleString(String),
    OctetString(Vec<u8>),
    Domain(Vec<u8>),
}

impl OdValue {
    /// Get the data type of this value.
    pub fn data_type(&self) -> DataType {
        match self {
            Self::Boolean(_) => DataType::Boolean,
            Self::Integer8(_) => DataType::Integer8,
            Self::Integer16(_) => DataType::Integer16,
            Self::Integer32(_) => DataType::Integer32,
            Self::Integer64(_) => DataType::Integer64,
            Self::Unsigned8(_) => DataType::Unsigned8,
            Self::Unsigned16(_) => DataType::Unsigned16,
            Self::Unsigned32(_) => DataType::Unsigned32,
            Self::Unsigned64(_) => DataType::Unsigned64,
            Self::Real32(_) => DataType::Real32,
            Self::Real64(_) => DataType::Real64,
            Self::VisibleString(_) => DataType::VisibleString,
            Self::OctetString(_) => DataType::OctetString,
            Self::Domain(_) => DataType::Domain,
        }
    }

    /// Encode value to bytes (little-endian).
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Boolean(v) => vec![*v as u8],
            Self::Integer8(v) => vec![*v as u8],
            Self::Integer16(v) => v.to_le_bytes().to_vec(),
            Self::Integer32(v) => v.to_le_bytes().to_vec(),
            Self::Integer64(v) => v.to_le_bytes().to_vec(),
            Self::Unsigned8(v) => vec![*v],
            Self::Unsigned16(v) => v.to_le_bytes().to_vec(),
            Self::Unsigned32(v) => v.to_le_bytes().to_vec(),
            Self::Unsigned64(v) => v.to_le_bytes().to_vec(),
            Self::Real32(v) => v.to_le_bytes().to_vec(),
            Self::Real64(v) => v.to_le_bytes().to_vec(),
            Self::VisibleString(s) => s.as_bytes().to_vec(),
            Self::OctetString(b) | Self::Domain(b) => b.clone(),
        }
    }

    /// Decode value from bytes.
    pub fn from_bytes(data_type: DataType, data: &[u8]) -> Option<Self> {
        match data_type {
            DataType::Boolean => data.first().map(|&b| Self::Boolean(b != 0)),
            DataType::Integer8 => data.first().map(|&b| Self::Integer8(b as i8)),
            DataType::Integer16 => data.get(..2).map(|b| Self::Integer16(i16::from_le_bytes([b[0], b[1]]))),
            DataType::Integer32 => data.get(..4).map(|b| Self::Integer32(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))),
            DataType::Integer64 => data.get(..8).map(|b| Self::Integer64(i64::from_le_bytes(b.try_into().unwrap()))),
            DataType::Unsigned8 => data.first().map(|&b| Self::Unsigned8(b)),
            DataType::Unsigned16 => data.get(..2).map(|b| Self::Unsigned16(u16::from_le_bytes([b[0], b[1]]))),
            DataType::Unsigned32 => data.get(..4).map(|b| Self::Unsigned32(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))),
            DataType::Unsigned64 => data.get(..8).map(|b| Self::Unsigned64(u64::from_le_bytes(b.try_into().unwrap()))),
            DataType::Real32 => data.get(..4).map(|b| Self::Real32(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))),
            DataType::Real64 => data.get(..8).map(|b| Self::Real64(f64::from_le_bytes(b.try_into().unwrap()))),
            DataType::VisibleString => Some(Self::VisibleString(String::from_utf8_lossy(data).to_string())),
            DataType::Domain => Some(Self::Domain(data.to_vec())),
            _ => None,
        }
    }
}

// === Type conversions for OdValue ===

impl From<bool> for OdValue {
    fn from(v: bool) -> Self { Self::Boolean(v) }
}
impl From<i8> for OdValue {
    fn from(v: i8) -> Self { Self::Integer8(v) }
}
impl From<i16> for OdValue {
    fn from(v: i16) -> Self { Self::Integer16(v) }
}
impl From<i32> for OdValue {
    fn from(v: i32) -> Self { Self::Integer32(v) }
}
impl From<i64> for OdValue {
    fn from(v: i64) -> Self { Self::Integer64(v) }
}
impl From<u8> for OdValue {
    fn from(v: u8) -> Self { Self::Unsigned8(v) }
}
impl From<u16> for OdValue {
    fn from(v: u16) -> Self { Self::Unsigned16(v) }
}
impl From<u32> for OdValue {
    fn from(v: u32) -> Self { Self::Unsigned32(v) }
}
impl From<u64> for OdValue {
    fn from(v: u64) -> Self { Self::Unsigned64(v) }
}
impl From<f32> for OdValue {
    fn from(v: f32) -> Self { Self::Real32(v) }
}
impl From<f64> for OdValue {
    fn from(v: f64) -> Self { Self::Real64(v) }
}
impl From<String> for OdValue {
    fn from(v: String) -> Self { Self::VisibleString(v) }
}
impl From<Vec<u8>> for OdValue {
    fn from(v: Vec<u8>) -> Self { Self::Domain(v) }
}

/// Try to extract a concrete type from OdValue.
impl TryFrom<OdValue> for bool {
    type Error = crate::error::OdError;
    fn try_from(v: OdValue) -> Result<Self, Self::Error> {
        match v {
            OdValue::Boolean(b) => Ok(b),
            _ => Err(crate::error::OdError::TypeMismatch {
                expected: DataType::Boolean,
                actual: v.data_type(),
            }),
        }
    }
}

impl TryFrom<OdValue> for u8 {
    type Error = crate::error::OdError;
    fn try_from(v: OdValue) -> Result<Self, Self::Error> {
        match v {
            OdValue::Unsigned8(v) => Ok(v),
            _ => Err(crate::error::OdError::TypeMismatch {
                expected: DataType::Unsigned8,
                actual: v.data_type(),
            }),
        }
    }
}

impl TryFrom<OdValue> for u16 {
    type Error = crate::error::OdError;
    fn try_from(v: OdValue) -> Result<Self, Self::Error> {
        match v {
            OdValue::Unsigned16(v) => Ok(v),
            _ => Err(crate::error::OdError::TypeMismatch {
                expected: DataType::Unsigned16,
                actual: v.data_type(),
            }),
        }
    }
}

impl TryFrom<OdValue> for u32 {
    type Error = crate::error::OdError;
    fn try_from(v: OdValue) -> Result<Self, Self::Error> {
        match v {
            OdValue::Unsigned32(v) => Ok(v),
            _ => Err(crate::error::OdError::TypeMismatch {
                expected: DataType::Unsigned32,
                actual: v.data_type(),
            }),
        }
    }
}

impl TryFrom<OdValue> for i16 {
    type Error = crate::error::OdError;
    fn try_from(v: OdValue) -> Result<Self, Self::Error> {
        match v {
            OdValue::Integer16(v) => Ok(v),
            _ => Err(crate::error::OdError::TypeMismatch {
                expected: DataType::Integer16,
                actual: v.data_type(),
            }),
        }
    }
}

impl TryFrom<OdValue> for i32 {
    type Error = crate::error::OdError;
    fn try_from(v: OdValue) -> Result<Self, Self::Error> {
        match v {
            OdValue::Integer32(v) => Ok(v),
            _ => Err(crate::error::OdError::TypeMismatch {
                expected: DataType::Integer32,
                actual: v.data_type(),
            }),
        }
    }
}

/// Object Dictionary trait — access to OD entries.
pub trait ObjectDictionary: Send {
    fn read(&self, index: u16, subindex: u8) -> Result<OdValue, crate::error::OdError>;
    fn write(&mut self, index: u16, subindex: u8, value: OdValue) -> Result<(), crate::error::OdError>;
    fn entry_info(&self, index: u16, subindex: u8) -> Result<EntryInfo, crate::error::OdError>;
}

/// CAN Driver trait — protocol-level CAN I/O.
///
/// This is the trait that the CANOpen protocol stack uses internally.
/// It operates on CanOpenFrame (COB-ID + 8-byte data).
///
/// For physical CAN bus access, see can-traits::CanBus.
/// A CanDriverAdapter bridges the two.
pub trait CanDriver: Send {
    fn send(&mut self, frame: &crate::frame::CanOpenFrame) -> Result<(), crate::error::CanOpenError>;
    fn recv(&mut self) -> Result<crate::frame::CanOpenFrame, crate::error::CanOpenError>;
    fn recv_async(&mut self) -> impl std::future::Future<Output = Result<crate::frame::CanOpenFrame, crate::error::CanOpenError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_od_value_roundtrip() {
        let val = OdValue::Unsigned32(0x00020192);
        let bytes = val.to_bytes();
        assert_eq!(bytes, [0x92, 0x01, 0x02, 0x00]);

        let decoded = OdValue::from_bytes(DataType::Unsigned32, &bytes).unwrap();
        assert_eq!(decoded, OdValue::Unsigned32(0x00020192));
    }

    #[test]
    fn test_od_value_conversions() {
        let val: OdValue = 42u16.into();
        assert_eq!(val.data_type(), DataType::Unsigned16);

        let extracted: u16 = val.try_into().unwrap();
        assert_eq!(extracted, 42u16);
    }

    #[test]
    fn test_od_value_type_mismatch() {
        let val = OdValue::Unsigned16(42);
        let result: Result<u32, _> = val.try_into();
        assert!(result.is_err());
    }
}
