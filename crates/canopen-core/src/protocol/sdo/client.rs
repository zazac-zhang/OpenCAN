//! SDO (Service Data Object) client implementation.

use crate::frame::{SdoData, SdoRequest, SdoResponse, SdoResponseData};
use crate::od::{DataType, OdValue};
use crate::{CanDriver, CanOpenError};
use std::time::Duration;
use tokio::time::timeout;

/// SDO client for reading/writing remote node object dictionaries.
pub struct SdoClient<C: CanDriver> {
    can: C,
    timeout: Duration,
}

impl<C: CanDriver> SdoClient<C> {
    pub fn new(can: C, timeout: Duration) -> Self {
        Self { can, timeout }
    }

    /// SDO Upload — read a value from remote node's OD.
    pub async fn upload(
        &mut self,
        node_id: u8,
        index: u16,
        subindex: u8,
    ) -> Result<OdValue, CanOpenError> {
        self.upload_with_type(node_id, index, subindex, DataType::Unsigned32)
            .await
    }

    /// SDO Upload with explicit data type hint.
    pub async fn upload_with_type(
        &mut self,
        node_id: u8,
        index: u16,
        subindex: u8,
        data_type: DataType,
    ) -> Result<OdValue, CanOpenError> {
        // Send initiate upload request
        let request = SdoRequest {
            node_id,
            index,
            subindex,
            data: SdoData::UploadRequest,
        };
        let frame = request.encode();
        self.can.send(&frame)?;

        // Wait for response (with timeout)
        let response_frame = self.recv_with_timeout().await?;
        let response = SdoResponse::decode(&response_frame)
            .ok_or_else(|| CanOpenError::Protocol("Invalid SDO response".to_string()))?;

        if response.index != index || response.subindex != subindex {
            return Err(CanOpenError::Protocol(
                "SDO response index/subindex mismatch".to_string(),
            ));
        }

        match response.data {
            SdoResponseData::Expedited { data, size } => {
                let bytes = if let Some(s) = size {
                    &data[..s as usize]
                } else {
                    &data
                };
                OdValue::from_bytes(data_type, bytes).ok_or_else(|| {
                    CanOpenError::Protocol("Failed to decode expedited data".to_string())
                })
            }
            SdoResponseData::SegmentedInitiated { size } => {
                // Segmented upload — read segments
                self.upload_segments(node_id, size as usize).await
            }
            SdoResponseData::Abort { code } => Err(CanOpenError::SdoAbort {
                code,
                reason: sdo_abort_reason(code),
            }),
            _ => Err(CanOpenError::Protocol(
                "Unexpected SDO response".to_string(),
            )),
        }
    }

    /// Read segmented upload data.
    async fn upload_segments(
        &mut self,
        node_id: u8,
        total_size: usize,
    ) -> Result<OdValue, CanOpenError> {
        let mut data = Vec::with_capacity(total_size);
        let mut toggle = false;

        loop {
            // Send upload segment request
            let mut req_data = [0u8; 8];
            req_data[0] = if toggle { 0x60 } else { 0x40 }; // toggle bit
            let frame =
                crate::frame::CanOpenFrame::new(0x600 + node_id as u16, req_data);
            self.can.send(&frame)?;

            // Receive segment (with timeout)
            let response_frame = self.recv_with_timeout().await?;
            let cmd = response_frame.data[0];
            let is_last = cmd & 0x01 != 0;
            let seg_size = if cmd & 0x0E != 0 {
                let n = (cmd >> 1) & 0x07;
                7 - n
            } else {
                7
            };

            data.extend_from_slice(&response_frame.data[1..1 + seg_size as usize]);
            toggle = !toggle;

            if is_last || data.len() >= total_size {
                break;
            }
        }

        Ok(OdValue::Domain(data))
    }

