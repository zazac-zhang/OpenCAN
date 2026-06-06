//! PDO monitor view.

use iced::widget::{column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length};
use crate::state::{App, Message};

/// PDO monitor view.
pub fn pdo_monitor(app: &App) -> Element<'_, Message> {
    let mut content = column![].spacing(4).padding(10);

    // Header
    content = content.push(
        row![
            text("PDO Monitor").size(16),
            text(format!("({} frames)", app.pdo_log.len())).size(12),
        ].spacing(8)
    );
    content = content.push(horizontal_rule(1));

    if !app.connected {
        content = content.push(text("Not connected. PDO data will appear here when connected.").size(12));
    } else if app.pdo_log.is_empty() {
        content = content.push(text("No PDO frames received yet.").size(12));
        content = content.push(
            text("PDOs appear when nodes send TPDO or receive RPDO.").size(11)
        );
    } else {
        // PDO type summary
        let mut tpdo1 = 0;
        let mut rpdo1 = 0;
        let mut tpdo2 = 0;
        let mut rpdo2 = 0;
        let mut tpdo3 = 0;
        let mut rpdo3 = 0;
        let mut tpdo4 = 0;
        let mut rpdo4 = 0;

        for entry in &app.pdo_log {
            match entry.cob_id {
                0x180..=0x1FF => tpdo1 += 1,
                0x200..=0x27F => rpdo1 += 1,
                0x280..=0x2FF => tpdo2 += 1,
                0x300..=0x37F => rpdo2 += 1,
                0x380..=0x3FF => tpdo3 += 1,
                0x400..=0x47F => rpdo3 += 1,
                0x480..=0x4FF => tpdo4 += 1,
                0x500..=0x57F => rpdo4 += 1,
                _ => {}
            }
        }

        content = content.push(text("PDO Summary:").size(12));
        content = content.push(
            row![
                text(format!("TPDO1: {}", tpdo1)).size(10),
                text(format!("RPDO1: {}", rpdo1)).size(10),
                text(format!("TPDO2: {}", tpdo2)).size(10),
                text(format!("RPDO2: {}", rpdo2)).size(10),
            ].spacing(8)
        );
        content = content.push(
            row![
                text(format!("TPDO3: {}", tpdo3)).size(10),
                text(format!("RPDO3: {}", rpdo3)).size(10),
                text(format!("TPDO4: {}", tpdo4)).size(10),
                text(format!("RPDO4: {}", rpdo4)).size(10),
            ].spacing(8)
        );

        content = content.push(horizontal_rule(1));

        // PDO frame table
        content = content.push(text("PDO Frames:").size(12));

        // Table header
        content = content.push(
            row![
                text("Time").size(9).width(70),
                text("COB-ID").size(9).width(50),
                text("Type").size(9).width(50),
                text("Node").size(9).width(30),
                text("Data").size(9).width(150),
                text("Decode").size(9),
            ].spacing(2)
        );
        content = content.push(horizontal_rule(1));

        // Show latest PDOs
        for entry in app.pdo_log.iter().rev().take(200) {
            let (pdo_type, node_id) = classify_pdo(entry.cob_id);

            // Try to decode data
            let decode = decode_pdo_data(entry);

            content = content.push(
                row![
                    text(entry.timestamp_str()).size(9).width(70),
                    text(format!("{:03X}", entry.cob_id)).size(9).width(50),
                    text(pdo_type).size(9).width(50),
                    text(format!("{}", node_id)).size(9).width(30),
                    text(entry.hex_data()).size(9).width(150),
                    text(decode).size(9),
                ].spacing(2)
            );
        }
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
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

/// Decode PDO data to human-readable format.
fn decode_pdo_data(entry: &crate::state::LogEntry) -> String {
    let data = &entry.data[..entry.dlc as usize];

    if data.len() >= 4 {
        // Try as U32
        let u32_val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        // Try as I32
        let i32_val = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        // Heuristic: if value looks like a position (large number) or velocity
        if i32_val.abs() > 1000 {
            format!("I32: {}", i32_val)
        } else {
            format!("U32: {}", u32_val)
        }
    } else if data.len() >= 2 {
        let u16_val = u16::from_le_bytes([data[0], data[1]]);
        format!("U16: {}", u16_val)
    } else {
        String::new()
    }
}
