//! Concrete Object Dictionary implementation.
//!
//! Provides [`ConcreteOd`], a BTreeMap-backed [`ObjectDictionary`] that can be
//! populated programmatically or built from an EDS file via `canopen-eds`.

use crate::error::OdError;
use crate::od::{AccessType, DataType, EntryInfo, ObjectDictionary, ObjectType, OdValue};
use std::collections::BTreeMap;

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
    /// Default value (typically from EDS). None if not specified.
    pub default_value: Option<OdValue>,
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
///     default_value: None,
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

    /// Remove an entry by index and subindex.
    /// Returns the removed entry if it existed.
    pub fn remove_entry(&mut self, index: u16, subindex: u8) -> Option<OdEntry> {
        self.entries.remove(&(index, subindex))
    }

    /// Check if an entry exists at the given index and subindex.
    pub fn contains(&self, index: u16, subindex: u8) -> bool {
        self.entries.contains_key(&(index, subindex))
    }

    /// Check if an index exists (any subindex).
    pub fn contains_index(&self, index: u16) -> bool {
        self.entries.keys().any(|(i, _)| *i == index)
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

    /// Get a mutable reference to an entry by index and subindex.
    pub fn get_entry_mut(&mut self, index: u16, subindex: u8) -> Option<&mut OdEntry> {
        self.entries.get_mut(&(index, subindex))
    }

    /// Iterate over all entries within an index range (inclusive).
    pub fn range(&self, start_index: u16, end_index: u16) -> impl Iterator<Item = &OdEntry> {
        self.entries
            .range((start_index, 0)..=(end_index, u8::MAX))
            .map(|(_, e)| e)
    }

    /// Get all subindices for a given index.
    pub fn subindices(&self, index: u16) -> impl Iterator<Item = &OdEntry> {
        self.entries
            .range((index, 0)..=(index, u8::MAX))
            .map(|(_, e)| e)
    }

    /// Create a standard DS301 device with common entries.
    ///
    /// Includes:
    /// - 0x1000: Device Type (default 0x00020192)
    /// - 0x1001: Error Register
    /// - 0x1018: Identity Object (4 subindices)
    pub fn standard_device(
        device_type: u32,
        vendor_id: u32,
        product_code: u32,
        revision: u32,
        serial: u32,
    ) -> Self {
        let mut od = Self::new();

        // 0x1000: Device Type
        od.add_entry(OdEntry {
            index: 0x1000,
            subindex: 0,
            object_type: ObjectType::Var,
            data_type: DataType::Unsigned32,
            access: AccessType::ReadOnly,
            name: "Device Type".to_string(),
            value: OdValue::Unsigned32(device_type),
            default_value: None,
        });

        // 0x1001: Error Register
        od.add_entry(OdEntry {
            index: 0x1001,
            subindex: 0,
            object_type: ObjectType::Var,
            data_type: DataType::Unsigned8,
            access: AccessType::ReadOnly,
            name: "Error Register".to_string(),
            value: OdValue::Unsigned8(0),
            default_value: None,
        });

        // 0x1018: Identity Object
        od.add_entry(OdEntry {
            index: 0x1018,
            subindex: 0,
            object_type: ObjectType::Var,
            data_type: DataType::Unsigned8,
            access: AccessType::ReadOnly,
            name: "Identity Object - Number of entries".to_string(),
            value: OdValue::Unsigned8(4),
            default_value: None,
        });
        od.add_entry(OdEntry {
            index: 0x1018,
            subindex: 1,
            object_type: ObjectType::Var,
            data_type: DataType::Unsigned32,
            access: AccessType::ReadOnly,
            name: "Vendor ID".to_string(),
            value: OdValue::Unsigned32(vendor_id),
            default_value: None,
        });
        od.add_entry(OdEntry {
            index: 0x1018,
            subindex: 2,
            object_type: ObjectType::Var,
            data_type: DataType::Unsigned32,
            access: AccessType::ReadOnly,
            name: "Product Code".to_string(),
            value: OdValue::Unsigned32(product_code),
            default_value: None,
        });
        od.add_entry(OdEntry {
            index: 0x1018,
            subindex: 3,
            object_type: ObjectType::Var,
            data_type: DataType::Unsigned32,
            access: AccessType::ReadOnly,
            name: "Revision Number".to_string(),
            value: OdValue::Unsigned32(revision),
            default_value: None,
        });
        od.add_entry(OdEntry {
            index: 0x1018,
            subindex: 4,
            object_type: ObjectType::Var,
            data_type: DataType::Unsigned32,
            access: AccessType::ReadOnly,
            name: "Serial Number".to_string(),
            value: OdValue::Unsigned32(serial),
            default_value: None,
        });

        od
    }
}

