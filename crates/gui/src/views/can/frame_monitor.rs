//! Frame monitor view (three-pane layout).

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Element, Length};
use crate::state::{App, Message};

/// Frame monitor view (three-pane layout).
pub fn frame_monitor(app: &App) -> Element<'_, Message> {
    let filtered: Vec<_> = app.can_log.iter()
        .filter(|e| app.log_filter.matches(e))
        .collect();

    // Filter bar
    let filter_bar = row![
        text("Filter:").size(11),
        text_input("Search COB-ID, data, or description...", &app.log_filter.text)
            .on_input(Message::LogFilterChanged)
            .width(Length::Fill),
        button(text("NMT").size(9)).on_press(Message::LogFilterToggleNmt),
        button(text("SDO").size(9)).on_press(Message::LogFilterToggleSdo),
        button(text("PDO").size(9)).on_press(Message::LogFilterTogglePdo),
        button(text("EMCY").size(9)).on_press(Message::LogFilterToggleEmcy),
        button(text("HB").size(9)).on_press(Message::LogFilterToggleHeartbeat),
    ].spacing(4);

    // Frame list (top pane)
    let mut frame_list = column![].spacing(1);

    // Table header
    frame_list = frame_list.push(
        row![
            text("Time").size(9).width(70),
            text("Direction").size(9).width(30),
            text("CAN ID").size(9).width(50),
            text("DLC").size(9).width(25),
            text("Data").size(9).width(150),
            text("Description").size(9),
        ].spacing(2)
    );
    frame_list = frame_list.push(horizontal_rule(1));

    // Show latest frames (newest first)
    for entry in filtered.iter().rev().take(200) {
        let _is_selected = app.selected_frame.as_ref()
            .map(|f| f.cob_id == entry.cob_id && f.timestamp_ms == entry.timestamp_ms)
            .unwrap_or(false);

        let dir_indicator = match entry.direction {
            crate::state::Direction::Rx => "Rx",
            crate::state::Direction::Tx => "Tx",
        };

        let desc = if entry.description.is_empty() {
            classify_cob_id(entry.cob_id)
        } else {
            entry.description.clone()
        };

        let row_content = row![
            text(entry.timestamp_str()).size(9).width(70),
            text(dir_indicator).size(9).width(30),
            text(format!("{:03X}", entry.cob_id)).size(9).width(50),
            text(format!("{}", entry.dlc)).size(9).width(25),
            text(entry.hex_data()).size(9).width(150),
            text(desc).size(9),
        ].spacing(2);

        let entry_clone = (*entry).clone();
        let btn = button(row_content)
            .on_press(Message::FrameSelected(entry_clone.clone()))
            .width(Length::Fill);

        frame_list = frame_list.push(btn);
    }

    // Frame detail (middle pane)
    let frame_detail: Element<'_, Message> = if let Some(ref frame) = app.selected_frame {
        frame_detail_view(frame)
    } else {
        container(
            text("Select a frame to view details").size(11)
        ).into()
    };

    // Hex dump (bottom pane)
    let hex_dump = if let Some(ref frame) = app.selected_frame {
        let mut dump = column![].spacing(2);
        dump = dump.push(text("Hex Dump:").size(10));

        // Show hex dump with offset
        let data = &frame.data[..frame.dlc as usize];
        let hex: Vec<String> = data.iter().map(|b| format!("{:02X}", b)).collect();
        let ascii: String = data.iter()
            .map(|&b| if b >= 0x20 && b < 0x7F { b as char } else { '.' })
            .collect();

        dump = dump.push(
            text(format!("00000000  {:<47}  {}", hex.join(" "), ascii)).size(10)
        );

        container(dump)
    } else {
        container(text("").size(10))
    };

    column![
        filter_bar,
        horizontal_rule(1),
        container(scrollable(frame_list)).height(Length::FillPortion(3)),
        horizontal_rule(1),
        frame_detail,
        horizontal_rule(1),
        hex_dump,
    ]
    .spacing(2)
    .padding(8)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Frame detail view.
