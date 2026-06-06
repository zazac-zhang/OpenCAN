//! Node panel view.

use iced::widget::{button, column, container, horizontal_rule, scrollable, text};
use iced::{Element, Length};
use crate::state::{App, Message};

/// Node panel view (left sidebar).
pub fn node_panel(app: &App) -> Element<'_, Message> {
    let mut panel = column![].spacing(4).padding(8);

    // Header
    panel = panel.push(text("Nodes").size(14));
    panel = panel.push(
        text(format!("{} found", app.nodes.len()))
            .size(11)
    );

    // Connection status
    let (conn_indicator, conn_text) = if app.connected {
        ("●", "Connected")
    } else {
        ("○", "Disconnected")
    };
    panel = panel.push(
        text(format!("{} {}", conn_indicator, conn_text))
            .size(11)
    );

    panel = panel.push(horizontal_rule(1));

    // Node list
    if app.nodes.is_empty() {
        panel = panel.push(
            text("  (no nodes)")
                .size(11)
        );
    } else {
        for node in &app.nodes {
            let is_selected = app.selected_node == Some(node.node_id);

            // Node button with status indicator
            let state_indicator = node.nmt_state.color_indicator();
            let label = format!("{} Node {}", state_indicator, node.node_id);

            let btn = if is_selected {
                button(
                    column![
                        text(format!("► {}", label)).size(11),
                        text(format!("   {}", node.nmt_state.as_str())).size(9),
                    ]
                )
                .on_press(Message::NodeSelected(node.node_id))
                .width(Length::Fill)
            } else {
                button(
                    column![
                        text(label).size(11),
                        text(format!("   {}", node.nmt_state.as_str())).size(9),
                    ]
                )
                .on_press(Message::NodeSelected(node.node_id))
                .width(Length::Fill)
            };
            panel = panel.push(btn);
        }
    }

    // Quick actions for selected node
    if let Some(node_id) = app.selected_node {
        panel = panel.push(horizontal_rule(1));
        panel = panel.push(text(format!("Node {} Actions:", node_id)).size(11));

        panel = panel.push(
            button(text("NMT Start").size(10))
                .on_press(Message::NmtStartNode(node_id))
                .width(Length::Fill)
        );
        panel = panel.push(
            button(text("NMT Stop").size(10))
                .on_press(Message::NmtStopNode(node_id))
                .width(Length::Fill)
        );
        panel = panel.push(
            button(text("NMT Reset").size(10))
                .on_press(Message::NmtResetNode(node_id))
                .width(Length::Fill)
        );
    }

    // Filter info
    if app.log_filter.active_filter_count() > 0 {
        panel = panel.push(horizontal_rule(1));
        panel = panel.push(text("Active Filters:").size(10));
        panel = panel.push(
            text(app.log_filter.summary())
                .size(9)
        );
        panel = panel.push(
            button(text("Clear Filters").size(10))
                .on_press(Message::LogFilterClear)
                .width(Length::Fill)
        );
    }

    container(scrollable(panel))
        .width(200)
        .height(Length::Fill)
        .into()
}
