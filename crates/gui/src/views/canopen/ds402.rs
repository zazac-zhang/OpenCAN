//! DS402 control view.

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Element, Length};
use crate::state::{App, Message, Ds402Mode};

/// DS402 control view.
pub fn ds402_control(app: &App) -> Element<'_, Message> {
    let node_id = app.selected_node.unwrap_or(1);
    let node = app.get_node(node_id);
    let ds402 = node.map(|n| &n.ds402);

    let mut content = column![].spacing(8).padding(10);

    // Header
    content = content.push(text(format!("DS402 Control — Node {}", node_id)).size(16));
    content = content.push(horizontal_rule(1));

    // State machine
    content = content.push(text("State Machine:").size(14));
    let state_str = ds402.map(|d| d.state.as_str()).unwrap_or("--");
    let status_word = ds402.map(|d| d.status_word).unwrap_or(0);

    content = content.push(
        row![
            text(format!("Current State: {}", state_str)).size(12),
            text(format!("Status Word: 0x{:04X}", status_word)).size(12),
        ].spacing(16)
    );

    // Status word bits
    content = content.push(text("Status Word Bits:").size(12));
    if let Some(ds402) = ds402 {
        for (mask, name, active) in ds402.status_bits() {
            let marker = if active { "●" } else { "○" };
            content = content.push(
                text(format!("  {} {} (0x{:04X})", marker, name, mask)).size(10)
            );
        }
    } else {
        content = content.push(text("  (no data)").size(10));
    }

    // State control buttons
    content = content.push(
        row![
            button(text("Read State").size(11)).on_press(Message::Ds402ReadState(node_id)),
            button(text("Enable").size(11)).on_press(Message::Ds402Enable(node_id)),
            button(text("Fault Reset").size(11)).on_press(Message::Ds402FaultReset(node_id)),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // Operation mode
    content = content.push(text("Operation Mode:").size(14));
    let mut mode_row = row![].spacing(4);
    for &mode in Ds402Mode::all() {
        let is_selected = app.ds402_state.selected_mode == mode;
        let btn = if is_selected {
            button(text(format!("[{}]", mode.short_name())).size(10))
                .on_press(Message::Ds402ModeChanged(mode))
        } else {
            button(text(mode.short_name()).size(10))
                .on_press(Message::Ds402ModeChanged(mode))
        };
        mode_row = mode_row.push(btn);
    }
    content = content.push(mode_row);
    content = content.push(
        text(format!("Selected: {}", app.ds402_state.selected_mode.name())).size(11)
    );

    content = content.push(horizontal_rule(1));

    // Position control
    content = content.push(text("Position Control:").size(14));
    content = content.push(
        row![
            text("Target:").size(11),
            text_input("0", &app.ds402_state.target_position)
                .on_input(Message::Ds402TargetPositionChanged)
                .width(120),
            button(text("Set").size(11)).on_press(Message::Ds402SetPosition(node_id)),
            button(text("Read Actual").size(11)).on_press(Message::Ds402ReadPosition(node_id)),
        ].spacing(4)
    );

    let actual_pos = ds402.map(|d| d.actual_position).unwrap_or(0);
    let target_pos = app.ds402_state.parsed_position();
    content = content.push(
        row![
            text(format!("  Actual: {}", actual_pos)).size(11),
            text(format!("  Target: {}", target_pos)).size(11),
            text(format!("  Error: {}", target_pos - actual_pos)).size(11),
        ].spacing(8)
    );

    // Position sparkline
    if let Some(ds402) = ds402 {
        if !ds402.position_history.is_empty() {
            content = content.push(text("Position History:").size(10));
            content = content.push(render_sparkline(&ds402.position_history, "pos"));
        }
    }

    content = content.push(horizontal_rule(1));

    // Velocity control
    content = content.push(text("Velocity Control:").size(14));
    content = content.push(
        row![
            text("Target:").size(11),
            text_input("0", &app.ds402_state.target_velocity)
                .on_input(Message::Ds402TargetVelocityChanged)
                .width(120),
            button(text("Set").size(11)).on_press(Message::Ds402SetVelocity(node_id)),
            button(text("Read Actual").size(11)).on_press(Message::Ds402ReadVelocity(node_id)),
        ].spacing(4)
    );

    let actual_vel = ds402.map(|d| d.actual_velocity).unwrap_or(0);
    let target_vel = app.ds402_state.parsed_velocity();
    content = content.push(
        row![
            text(format!("  Actual: {}", actual_vel)).size(11),
            text(format!("  Target: {}", target_vel)).size(11),
            text(format!("  Error: {}", target_vel - actual_vel)).size(11),
        ].spacing(8)
    );

    // Velocity sparkline
    if let Some(ds402) = ds402 {
        if !ds402.velocity_history.is_empty() {
            content = content.push(text("Velocity History:").size(10));
            content = content.push(render_sparkline(&ds402.velocity_history, "vel"));
        }
    }

    content = content.push(horizontal_rule(1));

    // Torque control
    content = content.push(text("Torque Control:").size(14));
    content = content.push(
        row![
            text("Target:").size(11),
            text_input("0", &app.ds402_state.target_torque)
                .on_input(Message::Ds402TargetTorqueChanged)
                .width(120),
            button(text("Set").size(11)).on_press(Message::Ds402SetTorque(node_id)),
            button(text("Read Actual").size(11)).on_press(Message::Ds402ReadTorque(node_id)),
        ].spacing(4)
    );

    let actual_torque = ds402.map(|d| d.actual_torque).unwrap_or(0);
    let target_torque = app.ds402_state.parsed_torque();
    content = content.push(
        row![
            text(format!("  Actual: {}", actual_torque)).size(11),
            text(format!("  Target: {}", target_torque)).size(11),
        ].spacing(8)
    );

    // Torque sparkline
    if let Some(ds402) = ds402 {
        if !ds402.torque_history.is_empty() {
            content = content.push(text("Torque History:").size(10));
            content = content.push(render_sparkline_i16(&ds402.torque_history, "torque"));
        }
    }

    content = content.push(horizontal_rule(1));

    // Trend Chart
    content = content.push(text("Trend Chart:").size(14));
    content = content.push(
        row![
            button(text("Position").size(10)).on_press(Message::Ds402ToggleRawValues),
            button(text("Velocity").size(10)).on_press(Message::Ds402ToggleRawValues),
            button(text("Torque").size(10)).on_press(Message::Ds402ToggleRawValues),
        ].spacing(4)
    );
    content = content.push(
        crate::views::canopen::trend_chart::trend_chart(&app.trend_chart)
            .map(|_| Message::Tick) // Placeholder message
    );

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render a sparkline for i32 data.
fn render_sparkline<'a>(data: &'a [i32], label: &str) -> Element<'a, Message> {
    if data.is_empty() {
        return text("").into();
    }

    let min = *data.iter().min().unwrap_or(&0);
    let max = *data.iter().max().unwrap_or(&0);
    let range = max - min;

    // Take last 50 points
    let display_data: Vec<i32> = if data.len() > 50 {
        data[data.len() - 50..].to_vec()
    } else {
        data.to_vec()
    };

    // Normalize to 0-7
    let normalized: Vec<u8> = if range == 0 {
        vec![4; display_data.len()]
    } else {
        display_data.iter()
            .map(|&v| ((v - min) * 7 / range) as u8)
            .collect()
    };

    // Build sparkline string
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let sparkline: String = normalized.iter()
        .map(|&n| chars[n.min(7) as usize])
        .collect();

    column![
        text(format!("{}: {} - {}", label, min, max)).size(9),
        text(sparkline).size(14),
    ]
    .into()
}

/// Render a sparkline for i16 data.
fn render_sparkline_i16<'a>(data: &'a [i16], label: &str) -> Element<'a, Message> {
    if data.is_empty() {
        return text("").into();
    }

    let min = *data.iter().min().unwrap_or(&0) as i32;
    let max = *data.iter().max().unwrap_or(&0) as i32;
    let range = max - min;

    // Take last 50 points
    let display_data: Vec<i32> = if data.len() > 50 {
        data[data.len() - 50..].iter().map(|&v| v as i32).collect()
    } else {
        data.iter().map(|&v| v as i32).collect()
    };

    // Normalize to 0-7
    let normalized: Vec<u8> = if range == 0 {
        vec![4; display_data.len()]
    } else {
        display_data.iter()
            .map(|&v| ((v - min) * 7 / range) as u8)
            .collect()
    };

    // Build sparkline string
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let sparkline: String = normalized.iter()
        .map(|&n| chars[n.min(7) as usize])
        .collect();

    column![
        text(format!("{}: {} - {}", label, min, max)).size(9),
        text(sparkline).size(14),
    ]
    .into()
}
