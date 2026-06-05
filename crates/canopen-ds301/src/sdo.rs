//! SDO (Service Data Object) client implementation.

use std::time::Duration;
use opencan_canopen_core::{CanDriver, CanOpenError};
use opencan_canopen_core::frame::{SdoRequest, SdoData, SdoResponse, SdoResponseData};
use opencan_canopen_core::od::{DataType, OdValue};

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
    pub async fn upload(&mut self, node_id: u8, index: u16, subindex: u8) -> Result<OdValue, CanOpenError> {
        self.upload_with_type(node_id, index, subindex, DataType::Unsigned32).await
    }

    /// SDO Upload with explicit data type hint.
    pub async fn upload_with_type(
        &mut self, node_id: u8, index: u16, subindex: u8, data_type: DataType,
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

        // Wait for response
        let response_frame = self.can.recv().await?;
        let response = SdoResponse::decode(&response_frame)
            .ok_or_else(|| CanOpenError::Protocol("Invalid SDO response".to_string()))?;

        if response.index != index || response.subindex != subindex {
            return Err(CanOpenError::Protocol("SDO response index/subindex mismatch".to_string()));
        }

        match response.data {
            SdoResponseData::Expedited { data, size } => {
                let bytes = if let Some(s) = size {
                    &data[..s as usize]
                } else {
                    &data
                };
                OdValue::from_bytes(data_type, bytes)
                    .ok_or_else(|| CanOpenError::Protocol("Failed to decode expedited data".to_string()))
            }
            SdoResponseData::SegmentedInitiated { size } => {
                // Segmented upload — read segments
                self.upload_segments(node_id, size as usize).await
            }
            SdoResponseData::Abort { code } => {
                Err(CanOpenError::SdoAbort {
                    code,
                    reason: sdo_abort_reason(code),
                })
            }
            _ => Err(CanOpenError::Protocol("Unexpected SDO response".to_string())),
        }
    }

    /// Read segmented upload data.
    async fn upload_segments(&mut self, _node_id: u8, total_size: usize) -> Result<OdValue, CanOpenError> {
        let mut data = Vec::with_capacity(total_size);
        let mut toggle = false;

        loop {
            // Send upload segment request
            let mut req_data = [0u8; 8];
            req_data[0] = if toggle { 0x60 } else { 0x40 }; // toggle bit
            let frame = opencan_canopen_core::frame::CanOpenFrame::new(0x600 + _node_id as u16, req_data);
            self.can.send(&frame)?;

            // Receive segment
            let response_frame = self.can.recv().await?;
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
    pub async fn download(&mut self, node_id: u8, index: u16, subindex: u8, value: &OdValue) -> Result<(), CanOpenError> {
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
            self.download_segments(node_id, index, subindex, &bytes).await
        }
    }

    /// Segmented download.
    async fn download_segments(
        &mut self, node_id: u8, index: u16, subindex: u8, data: &[u8],
    ) -> Result<(), CanOpenError> {
        // Send initiate segmented download
        let request = SdoRequest {
            node_id,
            index,
            subindex,
            data: SdoData::SegmentedInitiated { size: data.len() as u32 },
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

            // Wait for segment confirmation
            let response_frame = self.can.recv().await?;
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
                _ => return Err(CanOpenError::Protocol("Unexpected SDO response".to_string())),
            }

            offset += seg_len;
            toggle = !toggle;
        }

        Ok(())
    }

    /// Wait for download confirmation.
    async fn wait_download_confirm(&mut self, index: u16, subindex: u8) -> Result<(), CanOpenError> {
        let response_frame = self.can.recv().await?;
        let response = SdoResponse::decode(&response_frame)
            .ok_or_else(|| CanOpenError::Protocol("Invalid SDO response".to_string()))?;

        if response.index != index || response.subindex != subindex {
            return Err(CanOpenError::Protocol("SDO response index/subindex mismatch".to_string()));
        }

        match response.data {
            SdoResponseData::DownloadConfirmed => Ok(()),
            SdoResponseData::Abort { code } => Err(CanOpenError::SdoAbort {
                code,
                reason: sdo_abort_reason(code),
            }),
            _ => Err(CanOpenError::Protocol("Unexpected SDO response".to_string())),
        }
    }

    pub fn can(&self) -> &C {
        &self.can
    }
}

