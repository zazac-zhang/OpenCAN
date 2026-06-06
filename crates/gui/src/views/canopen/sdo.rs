//! SDO client view.

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Element, Length};
use crate::state::{App, Message};

/// SDO client view.
pub fn sdo_client(app: &App) -> Element<'_, Message> {
    let mut content = column![].spacing(8).padding(10);

    // Header
    content = content.push(text("SDO Client").size(16));
    content = content.push(horizontal_rule(1));

    // SDO read/write panel
    content = content.push(text("SDO Access:").size(14));

    // Node selection
    let node_str = app.selected_node
        .map(|n| n.to_string())
        .unwrap_or_default();
    content = content.push(
        row![
            text("Node ID:").size(11),
            text_input("1", &node_str)
                .width(40),
        ].spacing(4)
    );

    // Index/Subindex
    content = content.push(
        row![
            text("Index:").size(11),
            text_input("0x1000", &app.sdo_index)
                .on_input(Message::SdoIndexChanged)
                .width(80),
            text("Subindex:").size(11),
            text_input("0", &app.sdo_subindex)
                .on_input(Message::SdoSubindexChanged)
                .width(40),
        ].spacing(4)
    );

    // Data type (simplified for now)
    content = content.push(
        row![
            text("Data Type:").size(11),
            text(&app.sdo_data_type).size(11),
        ].spacing(4)
    );

    // Value input
    content = content.push(
        row![
            text("Value:").size(11),
            text_input("hex bytes", &app.sdo_value)
                .on_input(Message::SdoValueChanged)
                .width(Length::Fill),
        ].spacing(4)
    );

    // Action buttons
    content = content.push(
        row![
            button(text("Read").size(12)).on_press(Message::SdoRead),
            button(text("Write").size(12)).on_press(Message::SdoWrite),
        ].spacing(8)
    );

    content = content.push(horizontal_rule(1));

    // Quick read buttons
    content = content.push(text("Quick Read:").size(12));
    content = content.push(
        row![
            button(text("Device Type (1000:0)").size(10))
                .on_press(Message::SdoQuickRead(0x1000, 0)),
            button(text("Error Reg (1001:0)").size(10))
                .on_press(Message::SdoQuickRead(0x1001, 0)),
            button(text("Vendor ID (1018:1)").size(10))
                .on_press(Message::SdoQuickRead(0x1018, 1)),
        ].spacing(4)
    );
    content = content.push(
        row![
            button(text("Status Word (6041:0)").size(10))
                .on_press(Message::SdoQuickRead(0x6041, 0)),
            button(text("Actual Pos (6064:0)").size(10))
                .on_press(Message::SdoQuickRead(0x6064, 0)),
            button(text("Actual Vel (606C:0)").size(10))
                .on_press(Message::SdoQuickRead(0x606C, 0)),
        ].spacing(4)
    );
    content = content.push(
        row![
            button(text("Actual Torque (6077:0)").size(10))
                .on_press(Message::SdoQuickRead(0x6077, 0)),
            button(text("Mode (6060:0)").size(10))
                .on_press(Message::SdoQuickRead(0x6060, 0)),
            button(text("Heartbeat (1017:0)").size(10))
                .on_press(Message::SdoQuickRead(0x1017, 0)),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // SDO history
    content = content.push(
        row![
            text("SDO History:").size(14),
            text(format!("({} entries)", app.sdo_history.len())).size(11),
            button(text("Clear").size(10)).on_press(Message::SdoClearHistory),
        ].spacing(8)
    );

    if app.sdo_history.is_empty() {
        content = content.push(text("No SDO operations yet").size(11));
    } else {
        // History table header
        content = content.push(
            row![
                text("Node").size(9).width(30),
                text("Index:Sub").size(9).width(60),
                text("Type").size(9).width(30),
                text("Value").size(9).width(100),
                text("Result").size(9),
            ].spacing(2)
        );
        content = content.push(horizontal_rule(1));

        // Show latest entries
        for entry in app.sdo_history.iter().rev().take(50) {
            let op_type = if entry.is_read { "R" } else { "W" };
            let result = if entry.success {
                &entry.value
            } else {
                entry.error.as_deref().unwrap_or("Error")
            };

            content = content.push(
                row![
                    text(format!("{}", entry.node_id)).size(9).width(30),
                    text(format!("{:04X}:{:02X}", entry.index, entry.subindex)).size(9).width(60),
                    text(op_type).size(9).width(30),
                    text(result).size(9).width(100),
                    text(if entry.success { "OK" } else { "FAIL" }).size(9),
                ].spacing(2)
            );
        }
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
