//! Integration tests for the full CANOpen stack.
//!
//! Tests the complete flow through CanDriverAdapter + SdoClient + Ds402Device.

use std::time::Duration;
use opencan_canopen_core::testing::MockCanDriver;
use opencan_canopen_core::frame::CanOpenFrame;
use opencan_canopen_core::od::OdValue;
use opencan_canopen_core::CanOpenError;
use opencan_canopen_ds301::SdoClient;
use opencan_canopen_ds402::state_machine::{Ds402State, OperationMode};
use opencan_canopen_ds402::control::Ds402Device;

/// Helper: create an expedited SDO upload response for u16 value (cs=2, expedited, 4 bytes).
fn sdo_upload_response_u16(node_id: u8, index: u16, subindex: u8, value: u16) -> CanOpenFrame {
    let mut d = [0u8; 8];
    d[0] = 0x43; // cs=2, expedited, size indicated, 4 bytes
    d[1..3].copy_from_slice(&index.to_le_bytes());
    d[3] = subindex;
    d[4..6].copy_from_slice(&value.to_le_bytes());
    CanOpenFrame::new(0x580 + node_id as u16, d)
}

/// Helper: create an expedited SDO upload response (cs=2, expedited, 4 bytes).
fn sdo_upload_response(node_id: u8, index: u16, subindex: u8, data: [u8; 4]) -> CanOpenFrame {
    let mut d = [0u8; 8];
    d[0] = 0x43; // cs=2 (initiate upload response), expedited, size indicated, 4 bytes
    d[1..3].copy_from_slice(&index.to_le_bytes());
    d[3] = subindex;
    d[4..8].copy_from_slice(&data);
    CanOpenFrame::new(0x580 + node_id as u16, d)
}

/// Helper: create an SDO download confirmation (cs=1).
fn sdo_download_confirm(node_id: u8, index: u16, subindex: u8) -> CanOpenFrame {
    let mut d = [0u8; 8];
    d[0] = 0x20; // cs=1 (download confirmed)
    d[1..3].copy_from_slice(&index.to_le_bytes());
    d[3] = subindex;
    CanOpenFrame::new(0x580 + node_id as u16, d)
}

/// Helper: create an SDO abort response.
fn sdo_abort(node_id: u8, index: u16, subindex: u8, code: u32) -> CanOpenFrame {
    let mut d = [0u8; 8];
    d[0] = 0x80; // abort
    d[1..3].copy_from_slice(&index.to_le_bytes());
    d[3] = subindex;
    d[4..8].copy_from_slice(&code.to_le_bytes());
    CanOpenFrame::new(0x580 + node_id as u16, d)
}

#[tokio::test]
async fn test_full_sdo_read_device_type() {
    let mut mock = MockCanDriver::new();

    // Node 3 responds with Device Type = 0x00020192
    mock.enqueue(sdo_upload_response(3, 0x1000, 0, 0x00020192u32.to_le_bytes()));

    let mut client = SdoClient::new(mock, Duration::from_secs(1));
    let result = client.upload(3, 0x1000, 0).await.unwrap();

    assert_eq!(result, OdValue::Unsigned32(0x00020192));

    // Verify SDO request was sent correctly
    let tx = client.can().tx_log();
    assert_eq!(tx.len(), 1);
    assert_eq!(tx[0].cob_id, 0x603); // SDO client, node 3
    assert_eq!(tx[0].data[0], 0x40); // initiate upload
    assert_eq!(tx[0].data[1], 0x00); // index low
    assert_eq!(tx[0].data[2], 0x10); // index high
    assert_eq!(tx[0].data[3], 0x00); // subindex
}

#[tokio::test]
async fn test_full_sdo_write_control_word() {
    let mut mock = MockCanDriver::new();

    // Node 5 confirms download
    mock.enqueue(sdo_download_confirm(5, 0x6040, 0));

    let mut client = SdoClient::new(mock, Duration::from_secs(1));

    // Write control word 0x000F (Enable Operation)
    client.download(5, 0x6040, 0, &OdValue::Unsigned16(0x000F)).await.unwrap();

    // Verify request
    let tx = client.can().tx_log();
    assert_eq!(tx.len(), 1);
    assert_eq!(tx[0].cob_id, 0x605); // SDO client, node 5
    // cmd = 0x20 | 0x02 (expedited) | 0x01 (size indicated) | (4-2)<<2 = 0x2B
    assert_eq!(tx[0].data[0], 0x2B);
    assert_eq!(tx[0].data[1], 0x40); // index low
    assert_eq!(tx[0].data[2], 0x60); // index high
    assert_eq!(tx[0].data[3], 0x00); // subindex
    assert_eq!(tx[0].data[4], 0x0F); // value low
    assert_eq!(tx[0].data[5], 0x00); // value high
}

