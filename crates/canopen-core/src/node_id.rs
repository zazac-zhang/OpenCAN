//! CANOpen Node ID type.
//!
//! A valid CANOpen node ID is in the range 1..=127 (7-bit).
//! Node ID 0 is reserved for NMT broadcast.

use std::fmt;

/// A validated CANOpen node ID (1..=127).
///
/// # Example
/// ```
/// use opencan_canopen_core::node_id::NodeId;
///
/// let id = NodeId::new(5).unwrap();
/// assert_eq!(id.get(), 5);
///
/// // Node ID 0 is invalid (reserved for broadcast)
/// assert!(NodeId::new(0).is_err());
///
/// // Node ID 128 is invalid (exceeds 7-bit range)
/// assert!(NodeId::new(128).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u8);

/// Error returned when constructing a `NodeId` from an invalid value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidNodeId(pub u8);

impl fmt::Display for InvalidNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid node ID {}: must be in range 1..=127",
            self.0
        )
    }
}

impl std::error::Error for InvalidNodeId {}

impl NodeId {
    /// Minimum valid node ID.
    pub const MIN: u8 = 1;
    /// Maximum valid node ID.
    pub const MAX: u8 = 127;

    /// Create a new `NodeId`. Returns `Err` if value is 0 or > 127.
    pub fn new(id: u8) -> Result<Self, InvalidNodeId> {
        if (Self::MIN..=Self::MAX).contains(&id) {
            Ok(Self(id))
        } else {
            Err(InvalidNodeId(id))
        }
    }

    /// Create a `NodeId` without validation.
    ///
    /// # Safety
    /// The caller must ensure `id` is in range 1..=127.
    /// Using an invalid value may cause protocol errors.
    pub const fn new_unchecked(id: u8) -> Self {
        Self(id)
    }

    /// Get the raw node ID value.
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Get the NMT broadcast node ID (0).
    pub const fn broadcast() -> u8 {
        0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u8> for NodeId {
    type Error = InvalidNodeId;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<NodeId> for u8 {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

impl From<NodeId> for u16 {
    fn from(id: NodeId) -> Self {
        id.0 as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_node_ids() {
        assert_eq!(NodeId::new(1).unwrap().get(), 1);
        assert_eq!(NodeId::new(127).unwrap().get(), 127);
        assert_eq!(NodeId::new(64).unwrap().get(), 64);
    }

    #[test]
    fn test_invalid_node_ids() {
        assert!(NodeId::new(0).is_err());
        assert!(NodeId::new(128).is_err());
        assert!(NodeId::new(255).is_err());
    }

    #[test]
    fn test_node_id_conversions() {
        let id = NodeId::new(5).unwrap();
        let raw: u8 = id.into();
        assert_eq!(raw, 5);

        let wide: u16 = id.into();
        assert_eq!(wide, 5);

        let id2 = NodeId::try_from(10u8).unwrap();
        assert_eq!(id2.get(), 10);
    }

    #[test]
    fn test_node_id_display() {
        let id = NodeId::new(42).unwrap();
        assert_eq!(format!("{}", id), "42");
    }

    #[test]
    fn test_node_id_ordering() {
        let a = NodeId::new(1).unwrap();
        let b = NodeId::new(127).unwrap();
        assert!(a < b);
    }

    #[test]
    fn test_invalid_node_id_display() {
        let err = InvalidNodeId(0);
        assert_eq!(format!("{}", err), "invalid node ID 0: must be in range 1..=127");
    }
}
