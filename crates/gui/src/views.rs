//! View rendering functions.

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::state::{App, Message, View, NmtState};

/// Network overview page.
pub fn network_overview(app: &App) -> Element<'_, Message> {
    let mut content = column![text("Network Overview").size(20)].spacing(10).padding(10);

    if app.nodes.is_empty() {
        content = content.push(text("No nodes detected. Connect to a CAN bus to scan for nodes.").size(14));
    } else {
        for node in &app.nodes {
            let state_str = match node.nmt_state {
                NmtState::Unknown => "Unknown",
                NmtState::BootUp => "BootUp",
                NmtState::PreOperational => "Pre-Operational",
                NmtState::Operational => "Operational",
                NmtState::Stopped => "Stopped",
            };

            let node_card = container(
                column![
                    text(format!("Node {}", node.node_id)).size(16),
                    text(format!("State: {}", state_str)).size(12),
                    row![
                        button(text("Start").size(12)).on_press(Message::NmtStartNode(node.node_id)),
                        button(text("Stop").size(12)).on_press(Message::NmtStopNode(node.node_id)),
                        button(text("Reset").size(12)).on_press(Message::NmtResetNode(node.node_id)),
                        button(text("DS402").size(12)).on_press(Message::SwitchView(View::Ds402)),
                    ].spacing(4),
                ].spacing(4)
            )
            .padding(8)
            .width(Length::Fill);

            content = content.push(node_card);
        }
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Node detail page.
pub fn node_detail(app: &App) -> Element<'_, Message> {
    let node_id = app.selected_node.unwrap_or(1);

    let mut content = column![
        text(format!("Node {} Detail", node_id)).size(20),
        horizontal_rule(1),
    ].spacing(10).padding(10);

    // SDO Read/Write section
    content = content.push(text("SDO Access").size(16));

    let sdo_form = row![
        text_input("Index (0x1000)", &app.sdo_index)
            .on_input(Message::SdoIndexChanged)
            .width(100),
        text_input("Subindex", &app.sdo_subindex)
            .on_input(Message::SdoSubindexChanged)
            .width(60),
        button(text("Read").size(12)).on_press(Message::SdoRead),
        text_input("Value", &app.sdo_value)
            .on_input(Message::SdoValueChanged)
            .width(150),
        button(text("Write").size(12)).on_press(Message::SdoWrite),
    ].spacing(4);

    content = content.push(sdo_form);
    content = content.push(horizontal_rule(1));

    // OD cache display
    content = content.push(text("Cached OD Values:").size(14));
    for ((idx, sub), val) in &app.nodes.iter()
        .find(|n| n.node_id == node_id)
        .map(|n| &n.od_cache)
        .cloned()
        .unwrap_or_default()
    {
        content = content.push(
            text(format!("  {:04X}:{:02X} = {}", idx, sub, val)).size(12)
        );
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// DS402 panel.
pub fn ds402_panel(app: &App) -> Element<'_, Message> {
    let node_id = app.selected_node.unwrap_or(1);

    let content = column![
        text(format!("DS402 Control - Node {}", node_id)).size(20),
        horizontal_rule(1),
        text("State Machine:").size(14),
        text("[Not Connected]").size(12),
        horizontal_rule(1),
        text("Control:").size(14),
        row![
            button(text("Enable").size(12)).on_press(Message::Ds402Enable(node_id)),
            button(text("Fault Reset").size(12)).on_press(Message::Ds402FaultReset(node_id)),
        ].spacing(4),
        horizontal_rule(1),
        text("Status:").size(14),
        text("  Status Word: --").size(12),
        text("  Control Word: --").size(12),
        text("  Operation Mode: --").size(12),
        horizontal_rule(1),
        text("Real-time Data:").size(14),
        text("  Actual Position: --").size(12),
        text("  Actual Velocity: --").size(12),
        text("  Actual Torque: --").size(12),
    ].spacing(8).padding(10);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// PDO monitor page.
pub fn pdo_monitor(_app: &App) -> Element<'_, Message> {
    let content = column![
        text("PDO Monitor").size(20),
        horizontal_rule(1),
        text("No PDO data received yet.").size(14),
        text("Connect to a CAN bus and start receiving PDOs.").size(12),
    ].spacing(10).padding(10);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// CAN log page.
pub fn can_log(app: &App) -> Element<'_, Message> {
    let mut content = column![
        text("CAN Log").size(20),
        horizontal_rule(1),
    ].spacing(4).padding(10);

    if app.can_log.is_empty() {
        content = content.push(text("No CAN frames logged yet.").size(14));
    } else {
        for entry in app.can_log.iter().rev().take(100) {
            let hex_data: String = entry.data.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            content = content.push(
                text(format!(
                    "[{:>8}ms] {:03X} [{}] {}",
                    entry.timestamp_ms,
                    entry.cob_id,
                    entry.data.len(),
                    hex_data
                )).size(11)
            );
        }
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
