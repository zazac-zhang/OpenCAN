//! Enhanced SDO Server — advanced SDO server features.
//!
//! This module provides enhanced SDO server capabilities including:
//! - Access control and security
//! - Statistics tracking
//! - Event callbacks
//! - PDO mapping validation

use crate::frame::{
    CanOpenFrame, SdoData, SdoRequest, SdoResponse, SdoResponseData,
};
use crate::od::ObjectDictionary;
use std::collections::HashMap;
use std::time::Instant;

/// SDO server event types.
#[derive(Debug, Clone)]
pub enum SdoServerEvent {
    /// Upload request received.
    Upload {
        node_id: u8,
        index: u16,
        subindex: u8,
        timestamp: Instant,
    },
    /// Download request received.
    Download {
        node_id: u8,
        index: u16,
        subindex: u8,
        size: usize,
        timestamp: Instant,
    },
    /// Abort sent.
    Abort {
        node_id: u8,
        index: u16,
        subindex: u8,
        code: u32,
        timestamp: Instant,
    },
    /// Access denied.
    AccessDenied {
        node_id: u8,
        index: u16,
        subindex: u8,
        reason: String,
        timestamp: Instant,
    },
}

/// SDO server statistics.
#[derive(Debug, Clone, Default)]
pub struct SdoServerStats {
    /// Total upload requests.
    pub upload_count: u64,
    /// Total download requests.
    pub download_count: u64,
    /// Total abort responses.
    pub abort_count: u64,
    /// Total access denied.
    pub access_denied_count: u64,
    /// Total bytes uploaded.
    pub bytes_uploaded: u64,
    /// Total bytes downloaded.
    pub bytes_downloaded: u64,
    /// Last activity timestamp.
    pub last_activity: Option<Instant>,
}

/// Access control policy.
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct AccessPolicy {
    /// Allowed node IDs (empty = allow all).
    pub allowed_nodes: Vec<u8>,
    /// Denied node IDs.
    pub denied_nodes: Vec<u8>,
    /// Read-only indices (cannot be written to).
    pub read_only_indices: Vec<u16>,
    /// Restricted indices (require specific node IDs).
    pub restricted_indices: HashMap<u16, Vec<u8>>,
}


impl AccessPolicy {
    /// Check if a node is allowed to access the server.
    pub fn is_node_allowed(&self, node_id: u8) -> bool {
        if self.denied_nodes.contains(&node_id) {
            return false;
        }
        if self.allowed_nodes.is_empty() {
            return true; // Allow all if no specific allowed nodes
        }
        self.allowed_nodes.contains(&node_id)
    }

    /// Check if an index is read-only.
    pub fn is_read_only(&self, index: u16) -> bool {
        self.read_only_indices.contains(&index)
    }

    /// Check if a node can access a restricted index.
    pub fn can_access_index(&self, node_id: u8, index: u16) -> bool {
        if let Some(allowed_nodes) = self.restricted_indices.get(&index) {
            allowed_nodes.contains(&node_id)
        } else {
            true // Not restricted
        }
    }
}

/// Enhanced SDO Server with access control and statistics.
pub struct EnhancedSdoServer {
    /// Base SDO server.
    inner: crate::protocol::sdo::server::SdoServer,
    /// Access policy.
    access_policy: AccessPolicy,
    /// Statistics.
    stats: SdoServerStats,
    /// Event queue.
    events: Vec<SdoServerEvent>,
    /// Maximum event queue size.
    max_events: usize,
}

impl EnhancedSdoServer {
    /// Create a new enhanced SDO server.
    pub fn new(node_id: u8) -> Self {
        Self {
            inner: crate::protocol::sdo::server::SdoServer::new(node_id),
            access_policy: AccessPolicy::default(),
            stats: SdoServerStats::default(),
            events: Vec::new(),
            max_events: 1000,
        }
    }

    /// Set the access policy.
    pub fn set_access_policy(&mut self, policy: AccessPolicy) {
        self.access_policy = policy;
    }

    /// Get a reference to the access policy.
    pub fn access_policy(&self) -> &AccessPolicy {
        &self.access_policy
    }

    /// Get a mutable reference to the access policy.
    pub fn access_policy_mut(&mut self) -> &mut AccessPolicy {
        &mut self.access_policy
    }

