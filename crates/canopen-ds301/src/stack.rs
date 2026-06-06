//! CANOpen stack — main loop, event processing, and protocol operations.
//!
//! The stack owns the CAN driver and provides:
//! - Frame classification and event emission
//! - SDO read/write operations (expedited + segmented)
//! - NMT master commands
//! - Node scanning
//! - Heartbeat monitoring + production

use std::time::Duration;
use opencan_canopen_core::{CanDriver, CanOpenError};
use opencan_canopen_core::frame::{
    CanOpenFrame, FrameClass, classify_frame, PdoFrame,
    NmtCommand, NmtCommandSpecifier, SdoRequest, SdoData, SdoResponse, SdoResponseData,
};
use opencan_canopen_core::od::{DataType, OdValue};

use crate::heartbeat::{HeartbeatConsumer, HeartbeatProducer, SyncProducer};
use crate::emcy::EmergencyHandler;
use crate::sdo::sdo_abort_reason;

/// CANOpen protocol events emitted by the stack.
#[derive(Debug, Clone)]
pub enum CanEvent {
    /// A node's heartbeat state changed.
    HeartbeatChanged { node_id: u8, alive: bool },
    /// A node's heartbeat timed out.
    HeartbeatTimeout { node_id: u8 },
    /// Emergency event from a node.
    Emergency { node_id: u8, error_code: u16 },
    /// PDO frame received.
    PdoReceived { pdo: PdoFrame },
    /// SDO operation completed.
    SdoComplete { node_id: u8, result: Result<Vec<u8>, String> },
}

/// Main CANOpen protocol stack.
///
/// Owns the CAN driver and provides high-level protocol operations.
pub struct CanopenStack<C: CanDriver> {
    can: C,
    node_id: u8,
    heartbeat: HeartbeatConsumer,
    emergency: EmergencyHandler,
    sdo_timeout: Duration,
    sync_producer: Option<SyncProducer>,
    heartbeat_producer: Option<HeartbeatProducer>,
}

impl<C: CanDriver> CanopenStack<C> {
    /// Create a new stack with the given CAN driver and local node ID.
    pub fn new(can: C, node_id: u8) -> Self {
        Self {
            can,
            node_id,
            heartbeat: HeartbeatConsumer::new(Duration::from_secs(1)),
            emergency: EmergencyHandler::new(1000),
            sdo_timeout: Duration::from_secs(5),
            sync_producer: None,
            heartbeat_producer: None,
        }
    }

    /// Set the SDO timeout duration.
    pub fn set_sdo_timeout(&mut self, timeout: Duration) {
        self.sdo_timeout = timeout;
    }

    /// Set the default heartbeat timeout.
    pub fn set_heartbeat_timeout(&mut self, timeout: Duration) {
        self.heartbeat = HeartbeatConsumer::new(timeout);
    }

    /// Set expected heartbeat period for a specific node.
    pub fn set_heartbeat_period(&mut self, node_id: u8, period: Duration) {
        self.heartbeat.set_period(node_id, period);
    }

    // === Frame Processing ===

    /// Process one CAN frame — call this in a loop for incoming frames.
    pub fn process(&mut self, frame: &CanOpenFrame) -> Vec<CanEvent> {
        let mut events = Vec::new();

        match classify_frame(frame) {
            FrameClass::Heartbeat(hb) => {
                let changed = self.heartbeat.update(&hb);
                if changed {
                    events.push(CanEvent::HeartbeatChanged {
                        node_id: hb.node_id,
                        alive: self.heartbeat.is_alive(hb.node_id),
                    });
                }
            }
            FrameClass::Emergency(emcy) => {
                self.emergency.record(&emcy);
                events.push(CanEvent::Emergency {
                    node_id: emcy.node_id,
                    error_code: emcy.error_code,
                });
            }
            FrameClass::Pdo(pdo) => {
                events.push(CanEvent::PdoReceived { pdo });
            }
            FrameClass::Nmt(_) => {
                // NMT commands from other masters — ignore for now
            }
            FrameClass::SdoResponse(_) => {
                // SDO responses are handled by SDO operations directly
            }
            FrameClass::Sync => {
                // TODO: Handle SYNC
            }
            FrameClass::Timestamp => {
                // TODO: Handle TIME
            }
            FrameClass::Unknown => {
                // Unknown frame — log if needed
            }
        }

        // Check for heartbeat timeouts
        for (node_id, _elapsed) in self.heartbeat.check_timeouts() {
            events.push(CanEvent::HeartbeatTimeout { node_id });
        }

        events
    }

