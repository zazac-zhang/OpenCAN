//! EDS (Electronic Data Sheet) file parser.
//!
//! EDS files are INI-format files describing CANOpen device object dictionaries.
//! This parser extracts the fields relevant for debugging tools.

use crate::eds::model::{EdsEntry, EdsFile, EdsSubEntry};
use crate::od::ObjectType;
use std::collections::BTreeMap;

/// Parse an EDS file from a string.
pub fn parse_eds(content: &str) -> Result<EdsFile, String> {
    let mut entries = BTreeMap::new();
    let mut sub_entries = BTreeMap::new();

    let mut current_section: Option<String> = None;
    let mut current_props: BTreeMap<String, String> = BTreeMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            // Process previous section
            if let Some(ref section) = current_section {
                process_section(section, &current_props, &mut entries, &mut sub_entries);
            }

            current_section = Some(line[1..line.len() - 1].to_string());
            current_props.clear();
            continue;
        }

        // Key=Value pair
        if let Some((key, value)) = line.split_once('=') {
            current_props.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    // Process last section
    if let Some(ref section) = current_section {
        process_section(section, &current_props, &mut entries, &mut sub_entries);
    }

    Ok(EdsFile {
        entries,
        sub_entries,
    })
}

fn process_section(
    section: &str,
    props: &BTreeMap<String, String>,
    entries: &mut BTreeMap<u16, EdsEntry>,
    sub_entries: &mut BTreeMap<(u16, u8), EdsSubEntry>,
) {
    // Skip metadata sections
    if section == "FileInfo"
        || section == "DeviceInfo"
        || section == "DummyUsage"
        || section == "Comments"
        || section == "DeviceCommissioning"
    {
        return;
    }

    // Parse main entry: [1000], [1018], etc.
    if let Ok(index) = u16::from_str_radix(section, 16) {
        let parameter_name = props.get("ParameterName").cloned().unwrap_or_default();

        let object_type = match props.get("ObjectType").map(|s| s.as_str()) {
            Some("7") => ObjectType::Var,
            Some("8") => ObjectType::Array,
            Some("9") => ObjectType::Record,
            _ => ObjectType::Var,
        };

        let sub_number = props.get("SubNumber").and_then(|s| s.parse::<u8>().ok());

        let data_type: Option<u16> = props.get("DataType").and_then(|s| {
            if s.starts_with("0x") || s.starts_with("0X") {
                u16::from_str_radix(&s[2..], 16).ok()
            } else {
                s.parse::<u16>().ok()
            }
        });

        let access_type = props.get("AccessType").cloned();
        let default_value = props.get("DefaultValue").cloned();

        entries.insert(
            index,
            EdsEntry {
                index,
                parameter_name,
                object_type,
                sub_number,
                data_type,
                access_type,
                default_value,
            },
        );
    }

    // Parse sub-entries: [1018sub0], [1018sub1], etc.
    // Note: index is hex (e.g., 1018 = 0x1018), but subindex is decimal (e.g., sub10 = 10)
    if let Some((idx_str, sub_str)) = section.split_once("sub")
        && let (Ok(index), Ok(subindex)) = (u16::from_str_radix(idx_str, 16), sub_str.parse::<u8>())
    {
        let parameter_name = props.get("ParameterName").cloned().unwrap_or_default();
        let data_type: Option<u16> = props.get("DataType").and_then(|s| {
            if s.starts_with("0x") || s.starts_with("0X") {
                u16::from_str_radix(&s[2..], 16).ok()
            } else {
                s.parse::<u16>().ok()
            }
        });
        let access_type = props.get("AccessType").cloned();
        let default_value = props.get("DefaultValue").cloned();

        sub_entries.insert(
            (index, subindex),
            EdsSubEntry {
                index,
                subindex,
                parameter_name,
                data_type,
                access_type,
                default_value,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_eds() {
        let eds_content = r#"
[FileInfo]
FileName=Test.eds
FileVersion=1

[DeviceInfo]
VendorName=Test Corp

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

        // Check main entries
        assert!(eds.entries.contains_key(&0x1000));
        assert!(eds.entries.contains_key(&0x1018));

        let device_type = &eds.entries[&0x1000];
        assert_eq!(device_type.parameter_name, "Device Type");
        assert_eq!(device_type.object_type, ObjectType::Var);

        let identity = &eds.entries[&0x1018];
        assert_eq!(identity.parameter_name, "Identity Object");
        assert_eq!(identity.object_type, ObjectType::Record);
        assert_eq!(identity.sub_number, Some(4));

        // Check sub-entries
        assert!(eds.sub_entries.contains_key(&(0x1018, 0)));
        assert!(eds.sub_entries.contains_key(&(0x1018, 1)));

        let vendor_id = &eds.sub_entries[&(0x1018, 1)];
        assert_eq!(vendor_id.parameter_name, "Vendor ID");
        assert_eq!(vendor_id.data_type, Some(0x0007));
        assert_eq!(vendor_id.default_value.as_deref(), Some("0x12345678"));
    }

    #[test]
    fn test_parse_empty_eds() {
        let eds = parse_eds("").unwrap();
        assert!(eds.entries.is_empty());
    }

    #[test]
    fn test_parse_with_comments() {
        let eds_content = r#"
; This is a comment
# This is also a comment

[1000]
ParameterName=Device Type
ObjectType=7
"#;

        let eds = parse_eds(eds_content).unwrap();
        assert!(eds.entries.contains_key(&0x1000));
    }

    #[test]
    fn test_subindex_decimal_parsing() {
        let eds_content = r#"
[1018]
ParameterName=Identity Object
ObjectType=9
SubNumber=12

[1018sub0]
ParameterName=Max Sub-index
DataType=0x0005
AccessType=ro
DefaultValue=12

[1018sub10]
ParameterName=Vendor Specific 10
DataType=0x0007
AccessType=ro
DefaultValue=0xAAAAAAAA

[1018sub11]
ParameterName=Vendor Specific 11
DataType=0x0007
AccessType=ro
DefaultValue=0xBBBBBBBB
"#;

        let eds = parse_eds(eds_content).unwrap();

        // sub10 should be decimal 10, not hex 0x10 (16)
        assert!(
            eds.sub_entries.contains_key(&(0x1018, 10)),
            "sub10 should parse as decimal 10"
        );
        assert!(
            !eds.sub_entries.contains_key(&(0x1018, 16)),
            "sub10 should NOT parse as hex 0x10 (16)"
        );

        // sub11 should be decimal 11
        assert!(
            eds.sub_entries.contains_key(&(0x1018, 11)),
            "sub11 should parse as decimal 11"
        );

        let sub10 = &eds.sub_entries[&(0x1018, 10)];
        assert_eq!(sub10.parameter_name, "Vendor Specific 10");
        assert_eq!(sub10.default_value.as_deref(), Some("0xAAAAAAAA"));
    }
}
