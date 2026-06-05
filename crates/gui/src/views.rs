//! View rendering functions.

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::state::{App, Message, View};

/// Network overview page.
pub fn network_overview(app: &App) -> Element<'_, Message> {
    let mut content = column![text("Network Overview").size(20)].spacing(8).padding(10);

    if !app.connected {
        content = content.push(
            text("Not connected. Click 'Connect (Mock)' in the sidebar to start.").size(14)
        );
        return container(scrollable(content)).width(Length::Fill).height(Length::Fill).into();
    }

    if app.nodes.is_empty() {
        content = content.push(text("No nodes detected.").size(14));
        content = content.push(
            button(text("Scan Nodes").size(14))
                .on_press(Message::ScanNodes)
        );
    } else {
        content = content.push(
            row![
                text(format!("{} nodes found", app.nodes.len())).size(14),
                button(text("Rescan").size(12)).on_press(Message::ScanNodes),
            ].spacing(8)
        );

        for node in &app.nodes {
            let state_str = node.nmt_state.as_str();
            let ds402_str = if node.ds402.state.is_empty() { "" } else { &node.ds402.state };

            let node_card = container(
                column![
                    row![
                        text(format!("Node {}", node.node_id)).size(16),
                        text(format!("[{}]", state_str)).size(12),
                        if !ds402_str.is_empty() {
                            text(format!("DS402: {}", ds402_str)).size(11)
                        } else {
                            text("").size(11)
                        },
                    ].spacing(8),
                    row![
                        button(text("Start").size(11)).on_press(Message::NmtStartNode(node.node_id)),
                        button(text("Stop").size(11)).on_press(Message::NmtStopNode(node.node_id)),
                        button(text("Reset").size(11)).on_press(Message::NmtResetNode(node.node_id)),
                        button(text("Detail →").size(11)).on_press(Message::NodeSelected(node.node_id)),
                        button(text("DS402 →").size(11)).on_press(Message::SwitchView(View::Ds402)),
                    ].spacing(4),
                ].spacing(4)
            )
            .padding(8)
            .width(Length::Fill);

            content = content.push(node_card);
        }
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// Node detail page.
pub fn node_detail(app: &App) -> Element<'_, Message> {
    let node_id = app.selected_node.unwrap_or(1);
    let node = app.nodes.iter().find(|n| n.node_id == node_id);

    let mut content = column![
        text(format!("Node {} Detail", node_id)).size(20),
        horizontal_rule(1),
    ].spacing(8).padding(10);

    // Node info
    if let Some(node) = node {
        content = content.push(text(format!("State: {}", node.nmt_state.as_str())).size(14));
        if let Some(dt) = node.device_type {
            content = content.push(text(format!("Device Type: 0x{:08X}", dt)).size(12));
        }
        if let Some(vid) = node.vendor_id {
            content = content.push(text(format!("Vendor ID: 0x{:08X}", vid)).size(12));
        }
    }

    content = content.push(horizontal_rule(1));

    // SDO Read/Write section
    content = content.push(text("SDO Access").size(16));
    content = content.push(
        row![
            text("Index:").size(12),
            text_input("0x1000", &app.sdo_index)
                .on_input(Message::SdoIndexChanged)
                .width(80),
            text("Sub:").size(12),
            text_input("0", &app.sdo_subindex)
                .on_input(Message::SdoSubindexChanged)
                .width(40),
        ].spacing(4)
    );
    content = content.push(
        row![
            text("Value:").size(12),
            text_input("hex bytes", &app.sdo_value)
                .on_input(Message::SdoValueChanged)
                .width(200),
            button(text("Read").size(12)).on_press(Message::SdoRead),
            button(text("Write").size(12)).on_press(Message::SdoWrite),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // Quick read buttons
    content = content.push(text("Quick Read:").size(14));
    content = content.push(
        row![
            button(text("Device Type (1000)").size(11))
                .on_press(Message::SdoIndexChanged("1000".to_string())),
            button(text("Error Reg (1001)").size(11))
                .on_press(Message::SdoIndexChanged("1001".to_string())),
            button(text("Status Word (6041)").size(11))
                .on_press(Message::SdoIndexChanged("6041".to_string())),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // OD cache
    content = content.push(text("Cached OD Values:").size(14));
    if let Some(node) = node {
        if node.od_cache.is_empty() {
            content = content.push(text("  (empty — read an SDO to populate)").size(11));
        }
        for ((idx, sub), val) in &node.od_cache {
            content = content.push(
                text(format!("  {:04X}:{:02X} = {}", idx, sub, val)).size(11)
            );
        }
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// DS402 panel.
pub fn ds402_panel(app: &App) -> Element<'_, Message> {
    let node_id = app.selected_node.unwrap_or(1);
    let node = app.nodes.iter().find(|n| n.node_id == node_id);
    let ds402 = node.map(|n| &n.ds402);

    let mut content = column![
        text(format!("DS402 Control — Node {}", node_id)).size(20),
        horizontal_rule(1),
    ].spacing(8).padding(10);

    // State machine display
    content = content.push(text("State Machine:").size(16));
    let state_str = ds402.map(|d| d.state.as_str()).unwrap_or("--");
    let status_word = ds402.map(|d| d.status_word).unwrap_or(0);
    content = content.push(text(format!("  Current State: {}", state_str)).size(14));
    content = content.push(text(format!("  Status Word: 0x{:04X}", status_word)).size(12));

    content = content.push(
        row![
            button(text("Read State").size(12)).on_press(Message::Ds402ReadState(node_id)),
            button(text("Enable").size(12)).on_press(Message::Ds402Enable(node_id)),
            button(text("Fault Reset").size(12)).on_press(Message::Ds402FaultReset(node_id)),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // Position control
    content = content.push(text("Position Control:").size(16));
    content = content.push(
        row![
            text("Target:").size(12),
            text_input("0", &app.ds402_state.target_position)
                .on_input(Message::Ds402TargetPositionChanged)
                .width(100),
            button(text("Set").size(12)).on_press(Message::Ds402SetPosition(node_id)),
            button(text("Read Actual").size(12)).on_press(Message::Ds402ReadPosition(node_id)),
        ].spacing(4)
    );
    content = content.push(
        text(format!("  Actual Position: {}", ds402.map(|d| d.actual_position).unwrap_or(0))).size(12)
    );

    content = content.push(horizontal_rule(1));

    // Velocity control
    content = content.push(text("Velocity Control:").size(16));
    content = content.push(
        row![
            text("Target:").size(12),
            text_input("0", &app.ds402_state.target_velocity)
                .on_input(Message::Ds402TargetVelocityChanged)
                .width(100),
            button(text("Set").size(12)).on_press(Message::Ds402SetVelocity(node_id)),
            button(text("Read Actual").size(12)).on_press(Message::Ds402ReadVelocity(node_id)),
        ].spacing(4)
    );
    content = content.push(
        text(format!("  Actual Velocity: {}", ds402.map(|d| d.actual_velocity).unwrap_or(0))).size(12)
    );

    content = content.push(horizontal_rule(1));

    // Status word breakdown
    content = content.push(text("Status Word Bits:").size(14));
    let bits = [
        (0x0001, "Ready To Switch On"),
        (0x0002, "Switched On"),
        (0x0004, "Operation Enabled"),
        (0x0008, "Fault"),
        (0x0010, "Voltage Enabled"),
        (0x0020, "Quick Stop"),
        (0x0040, "Switch On Disabled"),
        (0x0080, "Warning"),
        (0x0200, "Remote"),
        (0x0400, "Target Reached"),
        (0x0800, "Internal Limit Active"),
    ];
    for (mask, name) in bits {
        let active = status_word & mask != 0;
        let marker = if active { "●" } else { "○" };
        content = content.push(text(format!("  {} {} ({:04X})", marker, name, mask)).size(10));
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// PDO monitor page.
pub fn pdo_monitor(_app: &App) -> Element<'_, Message> {
    let content = column![
        text("PDO Monitor").size(20),
        horizontal_rule(1),
        text("PDO monitoring requires CAN frame subscription.").size(14),
        text("Connect to a CAN bus and PDOs will appear here.").size(12),
        horizontal_rule(1),
        text("PDO Mapping (when available):").size(14),
        text("  TPDO1 (0x180): --").size(12),
        text("  RPDO1 (0x200): --").size(12),
        text("  TPDO2 (0x280): --").size(12),
        text("  RPDO2 (0x300): --").size(12),
        text("  TPDO3 (0x380): --").size(12),
        text("  RPDO3 (0x400): --").size(12),
        text("  TPDO4 (0x480): --").size(12),
        text("  RPDO4 (0x500): --").size(12),
    ].spacing(8).padding(10);

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// CAN log page.
pub fn can_log(app: &App) -> Element<'_, Message> {
    let mut content = column![
        row![
            text("CAN Log").size(20),
            text(format!("({} frames)", app.can_log.len())).size(14),
        ].spacing(8),
        horizontal_rule(1),
    ].spacing(4).padding(10);

    if app.can_log.is_empty() {
        content = content.push(text("No CAN frames logged yet.").size(14));
    } else {
        // Show latest 200 frames (newest first)
        for entry in app.can_log.iter().rev().take(200) {
            let hex_data: String = entry.data.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            let desc = if entry.description.is_empty() {
                classify_cob_id(entry.cob_id)
            } else {
                entry.description.clone()
            };

            content = content.push(
                text(format!(
                    "[{:>8}ms] {:03X} [{}] {}  {}",
                    entry.timestamp_ms,
                    entry.cob_id,
                    entry.data.len(),
                    hex_data,
                    desc
                )).size(10)
            );
        }
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
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
        _ => format!("Unknown"),
    }
}