    // === NMT Master ===

    /// Send NMT command to start a remote node.
    pub fn nmt_start(&mut self, node_id: u8) -> Result<(), CanOpenError> {
        let cmd = NmtCommand {
            command: NmtCommandSpecifier::EnterOperational,
            node_id,
        };
        self.can.send(&cmd.encode())
    }

    /// Send NMT command to stop a remote node.
    pub fn nmt_stop(&mut self, node_id: u8) -> Result<(), CanOpenError> {
        let cmd = NmtCommand {
            command: NmtCommandSpecifier::EnterStopped,
            node_id,
        };
        self.can.send(&cmd.encode())
    }

    /// Send NMT command to reset a remote node.
    pub fn nmt_reset(&mut self, node_id: u8) -> Result<(), CanOpenError> {
        let cmd = NmtCommand {
            command: NmtCommandSpecifier::ResetNode,
            node_id,
        };
        self.can.send(&cmd.encode())
    }

    /// Send NMT command to reset communication on a remote node.
    pub fn nmt_reset_communication(&mut self, node_id: u8) -> Result<(), CanOpenError> {
        let cmd = NmtCommand {
            command: NmtCommandSpecifier::ResetCommunication,
            node_id,
        };
        self.can.send(&cmd.encode())
    }

    /// Broadcast NMT command to all nodes (node_id = 0).
    pub fn nmt_broadcast(&mut self, command: NmtCommandSpecifier) -> Result<(), CanOpenError> {
        let cmd = NmtCommand {
            command,
            node_id: 0,
        };
        self.can.send(&cmd.encode())
    }

    // === SDO Operations ===

    /// SDO Upload — read a value from a remote node's object dictionary.
    ///
    /// This is the high-level API that handles expedited and segmented transfers.
    pub async fn sdo_upload(
        &mut self, node_id: u8, index: u16, subindex: u8, data_type: DataType,
    ) -> Result<OdValue, CanOpenError> {
        // Send initiate upload request
        let request = SdoRequest {
            node_id,
            index,
            subindex,
            data: SdoData::UploadRequest,
        };
        self.can.send(&request.encode())?;

        // Wait for response with timeout
        let response_frame = self.recv_with_timeout().await?;
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
                self.sdo_upload_segments(node_id, size as usize).await
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

    /// SDO Upload with default Unsigned32 type.
    pub async fn sdo_upload_u32(
        &mut self, node_id: u8, index: u16, subindex: u8,
    ) -> Result<OdValue, CanOpenError> {
        self.sdo_upload(node_id, index, subindex, DataType::Unsigned32).await
    }

    /// Read segmented upload data.
    async fn sdo_upload_segments(
        &mut self, node_id: u8, total_size: usize,
    ) -> Result<OdValue, CanOpenError> {
        let mut data = Vec::with_capacity(total_size);
        let mut toggle = false;

        loop {
            // Send upload segment request
            let mut req_data = [0u8; 8];
            req_data[0] = if toggle { 0x60 } else { 0x40 };
            let frame = CanOpenFrame::new(0x600 + node_id as u16, req_data);
            self.can.send(&frame)?;

            // Receive segment with timeout
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

    /// SDO Download — write a value to a remote node's object dictionary.
    pub async fn sdo_download(
        &mut self, node_id: u8, index: u16, subindex: u8, value: &OdValue,
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
            self.sdo_wait_download_confirm(index, subindex).await
        } else {
            // Segmented download
            self.sdo_download_segments(node_id, index, subindex, &bytes).await
        }
    }

    /// Segmented download.
    async fn sdo_download_segments(
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
        self.sdo_wait_download_confirm(index, subindex).await?;

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

            // Wait for segment confirmation with timeout
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
                _ => return Err(CanOpenError::Protocol("Unexpected SDO response".to_string())),
            }

            offset += seg_len;
            toggle = !toggle;
        }

        Ok(())
    }