/// Builder for constructing a [`ConcreteOd`] with a fluent API.
///
/// # Example
/// ```
/// use opencan_canopen_core::concrete_od::OdBuilder;
/// use opencan_canopen_core::od::*;
///
/// let od = OdBuilder::new()
///     .add_var(0x1000, "Device Type", AccessType::ReadOnly, OdValue::Unsigned32(0x00020192))
///     .add_var(0x6040, "Control Word", AccessType::ReadWrite, OdValue::Unsigned16(0))
///     .build();
///
/// assert_eq!(od.len(), 2);
/// assert_eq!(od.read(0x1000, 0).unwrap(), OdValue::Unsigned32(0x00020192));
/// ```
pub struct OdBuilder {
    od: ConcreteOd,
}

impl OdBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            od: ConcreteOd::new(),
        }
    }

    /// Add a simple variable entry (subindex 0, Var type).
    pub fn add_var(mut self, index: u16, name: &str, access: AccessType, value: OdValue) -> Self {
        let data_type = value.data_type().unwrap_or(DataType::Domain);
        let default_value = Some(value.clone());
        self.od.add_entry(OdEntry {
            index,
            subindex: 0,
            object_type: ObjectType::Var,
            data_type,
            access,
            name: name.to_string(),
            value,
            default_value,
        });
        self
    }

    /// Add an entry with full control over all fields.
    pub fn add_entry(mut self, entry: OdEntry) -> Self {
        self.od.add_entry(entry);
        self
    }

    /// Build and return the ConcreteOd.
    pub fn build(self) -> ConcreteOd {
        self.od
    }
}

impl Default for OdBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectDictionary for ConcreteOd {
    fn read(&self, index: u16, subindex: u8) -> Result<OdValue, OdError> {
        match self.entries.get(&(index, subindex)) {
            Some(entry) => Ok(entry.value.clone()),
            None => {
                // Single BTreeMap range lookup to distinguish index vs subindex error
                if self
                    .entries
                    .range((index, 0)..=(index, u8::MAX))
                    .next()
                    .is_some()
                {
                    Err(OdError::SubindexNotFound { index, subindex })
                } else {
                    Err(OdError::IndexNotFound { index })
                }
            }
        }
    }

    fn write(&mut self, index: u16, subindex: u8, value: OdValue) -> Result<(), OdError> {
        let entry = self
            .entries
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

        // Validate data type (skip for OdValue::None which has no inherent type)
        if let Some(value_type) = value.data_type()
            && value_type != entry.data_type
        {
            return Err(OdError::TypeMismatch {
                expected: entry.data_type,
                actual: value_type,
            });
        }

        entry.value = value;
        Ok(())
    }

