//! Network management view.

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length};
use crate::state::{App, Message};

/// Network management view.
pub fn network_management(app: &App) -> Element<'_, Message> {
    let mut content = column![].spacing(8).padding(10);

    // Header
    content = content.push(text("Network Management").size(16));
    content = content.push(horizontal_rule(1));

    if !app.connected {
        content = content.push(text("Not connected. Please connect first.").size(12));
        return container(scrollable(content)).width(Length::Fill).height(Length::Fill).into();
    }

    // Batch operations
    content = content.push(text("Quick Actions:").size(14));
    content = content.push(
        row![
            button(text("Scan Nodes").size(11)).on_press(Message::ScanNodes),
            button(text("Start All").size(11)).on_press(Message::NmtStartAll),
            button(text("Stop All").size(11)).on_press(Message::NmtStopAll),
            button(text("Reset All").size(11)).on_press(Message::NmtResetAll),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // Node table
    content = content.push(
        row![
            text("Nodes:").size(14),
            text(format!("({} found)", app.nodes.len())).size(11),
        ].spacing(8)
    );

    if app.nodes.is_empty() {
        content = content.push(text("No nodes detected. Click 'Scan Nodes' to discover nodes.").size(12));
    } else {
        // Table header
        content = content.push(
            row![
                text("ID").size(9).width(30),
                text("NMT State").size(9).width(80),
                text("Device Type").size(9).width(70),
                text("Vendor").size(9).width(60),
                text("Heartbeat").size(9).width(60),
                text("DS402").size(9).width(40),
                text("Actions").size(9),
            ].spacing(2)
        );
        content = content.push(horizontal_rule(1));

        // Node list
        for node in &app.nodes {
            let device_type = node.device_type
                .map(|dt| format!("0x{:08X}", dt))
                .unwrap_or_else(|| "-".to_string());
            let vendor_id = node.vendor_id
                .map(|vid| format!("0x{:08X}", vid))
                .unwrap_or_else(|| "-".to_string());
            let heartbeat = node.heartbeat_period
                .map(|p| format!("{}ms", p))
                .unwrap_or_else(|| "-".to_string());
            let ds402_status = if node.has_ds402() { "●" } else { "-" };

            let is_selected = app.selected_node == Some(node.node_id);

            let row_content = row![
                text(format!("{}", node.node_id)).size(10).width(30),
                text(format!("{} {}", node.nmt_state.color_indicator(), node.nmt_state.as_str())).size(10).width(80),
                text(device_type).size(10).width(70),
                text(vendor_id).size(10).width(60),
                text(heartbeat).size(10).width(60),
                text(ds402_status).size(10).width(40),
                row![
                    button(text("Start").size(9)).on_press(Message::NmtStartNode(node.node_id)),
                    button(text("Stop").size(9)).on_press(Message::NmtStopNode(node.node_id)),
                    button(text("Reset").size(9)).on_press(Message::NmtResetNode(node.node_id)),
                ].spacing(2),
            ].spacing(2);

            let btn = if is_selected {
                button(row_content)
                    .on_press(Message::NodeSelected(node.node_id))
                    .width(Length::Fill)
            } else {
                button(row_content)
                    .on_press(Message::NodeSelected(node.node_id))
                    .width(Length::Fill)
            };

            content = content.push(btn);
        }
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
