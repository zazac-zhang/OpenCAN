//! Object Dictionary types and trait.
//!
//! The Object Dictionary (OD) is the core data structure in CANOpen.
//! It organizes all device parameters as a collection of objects,
//! each identified by an Index (16-bit) and Subindex (8-bit).

/// CANOpen data types (DS301 Table 37).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
#[non_exhaustive]
pub enum DataType {
    Boolean = 0x0001,
    Integer8 = 0x0002,
    Integer16 = 0x0003,
    Integer32 = 0x0004,
    Unsigned8 = 0x0005,
    Unsigned16 = 0x0006,
    Unsigned32 = 0x0007,
    Real32 = 0x0008,
    VisibleString = 0x0009,
    OctetString = 0x000A,
    UnicodeString = 0x000B,
    TimeOfDay = 0x000C,
    TimeDifference = 0x000D,
    Domain = 0x000F,
    Integer24 = 0x0010,
    Real64 = 0x0011,
    Integer40 = 0x0012,
    Integer48 = 0x0013,
    Integer56 = 0x0014,
    Integer64 = 0x0015,
    Unsigned24 = 0x0016,
    Unsigned40 = 0x0017,
    Unsigned48 = 0x0018,
    Unsigned56 = 0x0019,
    Unsigned64 = 0x001A,
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

    /// Get the fixed byte size for this data type.
    ///
    /// Returns `None` for variable-length types (VisibleString, OctetString,
    /// UnicodeString, TimeOfDay, TimeDifference, Domain).
    pub fn byte_size(&self) -> Option<usize> {
        match self {
            Self::Boolean => Some(1),
            Self::Integer8 | Self::Unsigned8 => Some(1),
            Self::Integer16 | Self::Unsigned16 => Some(2),
            Self::Integer24 | Self::Unsigned24 => Some(3),
            Self::Integer32 | Self::Unsigned32 | Self::Real32 => Some(4),
            Self::Integer40 | Self::Unsigned40 => Some(5),
            Self::Integer48 | Self::Unsigned48 => Some(6),
            Self::Integer56 | Self::Unsigned56 => Some(7),
            Self::Integer64 | Self::Unsigned64 | Self::Real64 => Some(8),
            // Variable-length types and unknown future types
            _ => None,
        }
    }

    /// Check if this data type is a numeric type (integer or real).
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Boolean
                | Self::Integer8
                | Self::Integer16
                | Self::Integer24
                | Self::Integer32
                | Self::Integer40
                | Self::Integer48
                | Self::Integer56
                | Self::Integer64
                | Self::Unsigned8
                | Self::Unsigned16
                | Self::Unsigned24
                | Self::Unsigned32
                | Self::Unsigned40
                | Self::Unsigned48
                | Self::Unsigned56
                | Self::Unsigned64
                | Self::Real32
                | Self::Real64
        )
    }

    /// Check if this data type is a signed integer type.
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Self::Integer8
                | Self::Integer16
                | Self::Integer24
                | Self::Integer32
                | Self::Integer40
                | Self::Integer48
                | Self::Integer56
                | Self::Integer64
        )
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

impl std::fmt::Display for AccessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "ro"),
            Self::WriteOnly => write!(f, "wo"),
            Self::ReadWrite => write!(f, "rw"),
            Self::Constant => write!(f, "const"),
        }
    }
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

