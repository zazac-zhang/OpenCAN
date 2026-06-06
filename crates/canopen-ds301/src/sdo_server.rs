//! SDO Server — responds to SDO requests from other nodes.
//!
//! The SDO server listens on COB-ID 0x600 + node_id for incoming SDO requests,
//! reads/writes the local ObjectDictionary, and sends responses on 0x580 + node_id.
//!
//! Supports: Expedited and Segmented transfers.

use opencan_canopen_core::frame::{
    CanOpenFrame, SdoData, SdoRequest, SdoResponse, SdoResponseData,
};
use opencan_canopen_core::od::{ObjectDictionary, OdValue};

/// SDO Server — responds to SDO requests from other nodes.
pub struct SdoServer {
    node_id: u8,
    /// Segmented upload state: (index, subindex, data, offset)
    upload_state: Option<UploadState>,
    /// Segmented download state: (index, subindex, data)
    download_state: Option<DownloadState>,
    /// Block upload state
    block_upload: Option<BlockUploadState>,
}

/// State for segmented upload (server → client).
struct UploadState {
    index: u16,
    subindex: u8,
    data: Vec<u8>,
    offset: usize,
    toggle: bool,
}

/// State for segmented download (client → server).
struct DownloadState {
    index: u16,
    subindex: u8,
    data: Vec<u8>,
    expected_size: usize,
    toggle: bool,
}

/// State for block upload.
/// TODO: Implement block transfer per CiA 301 specification.
#[allow(dead_code)]
struct BlockUploadState {
    index: u16,
    subindex: u8,
    data: Vec<u8>,
    offset: usize,
    seq: u8,
    block_size: u8,
}

