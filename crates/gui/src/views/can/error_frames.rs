//! Error frames view.

use iced::widget::{column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length};
use crate::state::{App, Message};

/// Error frames view.
pub fn error_frames(app: &App) -> Element<'_, Message> {
    let mut content = column![].spacing(4).padding(10);

    // Header
    content = content.push(
        row![
            text("Error Frames").size(16),
            text(format!("({} frames)", app.error_frames.len())).size(12),
        ].spacing(8)
    );
    content = content.push(horizontal_rule(1));

    if app.error_frames.is_empty() {
        content = content.push(text("No error frames recorded").size(12));
        content = content.push(
            text("Error frames will appear here when CAN errors occur").size(11)
        );
    } else {
        // Summary
        let mut bit_errors = 0;
        let mut stuff_errors = 0;
        let mut crc_errors = 0;
        let mut form_errors = 0;
        let mut ack_errors = 0;

        for err in &app.error_frames {
            match err.error_type {
                crate::state::ErrorType::BitError => bit_errors += 1,
                crate::state::ErrorType::StuffError => stuff_errors += 1,
                crate::state::ErrorType::CrcError => crc_errors += 1,
                crate::state::ErrorType::FormError => form_errors += 1,
                crate::state::ErrorType::AckError => ack_errors += 1,
                _ => {}
            }
        }

        content = content.push(text("Error Summary:").size(14));
        content = content.push(
            row![
                text(format!("Bit: {}", bit_errors)).size(11),
                text(format!("Stuff: {}", stuff_errors)).size(11),
                text(format!("CRC: {}", crc_errors)).size(11),
                text(format!("Form: {}", form_errors)).size(11),
                text(format!("ACK: {}", ack_errors)).size(11),
            ].spacing(8)
        );

        content = content.push(horizontal_rule(1));

        // Error frame table
        content = content.push(text("Error Frame Log:").size(14));

        // Table header
        content = content.push(
            row![
                text("Time").size(9).width(70),
                text("Type").size(9).width(60),
                text("Flag").size(9).width(40),
                text("TEC").size(9).width(40),
                text("REC").size(9).width(40),
                text("Description").size(9),
            ].spacing(2)
        );
        content = content.push(horizontal_rule(1));

        // Show latest error frames
        for err in app.error_frames.iter().rev().take(100) {
            content = content.push(
                row![
                    text(err.timestamp_str()).size(9).width(70),
                    text(err.error_type.as_str()).size(9).width(60),
                    text(format!("0x{:02X}", err.error_flag)).size(9).width(40),
                    text(format!("{}", err.tec)).size(9).width(40),
                    text(format!("{}", err.rec)).size(9).width(40),
                    text(&err.description).size(9),
                ].spacing(2)
            );
        }
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
