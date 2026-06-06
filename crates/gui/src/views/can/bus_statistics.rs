//! Bus statistics view.

use iced::widget::{column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length};
use crate::state::{App, Message};

/// Bus statistics view.
pub fn bus_statistics(app: &App) -> Element<'_, Message> {
    let stats = &app.bus_stats;

    let mut content = column![].spacing(10).padding(10);

    // Header
    content = content.push(text("Bus Statistics").size(16));
    content = content.push(horizontal_rule(1));

    // Bus load
    content = content.push(text("Bus Load").size(14));
    content = content.push(
        row![
            text(format!("Current: {:.1}%", stats.bus_load_percent)).size(12),
            text(format!("Peak: {:.1}%", stats.peak_bus_load)).size(12),
            text(format!("Average: {:.1}%", stats.avg_bus_load())).size(12),
        ].spacing(16)
    );

    // Load bar visualization
    let load_bar = format!("[{}{}]",
        "█".repeat((stats.bus_load_percent / 5.0) as usize),
        "░".repeat(20 - (stats.bus_load_percent / 5.0) as usize)
    );
    content = content.push(text(load_bar).size(12));

    content = content.push(horizontal_rule(1));

    // Frame rate
    content = content.push(text("Frame Rate").size(14));
    content = content.push(
        row![
            text(format!("Current: {} fps", stats.frame_rate)).size(12),
            text(format!("Peak: {} fps", stats.peak_frame_rate)).size(12),
            text(format!("Average: {} fps", stats.avg_frame_rate())).size(12),
        ].spacing(16)
    );

    content = content.push(horizontal_rule(1));

    // Frame counts
    content = content.push(text("Frame Counts").size(14));
    content = content.push(
        row![
            text(format!("Total: {}", stats.frame_count)).size(12),
            text(format!("TX: {}", stats.tx_frames)).size(12),
            text(format!("RX: {}", stats.rx_frames)).size(12),
        ].spacing(16)
    );

    content = content.push(horizontal_rule(1));

    // Error counts
    content = content.push(text("Errors").size(14));
    content = content.push(
        row![
            text(format!("TX Errors: {}", stats.tx_errors)).size(12),
            text(format!("RX Errors: {}", stats.rx_errors)).size(12),
            text(format!("Error Frames: {}", stats.error_frame_count)).size(12),
        ].spacing(16)
    );

    if let Some(ref error) = stats.last_error {
        content = content.push(text(format!("Last Error: {}", error)).size(11));
    }

    content = content.push(horizontal_rule(1));

    // Bus state
    content = content.push(text("Bus State").size(14));
    content = content.push(text(format!("State: {}", stats.bus_state.as_str())).size(12));

    // Bitrate
    content = content.push(text(format!("Bitrate: {} kbps", stats.bitrate / 1000)).size(12));

    // History visualization
    if !stats.frame_rate_history.is_empty() {
        content = content.push(horizontal_rule(1));
        content = content.push(text("Frame Rate History (last 60s)").size(14));

        // Simple sparkline
        let sparkline = render_sparkline(&stats.frame_rate_history);
        content = content.push(sparkline);
    }

    if !stats.bus_load_history.is_empty() {
        content = content.push(text("Bus Load History (last 60s)").size(14));

        let sparkline = render_load_sparkline(&stats.bus_load_history);
        content = content.push(sparkline);
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render a simple sparkline for frame rate.
fn render_sparkline(data: &[u32]) -> iced::Element<'_, Message> {
    if data.is_empty() {
        return text("").into();
    }

    let max = *data.iter().max().unwrap_or(&1);
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let sparkline: String = data.iter()
        .map(|&v| {
            let idx = if max == 0 { 0 } else { (v * 7 / max) as usize };
            chars[idx.min(7)]
        })
        .collect();

    column![
        text(format!("Max: {} fps", max)).size(9),
        text(sparkline).size(14),
    ].into()
}

/// Render a simple sparkline for bus load.
fn render_load_sparkline(data: &[f32]) -> iced::Element<'_, Message> {
    if data.is_empty() {
        return text("").into();
    }

    let max = data.iter().cloned().fold(0.0f32, f32::max);
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let sparkline: String = data.iter()
        .map(|&v| {
            let idx = if max == 0.0 { 0 } else { (v * 7.0 / max) as usize };
            chars[idx.min(7)]
        })
        .collect();

    column![
        text(format!("Max: {:.1}%", max)).size(9),
        text(sparkline).size(14),
    ].into()
}