    fn entry_info(&self, index: u16, subindex: u8) -> Result<EntryInfo, OdError> {
        let entry = self
            .entries
            .get(&(index, subindex))
            .ok_or(OdError::SubindexNotFound { index, subindex })?;

        Ok(EntryInfo {
            index: entry.index,
            subindex: entry.subindex,
            object_type: entry.object_type,
            data_type: entry.data_type,
            access: entry.access,
            default_value: entry.default_value.clone(),
            name: entry.name.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_od() -> ConcreteOd {
        let mut od = ConcreteOd::new();
        od.add_entry(OdEntry {
            index: 0x1000,
            subindex: 0,
            object_type: ObjectType::Var,
            data_type: DataType::Unsigned32,
            access: AccessType::ReadOnly,
            name: "Device Type".to_string(),
            value: OdValue::Unsigned32(0x00020192),
            default_value: None,
        });
        od.add_entry(OdEntry {
            index: 0x6040,
            subindex: 0,
            object_type: ObjectType::Var,
            data_type: DataType::Unsigned16,
            access: AccessType::ReadWrite,
            name: "Control Word".to_string(),
            value: OdValue::Unsigned16(0),
            default_value: None,
        });
        od.add_entry(OdEntry {
            index: 0x6041,
            subindex: 0,
            object_type: ObjectType::Var,
            data_type: DataType::Unsigned16,
            access: AccessType::ReadOnly,
            name: "Status Word".to_string(),
            value: OdValue::Unsigned16(0),
            default_value: None,
        });
        od
    }

    #[test]
    fn test_remove_entry() {
        let mut od = make_test_od();
        assert_eq!(od.len(), 3);

        let removed = od.remove_entry(0x6040, 0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "Control Word");
        assert_eq!(od.len(), 2);
        assert!(!od.contains(0x6040, 0));

        // Removing non-existent entry returns None
        assert!(od.remove_entry(0x9999, 0).is_none());
    }

    #[test]
    fn test_contains() {
        let od = make_test_od();
        assert!(od.contains(0x1000, 0));
        assert!(!od.contains(0x1000, 1));
        assert!(!od.contains(0x9999, 0));
    }

    #[test]
    fn test_contains_index() {
        let od = make_test_od();
        assert!(od.contains_index(0x1000));
        assert!(od.contains_index(0x6040));
        assert!(!od.contains_index(0x2000));
    }

    #[test]
    fn test_range() {
        let od = make_test_od();
        let entries: Vec<_> = od.range(0x6000, 0x6FFF).collect();
        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .all(|e| e.index >= 0x6000 && e.index <= 0x6FFF)
        );
    }

    #[test]
    fn test_subindices() {
        let mut od = ConcreteOd::new();
        for sub in 0..5 {
            od.add_entry(OdEntry {
                index: 0x1018,
                subindex: sub,
                object_type: ObjectType::Var,
                data_type: DataType::Unsigned32,
                access: AccessType::ReadOnly,
                name: format!("Sub {}", sub),
                value: OdValue::Unsigned32(sub as u32),
                default_value: None,
            });
        }

        let subs: Vec<_> = od.subindices(0x1018).collect();
        assert_eq!(subs.len(), 5);
    }

    #[test]
    fn test_get_entry_mut() {
        let mut od = make_test_od();
        let entry = od.get_entry_mut(0x6040, 0).unwrap();
        entry.value = OdValue::Unsigned16(0x000F);

        assert_eq!(od.read(0x6040, 0).unwrap(), OdValue::Unsigned16(0x000F));
    }

    #[test]
    fn test_standard_device() {
        let od =
            ConcreteOd::standard_device(0x00020192, 0x12345678, 0xABCDEF01, 0x00010002, 0x00000042);

        assert_eq!(od.read(0x1000, 0).unwrap(), OdValue::Unsigned32(0x00020192));
        assert_eq!(od.read(0x1001, 0).unwrap(), OdValue::Unsigned8(0));
        assert_eq!(od.read(0x1018, 0).unwrap(), OdValue::Unsigned8(4));
        assert_eq!(od.read(0x1018, 1).unwrap(), OdValue::Unsigned32(0x12345678));
        assert_eq!(od.read(0x1018, 2).unwrap(), OdValue::Unsigned32(0xABCDEF01));
        assert_eq!(od.read(0x1018, 3).unwrap(), OdValue::Unsigned32(0x00010002));
        assert_eq!(od.read(0x1018, 4).unwrap(), OdValue::Unsigned32(0x00000042));
    }

    #[test]
    fn test_builder() {
        let od = OdBuilder::new()
            .add_var(
                0x1000,
                "Device Type",
                AccessType::ReadOnly,
                OdValue::Unsigned32(0x00020192),
            )
            .add_var(
                0x6040,
                "Control Word",
                AccessType::ReadWrite,
                OdValue::Unsigned16(0),
            )
            .build();

        assert_eq!(od.len(), 2);
        assert_eq!(od.read(0x1000, 0).unwrap(), OdValue::Unsigned32(0x00020192));
        assert_eq!(od.read(0x6040, 0).unwrap(), OdValue::Unsigned16(0));
    }
}