    /// SDO Download — write a value to remote node's OD.
    pub async fn download(
        &mut self,
        node_id: u8,
        index: u16,
        subindex: u8,
        value: &OdValue,
    ) -> Result<(), CanOpenError> {
        let bytes = value.to_bytes();

        if bytes.len() <= 4 {
            // Expedited transfer
            let mut data = [0u8; 4];
            data[..bytes.len()].copy_from_slice(&bytes);

            let request = SdoRequest {
                node_id,
                index,
                subindex,
                data: SdoData::Expedited {
                    data,
                    size: Some(bytes.len() as u8),
                },
            };
            self.can.send(&request.encode())?;
            self.wait_download_confirm(index, subindex).await
        } else {
            // Segmented download
            self.download_segments(node_id, index, subindex, &bytes)
                .await
        }
    }

    /// Segmented download.
    async fn download_segments(
        &mut self,
        node_id: u8,
        index: u16,
        subindex: u8,
        data: &[u8],
    ) -> Result<(), CanOpenError> {
        // Send initiate segmented download
        let request = SdoRequest {
            node_id,
            index,
            subindex,
            data: SdoData::SegmentedInitiated {
                size: data.len() as u32,
            },
        };
        self.can.send(&request.encode())?;
        self.wait_download_confirm(index, subindex).await?;

        // Send segments
        let mut offset = 0;
        let mut toggle = false;

        while offset < data.len() {
            let remaining = data.len() - offset;
            let seg_len = remaining.min(7);
            let is_last = offset + seg_len >= data.len();

            let mut seg_data = [0u8; 7];
            seg_data[..seg_len].copy_from_slice(&data[offset..offset + seg_len]);

            let request = SdoRequest {
                node_id,
                index,
                subindex,
                data: SdoData::Segment {
                    toggle,
                    last: is_last,
                    data: seg_data,
                    size: Some(seg_len as u8),
                },
            };
            self.can.send(&request.encode())?;

            // Wait for segment confirmation (with timeout)
            let response_frame = self.recv_with_timeout().await?;
            let response = SdoResponse::decode(&response_frame)
                .ok_or_else(|| CanOpenError::Protocol("Invalid SDO response".to_string()))?;

            match response.data {
                SdoResponseData::DownloadConfirmed => {}
                SdoResponseData::Abort { code } => {
                    return Err(CanOpenError::SdoAbort {
                        code,
                        reason: sdo_abort_reason(code),
                    });
                }
                _ => {
                    return Err(CanOpenError::Protocol(
                        "Unexpected SDO response".to_string(),
                    ));
                }
            }

            offset += seg_len;
            toggle = !toggle;
        }

        Ok(())
    }

    /// Wait for download confirmation.
    async fn wait_download_confirm(
        &mut self,
        index: u16,
        subindex: u8,
    ) -> Result<(), CanOpenError> {
        let response_frame = self.recv_with_timeout().await?;
        let response = SdoResponse::decode(&response_frame)
            .ok_or_else(|| CanOpenError::Protocol("Invalid SDO response".to_string()))?;

        if response.index != index || response.subindex != subindex {
            return Err(CanOpenError::Protocol(
                "SDO response index/subindex mismatch".to_string(),
            ));
        }

        match response.data {
            SdoResponseData::DownloadConfirmed => Ok(()),
            SdoResponseData::Abort { code } => Err(CanOpenError::SdoAbort {
                code,
                reason: sdo_abort_reason(code),
            }),
            _ => Err(CanOpenError::Protocol(
                "Unexpected SDO response".to_string(),
            )),
        }
    }

    /// Receive a frame with timeout.
    async fn recv_with_timeout(
        &mut self,
    ) -> Result<crate::frame::CanOpenFrame, CanOpenError> {
        match timeout(self.timeout, self.can.recv()).await {
            Ok(result) => result,
            Err(_) => Err(CanOpenError::SdoTimeout(self.timeout)),
        }
    }

