//! Concrete Object Dictionary implementation.
//!
//! Provides [`ConcreteOd`], a BTreeMap-backed [`ObjectDictionary`] that can be
//! populated programmatically or built from an EDS file via `canopen-eds`.

use std::collections::BTreeMap;
use crate::od::{AccessType, DataType, EntryInfo, ObjectType, OdValue, ObjectDictionary};
use crate::error::OdError;

/// A single OD entry with metadata and current value.
#[derive(Debug, Clone)]
pub struct OdEntry {
    pub index: u16,
    pub subindex: u8,
    pub object_type: ObjectType,
    pub data_type: DataType,
    pub access: AccessType,
    pub name: String,
    pub value: OdValue,
}

/// Concrete Object Dictionary backed by a BTreeMap.
///
/// # Example
/// ```
/// use opencan_canopen_core::concrete_od::{ConcreteOd, OdEntry};
/// use opencan_canopen_core::od::*;
///
/// let mut od = ConcreteOd::new();
/// od.add_entry(OdEntry {
///     index: 0x1000,
///     subindex: 0,
///     object_type: ObjectType::Var,
///     data_type: DataType::Unsigned32,
///     access: AccessType::ReadOnly,
///     name: "Device Type".to_string(),
///     value: OdValue::Unsigned32(0x00020192),
/// });
///
/// let val = od.read(0x1000, 0).unwrap();
/// assert_eq!(val, OdValue::Unsigned32(0x00020192));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ConcreteOd {
    entries: BTreeMap<(u16, u8), OdEntry>,
}

impl ConcreteOd {
    /// Create a new empty Object Dictionary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entry to the OD. Overwrites if the entry already exists.
    pub fn add_entry(&mut self, entry: OdEntry) {
        self.entries.insert((entry.index, entry.subindex), entry);
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the OD is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = &OdEntry> {
        self.entries.values()
    }

    /// Get a reference to an entry by index and subindex.
    pub fn get_entry(&self, index: u16, subindex: u8) -> Option<&OdEntry> {
        self.entries.get(&(index, subindex))
    }
}

impl ObjectDictionary for ConcreteOd {
    fn read(&self, index: u16, subindex: u8) -> Result<OdValue, OdError> {
        self.entries
            .get(&(index, subindex))
            .map(|e| e.value.clone())
            .ok_or(OdError::SubindexNotFound { index, subindex })
    }

    fn write(&mut self, index: u16, subindex: u8, value: OdValue) -> Result<(), OdError> {
        let entry = self.entries
            .get_mut(&(index, subindex))
            .ok_or(OdError::SubindexNotFound { index, subindex })?;

        // Check access type
        match entry.access {
            AccessType::ReadOnly | AccessType::Constant => {
                return Err(OdError::AccessDenied {
                    index,
                    subindex,
                    access: entry.access,
                });
            }
            AccessType::WriteOnly | AccessType::ReadWrite => {}
        }

        entry.value = value;
        Ok(())
    }

    fn entry_info(&self, index: u16, subindex: u8) -> Result<EntryInfo, OdError> {
        let entry = self.entries
            .get(&(index, subindex))
            .ok_or(OdError::SubindexNotFound { index, subindex })?;

        Ok(EntryInfo {
            index: entry.index,
            subindex: entry.subindex,
            object_type: entry.object_type,
            data_type: entry.data_type,
            access: entry.access,
            default_value: None,
            name: entry.name.clone(),
        })
    }
}