impl std::fmt::Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var => write!(f, "VAR"),
            Self::Array => write!(f, "ARRAY"),
            Self::Record => write!(f, "RECORD"),
        }
    }
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
    /// No value (entry exists but is empty, or SDO returned no data).
    None,
    Boolean(bool),
    Integer8(i8),
    Integer16(i16),
    /// 24-bit signed integer (stored as i32, serialized as 3 bytes LE).
    Integer24(i32),
    Integer32(i32),
    /// 40-bit signed integer (stored as i64, serialized as 5 bytes LE).
    Integer40(i64),
    /// 48-bit signed integer (stored as i64, serialized as 6 bytes LE).
    Integer48(i64),
    /// 56-bit signed integer (stored as i64, serialized as 7 bytes LE).
    Integer56(i64),
    Integer64(i64),
    Unsigned8(u8),
    Unsigned16(u16),
    /// 24-bit unsigned integer (stored as u32, serialized as 3 bytes LE).
    Unsigned24(u32),
    Unsigned32(u32),
    /// 40-bit unsigned integer (stored as u64, serialized as 5 bytes LE).
    Unsigned40(u64),
    /// 48-bit unsigned integer (stored as u64, serialized as 6 bytes LE).
    Unsigned48(u64),
    /// 56-bit unsigned integer (stored as u64, serialized as 7 bytes LE).
    Unsigned56(u64),
    Unsigned64(u64),
    Real32(f32),
    Real64(f64),
    VisibleString(String),
    OctetString(Vec<u8>),
    Domain(Vec<u8>),
}

impl OdValue {
    /// Get the data type of this value.
    pub fn data_type(&self) -> Option<DataType> {
        match self {
            Self::None => None,
            Self::Boolean(_) => Some(DataType::Boolean),
            Self::Integer8(_) => Some(DataType::Integer8),
            Self::Integer16(_) => Some(DataType::Integer16),
            Self::Integer24(_) => Some(DataType::Integer24),
            Self::Integer32(_) => Some(DataType::Integer32),
            Self::Integer40(_) => Some(DataType::Integer40),
            Self::Integer48(_) => Some(DataType::Integer48),
            Self::Integer56(_) => Some(DataType::Integer56),
            Self::Integer64(_) => Some(DataType::Integer64),
            Self::Unsigned8(_) => Some(DataType::Unsigned8),
            Self::Unsigned16(_) => Some(DataType::Unsigned16),
            Self::Unsigned24(_) => Some(DataType::Unsigned24),
            Self::Unsigned32(_) => Some(DataType::Unsigned32),
            Self::Unsigned40(_) => Some(DataType::Unsigned40),
            Self::Unsigned48(_) => Some(DataType::Unsigned48),
            Self::Unsigned56(_) => Some(DataType::Unsigned56),
            Self::Unsigned64(_) => Some(DataType::Unsigned64),
            Self::Real32(_) => Some(DataType::Real32),
            Self::Real64(_) => Some(DataType::Real64),
            Self::VisibleString(_) => Some(DataType::VisibleString),
            Self::OctetString(_) => Some(DataType::OctetString),
            Self::Domain(_) => Some(DataType::Domain),
        }
    }