fn frame_detail_view(frame: &crate::state::LogEntry) -> Element<'_, Message> {
    let mut detail = column![].spacing(2);
    detail = detail.push(text("Frame Details:").size(10));

    // Basic info
    detail = detail.push(
        text(format!("CAN ID: 0x{:03X} ({})", frame.cob_id, classify_cob_id(frame.cob_id)))
            .size(10)
    );
    detail = detail.push(text(format!("Time: {}ms", frame.timestamp_ms)).size(10));
    detail = detail.push(text(format!("DLC: {}", frame.dlc)).size(10));

    // Decode COB-ID
    let (func, node_id) = decode_cob_id(frame.cob_id);
    detail = detail.push(text(format!("Function: {}", func)).size(10));
    detail = detail.push(text(format!("Node ID: {}", node_id)).size(10));

    // SDO decode
    if frame.is_sdo() && frame.dlc >= 4 {
        detail = detail.push(horizontal_rule(1));
        detail = detail.push(text("SDO Decode:").size(10));

        let cmd = frame.data[0];
        let index = (frame.data[2] as u16) << 8 | frame.data[1] as u16;
        let subindex = frame.data[3];

        detail = detail.push(text(format!("  Command: 0x{:02X}", cmd)).size(9));
        detail = detail.push(text(format!("  Index: 0x{:04X}", index)).size(9));
        detail = detail.push(text(format!("  Subindex: 0x{:02X}", subindex)).size(9));

        // SDO command type
        let cmd_type = match cmd & 0xE0 {
            0x20 => "Download Segment",
            0x40 => "Initiate Upload",
            0x60 => "Initiate Download",
            0x80 => "Abort",
            0xA0 => "Upload Segment",
            0xC0 => "Download Response",
            _ => "Unknown",
        };
        detail = detail.push(text(format!("  Type: {}", cmd_type)).size(9));

        // Expedited flag
        if cmd & 0x02 != 0 {
            detail = detail.push(text("  Expedited: Yes").size(9));
            let size = 4 - ((cmd >> 2) & 0x03) as usize;
            if size > 0 && frame.dlc as usize > 4 {
                let data_start = 4;
                let data_end = (data_start + size).min(frame.dlc as usize);
                let data: Vec<String> = frame.data[data_start..data_end].iter()
                    .map(|b| format!("{:02X}", b))
                    .collect();
                detail = detail.push(text(format!("  Data: {}", data.join(" "))).size(9));

                // Try to decode as different types
                if size >= 4 {
                    let val = u32::from_le_bytes([
                        frame.data[4], frame.data[5], frame.data[6], frame.data[7]
                    ]);
                    detail = detail.push(text(format!("  U32: {}", val)).size(9));
                }
            }
        }
    }

    // PDO decode
    if frame.is_pdo() {
        detail = detail.push(horizontal_rule(1));
        detail = detail.push(text("PDO Data:").size(10));

        let (pdo_type, pdo_node) = classify_pdo(frame.cob_id);
        detail = detail.push(text(format!("  Type: {}", pdo_type)).size(9));
        detail = detail.push(text(format!("  Node: {}", pdo_node)).size(9));

        // Show data as different interpretations
        let data = &frame.data[..frame.dlc as usize];
        if data.len() >= 2 {
            let u16_val = u16::from_le_bytes([data[0], data[1]]);
            detail = detail.push(text(format!("  U16: {} (0x{:04X})", u16_val, u16_val)).size(9));
        }
        if data.len() >= 4 {
            let u32_val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            detail = detail.push(text(format!("  U32: {} (0x{:08X})", u32_val, u32_val)).size(9));
            let i32_val = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            detail = detail.push(text(format!("  I32: {}", i32_val)).size(9));
        }
    }

    // EMCY decode
    if frame.is_emcy() && frame.dlc >= 2 {
        detail = detail.push(horizontal_rule(1));
        detail = detail.push(text("EMCY Decode:").size(10));

        let error_code = (frame.data[1] as u16) << 8 | frame.data[0] as u16;
        detail = detail.push(text(format!("  Error Code: 0x{:04X}", error_code)).size(9));

        if frame.dlc >= 3 {
            let error_register = frame.data[2];
            detail = detail.push(text(format!("  Error Register: 0x{:02X}", error_register)).size(9));
        }
    }

    container(detail).into()
}