    /// SDO Block Upload — read a large value using block transfer.
    ///
    /// More efficient than segmented transfer for large data blocks.
    /// The server sends multiple segments in a block without waiting for
    /// individual acknowledgments.
    pub async fn block_upload(
        &mut self,
        node_id: u8,
        index: u16,
        subindex: u8,
    ) -> Result<OdValue, CanOpenError> {
        // Step 1: Initiate block upload request (cs=5)
        let request = SdoRequest {
            node_id,
            index,
            subindex,
            data: SdoData::BlockUploadRequest { crc_enabled: false },
        };
        self.can.send(&request.encode())?;

        // Step 2: Wait for initiate block upload response
        let response_frame = self.recv_with_timeout().await?;
        let response = SdoResponse::decode(&response_frame)
            .ok_or_else(|| CanOpenError::Protocol("Invalid SDO response".to_string()))?;

        if response.index != index || response.subindex != subindex {
            return Err(CanOpenError::Protocol(
                "SDO response index/subindex mismatch".to_string(),
            ));
        }

        let (_block_size, total_size, crc_supported) = match &response.data {
            SdoResponseData::BlockUploadInitiated {
                block_size,
                crc_supported,
                size,
            } => (*block_size, *size, *crc_supported),
            SdoResponseData::Abort { code } => {
                return Err(CanOpenError::SdoAbort {
                    code: *code,
                    reason: sdo_abort_reason(*code),
                });
            }
            _ => {
                return Err(CanOpenError::Protocol(
                    "Expected block upload initiated response".to_string(),
                ));
            }
        };

        // Step 3: Start block upload (cs=6)
        let start = SdoRequest {
            node_id,
            index,
            subindex,
            data: SdoData::BlockUploadStart,
        };
        self.can.send(&start.encode())?;

        // Step 4: Receive block segments
        let mut data = Vec::new();
        let mut last_seq = 0u8;

        loop {
            let seg_frame = self.recv_with_timeout().await?;
            let cmd = seg_frame.data[0];

            // Check if this is an end-of-block marker (cs=5 or cs=6)
            if cmd & 0xE0 == 0xA0 || cmd & 0xE0 == 0xC0 {
                // End block
                let end_resp = SdoResponse::decode(&seg_frame)
                    .ok_or_else(|| CanOpenError::Protocol("Invalid block end".to_string()))?;
                match end_resp.data {
                    SdoResponseData::BlockEnd { n, .. } => {
                        // Remove unused bytes from last segment
                        // n = number of bytes that do NOT contain data
                        if n > 0 && !data.is_empty() {
                            let trim = n as usize;
                            let new_len = data.len().saturating_sub(trim);
                            data.truncate(new_len);
                        }
                        break;
                    }
                    _ => {
                        return Err(CanOpenError::Protocol("Expected block end".to_string()));
                    }
                }
            }

            // Regular block segment (seq in cmd[6:0])
            let seq = cmd & 0x7F;
            if seq != (last_seq % 127) + 1 && seq != 1 {
                return Err(CanOpenError::Protocol(format!(
                    "Block sequence error: expected {}, got {}",
                    (last_seq % 127) + 1,
                    seq
                )));
            }
            last_seq = seq;

            data.extend_from_slice(&seg_frame.data[1..8]);
        }

        // Step 5: Confirm block transfer
        let confirm = SdoRequest {
            node_id,
            index,
            subindex,
            data: SdoData::BlockEnd {
                n: 0,
                crc: if crc_supported { Some(0) } else { None },
            },
        };
        self.can.send(&confirm.encode())?;

        // Trim to total size if known
        if let Some(size) = total_size {
            data.truncate(size as usize);
        }

        Ok(OdValue::Domain(data))
    }