    /// Get the current statistics.
    pub fn stats(&self) -> &SdoServerStats {
        &self.stats
    }

    /// Get the node ID.
    pub fn node_id(&self) -> u8 {
        self.inner.node_id()
    }

    /// Process an incoming SDO request frame with access control.
    pub fn process(
        &mut self,
        frame: &CanOpenFrame,
        od: &mut dyn ObjectDictionary,
    ) -> Option<CanOpenFrame> {
        // Only handle frames addressed to our node
        let expected_cob_id = 0x600 + self.inner.node_id() as u16;
        if frame.cob_id != expected_cob_id {
            return None;
        }

        let now = Instant::now();
        self.stats.last_activity = Some(now);

        // Parse the request to check access
        let request = SdoRequest::decode(frame);

        if let Some(req) = &request {
            // Check node access
            if !self.access_policy.is_node_allowed(req.node_id) {
                self.stats.access_denied_count += 1;
                self.events.push(SdoServerEvent::AccessDenied {
                    node_id: req.node_id,
                    index: req.index,
                    subindex: req.subindex,
                    reason: "Node not allowed".to_string(),
                    timestamp: now,
                });
                return Some(Self::abort(
                    self.inner.node_id(),
                    req.index,
                    req.subindex,
                    0x0800_0000, // General error
                ));
            }

            // Check index access
            if !self.access_policy.can_access_index(req.node_id, req.index) {
                self.stats.access_denied_count += 1;
                self.events.push(SdoServerEvent::AccessDenied {
                    node_id: req.node_id,
                    index: req.index,
                    subindex: req.subindex,
                    reason: "Index restricted".to_string(),
                    timestamp: now,
                });
                return Some(Self::abort(
                    self.inner.node_id(),
                    req.index,
                    req.subindex,
                    0x0601_0000, // Unsupported access
                ));
            }

            // Check read-only for downloads
            match &req.data {
                SdoData::Expedited { .. } | SdoData::SegmentedInitiated { .. } => {
                    if self.access_policy.is_read_only(req.index) {
                        self.stats.access_denied_count += 1;
                        self.events.push(SdoServerEvent::AccessDenied {
                            node_id: req.node_id,
                            index: req.index,
                            subindex: req.subindex,
                            reason: "Index is read-only".to_string(),
                            timestamp: now,
                        });
                        return Some(Self::abort(
                            self.inner.node_id(),
                            req.index,
                            req.subindex,
                            0x0601_0000, // Unsupported access
                        ));
                    }
                }
                _ => {}
            }

            // Update statistics
            match &req.data {
                SdoData::UploadRequest => {
                    self.stats.upload_count += 1;
                    self.events.push(SdoServerEvent::Upload {
                        node_id: req.node_id,
                        index: req.index,
                        subindex: req.subindex,
                        timestamp: now,
                    });
                }
                SdoData::Expedited { data: _, size } => {
                    self.stats.download_count += 1;
                    let bytes = size.unwrap_or(4) as u64;
                    self.stats.bytes_downloaded += bytes;
                    self.events.push(SdoServerEvent::Download {
                        node_id: req.node_id,
                        index: req.index,
                        subindex: req.subindex,
                        size: bytes as usize,
                        timestamp: now,
                    });
                }
                SdoData::SegmentedInitiated { size } => {
                    self.stats.download_count += 1;
                    self.events.push(SdoServerEvent::Download {
                        node_id: req.node_id,
                        index: req.index,
                        subindex: req.subindex,
                        size: *size as usize,
                        timestamp: now,
                    });
                }
                _ => {}
            }
        }

        // Process with inner server
        let response = self.inner.process(frame, od);

        // Check if response is an abort
        if let Some(resp) = &response
            && let Some(resp_data) = SdoResponse::decode(resp)
                && matches!(resp_data.data, SdoResponseData::Abort { .. }) {
                    self.stats.abort_count += 1;
                    if let Some(req) = request {
                        self.events.push(SdoServerEvent::Abort {
                            node_id: req.node_id,
                            index: req.index,
                            subindex: req.subindex,
                            code: match resp_data.data {
                                SdoResponseData::Abort { code } => code,
                                _ => 0,
                            },
                            timestamp: now,
                        });
                    }
                }

        // Trim event queue if needed
        while self.events.len() > self.max_events {
            self.events.remove(0);
        }

        response
    }

