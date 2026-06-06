//! Toolbar view.

use iced::widget::{button, container, row, text};
use iced::{Element, Length};
use crate::state::{App, Message};

/// Toolbar view.
pub fn toolbar(app: &App) -> Element<'_, Message> {
    let mut bar = row![].spacing(8).padding(8);

    // Connection buttons
    if !app.connected {
        bar = bar.push(
            button(text("Connect (Mock)").size(12))
                .on_press(Message::ConnectMock)
        );
        bar = bar.push(
            button(text("Connect...").size(12))
                .on_press(Message::ShowConnectionDialog)
        );
    } else {
        bar = bar.push(
            button(text("Disconnect").size(12))
                .on_press(Message::Disconnect)
        );
        bar = bar.push(
            button(text("Scan Nodes").size(12))
                .on_press(Message::ScanNodes)
        );
    }

    bar = bar.push(separator());

    // NMT quick actions
    if app.connected {
        bar = bar.push(
            button(text("Start All").size(12))
                .on_press(Message::NmtStartAll)
        );
        bar = bar.push(
            button(text("Stop All").size(12))
                .on_press(Message::NmtStopAll)
        );
        bar = bar.push(
            button(text("Reset All").size(12))
                .on_press(Message::NmtResetAll)
        );
    }

    bar = bar.push(separator());

    // Bitrate selector
    bar = bar.push(text("Bitrate:").size(12));
    for br in [125000u32, 250000, 500000, 1000000] {
        let label = format!("{}k", br / 1000);
        let is_selected = app.toolbar_bitrate == br;
        let btn = if is_selected {
            button(text(format!("[{}]", label)).size(11))
                .on_press(Message::BitrateChanged(br))
        } else {
            button(text(label).size(11))
                .on_press(Message::BitrateChanged(br))
        };
        bar = bar.push(btn);
    }

    bar = bar.push(separator());

    // Pause/Resume
    let pause_text = if app.paused { "▶ Resume" } else { "⏸ Pause" };
    bar = bar.push(
        button(text(pause_text).size(12))
            .on_press(Message::TogglePause)
    );

    // Log actions
    bar = bar.push(
        button(text("Clear").size(12))
            .on_press(Message::ClearLog)
    );
    bar = bar.push(
        button(text("Export").size(12))
            .on_press(Message::ExportLog)
    );
    bar = bar.push(
        button(text("Import").size(12))
            .on_press(Message::ImportLog)
    );

    bar = bar.push(separator());

    // Detail panel toggle
    let detail_text = if app.detail_collapsed { "◀ Show Detail" } else { "▶ Hide Detail" };
    bar = bar.push(
        button(text(detail_text).size(12))
            .on_press(Message::ToggleDetailPanel)
    );

    // Status indicator (right-aligned)
    bar = bar.push(
        container(text(if app.paused { "⏸ PAUSED" } else { "" }).size(11))
            .width(Length::Fill)
    );

    container(bar)
        .width(Length::Fill)
        .height(48)
        .into()
}

fn separator() -> Element<'static, Message> {
    text("│").size(12).into()
}