    /// SDO Block Download — write a large value using block transfer.
    ///
    /// More efficient than segmented transfer for large data blocks.
    /// The client sends multiple segments in a block without waiting for
    /// individual acknowledgments.
    pub async fn block_download(
        &mut self,
        node_id: u8,
        index: u16,
        subindex: u8,
        value: &OdValue,
    ) -> Result<(), CanOpenError> {
        let bytes = value.to_bytes();
        if bytes.len() <= 4 {
            // Use expedited for small data
            return self.download(node_id, index, subindex, value).await;
        }

        // Step 1: Initiate block download request (cs=6)
        let request = SdoRequest {
            node_id,
            index,
            subindex,
            data: SdoData::BlockDownloadRequest {
                crc_enabled: false,
                size: Some(bytes.len() as u32),
            },
        };
        self.can.send(&request.encode())?;

        // Step 2: Wait for block download confirmed
        let response_frame = self.recv_with_timeout().await?;
        let response = SdoResponse::decode(&response_frame)
            .ok_or_else(|| CanOpenError::Protocol("Invalid SDO response".to_string()))?;

        if response.index != index || response.subindex != subindex {
            return Err(CanOpenError::Protocol(
                "SDO response index/subindex mismatch".to_string(),
            ));
        }

        match &response.data {
            SdoResponseData::BlockDownloadConfirmed { .. } => {}
            SdoResponseData::Abort { code } => {
                return Err(CanOpenError::SdoAbort {
                    code: *code,
                    reason: sdo_abort_reason(*code),
                });
            }
            _ => {
                return Err(CanOpenError::Protocol(
                    "Expected block download confirmed".to_string(),
                ));
            }
        }

        // Step 3: Send block segments
        let mut offset = 0;
        let mut seq = 0u8;

        while offset < bytes.len() {
            seq = (seq % 127) + 1;
            let remaining = bytes.len() - offset;
            let seg_len = remaining.min(7);

            let mut seg_data = [0u8; 7];
            seg_data[..seg_len].copy_from_slice(&bytes[offset..offset + seg_len]);

            let segment = SdoRequest {
                node_id,
                index,
                subindex,
                data: SdoData::BlockSegment {
                    seq,
                    data: seg_data,
                },
            };
            self.can.send(&segment.encode())?;

            offset += seg_len;
        }

        // Step 4: Send end of block
        let unused_bytes = (7 - (bytes.len() % 7)) % 7;
        let end = SdoRequest {
            node_id,
            index,
            subindex,
            data: SdoData::BlockEnd {
                n: unused_bytes as u8,
                crc: None,
            },
        };
        self.can.send(&end.encode())?;

        // Step 5: Wait for end-of-block confirmation
        let confirm_frame = self.recv_with_timeout().await?;
        let confirm = SdoResponse::decode(&confirm_frame)
            .ok_or_else(|| CanOpenError::Protocol("Invalid SDO response".to_string()))?;

        match &confirm.data {
            SdoResponseData::BlockEnd { .. } => Ok(()),
            SdoResponseData::Abort { code } => Err(CanOpenError::SdoAbort {
                code: *code,
                reason: sdo_abort_reason(*code),
            }),
            _ => Err(CanOpenError::Protocol(
                "Expected block end confirmation".to_string(),
            )),
        }
    }

    pub fn can(&self) -> &C {
        &self.can
    }

    pub fn can_mut(&mut self) -> &mut C {
        &mut self.can
    }
}

