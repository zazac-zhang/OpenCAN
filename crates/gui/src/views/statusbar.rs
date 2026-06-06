//! Status bar view.

use iced::widget::{container, row, text};
use iced::{Element, Length};
use crate::state::{App, Message};

/// Status bar view (bottom).
pub fn statusbar(app: &App) -> Element<'_, Message> {
    let mut status = row![].spacing(8).padding(4);

    // Connection status
    let conn_text = if app.connected {
        "● Connected"
    } else {
        "○ Disconnected"
    };
    status = status.push(text(conn_text).size(10));

    status = status.push(separator());

    // Bus statistics
    if app.connected {
        status = status.push(
            text(format!("Load: {:.1}%", app.bus_stats.bus_load_percent)).size(10)
        );
        status = status.push(
            text(format!("Rate: {} fps", app.bus_stats.frame_rate)).size(10)
        );
        status = status.push(
            text(format!("Frames: {}", app.bus_stats.frame_count)).size(10)
        );

        if app.bus_stats.tx_errors > 0 || app.bus_stats.rx_errors > 0 {
            status = status.push(
                text(format!("Err: {}/{}", app.bus_stats.tx_errors, app.bus_stats.rx_errors)).size(10)
            );
        }

        if app.bus_stats.error_frame_count > 0 {
            status = status.push(
                text(format!("ErrFrames: {}", app.bus_stats.error_frame_count)).size(10)
            );
        }

        status = status.push(separator());

        // Bus state
        let bus_state = app.bus_stats.bus_state.as_str();
        status = status.push(text(format!("Bus: {}", bus_state)).size(10));
    }

    status = status.push(separator());

    // Node count
    status = status.push(
        text(format!("Nodes: {}", app.nodes.len())).size(10)
    );

    // EMCY count
    if !app.emcy_log.is_empty() {
        status = status.push(
            text(format!("EMCY: {}", app.emcy_log.len())).size(10)
        );
    }

    status = status.push(separator());

    // Last operation/status message
    let msg_display = if app.status_message.len() > 60 {
        format!("{}...", &app.status_message[..57])
    } else {
        app.status_message.clone()
    };
    status = status.push(
        text(msg_display).size(10)
    );

    // Right-aligned: filter info
    status = status.push(
        container(text("").size(10))
            .width(Length::Fill)
    );

    if app.log_filter.active_filter_count() > 0 {
        status = status.push(
            text(format!("Filters: {}", app.log_filter.active_filter_count())).size(10)
        );
    }

    container(status)
        .width(Length::Fill)
        .height(32)
        .into()
}

fn separator() -> Element<'static, Message> {
    text("│").size(10).into()
}
