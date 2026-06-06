//! SDO data types.

/// CANOpen data types for SDO access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdoDataType {
    Boolean,
    Integer8,
    Integer16,
    Integer32,
    Integer64,
    Unsigned8,
    Unsigned16,
    Unsigned32,
    Unsigned64,
    Real32,
    Real64,
    VisibleString,
    OctetString,
    UnicodeString,
    TimeOfDay,
    TimeDifference,
    Domain,
}

impl SdoDataType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Boolean => "BOOLEAN",
            Self::Integer8 => "INT8",
            Self::Integer16 => "INT16",
            Self::Integer32 => "INT32",
            Self::Integer64 => "INT64",
            Self::Unsigned8 => "UNS8",
            Self::Unsigned16 => "UNS16",
            Self::Unsigned32 => "UNS32",
            Self::Unsigned64 => "UNS64",
            Self::Real32 => "REAL32",
            Self::Real64 => "REAL64",
            Self::VisibleString => "VISIBLE_STRING",
            Self::OctetString => "OCTET_STRING",
            Self::UnicodeString => "UNICODE_STRING",
            Self::TimeOfDay => "TIME_OF_DAY",
            Self::TimeDifference => "TIME_DIFFERENCE",
            Self::Domain => "DOMAIN",
        }
    }

    pub fn byte_size(&self) -> Option<usize> {
        match self {
            Self::Boolean => Some(1),
            Self::Integer8 | Self::Unsigned8 => Some(1),
            Self::Integer16 | Self::Unsigned16 => Some(2),
            Self::Integer32 | Self::Unsigned32 | Self::Real32 => Some(4),
            Self::Integer64 | Self::Unsigned64 | Self::Real64 => Some(8),
            _ => None,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self,
            Self::Boolean | Self::Integer8 | Self::Integer16 | Self::Integer32 | Self::Integer64 |
            Self::Unsigned8 | Self::Unsigned16 | Self::Unsigned32 | Self::Unsigned64 |
            Self::Real32 | Self::Real64
        )
    }

    pub fn is_signed(&self) -> bool {
        matches!(self,
            Self::Integer8 | Self::Integer16 | Self::Integer32 | Self::Integer64 |
            Self::Real32 | Self::Real64
        )
    }

    pub fn all() -> &'static [SdoDataType] {
        &[
            Self::Unsigned8,
            Self::Unsigned16,
            Self::Unsigned32,
            Self::Unsigned64,
            Self::Integer8,
            Self::Integer16,
            Self::Integer32,
            Self::Integer64,
            Self::Boolean,
            Self::Real32,
            Self::Real64,
            Self::VisibleString,
            Self::OctetString,
            Self::Domain,
        ]
    }

    /// Parse value from hex string.
    pub fn parse_hex(&self, hex: &str) -> Vec<u8> {
        let bytes: Vec<u8> = hex.split_whitespace()
            .filter_map(|b| u8::from_str_radix(b, 16).ok())
            .collect();

        // Ensure correct length
        if let Some(size) = self.byte_size() {
            if bytes.len() < size {
                let mut padded = bytes;
                padded.resize(size, 0);
                return padded;
            }
        }

        bytes
    }

    /// Format value from bytes.
    pub fn format_value(&self, data: &[u8]) -> String {
        match self {
            Self::Boolean => {
                if data.len() >= 1 && data[0] != 0 { "TRUE".to_string() } else { "FALSE".to_string() }
            }
            Self::Integer8 => {
                if data.len() >= 1 { (data[0] as i8).to_string() } else { "?".to_string() }
            }
            Self::Integer16 => {
                if data.len() >= 2 {
                    i16::from_le_bytes([data[0], data[1]]).to_string()
                } else { "?".to_string() }
            }
            Self::Integer32 => {
                if data.len() >= 4 {
                    i32::from_le_bytes([data[0], data[1], data[2], data[3]]).to_string()
                } else { "?".to_string() }
            }
            Self::Integer64 => {
                if data.len() >= 8 {
                    i64::from_le_bytes([
                        data[0], data[1], data[2], data[3],
                        data[4], data[5], data[6], data[7]
                    ]).to_string()
                } else { "?".to_string() }
            }
            Self::Unsigned8 => {
                if data.len() >= 1 { data[0].to_string() } else { "?".to_string() }
            }
            Self::Unsigned16 => {
                if data.len() >= 2 {
                    u16::from_le_bytes([data[0], data[1]]).to_string()
                } else { "?".to_string() }
            }
            Self::Unsigned32 => {
                if data.len() >= 4 {
                    u32::from_le_bytes([data[0], data[1], data[2], data[3]]).to_string()
                } else { "?".to_string() }
            }
            Self::Unsigned64 => {
                if data.len() >= 8 {
                    u64::from_le_bytes([
                        data[0], data[1], data[2], data[3],
                        data[4], data[5], data[6], data[7]
                    ]).to_string()
                } else { "?".to_string() }
            }
            Self::Real32 => {
                if data.len() >= 4 {
                    f32::from_le_bytes([data[0], data[1], data[2], data[3]]).to_string()
                } else { "?".to_string() }
            }
            Self::Real64 => {
                if data.len() >= 8 {
                    f64::from_le_bytes([
                        data[0], data[1], data[2], data[3],
                        data[4], data[5], data[6], data[7]
                    ]).to_string()
                } else { "?".to_string() }
            }
            Self::VisibleString => {
                String::from_utf8_lossy(data).to_string()
            }
            _ => data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "),
        }
    }
}