/// Get human-readable SDO abort reason.
///
/// Delegates to [`super::abort::abort_reason`].
pub fn sdo_abort_reason(code: u32) -> &'static str {
    super::abort::abort_reason(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockCanDriver;

    #[tokio::test]
    async fn test_sdo_expedited_upload() {
        let mut mock = MockCanDriver::new();

        // Pre-load: target node responds with expedited data (cs=3, expedited, size=4)
        // Device Type 0x1000 = 0x00020192 (CANOpen device)
        mock.enqueue(crate::frame::CanOpenFrame::new(
            0x583,
            [0x43, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00],
        ));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let result = client.upload(3, 0x1000, 0).await.unwrap();

        assert_eq!(result, OdValue::Unsigned32(0x00020192));

        // Verify request was sent
        let tx = client.can().tx_log();
        assert_eq!(tx.len(), 1);
        assert_eq!(tx[0].cob_id, 0x603);
        assert_eq!(tx[0].data[0], 0x40); // initiate upload
        assert_eq!(tx[0].data[1], 0x00); // index low
        assert_eq!(tx[0].data[2], 0x10); // index high
        assert_eq!(tx[0].data[3], 0x00); // subindex
    }

    #[tokio::test]
    async fn test_sdo_expedited_download() {
        let mut mock = MockCanDriver::new();

        // Pre-load: download confirmation
        mock.enqueue(crate::frame::CanOpenFrame::new(
            0x583,
            [0x20, 0x40, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00],
        ));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));

        // Write control word 0x0006 (Shutdown)
        client
            .download(3, 0x6040, 0, &OdValue::Unsigned16(0x0006))
            .await
            .unwrap();

        // Verify request
        let tx = client.can().tx_log();
        assert_eq!(tx.len(), 1);
        assert_eq!(tx[0].cob_id, 0x603);
        // cmd = 0x20 | 0x02 (expedited) | 0x01 (size indicated) | (4-2)<<2 = 0x2B
        assert_eq!(tx[0].data[0], 0x2B);
        assert_eq!(tx[0].data[4], 0x06);
        assert_eq!(tx[0].data[5], 0x00);
    }

    #[tokio::test]
    async fn test_sdo_abort() {
        let mut mock = MockCanDriver::new();

        // Pre-load: abort response (object does not exist)
        mock.enqueue(crate::frame::CanOpenFrame::new(
            0x583,
            [0x80, 0x00, 0x10, 0x00, 0x00, 0x00, 0x02, 0x06],
        ));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let err = client.upload(3, 0x1000, 0).await.unwrap_err();

        match err {
            CanOpenError::SdoAbort { code, reason } => {
                assert_eq!(code, 0x0602_0000);
                assert_eq!(reason, "Object does not exist");
            }
            e => panic!("Expected SdoAbort, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_sdo_segmented_upload() {
        let mut mock = MockCanDriver::new();

        // Pre-load: segmented initiated response (cs=2, not expedited, size indicated)
        // 0x41 = 0b0100_0001 → cs=2 (0b010), e=0, s=1
        let mut data = [0u8; 8];
        data[0] = 0x41;
        data[1] = 0x00;
        data[2] = 0x10; // index 0x1000
        data[3] = 0x00; // subindex
        data[4] = 20;
        data[5] = 0;
        data[6] = 0;
        data[7] = 0; // size = 20
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, data));

        // Segment 1: 7 bytes, not last (cs=0, n=0, t=0)
        let mut seg1 = [0u8; 8];
        seg1[0] = 0x00;
        seg1[1..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, seg1));

        // Segment 2: 7 bytes, not last (cs=0, n=0, t=1)
        let mut seg2 = [0u8; 8];
        seg2[0] = 0x10;
        seg2[1..8].copy_from_slice(&[8, 9, 10, 11, 12, 13, 14]);
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, seg2));

        // Segment 3: 6 bytes, last (cs=0, n=1, t=0, c=1)
        // cmd = 0b0000_0011 = 0x03
        let mut seg3 = [0u8; 8];
        seg3[0] = 0x03;
        seg3[1..7].copy_from_slice(&[15, 16, 17, 18, 19, 20]);
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, seg3));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let result = client.upload(3, 0x1000, 0).await.unwrap();

        match result {
            OdValue::Domain(data) => {
                assert_eq!(data.len(), 20);
                assert_eq!(
                    data,
                    vec![
                        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20
                    ]
                );
            }
            other => panic!("Expected Domain data, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_sdo_segmented_download() {
        let mut mock = MockCanDriver::new();

        // Pre-load: initiate segmented download confirmation (cs=1)
        mock.enqueue(crate::frame::CanOpenFrame::new(
            0x583,
            [0x20, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00],
        ));

        // Segment confirmations (cs=1) — 3 segments for 15 bytes
        for _ in 0..3 {
            mock.enqueue(crate::frame::CanOpenFrame::new(
                0x583,
                [0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ));
        }

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        client
            .download(3, 0x1000, 0, &OdValue::Domain(data))
            .await
            .unwrap();

        // Should have sent: initiate + 3 segments = 4 frames
        let tx = client.can().tx_log();
        assert_eq!(tx.len(), 4);
    }

    #[tokio::test]
    async fn test_sdo_block_upload() {
        let mut mock = MockCanDriver::new();

        // Response: initiate block upload (cs=5, block_size=3, no size)
        let mut resp1 = [0u8; 8];
        resp1[0] = 0xA0; // cs=5, no CRC, no size
        resp1[1] = 0x00; // index low
        resp1[2] = 0x20; // index high (0x2000)
        resp1[3] = 0x00; // subindex
        resp1[4] = 0x03; // block_size = 3
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, resp1));

        // Block segments (3 segments)
        let mut seg1 = [0u8; 8];
        seg1[0] = 0x01; // seq=1
        seg1[1..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, seg1));

        let mut seg2 = [0u8; 8];
        seg2[0] = 0x02; // seq=2
        seg2[1..8].copy_from_slice(&[8, 9, 10, 11, 12, 13, 14]);
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, seg2));

        let mut seg3 = [0u8; 8];
        seg3[0] = 0x03; // seq=3
        seg3[1..8].copy_from_slice(&[15, 16, 17, 18, 19, 20, 0]);
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, seg3));

        // End block (cs=5, n=1 unused byte in last segment)
        let mut end = [0u8; 8];
        end[0] = 0xA3; // cs=5, no CRC, n=1 (bits 3-1 = 001)
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, end));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let result = client.block_upload(3, 0x2000, 0).await.unwrap();

        match result {
            OdValue::Domain(data) => {
                assert_eq!(data.len(), 20);
                assert_eq!(data[0], 1);
                assert_eq!(data[19], 20);
            }
            other => panic!("Expected Domain data, got: {:?}", other),
        }

        // Verify: initiate + start + end_confirm = 3 frames sent
        let tx = client.can().tx_log();
        assert_eq!(tx.len(), 3);
        assert_eq!(tx[0].data[0], 0xA0); // initiate block upload
        assert_eq!(tx[1].data[0], 0xC0); // start block upload
        assert_eq!(tx[2].data[0], 0xC1); // end confirm (cs=1, n=0, no crc)
    }

    #[tokio::test]
    async fn test_sdo_block_download() {
        let mut mock = MockCanDriver::new();

        // Response: block download confirmed (cs=4)
        let mut resp1 = [0u8; 8];
        resp1[0] = 0x80; // cs=4, no CRC
        resp1[1] = 0x00; // index low
        resp1[2] = 0x20; // index high (0x2000)
        resp1[3] = 0x00; // subindex
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, resp1));

        // End block confirmation (cs=5, n=0)
        let mut end_confirm = [0u8; 8];
        end_confirm[0] = 0xA1; // cs=5, no CRC, n=0
        mock.enqueue(crate::frame::CanOpenFrame::new(
            0x583,
            end_confirm,
        ));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        client
            .block_download(3, 0x2000, 0, &OdValue::Domain(data))
            .await
            .unwrap();

        // Verify: initiate + 3 segments + end = 5 frames sent
        let tx = client.can().tx_log();
        assert_eq!(tx.len(), 5);
        assert_eq!(tx[0].data[0], 0xC2); // initiate block download (size indicated)
        assert_eq!(tx[1].data[0], 0x01); // segment seq=1
        assert_eq!(tx[2].data[0], 0x02); // segment seq=2
        assert_eq!(tx[3].data[0], 0x03); // segment seq=3
        assert_eq!(tx[4].data[0], 0xCD); // end block (cs=1, n=6 unused bytes)
    }

    #[tokio::test]
    async fn test_sdo_block_upload_small_data() {
        let mut mock = MockCanDriver::new();

        // Response: initiate block upload (cs=5, block_size=1)
        let mut resp1 = [0u8; 8];
        resp1[0] = 0xA0;
        resp1[1] = 0x00;
        resp1[2] = 0x20;
        resp1[3] = 0x00;
        resp1[4] = 0x01; // block_size = 1
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, resp1));

        // Single segment
        let mut seg = [0u8; 8];
        seg[0] = 0x01; // seq=1
        seg[1..4].copy_from_slice(&[0xAA, 0xBB, 0xCC]);
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, seg));

        // End block (n=4 unused bytes)
        let mut end = [0u8; 8];
        end[0] = 0xA9; // cs=5, n=4
        mock.enqueue(crate::frame::CanOpenFrame::new(0x583, end));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let result = client.block_upload(3, 0x2000, 0).await.unwrap();

        match result {
            OdValue::Domain(data) => {
                assert_eq!(data.len(), 3); // 7 - 4 unused = 3 bytes
                assert_eq!(data, vec![0xAA, 0xBB, 0xCC]);
            }
            other => panic!("Expected Domain data, got: {:?}", other),
        }
    }

    // === Error path tests ===

    #[tokio::test]
    async fn test_sdo_upload_timeout() {
        let mock = MockCanDriver::new();
        // No frames enqueued → recv will timeout
        let mut client = SdoClient::new(mock, Duration::from_millis(50));
        let result = client.upload(3, 0x1000, 0).await;
        assert!(result.is_err());
        // Timeout can be either SdoTimeout or Timeout depending on recv impl
        match result.unwrap_err() {
            CanOpenError::SdoTimeout(_) | CanOpenError::Timeout => {}
            other => panic!("Expected timeout error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_sdo_upload_abort_response() {
        let mut mock = MockCanDriver::new();
        // Abort response: object does not exist
        mock.enqueue(crate::frame::CanOpenFrame::new(
            0x583,
            [0x80, 0x00, 0x10, 0x00, 0x00, 0x00, 0x02, 0x06],
        ));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let result = client.upload(3, 0x1000, 0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CanOpenError::SdoAbort { code, .. } => {
                assert_eq!(code, 0x0602_0000); // Object does not exist
            }
            other => panic!("Expected SdoAbort, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_sdo_upload_index_mismatch() {
        let mut mock = MockCanDriver::new();
        // Response with wrong index (0x2000 instead of 0x1000)
        mock.enqueue(crate::frame::CanOpenFrame::new(
            0x583,
            [0x43, 0x00, 0x20, 0x00, 0x92, 0x01, 0x02, 0x00],
        ));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let result = client.upload(3, 0x1000, 0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CanOpenError::Protocol(msg) => {
                assert!(msg.contains("mismatch"));
            }
            other => panic!("Expected Protocol error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_sdo_download_timeout() {
        let mock = MockCanDriver::new();
        let mut client = SdoClient::new(mock, Duration::from_millis(50));
        let result = client
            .download(3, 0x6040, 0, &OdValue::Unsigned16(0x0006))
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CanOpenError::SdoTimeout(_) | CanOpenError::Timeout => {}
            other => panic!("Expected timeout error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_sdo_download_abort_response() {
        let mut mock = MockCanDriver::new();
        // Abort: read-only object
        mock.enqueue(crate::frame::CanOpenFrame::new(
            0x583,
            [0x80, 0x40, 0x60, 0x00, 0x02, 0x00, 0x04, 0x06],
        ));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let result = client
            .download(3, 0x6040, 0, &OdValue::Unsigned16(0x0006))
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CanOpenError::SdoAbort { code, .. } => {
                assert_eq!(code, 0x0604_0002); // Read-only object
            }
            other => panic!("Expected SdoAbort, got: {:?}", other),
        }
    }
}
