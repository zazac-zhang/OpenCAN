//! SDO (Service Data Object) client implementation.

use std::time::Duration;
use opencan_canopen_core::{CanDriver, CanOpenError};
use opencan_canopen_core::frame::{SdoRequest, SdoData, SdoResponse, SdoResponseData};
use opencan_canopen_core::od::OdValue;

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
        let response_frame = self.can.recv_async().await?;

        let response = SdoResponse::decode(&response_frame)
            .ok_or_else(|| CanOpenError::Protocol("Invalid SDO response".to_string()))?;

        match response.data {
            SdoResponseData::Expedited { data, size } => {
                let bytes = if let Some(s) = size {
                    &data[..s as usize]
                } else {
                    &data
                };
                // Default to reading raw bytes as Unsigned32 for now
                // Real implementation would use OD entry info for type
                Ok(OdValue::from_bytes(
                    opencan_canopen_core::od::DataType::Unsigned32,
                    bytes,
                ).unwrap_or(OdValue::Domain(bytes.to_vec())))
            }
            SdoResponseData::SegmentedInitiated { .. } => {
                // TODO: Implement segmented transfer
                Err(CanOpenError::Protocol("Segmented transfer not yet implemented".to_string()))
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
            let frame = request.encode();
            self.can.send(&frame)?;

            // Wait for confirmation
            let response_frame = self.can.recv_async().await?;
            let response = SdoResponse::decode(&response_frame)
                .ok_or_else(|| CanOpenError::Protocol("Invalid SDO response".to_string()))?;

            match response.data {
                SdoResponseData::DownloadConfirmed => Ok(()),
                SdoResponseData::Abort { code } => Err(CanOpenError::SdoAbort {
                    code,
                    reason: sdo_abort_reason(code),
                }),
                _ => Err(CanOpenError::Protocol("Unexpected SDO response".to_string())),
            }
        } else {
            // TODO: Implement segmented download
            Err(CanOpenError::Protocol("Segmented download not yet implemented".to_string()))
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
