//! Heartbeat monitor view.

use iced::widget::{column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length};
use crate::state::{App, Message};

/// Heartbeat monitor view.
pub fn heartbeat_monitor(app: &App) -> Element<'_, Message> {
    let mut content = column![].spacing(4).padding(10);

    // Header
    content = content.push(text("Heartbeat Monitor").size(16));
    content = content.push(horizontal_rule(1));

    if app.heartbeat_status.is_empty() {
        content = content.push(text("No heartbeat data available").size(12));
        content = content.push(
            text("Heartbeat status will appear here when nodes send heartbeat messages").size(11)
        );
    } else {
        // Summary
        let online_count = app.heartbeat_status.iter().filter(|h| h.alive).count();
        let offline_count = app.heartbeat_status.len() - online_count;

        content = content.push(text("Summary:").size(14));
        content = content.push(
            row![
                text(format!("Online: {}", online_count)).size(12),
                text(format!("Offline: {}", offline_count)).size(12),
                text(format!("Total: {}", app.heartbeat_status.len())).size(12),
            ].spacing(16)
        );

        content = content.push(horizontal_rule(1));

        // Heartbeat table
        content = content.push(text("Node Status:").size(14));

        // Table header
        content = content.push(
            row![
                text("Node").size(9).width(40),
                text("Status").size(9).width(50),
                text("Producer Period").size(9).width(80),
                text("Last Heartbeat").size(9).width(80),
                text("Missed").size(9).width(40),
            ].spacing(2)
        );
        content = content.push(horizontal_rule(1));

        // Show heartbeat status for each node
        for hb in &app.heartbeat_status {
            let period = hb.producer_period_ms
                .map(|p| format!("{}ms", p))
                .unwrap_or_else(|| "-".to_string());
            let last = hb.last_heartbeat_ms
                .map(|t| format!("{}ms", t))
                .unwrap_or_else(|| "-".to_string());

            content = content.push(
                row![
                    text(format!("{}", hb.node_id)).size(10).width(40),
                    text(format!("{} {}", hb.status_indicator(), hb.status_text())).size(10).width(50),
                    text(period).size(10).width(80),
                    text(last).size(10).width(80),
                    text(format!("{}", hb.missed_count)).size(10).width(40),
                ].spacing(2)
            );
        }

        content = content.push(horizontal_rule(1));

        // Legend
        content = content.push(text("Legend:").size(12));
        content = content.push(text("  ● Online - heartbeat received within timeout").size(10));
        content = content.push(text("  ○ Offline - heartbeat not received or timeout").size(10));
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
