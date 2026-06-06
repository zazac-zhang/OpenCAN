//! Connection dialog view.

use iced::widget::{button, column, container, horizontal_rule, row, text, text_input};
use iced::{Element};
use crate::state::{App, Message, CanBackend};

/// Connection dialog overlay.
pub fn connection_dialog(app: &App) -> Element<'_, Message> {
    let dialog = &app.connection_dialog;

    let mut content = column![].spacing(8).padding(16);

    // Header
    content = content.push(text("Connect to CAN Bus").size(16));
    content = content.push(horizontal_rule(1));

    // Error message
    if let Some(ref error) = dialog.error {
        content = content.push(
            text(format!("Error: {}", error)).size(12)
        );
    }

    // Backend selection
    content = content.push(text("Backend:").size(12));
    let mut backend_row = row![].spacing(4);
    for b in CanBackend::all() {
        let label = b.name();
        let is_selected = *b == dialog.selected_backend;
        let btn = if is_selected {
            button(text(format!("[{}]", label)).size(11))
                .on_press(Message::ConnectionBackendChanged(*b))
        } else {
            button(text(label).size(11))
                .on_press(Message::ConnectionBackendChanged(*b))
        };
        backend_row = backend_row.push(btn);
    }
    content = content.push(backend_row);

    // Backend description
    content = content.push(
        text(dialog.selected_backend.description())
            .size(10)
    );

    content = content.push(horizontal_rule(1));

    // Channel input
    if dialog.selected_backend.requires_channel() {
        content = content.push(
            row![
                text("Channel:").size(12),
                text_input(dialog.selected_backend.default_channel(), &dialog.channel)
                    .on_input(Message::ConnectionChannelChanged)
                    .width(200),
            ].spacing(8)
        );
    }

    // Bitrate input
    content = content.push(
        row![
            text("Bitrate:").size(12),
            text_input("500000", &dialog.bitrate)
                .on_input(Message::ConnectionBitrateChanged)
                .width(200),
            text("bps").size(10),
        ].spacing(4)
    );

    // Bitrate quick select
    let mut bitrate_row = row![].spacing(4);
    for br in [125000u32, 250000, 500000, 1000000] {
        let label = format!("{}k", br / 1000);
        bitrate_row = bitrate_row.push(
            button(text(label).size(10))
                .on_press(Message::ConnectionBitrateChanged(br.to_string()))
        );
    }
    content = content.push(bitrate_row);

    // Node ID input
    content = content.push(
        row![
            text("Node ID:").size(12),
            text_input("0", &dialog.node_id)
                .on_input(Message::ConnectionNodeIdChanged)
                .width(60),
            text("(0 = master, 1-127 = slave)").size(10),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // Connecting indicator
    if dialog.connecting {
        content = content.push(text("Connecting...").size(12));
    }

    // Action buttons
    content = content.push(
        row![
            button(text("Connect").size(14))
                .on_press(Message::ConnectionConnect),
            button(text("Cancel").size(14))
                .on_press(Message::HideConnectionDialog),
        ].spacing(8)
    );

    // Wrap in a styled container
    container(content)
        .width(450)
        .padding(4)
        .into()
}