#[tokio::test]
async fn test_sdo_abort_object_not_found() {
    let mut mock = MockCanDriver::new();

    mock.enqueue(sdo_abort(3, 0x1000, 0, 0x06020000));

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
async fn test_sdo_abort_read_only() {
    let mut mock = MockCanDriver::new();

    mock.enqueue(sdo_abort(3, 0x1000, 0, 0x06010002));

    let mut client = SdoClient::new(mock, Duration::from_secs(1));
    let err = client.upload(3, 0x1000, 0).await.unwrap_err();

    match err {
        CanOpenError::SdoAbort { code, reason } => {
            assert_eq!(code, 0x0601_0002);
            assert_eq!(reason, "Attempt to write a read only object");
        }
        e => panic!("Expected SdoAbort, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_ds402_enable_sequence() {
    let mut mock = MockCanDriver::new();

    // Read status word → OperationEnabled (0x0027)
    mock.enqueue(sdo_upload_response_u16(3, 0x6041, 0, 0x0027));

    // Enable sequence: Shutdown → SwitchOn → EnableOperation
    mock.enqueue(sdo_download_confirm(3, 0x6040, 0)); // Shutdown
    mock.enqueue(sdo_download_confirm(3, 0x6040, 0)); // Switch On
    mock.enqueue(sdo_download_confirm(3, 0x6040, 0)); // Enable Operation

    let client = SdoClient::new(mock, Duration::from_secs(1));
    let mut device = Ds402Device::new(client, 3);

    // Read current state
    let state = device.state().await.unwrap();
    assert_eq!(state, Ds402State::OperationEnabled);

    // Execute enable sequence
    device.enable().await.unwrap();

    // Verify control words were sent
    let tx = device.sdo().can().tx_log();
    assert_eq!(tx.len(), 4); // read + 3 writes

    // Check the control words
    // First write: Shutdown (0x0006)
    assert_eq!(tx[1].data[4], 0x06);
    assert_eq!(tx[1].data[5], 0x00);

    // Second write: Switch On (0x0007)
    assert_eq!(tx[2].data[4], 0x07);
    assert_eq!(tx[2].data[5], 0x00);

    // Third write: Enable Operation (0x000F)
    assert_eq!(tx[3].data[4], 0x0F);
    assert_eq!(tx[3].data[5], 0x00);
}

#[tokio::test]
async fn test_ds402_set_mode_and_position() {
    let mut mock = MockCanDriver::new();

    // Set mode to CSP (0x08)
    mock.enqueue(sdo_download_confirm(3, 0x6060, 0));

    // Set target position
    mock.enqueue(sdo_download_confirm(3, 0x607A, 0));

    // Read actual position (i32)
    mock.enqueue(sdo_upload_response(3, 0x6064, 0, 12345i32.to_le_bytes()));

    let client = SdoClient::new(mock, Duration::from_secs(1));
    let mut device = Ds402Device::new(client, 3);

    // Set CSP mode
    device.set_mode(OperationMode::CyclicSyncPosition).await.unwrap();

    // Set target position
    device.set_target_position(10000).await.unwrap();

    // Read actual position
    let pos = device.actual_position().await.unwrap();
    assert_eq!(pos, 12345);

    // Verify all requests
    let tx = device.sdo().can().tx_log();
    assert_eq!(tx.len(), 3);
}

#[tokio::test]
async fn test_ds402_state_from_status_word() {
    // Test all state transitions from status word
    assert_eq!(Ds402State::from_status_word(0x0000), Ds402State::NotReadyToSwitchOn);
    assert_eq!(Ds402State::from_status_word(0x0040), Ds402State::SwitchOnDisabled);
    assert_eq!(Ds402State::from_status_word(0x0021), Ds402State::ReadyToSwitchOn);
    assert_eq!(Ds402State::from_status_word(0x0023), Ds402State::SwitchedOn);
    assert_eq!(Ds402State::from_status_word(0x0027), Ds402State::OperationEnabled);
    assert_eq!(Ds402State::from_status_word(0x0007), Ds402State::QuickStopActive);
    assert_eq!(Ds402State::from_status_word(0x000F), Ds402State::FaultReactionActive);
    assert_eq!(Ds402State::from_status_word(0x0008), Ds402State::Fault);
}

#[tokio::test]
async fn test_multiple_sdos_sequential() {
    let mut mock = MockCanDriver::new();

    // Multiple SDO reads
    mock.enqueue(sdo_upload_response(1, 0x1000, 0, 0x00020192u32.to_le_bytes()));
    mock.enqueue(sdo_upload_response(1, 0x1001, 0, [0x00, 0, 0, 0])); // Error register
    mock.enqueue(sdo_upload_response(1, 0x1018, 1, 0x12345678u32.to_le_bytes())); // Vendor ID

    let mut client = SdoClient::new(mock, Duration::from_secs(1));

    let dt = client.upload(1, 0x1000, 0).await.unwrap();
    assert_eq!(dt, OdValue::Unsigned32(0x00020192));

    let err = client.upload(1, 0x1001, 0).await.unwrap();
    // Error register is 1 byte, but upload defaults to Unsigned32
    // The raw bytes [0x00, 0, 0, 0] = Unsigned32(0)
    assert_eq!(err, OdValue::Unsigned32(0));

    let vid = client.upload(1, 0x1018, 1).await.unwrap();
    assert_eq!(vid, OdValue::Unsigned32(0x12345678));

    assert_eq!(client.can().tx_log().len(), 3);
}
