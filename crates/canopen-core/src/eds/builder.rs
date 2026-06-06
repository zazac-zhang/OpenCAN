//! EDS → Object Dictionary builder.
//!
//! Converts a parsed [`EdsFile`] into a [`ConcreteOd`] that can be used
//! by the CANOpen protocol stack and GUI.

use crate::eds::model::EdsFile;
use crate::concrete_od::{ConcreteOd, OdEntry};
use crate::od::{AccessType, DataType, ObjectType, OdValue};

/// Build a [`ConcreteOd`] from a parsed EDS file.
///
/// # Example
/// ```no_run
/// # use opencan_canopen_core::eds::parser::parse_eds;
/// # use opencan_canopen_core::eds::builder::build_od;
/// let eds_content = std::fs::read_to_string("device.eds").unwrap();
/// let eds = parse_eds(&eds_content).unwrap();
/// let od = build_od(&eds);
/// ```
pub fn build_od(eds: &EdsFile) -> ConcreteOd {
    let mut od = ConcreteOd::new();

    for (&index, entry) in &eds.entries {
        // Determine access type from sub-entries (use first sub-entry's access)
        // Determine data type: prefer sub-entry, fall back to main entry
        let data_type = eds
            .sub_entries
            .iter()
            .find(|((idx, _), _)| *idx == index)
            .and_then(|(_, sub)| sub.data_type.and_then(DataType::from_u16))
            .or_else(|| entry.data_type.and_then(DataType::from_u16))
            .unwrap_or(DataType::Unsigned32);

        // Determine access type: prefer sub-entry, fall back to main entry
        let access = eds
            .sub_entries
            .iter()
            .find(|((idx, _), _)| *idx == index)
            .and_then(|(_, sub)| parse_access_type(sub.access_type.as_deref().unwrap_or("rw")))
            .or_else(|| parse_access_type(entry.access_type.as_deref().unwrap_or("rw")))
            .unwrap_or(AccessType::ReadWrite);

        // Parse default value: prefer sub-entry, fall back to main entry
        let default_value = eds
            .sub_entries
            .iter()
            .find(|((idx, _), _)| *idx == index)
            .and_then(|(_, sub)| sub.default_value.as_deref())
            .or(entry.default_value.as_deref())
            .and_then(|v| parse_default_value(v, data_type));

        // Add the main entry (subindex 0) for Var types.
        // For Array/Record, subindex 0 comes from the sub-entries.
        if entry.object_type == ObjectType::Var {
            od.add_entry(OdEntry {
                index,
                subindex: 0,
                object_type: entry.object_type,
                data_type,
                access,
                name: entry.parameter_name.clone(),
                value: default_value.clone().unwrap_or(OdValue::None),
                default_value: default_value.clone(),
            });
        }

        // Add sub-entries (including subindex 0 for Array/Record)
        let has_subs = eds.sub_entries.iter().any(|((idx, _), _)| *idx == index);
        if has_subs {
            for ((idx, sub), sub_entry) in &eds.sub_entries {
                if *idx != index {
                    continue;
                }

                let sub_data_type = sub_entry
                    .data_type
                    .and_then(DataType::from_u16)
                    .unwrap_or(data_type);

                let sub_access =
                    parse_access_type(sub_entry.access_type.as_deref().unwrap_or("rw"))
                        .unwrap_or(AccessType::ReadWrite);

                let sub_value = sub_entry
                    .default_value
                    .as_deref()
                    .and_then(|v| parse_default_value(v, sub_data_type))
                    .unwrap_or(OdValue::None);

                od.add_entry(OdEntry {
                    index: *idx,
                    subindex: *sub,
                    object_type: entry.object_type,
                    data_type: sub_data_type,
                    access: sub_access,
                    name: sub_entry.parameter_name.clone(),
                    value: sub_value.clone(),
                    default_value: Some(sub_value),
                });
            }
        } else if entry.object_type != ObjectType::Var {
            // Array/Record with no sub-entries defined — add subindex 0 anyway
            od.add_entry(OdEntry {
                index,
                subindex: 0,
                object_type: entry.object_type,
                data_type,
                access,
                name: entry.parameter_name.clone(),
                value: default_value.clone().unwrap_or(OdValue::None),
                default_value,
            });
        }
    }

    od
}

/// Parse an EDS access type string to [`AccessType`].
fn parse_access_type(s: &str) -> Option<AccessType> {
    match s.to_lowercase().as_str() {
        "ro" | "const" => Some(AccessType::ReadOnly),
        "wo" => Some(AccessType::WriteOnly),
        "rw" => Some(AccessType::ReadWrite),
        _ => Some(AccessType::ReadWrite), // Default to read-write for unknown
    }
}

/// Parse an EDS default value string to [`OdValue`].
fn parse_default_value(s: &str, data_type: DataType) -> Option<OdValue> {
    let s = s.trim();

    match data_type {
        DataType::Boolean => {
            let val = s.parse::<u8>().ok()?;
            Some(OdValue::Boolean(val != 0))
        }
        DataType::Integer8 => {
            let val = parse_numeric::<i8>(s)?;
            Some(OdValue::Integer8(val))
        }
        DataType::Integer16 => {
            let val = parse_numeric::<i16>(s)?;
            Some(OdValue::Integer16(val))
        }
        DataType::Integer32 => {
            let val = parse_numeric::<i32>(s)?;
            Some(OdValue::Integer32(val))
        }
        DataType::Integer64 => {
            let val = parse_numeric::<i64>(s)?;
            Some(OdValue::Integer64(val))
        }
        DataType::Unsigned8 => {
            let val = parse_numeric::<u8>(s)?;
            Some(OdValue::Unsigned8(val))
        }
        DataType::Unsigned16 => {
            let val = parse_numeric::<u16>(s)?;
            Some(OdValue::Unsigned16(val))
        }
        DataType::Unsigned32 => {
            let val = parse_numeric::<u32>(s)?;
            Some(OdValue::Unsigned32(val))
        }
        DataType::Unsigned64 => {
            let val = parse_numeric::<u64>(s)?;
            Some(OdValue::Unsigned64(val))
        }
        DataType::Real32 => {
            let val = s.parse::<f32>().ok()?;
            Some(OdValue::Real32(val))
        }
        DataType::Real64 => {
            let val = s.parse::<f64>().ok()?;
            Some(OdValue::Real64(val))
        }
        DataType::VisibleString => Some(OdValue::VisibleString(s.to_string())),
        DataType::Domain => {
            // Domain values are typically hex strings
            let bytes = parse_hex_bytes(s);
            if bytes.is_empty() {
                None
            } else {
                Some(OdValue::Domain(bytes))
            }
        }
        _ => None,
    }
}