impl SdoServer {
    /// Create a new SDO server for the given node ID.
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
            upload_state: None,
            download_state: None,
            block_upload: None,
        }
    }

    /// Process an incoming SDO request frame.
    ///
    /// Returns `Some(response_frame)` if this was an SDO request to us,
    /// `None` otherwise (pass through to other processing).
    pub fn process(
        &mut self,
        frame: &CanOpenFrame,
        od: &mut dyn ObjectDictionary,
    ) -> Option<CanOpenFrame> {
        // Only handle frames addressed to our node
        let expected_cob_id = 0x600 + self.node_id as u16;
        if frame.cob_id != expected_cob_id {
            return None;
        }

        let cmd = frame.data[0];

        // Check for upload segment request (cs=2: 0x40 or cs=3: 0x60)
        if (cmd & 0xE0 == 0x40 || cmd & 0xE0 == 0x60) && self.upload_state.is_some() {
            return self.handle_upload_segment(frame);
        }

        // Check for download segment (cs=0, cmd=0x00-0x1F)
        if cmd & 0xE0 == 0x00 && self.download_state.is_some() {
            return self.handle_download_segment(frame, od);
        }

        // Parse as initial request
        let request = match SdoRequest::decode(frame) {
            Some(r) => r,
            None => {
                return Some(Self::abort(self.node_id, 0, 0, 0x0504_0001)); // CS not valid
            }
        };

        match &request.data {
            SdoData::UploadRequest => self.handle_upload(request.index, request.subindex, od),
            SdoData::Expedited { data, size } => {
                self.handle_expedited_download(request.index, request.subindex, data, *size, od)
            }
            SdoData::SegmentedInitiated { size } => {
                self.handle_segmented_download_init(request.index, request.subindex, *size as usize)
            }
            SdoData::Abort { .. } => {
                // Client sent abort — clear any pending state
                self.upload_state = None;
                self.download_state = None;
                None
            }
            _ => Some(Self::abort(
                self.node_id,
                request.index,
                request.subindex,
                0x0504_0001,
            )),
        }
    }

    /// Handle upload request (client reads from our OD).
    fn handle_upload(
        &mut self,
        index: u16,
        subindex: u8,
        od: &mut dyn ObjectDictionary,
    ) -> Option<CanOpenFrame> {
        let value = match od.read(index, subindex) {
            Ok(v) => v,
            Err(e) => {
                return Some(Self::abort(
                    self.node_id,
                    index,
                    subindex,
                    od_error_to_sdo_abort(&e),
                ));
            }
        };

        let bytes = value.to_bytes();

        if bytes.len() <= 4 {
            // Expedited response
            let mut data = [0u8; 4];
            let len = bytes.len();
            data[..len].copy_from_slice(&bytes);

            Some(
                SdoResponse {
                    node_id: self.node_id,
                    index,
                    subindex,
                    data: SdoResponseData::Expedited {
                        data,
                        size: Some(len as u8),
                    },
                }
                .encode(),
            )
        } else {
            // Segmented upload — initiate
            let total_size = bytes.len();
            self.upload_state = Some(UploadState {
                index,
                subindex,
                data: bytes,
                offset: 0,
                toggle: false,
            });

            Some(
                SdoResponse {
                    node_id: self.node_id,
                    index,
                    subindex,
                    data: SdoResponseData::SegmentedInitiated {
                        size: total_size as u32,
                    },
                }
                .encode(),
            )
        }
    }

    /// Handle upload segment request (client continues reading).
    fn handle_upload_segment(&mut self, frame: &CanOpenFrame) -> Option<CanOpenFrame> {
        let state = match &mut self.upload_state {
            Some(s) => s,
            None => return None,
        };

        let cmd = frame.data[0];
        let client_toggle = cmd & 0x10 != 0;

        // Verify toggle bit
        if client_toggle != state.toggle {
            let idx = state.index;
            let sub = state.subindex;
            self.upload_state = None;
            return Some(Self::abort(self.node_id, idx, sub, 0x0503_0000));
        }

        let remaining = state.data.len() - state.offset;
        let seg_len = remaining.min(7);
        let is_last = state.offset + seg_len >= state.data.len();

        let mut seg_data = [0u8; 7];
        seg_data[..seg_len].copy_from_slice(&state.data[state.offset..state.offset + seg_len]);

        // Copy values before modifying state
        let node_id = self.node_id;
        let index = state.index;
        let subindex = state.subindex;

        state.offset += seg_len;
        state.toggle = !state.toggle;

        if is_last {
            self.upload_state = None;
        }

        Some(
            SdoResponse {
                node_id,
                index,
                subindex,
                data: SdoResponseData::Segment {
                    toggle: client_toggle,
                    last: is_last,
                    data: seg_data,
                    size: Some(seg_len as u8),
                },
            }
            .encode(),
        )
    }

    /// Handle expedited download (client writes ≤4 bytes to our OD).
    fn handle_expedited_download(
        &mut self,
        index: u16,
        subindex: u8,
        data: &[u8; 4],
        size: Option<u8>,
        od: &mut dyn ObjectDictionary,
    ) -> Option<CanOpenFrame> {
        // Get the entry info to determine data type
        let info = match od.entry_info(index, subindex) {
            Ok(i) => i,
            Err(e) => {
                return Some(Self::abort(
                    self.node_id,
                    index,
                    subindex,
                    od_error_to_sdo_abort(&e),
                ));
            }
        };

        let bytes = if let Some(s) = size {
            &data[..s as usize]
        } else {
            data.as_slice()
        };

        let value = match OdValue::from_bytes(info.data_type, bytes) {
            Some(v) => v,
            None => {
                return Some(Self::abort(self.node_id, index, subindex, 0x0607_0010)); // Data type mismatch
            }
        };

        match od.write(index, subindex, value) {
            Ok(()) => Some(
                SdoResponse {
                    node_id: self.node_id,
                    index,
                    subindex,
                    data: SdoResponseData::DownloadConfirmed,
                }
                .encode(),
            ),
            Err(e) => Some(Self::abort(
                self.node_id,
                index,
                subindex,
                od_error_to_sdo_abort(&e),
            )),
        }
    }

    /// Handle segmented download initiate (client starts writing >4 bytes).
    fn handle_segmented_download_init(
        &mut self,
        index: u16,
        subindex: u8,
        size: usize,
    ) -> Option<CanOpenFrame> {
        self.download_state = Some(DownloadState {
            index,
            subindex,
            data: Vec::with_capacity(size),
            expected_size: size,
            toggle: false,
        });

        Some(
            SdoResponse {
                node_id: self.node_id,
                index,
                subindex,
                data: SdoResponseData::DownloadConfirmed,
            }
            .encode(),
        )
    }

    /// Handle download segment (client continues writing).
    fn handle_download_segment(
        &mut self,
        frame: &CanOpenFrame,
        od: &mut dyn ObjectDictionary,
    ) -> Option<CanOpenFrame> {
        let state = match &mut self.download_state {
            Some(s) => s,
            None => return None,
        };

        let cmd = frame.data[0];
        let client_toggle = cmd & 0x10 != 0;
        let is_last = cmd & 0x01 != 0;
        let seg_size = if cmd & 0x0E != 0 {
            (7 - ((cmd >> 1) & 0x07)) as usize
        } else {
            7
        };

        // Verify toggle bit
        if client_toggle != state.toggle {
            return Some(Self::abort(
                self.node_id,
                state.index,
                state.subindex,
                0x0503_0000,
            ));
        }

        state.data.extend_from_slice(&frame.data[1..1 + seg_size]);
        state.toggle = !state.toggle;

        if is_last || state.data.len() >= state.expected_size {
            // Download complete — write to OD
            let info = match od.entry_info(state.index, state.subindex) {
                Ok(i) => i,
                Err(e) => {
                    let resp = Self::abort(
                        self.node_id,
                        state.index,
                        state.subindex,
                        od_error_to_sdo_abort(&e),
                    );
                    self.download_state = None;
                    return Some(resp);
                }
            };

            let value = match OdValue::from_bytes(info.data_type, &state.data) {
                Some(v) => v,
                None => {
                    let resp = Self::abort(self.node_id, state.index, state.subindex, 0x0607_0010);
                    self.download_state = None;
                    return Some(resp);
                }
            };

            let index = state.index;
            let subindex = state.subindex;
            self.download_state = None;

            match od.write(index, subindex, value) {
                Ok(()) => Some(
                    SdoResponse {
                        node_id: self.node_id,
                        index,
                        subindex,
                        data: SdoResponseData::DownloadConfirmed,
                    }
                    .encode(),
                ),
                Err(e) => Some(Self::abort(
                    self.node_id,
                    index,
                    subindex,
                    od_error_to_sdo_abort(&e),
                )),
            }
        } else {
            // More segments expected
            Some(
                SdoResponse {
                    node_id: self.node_id,
                    index: state.index,
                    subindex: state.subindex,
                    data: SdoResponseData::DownloadConfirmed,
                }
                .encode(),
            )
        }
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

    /// Get the node ID.
    pub fn node_id(&self) -> u8 {
        self.node_id
    }

    /// Clear any pending segmented transfer state.
    pub fn reset(&mut self) {
        self.upload_state = None;
        self.download_state = None;
        self.block_upload = None;
    }
}