/// Classify COB-ID to human-readable description.
fn classify_cob_id(cob_id: u16) -> String {
    match cob_id {
        0x000 => "NMT".to_string(),
        0x080 => "SYNC".to_string(),
        0x081..=0x0FF => format!("EMCY node {}", cob_id - 0x080),
        0x100..=0x17F => "TIME".to_string(),
        0x180..=0x1FF => format!("TPDO1 node {}", cob_id - 0x180),
        0x200..=0x27F => format!("RPDO1 node {}", cob_id - 0x200),
        0x280..=0x2FF => format!("TPDO2 node {}", cob_id - 0x280),
        0x300..=0x37F => format!("RPDO2 node {}", cob_id - 0x300),
        0x380..=0x3FF => format!("TPDO3 node {}", cob_id - 0x380),
        0x400..=0x47F => format!("RPDO3 node {}", cob_id - 0x400),
        0x480..=0x4FF => format!("TPDO4 node {}", cob_id - 0x480),
        0x500..=0x57F => format!("RPDO4 node {}", cob_id - 0x500),
        0x580..=0x5FF => format!("SDO server node {}", cob_id - 0x580),
        0x600..=0x67F => format!("SDO client node {}", cob_id - 0x600),
        0x700..=0x77F => format!("HEARTBEAT node {}", cob_id - 0x700),
        _ => "Unknown".to_string(),
    }
}

/// Decode COB-ID to function and node ID.
fn decode_cob_id(cob_id: u16) -> (String, u8) {
    match cob_id {
        0x000 => ("NMT".to_string(), 0),
        0x080 => ("SYNC".to_string(), 0),
        0x081..=0x0FF => ("EMCY".to_string(), (cob_id - 0x080) as u8),
        0x100..=0x17F => ("TIME".to_string(), 0),
        0x180..=0x1FF => ("TPDO1".to_string(), (cob_id - 0x180) as u8),
        0x200..=0x27F => ("RPDO1".to_string(), (cob_id - 0x200) as u8),
        0x280..=0x2FF => ("TPDO2".to_string(), (cob_id - 0x280) as u8),
        0x300..=0x37F => ("RPDO2".to_string(), (cob_id - 0x300) as u8),
        0x380..=0x3FF => ("TPDO3".to_string(), (cob_id - 0x380) as u8),
        0x400..=0x47F => ("RPDO3".to_string(), (cob_id - 0x400) as u8),
        0x480..=0x4FF => ("TPDO4".to_string(), (cob_id - 0x480) as u8),
        0x500..=0x57F => ("RPDO4".to_string(), (cob_id - 0x500) as u8),
        0x580..=0x5FF => ("SDO Server".to_string(), (cob_id - 0x580) as u8),
        0x600..=0x67F => ("SDO Client".to_string(), (cob_id - 0x600) as u8),
        0x700..=0x77F => ("Heartbeat".to_string(), (cob_id - 0x700) as u8),
        _ => ("Unknown".to_string(), 0),
    }
}

/// Classify a PDO COB-ID to type and node ID.
fn classify_pdo(cob_id: u16) -> (String, u8) {
    match cob_id {
        0x180..=0x1FF => ("TPDO1".to_string(), (cob_id - 0x180) as u8),
        0x200..=0x27F => ("RPDO1".to_string(), (cob_id - 0x200) as u8),
        0x280..=0x2FF => ("TPDO2".to_string(), (cob_id - 0x280) as u8),
        0x300..=0x37F => ("RPDO2".to_string(), (cob_id - 0x300) as u8),
        0x380..=0x3FF => ("TPDO3".to_string(), (cob_id - 0x380) as u8),
        0x400..=0x47F => ("RPDO3".to_string(), (cob_id - 0x400) as u8),
        0x480..=0x4FF => ("TPDO4".to_string(), (cob_id - 0x480) as u8),
        0x500..=0x57F => ("RPDO4".to_string(), (cob_id - 0x500) as u8),
        _ => ("?".to_string(), 0),
    }
}