/// Get human-readable SDO abort reason.
pub fn sdo_abort_reason(code: u32) -> &'static str {
    match code {
        0x0503_0000 => "Toggle bit not alternated",
        0x0504_0000 => "SDO protocol timed out",
        0x0504_0001 => "Command specifier not valid or unknown",
        0x0504_0002 => "Invalid block size",
        0x0504_0003 => "Invalid sequence number",
        0x0504_0004 => "CRC error",
        0x0504_0005 => "Out of memory",
        0x0601_0000 => "Unsupported access to an object",
        0x0601_0001 => "Attempt to read a write only object",
        0x0601_0002 => "Attempt to write a read only object",
        0x0602_0000 => "Object does not exist",
        0x0604_0041 => "Object cannot be mapped to the PDO",
        0x0604_0042 => "Number and length of objects exceed PDO",
        0x0604_0043 => "General parameter incompatibility",
        0x0606_0000 => "Access failed due to hardware error",
        0x0607_0010 => "Data type does not match, length too high",
        0x0607_0012 => "Data type does not match, length too low",
        0x0609_0011 => "Sub-index does not exist",
        0x0609_0030 => "Value range of parameter exceeded",
        0x0609_0031 => "Value of parameter written too high",
        0x0609_0032 => "Value of parameter written too low",
        0x0609_0036 => "Maximum value is less than minimum value",
        0x0800_0000 => "General error",
        0x0800_0020 => "Data cannot be transferred or stored",
        0x0800_0021 => "Data cannot be transferred because of local control",
        0x0800_0022 => "Data cannot be transferred because of device state",
        _ => "Unknown abort code",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencan_canopen_core::testing::MockCanDriver;

    #[tokio::test]
    async fn test_sdo_expedited_upload() {
        let mut mock = MockCanDriver::new();

        // Pre-load: target node responds with expedited data (cs=3, expedited, size=4)
        // Device Type 0x1000 = 0x00020192 (CANOpen device)
        mock.enqueue(opencan_canopen_core::frame::CanOpenFrame::new(
            0x583,
            [0x63, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00],
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
        mock.enqueue(opencan_canopen_core::frame::CanOpenFrame::new(
            0x583,
            [0x20, 0x40, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00],
        ));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));

        // Write control word 0x0006 (Shutdown)
        client.download(3, 0x6040, 0, &OdValue::Unsigned16(0x0006)).await.unwrap();

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
        mock.enqueue(opencan_canopen_core::frame::CanOpenFrame::new(
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

        // Pre-load: segmented initiated response (cs=3, not expedited, size indicated)
        // 0x61 = 0b0110_0001 → cs=3 (0b011), e=0, s=1
        let mut data = [0u8; 8];
        data[0] = 0x61;
        data[1] = 0x00; data[2] = 0x10; // index 0x1000
        data[3] = 0x00; // subindex
        data[4] = 20; data[5] = 0; data[6] = 0; data[7] = 0; // size = 20
        mock.enqueue(opencan_canopen_core::frame::CanOpenFrame::new(0x583, data));

        // Segment 1: 7 bytes, not last (cs=0, n=0, t=0)
        let mut seg1 = [0u8; 8];
        seg1[0] = 0x00;
        seg1[1..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        mock.enqueue(opencan_canopen_core::frame::CanOpenFrame::new(0x583, seg1));

        // Segment 2: 7 bytes, not last (cs=0, n=0, t=1)
        let mut seg2 = [0u8; 8];
        seg2[0] = 0x10;
        seg2[1..8].copy_from_slice(&[8, 9, 10, 11, 12, 13, 14]);
        mock.enqueue(opencan_canopen_core::frame::CanOpenFrame::new(0x583, seg2));

        // Segment 3: 6 bytes, last (cs=0, n=1, t=0, c=1)
        // cmd = 0b0000_0011 = 0x03
        let mut seg3 = [0u8; 8];
        seg3[0] = 0x03;
        seg3[1..7].copy_from_slice(&[15, 16, 17, 18, 19, 20]);
        mock.enqueue(opencan_canopen_core::frame::CanOpenFrame::new(0x583, seg3));

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let result = client.upload(3, 0x1000, 0).await.unwrap();

        match result {
            OdValue::Domain(data) => {
                assert_eq!(data.len(), 20);
                assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]);
            }
            other => panic!("Expected Domain data, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_sdo_segmented_download() {
        let mut mock = MockCanDriver::new();

        // Pre-load: initiate segmented download confirmation (cs=1)
        mock.enqueue(opencan_canopen_core::frame::CanOpenFrame::new(
            0x583,
            [0x20, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00],
        ));

        // Segment confirmations (cs=1) — 3 segments for 15 bytes
        for _ in 0..3 {
            mock.enqueue(opencan_canopen_core::frame::CanOpenFrame::new(
                0x583,
                [0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ));
        }

        let mut client = SdoClient::new(mock, Duration::from_secs(1));
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        client.download(3, 0x1000, 0, &OdValue::Domain(data)).await.unwrap();

        // Should have sent: initiate + 3 segments = 4 frames
        let tx = client.can().tx_log();
        assert_eq!(tx.len(), 4);
    }
}