    /// Wait for SDO download confirmation.
    async fn sdo_wait_download_confirm(
        &mut self, index: u16, subindex: u8,
    ) -> Result<(), CanOpenError> {
        let response_frame = self.recv_with_timeout().await?;
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

    // === Node Scanning ===

    /// Scan for active nodes on the CANOpen network.
    ///
    /// Sends an SDO read of the Device Type (0x1000) to each possible node ID (1..127).
    /// Returns a list of node IDs that responded.
    pub async fn scan_nodes(&mut self) -> Result<Vec<u8>, CanOpenError> {
        let mut found = Vec::new();

        for node_id in 1..=127 {
            // Try to read Device Type (0x1000:00)
            let request = SdoRequest {
                node_id,
                index: 0x1000,
                subindex: 0,
                data: SdoData::UploadRequest,
            };
            self.can.send(&request.encode())?;

            // Short timeout for scanning
            let result = tokio::time::timeout(Duration::from_millis(50), self.can.recv()).await;

            match result {
                Ok(Ok(frame)) => {
                    if let Some(resp) = SdoResponse::decode(&frame)
                        && resp.index == 0x1000 && resp.subindex == 0
                    {
                        match resp.data {
                            SdoResponseData::Expedited { .. }
                            | SdoResponseData::SegmentedInitiated { .. } => {
                                found.push(node_id);
                            }
                            SdoResponseData::Abort { .. } => {
                                // Node responded (even with abort) — it exists
                                found.push(node_id);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    // Timeout or error — node not responding
                }
            }
        }

        Ok(found)
    }

    // === Heartbeat Production ===

    /// Send a heartbeat frame for the local node.
    pub fn send_heartbeat(&mut self, state: opencan_canopen_core::frame::NmtState) -> Result<(), CanOpenError> {
        let hb = opencan_canopen_core::frame::HeartbeatFrame {
            node_id: self.node_id,
            state,
        };
        self.can.send(&hb.encode())
    }

    /// Enable periodic heartbeat production.
    pub fn enable_heartbeat_production(&mut self, period: Duration) {
        self.heartbeat_producer = Some(HeartbeatProducer::new(period));
    }

    /// Get the heartbeat producer, if enabled.
    pub fn heartbeat_producer(&self) -> Option<&HeartbeatProducer> {
        self.heartbeat_producer.as_ref()
    }

    /// Get the heartbeat producer mutably, if enabled.
    pub fn heartbeat_producer_mut(&mut self) -> Option<&mut HeartbeatProducer> {
        self.heartbeat_producer.as_mut()
    }

    // === SYNC Production ===

    /// Enable periodic SYNC frame production.
    pub fn enable_sync_production(&mut self, period: Duration) {
        self.sync_producer = Some(SyncProducer::new(period));
    }

    /// Disable SYNC production.
    pub fn disable_sync_production(&mut self) {
        self.sync_producer = None;
    }

    /// Get the SYNC producer, if enabled.
    pub fn sync_producer(&self) -> Option<&SyncProducer> {
        self.sync_producer.as_ref()
    }

    /// Get the SYNC producer mutably, if enabled.
    pub fn sync_producer_mut(&mut self) -> Option<&mut SyncProducer> {
        self.sync_producer.as_mut()
    }

    /// Check if it's time to send a SYNC, and if so, send it.
    /// Returns true if a SYNC was sent.
    pub fn poll_sync(&mut self) -> Result<bool, CanOpenError> {
        if let Some(ref mut producer) = self.sync_producer
            && producer.should_send() {
                let frame = producer.build_frame();
                self.can.send(&frame)?;
                return Ok(true);
            }
        Ok(false)
    }

    /// Check if it's time to send a heartbeat, and if so, send it.
    /// Returns true if a heartbeat was sent.
    pub fn poll_heartbeat(&mut self, state: opencan_canopen_core::frame::NmtState) -> Result<bool, CanOpenError> {
        let should_send = self.heartbeat_producer
            .as_ref()
            .is_some_and(|p| p.should_send());

        if should_send {
            self.send_heartbeat(state)?;
            if let Some(ref mut producer) = self.heartbeat_producer {
                producer.mark_sent();
            }
            return Ok(true);
        }
        Ok(false)
    }

    // === Accessors ===

    /// Get the node ID of this stack.
    pub fn node_id(&self) -> u8 {
        self.node_id
    }

    /// Get a reference to the heartbeat consumer.
    pub fn heartbeat(&self) -> &HeartbeatConsumer {
        &self.heartbeat
    }

    /// Get a reference to the emergency handler.
    pub fn emergency(&self) -> &EmergencyHandler {
        &self.emergency
    }

    /// Get a reference to the CAN driver.
    pub fn can(&self) -> &C {
        &self.can
    }

    /// Get a mutable reference to the CAN driver.
    pub fn can_mut(&mut self) -> &mut C {
        &mut self.can
    }

    /// Receive a frame with timeout.
    async fn recv_with_timeout(&mut self) -> Result<CanOpenFrame, CanOpenError> {
        match tokio::time::timeout(self.sdo_timeout, self.can.recv()).await {
            Ok(result) => result,
            Err(_) => Err(CanOpenError::SdoTimeout(self.sdo_timeout)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencan_canopen_core::testing::MockCanDriver;
    use opencan_canopen_core::frame::{HeartbeatFrame, NmtState, EmergencyFrame};
    use opencan_canopen_core::od::OdValue;

    /// Helper: create an expedited SDO upload response.
    fn sdo_upload_response(node_id: u8, index: u16, subindex: u8, data: [u8; 4]) -> CanOpenFrame {
        let mut d = [0u8; 8];
        d[0] = 0x43; // cs=2, expedited, size indicated, 4 bytes
        d[1..3].copy_from_slice(&index.to_le_bytes());
        d[3] = subindex;
        d[4..8].copy_from_slice(&data);
        CanOpenFrame::new(0x580 + node_id as u16, d)
    }

    /// Helper: create an SDO download confirmation.
    fn sdo_download_confirm(node_id: u8, index: u16, subindex: u8) -> CanOpenFrame {
        let mut d = [0u8; 8];
        d[0] = 0x20; // cs=1 (download confirmed)
        d[1..3].copy_from_slice(&index.to_le_bytes());
        d[3] = subindex;
        CanOpenFrame::new(0x580 + node_id as u16, d)
    }

    #[tokio::test]
    async fn test_stack_sdo_upload() {
        let mut mock = MockCanDriver::new();
        mock.enqueue(sdo_upload_response(3, 0x1000, 0, 0x00020192u32.to_le_bytes()));

        let mut stack = CanopenStack::new(mock, 0);
        let result = stack.sdo_upload(3, 0x1000, 0, DataType::Unsigned32).await.unwrap();
        assert_eq!(result, OdValue::Unsigned32(0x00020192));

        let tx = stack.can().tx_log();
        assert_eq!(tx.len(), 1);
        assert_eq!(tx[0].cob_id, 0x603);
        assert_eq!(tx[0].data[0], 0x40); // initiate upload
    }

    #[tokio::test]
    async fn test_stack_sdo_download() {
        let mut mock = MockCanDriver::new();
        mock.enqueue(sdo_download_confirm(3, 0x6040, 0));

        let mut stack = CanopenStack::new(mock, 0);
        stack.sdo_download(3, 0x6040, 0, &OdValue::Unsigned16(0x000F)).await.unwrap();

        let tx = stack.can().tx_log();
        assert_eq!(tx.len(), 1);
        assert_eq!(tx[0].cob_id, 0x603);
        assert_eq!(tx[0].data[0], 0x2B); // expedited, 2 bytes
        assert_eq!(tx[0].data[4], 0x0F);
    }

    #[tokio::test]
    async fn test_stack_nmt_start() {
        let mock = MockCanDriver::new();
        let mut stack = CanopenStack::new(mock, 0);

        stack.nmt_start(5).unwrap();

        let tx = stack.can().tx_log();
        assert_eq!(tx.len(), 1);
        assert_eq!(tx[0].cob_id, 0x000); // NMT
        assert_eq!(tx[0].data[0], 0x01); // EnterOperational
        assert_eq!(tx[0].data[1], 5);   // node_id
    }

    #[tokio::test]
    async fn test_stack_nmt_broadcast() {
        let mock = MockCanDriver::new();
        let mut stack = CanopenStack::new(mock, 0);

        stack.nmt_broadcast(NmtCommandSpecifier::ResetCommunication).unwrap();

        let tx = stack.can().tx_log();
        assert_eq!(tx.len(), 1);
        assert_eq!(tx[0].cob_id, 0x000);
        assert_eq!(tx[0].data[0], 0x82); // ResetCommunication
        assert_eq!(tx[0].data[1], 0);    // broadcast
    }

    #[tokio::test]
    async fn test_stack_process_heartbeat() {
        let mock = MockCanDriver::new();
        let mut stack = CanopenStack::new(mock, 0);

        let hb = HeartbeatFrame { node_id: 5, state: NmtState::Operational };
        let events = stack.process(&hb.encode());
        assert_eq!(events.len(), 1);
        match &events[0] {
            CanEvent::HeartbeatChanged { node_id, alive } => {
                assert_eq!(*node_id, 5);
                assert!(*alive);
            }
            _ => panic!("Expected HeartbeatChanged"),
        }
    }

    #[tokio::test]
    async fn test_stack_process_emergency() {
        let mock = MockCanDriver::new();
        let mut stack = CanopenStack::new(mock, 0);

        let emcy = EmergencyFrame {
            node_id: 3,
            error_code: 0x1000,
            error_register: 0x01,
            data: [0, 0, 0, 0, 0],
        };
        let events = stack.process(&emcy.encode());
        assert_eq!(events.len(), 1);
        match &events[0] {
            CanEvent::Emergency { node_id, error_code } => {
                assert_eq!(*node_id, 3);
                assert_eq!(*error_code, 0x1000);
            }
            _ => panic!("Expected Emergency"),
        }
    }

    #[tokio::test]
    async fn test_stack_sdo_abort() {
        let mut mock = MockCanDriver::new();
        // Abort: object does not exist
        let mut d = [0u8; 8];
        d[0] = 0x80;
        d[1..3].copy_from_slice(&0x1000u16.to_le_bytes());
        d[3] = 0;
        d[4..8].copy_from_slice(&0x06020000u32.to_le_bytes());
        mock.enqueue(CanOpenFrame::new(0x583, d));

        let mut stack = CanopenStack::new(mock, 0);
        let err = stack.sdo_upload(3, 0x1000, 0, DataType::Unsigned32).await.unwrap_err();

        match err {
            CanOpenError::SdoAbort { code, reason } => {
                assert_eq!(code, 0x0602_0000);
                assert_eq!(reason, "Object does not exist");
            }
            e => panic!("Expected SdoAbort, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_stack_send_heartbeat() {
        let mock = MockCanDriver::new();
        let mut stack = CanopenStack::new(mock, 5);

        stack.send_heartbeat(NmtState::Operational).unwrap();

        let tx = stack.can().tx_log();
        assert_eq!(tx.len(), 1);
        assert_eq!(tx[0].cob_id, 0x705); // heartbeat node 5
        assert_eq!(tx[0].data[0], 0x05); // Operational
    }
}
