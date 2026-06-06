//! Detail panel view.

use iced::widget::{button, column, container, horizontal_rule, scrollable, text};
use iced::{Element, Length};
use crate::state::{App, Message};

/// Detail panel view (right sidebar).
pub fn detail_panel(app: &App) -> Element<'_, Message> {
    let mut content = column![].spacing(4).padding(8);

    // Header
    content = content.push(
        button(text("◀ Hide").size(10))
            .on_press(Message::ToggleDetailPanel)
    );
    content = content.push(text("Details").size(14));
    content = content.push(horizontal_rule(1));

    if let Some(node_id) = app.selected_node {
        let node = app.get_node(node_id);

        if let Some(node) = node {
            // Node info
            content = content.push(text(format!("Node {}", node.node_id)).size(13));
            content = content.push(
                text(format!("{} {}", node.nmt_state.color_indicator(), node.nmt_state.as_str()))
                    .size(11)
            );

            if let Some(dt) = node.device_type {
                content = content.push(text(format!("Device Type: 0x{:08X}", dt)).size(10));
            }
            if let Some(vid) = node.vendor_id {
                content = content.push(text(format!("Vendor ID: 0x{:08X}", vid)).size(10));
            }
            if let Some(ref name) = node.product_name {
                content = content.push(text(format!("Product: {}", name)).size(10));
            }
            if let Some(err) = node.error_register {
                content = content.push(text(format!("Error Reg: 0x{:02X}", err)).size(10));
            }

            content = content.push(horizontal_rule(1));

            // OD cache
            content = content.push(text("OD Cache:").size(12));
            if node.od_cache.is_empty() {
                content = content.push(text("  (empty)").size(10));
            } else {
                content = content.push(
                    text(format!("  {} entries", node.od_cache.len())).size(10)
                );
                // Show first 20 entries
                for ((idx, sub), entry) in node.od_cache.iter().take(20) {
                    let display = entry.display();
                    content = content.push(
                        text(format!("  {:04X}:{:02X} {}", idx, sub, display)).size(9)
                    );
                }
                if node.od_cache.len() > 20 {
                    content = content.push(
                        text(format!("  ... and {} more", node.od_cache.len() - 20)).size(9)
                    );
                }
            }

            // DS402 quick status
            if node.has_ds402() {
                content = content.push(horizontal_rule(1));
                content = content.push(text("DS402:").size(12));
                content = content.push(
                    text(format!("  State: {}", node.ds402.state)).size(10)
                );
                content = content.push(
                    text(format!("  Status: 0x{:04X}", node.ds402.status_word)).size(10)
                );
                content = content.push(
                    text(format!("  Pos: {}", node.ds402.actual_position)).size(10)
                );
                content = content.push(
                    text(format!("  Vel: {}", node.ds402.actual_velocity)).size(10)
                );
                content = content.push(
                    text(format!("  Torque: {}", node.ds402.actual_torque)).size(10)
                );
            }

            // Heartbeat info
            if let Some(period) = node.heartbeat_period {
                content = content.push(horizontal_rule(1));
                content = content.push(text("Heartbeat:").size(12));
                content = content.push(
                    text(format!("  Period: {}ms", period)).size(10)
                );
                let hb_status = app.heartbeat_status.iter()
                    .find(|h| h.node_id == node_id);
                if let Some(hb) = hb_status {
                    content = content.push(
                        text(format!("  Status: {}", hb.status_text())).size(10)
                    );
                }
            }
        } else {
            content = content.push(
                text(format!("Node {} (not found)", node_id)).size(11)
            );
        }
    } else {
        content = content.push(text("Select a node").size(11));
    }

    container(scrollable(content))
        .width(200)
        .height(Length::Fill)
        .into()
}