    /// Encode value to bytes (little-endian).
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::None => Vec::new(),
            Self::Boolean(v) => vec![*v as u8],
            Self::Integer8(v) => vec![*v as u8],
            Self::Integer16(v) => v.to_le_bytes().to_vec(),
            Self::Integer24(v) => {
                let bytes = v.to_le_bytes();
                vec![bytes[0], bytes[1], bytes[2]]
            }
            Self::Integer32(v) => v.to_le_bytes().to_vec(),
            Self::Integer40(v) => {
                let bytes = v.to_le_bytes();
                vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]]
            }
            Self::Integer48(v) => {
                let bytes = v.to_le_bytes();
                vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]
            }
            Self::Integer56(v) => {
                let bytes = v.to_le_bytes();
                vec![
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                ]
            }
            Self::Integer64(v) => v.to_le_bytes().to_vec(),
            Self::Unsigned8(v) => vec![*v],
            Self::Unsigned16(v) => v.to_le_bytes().to_vec(),
            Self::Unsigned24(v) => {
                let bytes = v.to_le_bytes();
                vec![bytes[0], bytes[1], bytes[2]]
            }
            Self::Unsigned32(v) => v.to_le_bytes().to_vec(),
            Self::Unsigned40(v) => {
                let bytes = v.to_le_bytes();
                vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]]
            }
            Self::Unsigned48(v) => {
                let bytes = v.to_le_bytes();
                vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]
            }
            Self::Unsigned56(v) => {
                let bytes = v.to_le_bytes();
                vec![
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                ]
            }
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
            DataType::Integer16 => data
                .get(..2)
                .map(|b| Self::Integer16(i16::from_le_bytes([b[0], b[1]]))),
            DataType::Integer24 => data.get(..3).map(|b| {
                let raw = i32::from_le_bytes([b[0], b[1], b[2], 0]);
                Self::Integer24((raw << 8) >> 8) // sign-extend from 24-bit
            }),
            DataType::Integer32 => data
                .get(..4)
                .map(|b| Self::Integer32(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))),
            DataType::Integer40 => data.get(..5).map(|b| {
                let raw = i64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], 0, 0, 0]);
                Self::Integer40((raw << 24) >> 24) // sign-extend from 40-bit
            }),
            DataType::Integer48 => data.get(..6).map(|b| {
                let raw = i64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], 0, 0]);
                Self::Integer48((raw << 16) >> 16) // sign-extend from 48-bit
            }),
            DataType::Integer56 => data.get(..7).map(|b| {
                let raw = i64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], 0]);
                Self::Integer56((raw << 8) >> 8) // sign-extend from 56-bit
            }),
            DataType::Integer64 => data
                .get(..8)
                .map(|b| Self::Integer64(i64::from_le_bytes(b.try_into().unwrap()))),
            DataType::Unsigned8 => data.first().map(|&b| Self::Unsigned8(b)),
            DataType::Unsigned16 => data
                .get(..2)
                .map(|b| Self::Unsigned16(u16::from_le_bytes([b[0], b[1]]))),
            DataType::Unsigned24 => data
                .get(..3)
                .map(|b| Self::Unsigned24(u32::from_le_bytes([b[0], b[1], b[2], 0]))),
            DataType::Unsigned32 => data
                .get(..4)
                .map(|b| Self::Unsigned32(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))),
            DataType::Unsigned40 => data.get(..5).map(|b| {
                Self::Unsigned40(u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], 0, 0, 0]))
            }),
            DataType::Unsigned48 => data.get(..6).map(|b| {
                Self::Unsigned48(u64::from_le_bytes([
                    b[0], b[1], b[2], b[3], b[4], b[5], 0, 0,
                ]))
            }),
            DataType::Unsigned56 => data.get(..7).map(|b| {
                Self::Unsigned56(u64::from_le_bytes([
                    b[0], b[1], b[2], b[3], b[4], b[5], b[6], 0,
                ]))
            }),
            DataType::Unsigned64 => data
                .get(..8)
                .map(|b| Self::Unsigned64(u64::from_le_bytes(b.try_into().unwrap()))),
            DataType::Real32 => data
                .get(..4)
                .map(|b| Self::Real32(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))),
            DataType::Real64 => data
                .get(..8)
                .map(|b| Self::Real64(f64::from_le_bytes(b.try_into().unwrap()))),
            DataType::VisibleString => Some(Self::VisibleString(
                String::from_utf8_lossy(data).to_string(),
            )),
            DataType::OctetString => Some(Self::OctetString(data.to_vec())),
            DataType::Domain => Some(Self::Domain(data.to_vec())),
            // Variable-length types without OdValue variant
            DataType::UnicodeString | DataType::TimeOfDay | DataType::TimeDifference => None,
        }
    }

    // === Convenience extractors ===

    /// Try to extract a u8 value.
    pub fn try_as_u8(&self) -> Option<u8> {
        match self {
            Self::Unsigned8(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a u16 value.
    pub fn try_as_u16(&self) -> Option<u16> {
        match self {
            Self::Unsigned16(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a u32 value.
    pub fn try_as_u32(&self) -> Option<u32> {
        match self {
            Self::Unsigned32(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a u64 value.
    pub fn try_as_u64(&self) -> Option<u64> {
        match self {
            Self::Unsigned64(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract an i8 value.
    pub fn try_as_i8(&self) -> Option<i8> {
        match self {
            Self::Integer8(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract an i16 value.
    pub fn try_as_i16(&self) -> Option<i16> {
        match self {
            Self::Integer16(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract an i32 value.
    pub fn try_as_i32(&self) -> Option<i32> {
        match self {
            Self::Integer32(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a 24-bit signed integer value (stored as i32).
    pub fn try_as_i24(&self) -> Option<i32> {
        match self {
            Self::Integer24(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a 24-bit unsigned integer value (stored as u32).
    pub fn try_as_u24(&self) -> Option<u32> {
        match self {
            Self::Unsigned24(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract an i64 value.
    pub fn try_as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer64(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a 40-bit signed integer value (stored as i64).
    pub fn try_as_i40(&self) -> Option<i64> {
        match self {
            Self::Integer40(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a 48-bit signed integer value (stored as i64).
    pub fn try_as_i48(&self) -> Option<i64> {
        match self {
            Self::Integer48(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a 56-bit signed integer value (stored as i64).
    pub fn try_as_i56(&self) -> Option<i64> {
        match self {
            Self::Integer56(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a 40-bit unsigned integer value (stored as u64).
    pub fn try_as_u40(&self) -> Option<u64> {
        match self {
            Self::Unsigned40(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a 48-bit unsigned integer value (stored as u64).
    pub fn try_as_u48(&self) -> Option<u64> {
        match self {
            Self::Unsigned48(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a 56-bit unsigned integer value (stored as u64).
    pub fn try_as_u56(&self) -> Option<u64> {
        match self {
            Self::Unsigned56(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a bool value.
    pub fn try_as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a f32 value.
    pub fn try_as_f32(&self) -> Option<f32> {
        match self {
            Self::Real32(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a f64 value.
    pub fn try_as_f64(&self) -> Option<f64> {
        match self {
            Self::Real64(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a string reference.
    pub fn try_as_str(&self) -> Option<&str> {
        match self {
            Self::VisibleString(s) => Some(s),
            _ => None,
        }
    }

    /// Try to extract a byte slice.
    pub fn try_as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::OctetString(b) | Self::Domain(b) => Some(b),
            _ => None,
        }
    }

    /// Check if this value is None (empty).
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

// === Type conversions for OdValue ===

impl From<bool> for OdValue {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}
impl From<i8> for OdValue {
    fn from(v: i8) -> Self {
        Self::Integer8(v)
    }
}
impl From<i16> for OdValue {
    fn from(v: i16) -> Self {
        Self::Integer16(v)
    }
}
impl From<i32> for OdValue {
    fn from(v: i32) -> Self {
        Self::Integer32(v)
    }
}
impl From<i64> for OdValue {
    fn from(v: i64) -> Self {
        Self::Integer64(v)
    }
}
impl From<u8> for OdValue {
    fn from(v: u8) -> Self {
        Self::Unsigned8(v)
    }
}
impl From<u16> for OdValue {
    fn from(v: u16) -> Self {
        Self::Unsigned16(v)
    }
}
impl From<u32> for OdValue {
    fn from(v: u32) -> Self {
        Self::Unsigned32(v)
    }
}
impl From<u64> for OdValue {
    fn from(v: u64) -> Self {
        Self::Unsigned64(v)
    }
}
impl From<f32> for OdValue {
    fn from(v: f32) -> Self {
        Self::Real32(v)
    }
}
impl From<f64> for OdValue {
    fn from(v: f64) -> Self {
        Self::Real64(v)
    }
}
impl From<String> for OdValue {
    fn from(v: String) -> Self {
        Self::VisibleString(v)
    }
}
impl From<Vec<u8>> for OdValue {
    fn from(v: Vec<u8>) -> Self {
        Self::Domain(v)
    }
}

/// Try to extract a concrete type from OdValue.
impl TryFrom<OdValue> for bool {
    type Error = crate::error::OdError;
    fn try_from(v: OdValue) -> Result<Self, Self::Error> {
        match v {
            OdValue::Boolean(b) => Ok(b),
            _ => Err(crate::error::OdError::TypeMismatch {
                expected: DataType::Boolean,
                actual: v.data_type().unwrap_or(DataType::Boolean),
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
                actual: v.data_type().unwrap_or(DataType::Boolean),
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
                actual: v.data_type().unwrap_or(DataType::Boolean),
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
                actual: v.data_type().unwrap_or(DataType::Boolean),
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
                actual: v.data_type().unwrap_or(DataType::Boolean),
            }),
        }
    }
}

impl TryFrom<OdValue> for i32 {
    type Error = crate::error::OdError;
    fn try_from(v: OdValue) -> Result<Self, Self::Error> {
        match v {
            OdValue::Integer32(v) => Ok(v),
            OdValue::Integer24(v) => Ok(v),
            _ => Err(crate::error::OdError::TypeMismatch {
                expected: DataType::Integer32,
                actual: v.data_type().unwrap_or(DataType::Boolean),
            }),
        }
    }
}

/// Object Dictionary trait — access to OD entries.
pub trait ObjectDictionary: Send {
    fn read(&self, index: u16, subindex: u8) -> Result<OdValue, crate::error::OdError>;
    fn write(
        &mut self,
        index: u16,
        subindex: u8,
        value: OdValue,
    ) -> Result<(), crate::error::OdError>;
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
    fn send(
        &mut self,
        frame: &crate::frame::CanOpenFrame,
    ) -> Result<(), crate::error::CanOpenError>;
    fn recv(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<crate::frame::CanOpenFrame, crate::error::CanOpenError>,
    > + Send;
}

/// Blanket impl: `&mut C` is a CanDriver if `C` is.
impl<C: CanDriver> CanDriver for &mut C {
    fn send(
        &mut self,
        frame: &crate::frame::CanOpenFrame,
    ) -> Result<(), crate::error::CanOpenError> {
        (**self).send(frame)
    }

    fn recv(
        &mut self,
    ) -> impl std::future::Future<
        Output = Result<crate::frame::CanOpenFrame, crate::error::CanOpenError>,
    > + Send {
        (**self).recv()
    }
}

impl std::fmt::Display for OdValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Boolean(v) => write!(f, "{}", v),
            Self::Integer8(v) => write!(f, "{}", v),
            Self::Integer16(v) => write!(f, "{}", v),
            Self::Integer24(v) => write!(f, "{}", v),
            Self::Integer32(v) => write!(f, "{}", v),
            Self::Integer40(v) => write!(f, "{}", v),
            Self::Integer48(v) => write!(f, "{}", v),
            Self::Integer56(v) => write!(f, "{}", v),
            Self::Integer64(v) => write!(f, "{}", v),
            Self::Unsigned8(v) => write!(f, "{}", v),
            Self::Unsigned16(v) => write!(f, "{}", v),
            Self::Unsigned24(v) => write!(f, "{}", v),
            Self::Unsigned32(v) => write!(f, "{}", v),
            Self::Unsigned40(v) => write!(f, "{}", v),
            Self::Unsigned48(v) => write!(f, "{}", v),
            Self::Unsigned56(v) => write!(f, "{}", v),
            Self::Unsigned64(v) => write!(f, "{}", v),
            Self::Real32(v) => write!(f, "{}", v),
            Self::Real64(v) => write!(f, "{}", v),
            Self::VisibleString(s) => write!(f, "\"{}\"", s),
            Self::OctetString(b) => {
                write!(f, "OctetString[{}]", b.len())
            }
            Self::Domain(b) => {
                write!(f, "Domain[{}]", b.len())
            }
        }
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boolean => write!(f, "Boolean"),
            Self::Integer8 => write!(f, "Integer8"),
            Self::Integer16 => write!(f, "Integer16"),
            Self::Integer24 => write!(f, "Integer24"),
            Self::Integer32 => write!(f, "Integer32"),
            Self::Integer40 => write!(f, "Integer40"),
            Self::Integer48 => write!(f, "Integer48"),
            Self::Integer56 => write!(f, "Integer56"),
            Self::Integer64 => write!(f, "Integer64"),
            Self::Unsigned8 => write!(f, "Unsigned8"),
            Self::Unsigned16 => write!(f, "Unsigned16"),
            Self::Unsigned24 => write!(f, "Unsigned24"),
            Self::Unsigned32 => write!(f, "Unsigned32"),
            Self::Unsigned40 => write!(f, "Unsigned40"),
            Self::Unsigned48 => write!(f, "Unsigned48"),
            Self::Unsigned56 => write!(f, "Unsigned56"),
            Self::Unsigned64 => write!(f, "Unsigned64"),
            Self::Real32 => write!(f, "Real32"),
            Self::Real64 => write!(f, "Real64"),
            Self::VisibleString => write!(f, "VisibleString"),
            Self::OctetString => write!(f, "OctetString"),
            Self::UnicodeString => write!(f, "UnicodeString"),
            Self::TimeOfDay => write!(f, "TimeOfDay"),
            Self::TimeDifference => write!(f, "TimeDifference"),
            Self::Domain => write!(f, "Domain"),
        }
    }
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
        assert_eq!(val.data_type(), Some(DataType::Unsigned16));

        let extracted: u16 = val.try_into().unwrap();
        assert_eq!(extracted, 42u16);
    }

    #[test]
    fn test_od_value_type_mismatch() {
        let val = OdValue::Unsigned16(42);
        let result: Result<u32, _> = val.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_data_type_byte_size() {
        assert_eq!(DataType::Boolean.byte_size(), Some(1));
        assert_eq!(DataType::Integer8.byte_size(), Some(1));
        assert_eq!(DataType::Unsigned16.byte_size(), Some(2));
        assert_eq!(DataType::Integer24.byte_size(), Some(3));
        assert_eq!(DataType::Real32.byte_size(), Some(4));
        assert_eq!(DataType::Integer40.byte_size(), Some(5));
        assert_eq!(DataType::Unsigned48.byte_size(), Some(6));
        assert_eq!(DataType::Integer56.byte_size(), Some(7));
        assert_eq!(DataType::Integer64.byte_size(), Some(8));
        assert_eq!(DataType::Real64.byte_size(), Some(8));

        // Variable-length types return None
        assert_eq!(DataType::VisibleString.byte_size(), None);
        assert_eq!(DataType::Domain.byte_size(), None);
        assert_eq!(DataType::OctetString.byte_size(), None);
    }

    #[test]
    fn test_data_type_is_numeric() {
        assert!(DataType::Unsigned32.is_numeric());
        assert!(DataType::Real64.is_numeric());
        assert!(DataType::Boolean.is_numeric());
        assert!(!DataType::VisibleString.is_numeric());
        assert!(!DataType::Domain.is_numeric());
    }

    #[test]
    fn test_data_type_is_signed() {
        assert!(DataType::Integer16.is_signed());
        assert!(DataType::Integer64.is_signed());
        assert!(!DataType::Unsigned16.is_signed());
        assert!(!DataType::Boolean.is_signed());
        assert!(!DataType::Real32.is_signed());
    }

    #[test]
    fn test_od_value_try_as_convenience() {
        let val = OdValue::Unsigned32(1234);
        assert_eq!(val.try_as_u32(), Some(1234));
        assert_eq!(val.try_as_u16(), None);
        assert_eq!(val.try_as_str(), None);

        let val = OdValue::VisibleString("hello".to_string());
        assert_eq!(val.try_as_str(), Some("hello"));
        assert_eq!(val.try_as_u32(), None);

        let val = OdValue::Boolean(true);
        assert_eq!(val.try_as_bool(), Some(true));

        let val = OdValue::Real32(3.14);
        assert!(val.try_as_f32().is_some());
        assert_eq!(val.try_as_u32(), None);
    }

    #[test]
    fn test_od_value_is_none() {
        assert!(OdValue::None.is_none());
        assert!(!OdValue::Unsigned8(0).is_none());
    }

    #[test]
    fn test_od_value_try_as_bytes() {
        let val = OdValue::Domain(vec![1, 2, 3]);
        assert_eq!(val.try_as_bytes(), Some(&[1, 2, 3][..]));

        let val = OdValue::Unsigned32(0);
        assert_eq!(val.try_as_bytes(), None);
    }

    // === 40/48/56-bit type tests ===

    #[test]
    fn test_unsigned40_roundtrip() {
        let val = OdValue::Unsigned40(0x000000FF_FFFFFFFF);
        let bytes = val.to_bytes();
        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

        let decoded = OdValue::from_bytes(DataType::Unsigned40, &bytes).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(decoded.try_as_u40(), Some(0xFFFFFFFFFF));
    }

    #[test]
    fn test_unsigned48_roundtrip() {
        let val = OdValue::Unsigned48(0x0000FFFF_FFFFFFFF);
        let bytes = val.to_bytes();
        assert_eq!(bytes.len(), 6);

        let decoded = OdValue::from_bytes(DataType::Unsigned48, &bytes).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(decoded.try_as_u48(), Some(0xFFFFFFFFFFFF));
    }

    #[test]
    fn test_unsigned56_roundtrip() {
        let val = OdValue::Unsigned56(0x00FFFFFF_FFFFFFFF);
        let bytes = val.to_bytes();
        assert_eq!(bytes.len(), 7);

        let decoded = OdValue::from_bytes(DataType::Unsigned56, &bytes).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(decoded.try_as_u56(), Some(0xFFFFFFFFFFFFFF));
    }

    #[test]
    fn test_integer40_roundtrip() {
        // Positive
        let val = OdValue::Integer40(0x0000007F_FFFFFFFF);
        let bytes = val.to_bytes();
        assert_eq!(bytes.len(), 5);
        let decoded = OdValue::from_bytes(DataType::Integer40, &bytes).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(decoded.try_as_i40(), Some(0x7FFFFFFFFF));

        // Negative
        let val = OdValue::Integer40(-1);
        let bytes = val.to_bytes();
        assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let decoded = OdValue::from_bytes(DataType::Integer40, &bytes).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn test_integer48_roundtrip() {
        let val = OdValue::Integer48(-12345);
        let bytes = val.to_bytes();
        assert_eq!(bytes.len(), 6);
        let decoded = OdValue::from_bytes(DataType::Integer48, &bytes).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(decoded.try_as_i48(), Some(-12345));
    }

    #[test]
    fn test_integer56_roundtrip() {
        // Max positive 56-bit value: 0x007FFFFFFFFFFFFF
        let val = OdValue::Integer56(0x007F_FFFF_FFFF_FFFF);
        let bytes = val.to_bytes();
        assert_eq!(bytes.len(), 7);
        let decoded = OdValue::from_bytes(DataType::Integer56, &bytes).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(decoded.try_as_i56(), Some(0x7FFFFFFFFFFFFF));

        // Negative
        let val = OdValue::Integer56(-1);
        let bytes = val.to_bytes();
        assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let decoded = OdValue::from_bytes(DataType::Integer56, &bytes).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn test_40_48_56_byte_size() {
        assert_eq!(DataType::Integer40.byte_size(), Some(5));
        assert_eq!(DataType::Unsigned40.byte_size(), Some(5));
        assert_eq!(DataType::Integer48.byte_size(), Some(6));
        assert_eq!(DataType::Unsigned48.byte_size(), Some(6));
        assert_eq!(DataType::Integer56.byte_size(), Some(7));
        assert_eq!(DataType::Unsigned56.byte_size(), Some(7));
    }

    #[test]
    fn test_from_bytes_wrong_length_returns_none() {
        // Too short
        assert!(OdValue::from_bytes(DataType::Unsigned40, &[1, 2, 3]).is_none());
        assert!(OdValue::from_bytes(DataType::Integer48, &[1, 2, 3, 4, 5]).is_none());
        assert!(OdValue::from_bytes(DataType::Unsigned56, &[1, 2, 3, 4, 5, 6]).is_none());
        // Exact length should work
        assert!(OdValue::from_bytes(DataType::Unsigned40, &[1, 2, 3, 4, 5]).is_some());
    }
}