/// Convert an OD error to an SDO abort code.
fn od_error_to_sdo_abort(err: &opencan_canopen_core::error::OdError) -> u32 {
    use opencan_canopen_core::error::OdError;
    match err {
        OdError::IndexNotFound { .. } => 0x0602_0000,
        OdError::SubindexNotFound { .. } => 0x0609_0011,
        OdError::AccessDenied { .. } => 0x0601_0000,
        OdError::TypeMismatch { .. } => 0x0607_0010,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencan_canopen_core::concrete_od::{ConcreteOd, OdEntry};
    use opencan_canopen_core::od::{AccessType, DataType, ObjectType};

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
        od.add_entry(OdEntry {
            index: 0x2000,
            subindex: 0,
            object_type: ObjectType::Var,
            data_type: DataType::VisibleString,
            access: AccessType::ReadWrite,
            name: "Test String".to_string(),
            value: OdValue::VisibleString("Hello".to_string()),
            default_value: None,
        });
        od.add_entry(OdEntry {
            index: 0x2001,
            subindex: 0,
            object_type: ObjectType::Var,
            data_type: DataType::VisibleString,
            access: AccessType::ReadWrite,
            name: "Long String".to_string(),
            value: OdValue::VisibleString("This is a multi-segment test string!".to_string()),
            default_value: None,
        });
        od
    }

    #[test]
    fn test_expedited_upload() {
        let mut server = SdoServer::new(3);
        let mut od = make_od();

        // Client reads Device Type (0x1000:0)
        let request = SdoRequest {
            node_id: 3,
            index: 0x1000,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = request.encode();
        let response = server.process(&frame, &mut od).unwrap();

        // Should be on 0x583
        assert_eq!(response.cob_id, 0x583);

        // Decode response
        let resp = SdoResponse::decode(&response).unwrap();
        assert_eq!(resp.index, 0x1000);
        assert_eq!(resp.subindex, 0);
        match resp.data {
            SdoResponseData::Expedited { data, size } => {
                assert_eq!(size, Some(4));
                assert_eq!(data, 0x00020192u32.to_le_bytes());
            }
            _ => panic!("Expected expedited response"),
        }
    }

    #[test]
    fn test_expedited_download() {
        let mut server = SdoServer::new(3);
        let mut od = make_od();

        // Client writes Control Word (0x6040:0) = 0x000F
        let request = SdoRequest {
            node_id: 3,
            index: 0x6040,
            subindex: 0,
            data: SdoData::Expedited {
                data: [0x0F, 0x00, 0, 0],
                size: Some(2),
            },
        };
        let frame = request.encode();
        let response = server.process(&frame, &mut od).unwrap();

        let resp = SdoResponse::decode(&response).unwrap();
        assert!(matches!(resp.data, SdoResponseData::DownloadConfirmed));

        // Verify OD was written
        assert_eq!(od.read(0x6040, 0).unwrap(), OdValue::Unsigned16(0x000F));
    }

    #[test]
    fn test_upload_nonexistent_object() {
        let mut server = SdoServer::new(3);
        let mut od = make_od();

        let request = SdoRequest {
            node_id: 3,
            index: 0x9999,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = request.encode();
        let response = server.process(&frame, &mut od).unwrap();

        let resp = SdoResponse::decode(&response).unwrap();
        match resp.data {
            SdoResponseData::Abort { code } => assert_eq!(code, 0x0602_0000),
            _ => panic!("Expected abort"),
        }
    }

    #[test]
    fn test_write_read_only_object() {
        let mut server = SdoServer::new(3);
        let mut od = make_od();

        // Try to write to Device Type (read-only)
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
        let response = server.process(&frame, &mut od).unwrap();

        let resp = SdoResponse::decode(&response).unwrap();
        match resp.data {
            SdoResponseData::Abort { code } => assert_eq!(code, 0x0601_0000),
            _ => panic!("Expected abort for write to read-only"),
        }
    }

    #[test]
    fn test_wrong_node_id_ignored() {
        let mut server = SdoServer::new(3);
        let mut od = make_od();

        // Request for node 5, not us (node 3)
        let request = SdoRequest {
            node_id: 5,
            index: 0x1000,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = request.encode();
        assert!(server.process(&frame, &mut od).is_none());
    }

    #[test]
    fn test_segmented_upload() {
        let mut server = SdoServer::new(3);
        let mut od = make_od();

        // Client reads Test String (0x2000:0) = "Hello" (5 bytes, needs segmented)
        let request = SdoRequest {
            node_id: 3,
            index: 0x2000,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = request.encode();
        let response = server.process(&frame, &mut od).unwrap();

        // Should be segmented init
        let resp = SdoResponse::decode(&response).unwrap();
        match resp.data {
            SdoResponseData::SegmentedInitiated { size } => assert_eq!(size, 5),
            _ => panic!("Expected segmented initiated"),
        }

        // Request first segment (cs=2, toggle at bit 4 = 0x10, so 0x50 has toggle=1)
        let seg_req = CanOpenFrame::new(0x603, [0x40, 0, 0, 0, 0, 0, 0, 0]); // cs=2, toggle=0
        let response = server.process(&seg_req, &mut od).unwrap();
        let resp = SdoResponse::decode(&response).unwrap();
        match resp.data {
            SdoResponseData::Segment {
                toggle,
                last,
                data,
                size,
            } => {
                assert!(!toggle); // server echoes client toggle
                assert!(last); // 5 bytes fits in one segment
                assert_eq!(size, Some(5));
                assert_eq!(&data[..5], b"Hello");
            }
            _ => panic!("Expected segment response"),
        }
    }

    #[test]
    fn test_segmented_upload_multi_segment() {
        let mut server = SdoServer::new(3);
        let mut od = make_od();

        // Upload 0x2001:0 = "This is a multi-segment test string!" (38 bytes, needs 6 segments)
        let request = SdoRequest {
            node_id: 3,
            index: 0x2001,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        let frame = request.encode();
        let response = server.process(&frame, &mut od).unwrap();

        // Should be segmented init with total size
        let resp = SdoResponse::decode(&response).unwrap();
        match resp.data {
            SdoResponseData::SegmentedInitiated { size } => assert_eq!(size, 36),
            _ => panic!("Expected segmented initiated"),
        }

        let mut all_data = Vec::new();
        let mut toggle = false;
        let expected = b"This is a multi-segment test string!";

        loop {
            // Send segment request with alternating toggle
            // Upload segment request: cs=2 (0x40 base), toggle at bit 4 (0x10)
            let cmd = if toggle { 0x50 } else { 0x40 };
            let seg_req = CanOpenFrame::new(0x603, [cmd, 0, 0, 0, 0, 0, 0, 0]);
            let seg_num = all_data.len() / 7 + 1;
            let response = server.process(&seg_req, &mut od).unwrap();
            let resp = SdoResponse::decode(&response).unwrap();

            match resp.data {
                SdoResponseData::Segment {
                    toggle: t,
                    last,
                    data,
                    size,
                } => {
                    assert_eq!(t, toggle, "Server toggle mismatch on segment {}", seg_num);
                    let len = size.unwrap_or(7) as usize;
                    all_data.extend_from_slice(&data[..len]);
                    if last {
                        break;
                    }
                }
                other => panic!(
                    "Expected segment response on seg {}, got {:?} (abort code 0x{:08X})",
                    seg_num,
                    other,
                    match other {
                        SdoResponseData::Abort { code } => code,
                        _ => 0,
                    }
                ),
            }
            toggle = !toggle;
        }

        assert_eq!(
            &all_data[..],
            expected,
            "Multi-segment upload data mismatch"
        );
    }

    #[test]
    fn test_segmented_download() {
        let mut server = SdoServer::new(3);
        let mut od = make_od();

        // Client initiates segmented download to 0x2000:0 (7 bytes)
        let request = SdoRequest {
            node_id: 3,
            index: 0x2000,
            subindex: 0,
            data: SdoData::SegmentedInitiated { size: 7 },
        };
        let frame = request.encode();
        let response = server.process(&frame, &mut od).unwrap();

        let resp = SdoResponse::decode(&response).unwrap();
        assert!(matches!(resp.data, SdoResponseData::DownloadConfirmed));

        // Send segment: "World!" (6 bytes, last)
        let mut seg_data = [0u8; 7];
        seg_data[..6].copy_from_slice(b"World!");
        let _seg_frame = SdoRequest {
            node_id: 3,
            index: 0x2000,
            subindex: 0,
            data: SdoData::Segment {
                toggle: false,
                last: true,
                data: seg_data,
                size: Some(6),
            },
        }
        .encode();
        // The segment frame has cs=0, not cs=1, so it won't parse as SdoRequest
        // Let's build it manually
        let mut raw = [0u8; 8];
        raw[0] = 0x03; // toggle=0, last=1, n=1 (6 bytes of data)
        raw[1..7].copy_from_slice(b"World!");
        let seg_frame = CanOpenFrame::new(0x603, raw);

        let response = server.process(&seg_frame, &mut od).unwrap();
        let resp = SdoResponse::decode(&response).unwrap();
        assert!(matches!(resp.data, SdoResponseData::DownloadConfirmed));

        // Note: the string was "Hello" originally, now we wrote "World!" (6 bytes)
        // But the OD entry was initialized with "Hello". After download it should be "World!"
        // Actually we need to check: the segmented download wrote 6 bytes "World!" but
        // the from_bytes for VisibleString will interpret all bytes. Let's verify:
        match od.read(0x2000, 0).unwrap() {
            OdValue::VisibleString(s) => assert_eq!(s, "World!"),
            other => panic!("Expected VisibleString, got {:?}", other),
        }
    }

    #[test]
    fn test_toggle_bit_error() {
        let mut server = SdoServer::new(3);
        let mut od = make_od();

        // Initiate segmented upload
        let request = SdoRequest {
            node_id: 3,
            index: 0x2000,
            subindex: 0,
            data: SdoData::UploadRequest,
        };
        server.process(&request.encode(), &mut od);

        // Send segment with wrong toggle (should be false, send true)
        let seg_req = CanOpenFrame::new(0x603, [0x50, 0, 0, 0, 0, 0, 0, 0]); // cs=2, toggle=1 (wrong!)
        let response = server.process(&seg_req, &mut od).unwrap();
        let resp = SdoResponse::decode(&response).unwrap();
        match resp.data {
            SdoResponseData::Abort { code } => assert_eq!(code, 0x0503_0000),
            _ => panic!("Expected toggle error abort"),
        }
    }
}