/// Parse a numeric value from decimal or hex string.
fn parse_numeric<T: FromStrRadix + std::str::FromStr + Copy>(s: &str) -> Option<T> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        T::from_hex(&s[2..])
    } else {
        s.parse::<T>().ok()
    }
}

trait FromStrRadix: Sized {
    fn from_hex(s: &str) -> Option<Self>;
}

impl FromStrRadix for u8 {
    fn from_hex(s: &str) -> Option<Self> {
        u8::from_str_radix(s, 16).ok()
    }
}
impl FromStrRadix for u16 {
    fn from_hex(s: &str) -> Option<Self> {
        u16::from_str_radix(s, 16).ok()
    }
}
impl FromStrRadix for u32 {
    fn from_hex(s: &str) -> Option<Self> {
        u32::from_str_radix(s, 16).ok()
    }
}
impl FromStrRadix for u64 {
    fn from_hex(s: &str) -> Option<Self> {
        u64::from_str_radix(s, 16).ok()
    }
}
impl FromStrRadix for i8 {
    fn from_hex(s: &str) -> Option<Self> {
        i8::from_str_radix(s, 16).ok()
    }
}
impl FromStrRadix for i16 {
    fn from_hex(s: &str) -> Option<Self> {
        i16::from_str_radix(s, 16).ok()
    }
}
impl FromStrRadix for i32 {
    fn from_hex(s: &str) -> Option<Self> {
        i32::from_str_radix(s, 16).ok()
    }
}
impl FromStrRadix for i64 {
    fn from_hex(s: &str) -> Option<Self> {
        i64::from_str_radix(s, 16).ok()
    }
}

/// Parse a hex byte string (e.g. "01 02 03" or "010203") to Vec<u8>.
fn parse_hex_bytes(s: &str) -> Vec<u8> {
    let s = s.replace(' ', "");
    (0..s.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eds::parser::parse_eds;
    use crate::od::ObjectDictionary;

    #[test]
    fn test_build_od_basic() {
        let eds_content = r#"
[1000]
ParameterName=Device Type
ObjectType=7
DataType=0x0007
AccessType=ro
DefaultValue=0x00020192

[1018]
ParameterName=Identity Object
ObjectType=9
SubNumber=4

[1018sub0]
ParameterName=Max Sub-index
DataType=0x0005
AccessType=ro
DefaultValue=4

[1018sub1]
ParameterName=Vendor ID
DataType=0x0007
AccessType=ro
DefaultValue=0x12345678
"#;

        let eds = parse_eds(eds_content).unwrap();
        let od = build_od(&eds);

        // Check Device Type
        let dt = od.read(0x1000, 0).unwrap();
        assert_eq!(dt, OdValue::Unsigned32(0x00020192));

        // Check Identity Object sub-entries
        let max_sub = od.read(0x1018, 0).unwrap();
        assert_eq!(max_sub, OdValue::Unsigned8(4));

        let vendor_id = od.read(0x1018, 1).unwrap();
        assert_eq!(vendor_id, OdValue::Unsigned32(0x12345678));

        // Check entry info
        let info = od.entry_info(0x1000, 0).unwrap();
        assert_eq!(info.name, "Device Type");
        assert_eq!(info.data_type, DataType::Unsigned32);
    }

    #[test]
    fn test_build_od_empty() {
        let eds = parse_eds("").unwrap();
        let od = build_od(&eds);
        assert!(od.is_empty());
    }

    #[test]
    fn test_build_od_access_types() {
        let eds_content = r#"
[2000]
ParameterName=RW Param
ObjectType=7
DataType=0x0006
AccessType=rw
DefaultValue=100

[2001]
ParameterName=RO Param
ObjectType=7
DataType=0x0006
AccessType=ro
DefaultValue=200
"#;

        let eds = parse_eds(eds_content).unwrap();
        let mut od = build_od(&eds);

        // RW should be writable
        od.write(0x2000, 0, OdValue::Unsigned16(999)).unwrap();
        assert_eq!(od.read(0x2000, 0).unwrap(), OdValue::Unsigned16(999));

        // RO should not be writable
        assert!(od.write(0x2001, 0, OdValue::Unsigned16(999)).is_err());
    }

    #[test]
    fn test_parse_hex_values() {
        assert_eq!(parse_hex_bytes("01 02 03"), vec![1, 2, 3]);
        assert_eq!(parse_hex_bytes("010203"), vec![1, 2, 3]);
        assert_eq!(parse_hex_bytes(""), Vec::<u8>::new());
    }

    #[test]
    fn test_parse_numeric_hex() {
        assert_eq!(parse_numeric::<u32>("0x12345678"), Some(0x12345678));
        assert_eq!(parse_numeric::<u16>("0x0006"), Some(6));
        assert_eq!(parse_numeric::<u32>("12345"), Some(12345));
    }
}
