//! EDS data model.

use std::collections::BTreeMap;
use opencan_canopen_core::od::ObjectType;

/// Parsed EDS file structure.
pub struct EdsFile {
    pub entries: BTreeMap<u16, EdsEntry>,
    pub sub_entries: BTreeMap<(u16, u8), EdsSubEntry>,
}

/// EDS main entry (index).
pub struct EdsEntry {
    pub index: u16,
    pub parameter_name: String,
    pub object_type: ObjectType,
    pub sub_number: Option<u8>,
}

/// EDS sub-entry.
pub struct EdsSubEntry {
    pub index: u16,
    pub subindex: u8,
    pub parameter_name: String,
    pub data_type: Option<u16>,
    pub access_type: Option<String>,
    pub default_value: Option<String>,
}