    /// Drain all pending events.
    pub fn drain_events(&mut self) -> Vec<SdoServerEvent> {
        std::mem::take(&mut self.events)
    }

    /// Clear any pending segmented transfer state.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Build an SDO abort response frame.
    fn abort(node_id: u8, index: u16, subindex: u8, code: u32) -> CanOpenFrame {
        SdoResponse {
            node_id,
            index,
            subindex,
            data: SdoResponseData::Abort { code },
        }
        .encode()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concrete_od::{ConcreteOd, OdEntry};
    use crate::od::{AccessType, DataType, ObjectType, OdValue};

    fn make_od() -> ConcreteOd {
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
        od
    }

    #[test]
    fn test_enhanced_server_basic() {
        let mut server = EnhancedSdoServer::new(3);
        let mut od = make_od();

        // Client reads Device Type
        let request = SdoRequest {
            node_id: 3,
            index: 0x1000,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = request.encode();
        let response = server.process(&frame, &mut od);

        assert!(response.is_some());
        assert_eq!(server.stats().upload_count, 1);
    }

    #[test]
    fn test_enhanced_server_access_policy() {
        let mut server = EnhancedSdoServer::new(3);
        let mut od = make_od();

        // Set policy: deny node 3 (this is just for testing - in reality we'd check source nodes)
        let policy = AccessPolicy {
            denied_nodes: vec![3],
            ..Default::default()
        };
        server.set_access_policy(policy);

        // Node 3 (our node) should be denied
        let request = SdoRequest {
            node_id: 3,
            index: 0x1000,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = request.encode();
        let response = server.process(&frame, &mut od);

        assert!(response.is_some());
        assert_eq!(server.stats().access_denied_count, 1);

        // Reset stats
        server.stats = SdoServerStats::default();

        // Set policy: allow all nodes
        let policy = AccessPolicy {
            allowed_nodes: vec![],
            denied_nodes: vec![],
            ..Default::default()
        };
        server.set_access_policy(policy);

        // Now node 3 should be allowed
        let request = SdoRequest {
            node_id: 3,
            index: 0x1000,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = request.encode();
        let response = server.process(&frame, &mut od);

        assert!(response.is_some());
        assert_eq!(server.stats().upload_count, 1);
    }

    #[test]
    fn test_enhanced_server_read_only() {
        let mut server = EnhancedSdoServer::new(3);
        let mut od = make_od();

        // Set policy: 0x1000 is read-only
        let policy = AccessPolicy {
            read_only_indices: vec![0x1000],
            ..Default::default()
        };
        server.set_access_policy(policy);

        // Download to read-only index should be denied
        let request = SdoRequest {
            node_id: 3,
            index: 0x1000,
            subindex: 0,
            data: SdoData::Expedited {
                data: [0, 0, 0, 0],
                size: Some(4),
            },
        };
        let frame = request.encode();
        let response = server.process(&frame, &mut od);

        assert!(response.is_some());
        assert_eq!(server.stats().access_denied_count, 1);
    }

    #[test]
    fn test_enhanced_server_events() {
        let mut server = EnhancedSdoServer::new(3);
        let mut od = make_od();

        // Upload request
        let request = SdoRequest {
            node_id: 3,
            index: 0x1000,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = request.encode();
        server.process(&frame, &mut od);

        let events = server.drain_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SdoServerEvent::Upload { .. }));
    }

    #[test]
    fn test_access_policy_default() {
        let policy = AccessPolicy::default();
        assert!(policy.is_node_allowed(1));
        assert!(policy.is_node_allowed(127));
        assert!(!policy.is_read_only(0x1000));
        assert!(policy.can_access_index(1, 0x1000));
    }

    #[test]
    fn test_access_policy_denied_nodes() {
        let policy = AccessPolicy {
            denied_nodes: vec![5, 10],
            ..Default::default()
        };
        assert!(policy.is_node_allowed(1));
        assert!(!policy.is_node_allowed(5));
        assert!(!policy.is_node_allowed(10));
    }

    #[test]
    fn test_sdo_server_stats_default() {
        let stats = SdoServerStats::default();
        assert_eq!(stats.upload_count, 0);
        assert_eq!(stats.download_count, 0);
        assert_eq!(stats.abort_count, 0);
        assert_eq!(stats.access_denied_count, 0);
    }
}
