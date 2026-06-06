//! Helper functions for the GUI.

use iced::widget::{button, text};
use iced::Element;

use crate::state::Message;

/// Create a primary (highlighted) button.
pub fn button_primary(label: &str) -> iced::widget::Button<'_, Message> {
    button(text(label).size(12))
}

/// Create a secondary (normal) button.
pub fn button_secondary(label: &str) -> iced::widget::Button<'_, Message> {
    button(text(label).size(11))
}

/// Create a small button.
pub fn button_small(label: &str) -> iced::widget::Button<'_, Message> {
    button(text(label).size(10))
}

/// Format bytes as hex string.
pub fn bytes_to_hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
}

/// Parse hex string to bytes.
pub fn parse_hex_bytes(s: &str) -> Vec<u8> {
    s.split_whitespace()
        .filter_map(|b| u8::from_str_radix(b, 16).ok())
        .collect()
}

/// Format timestamp as seconds.milliseconds.
pub fn format_timestamp(ms: u64) -> String {
    let sec = ms / 1000;
    let millis = ms % 1000;
    format!("{:3}.{:03}", sec, millis)
}

/// Create a separator element.
pub fn separator() -> Element<'static, Message> {
    text("│").size(12).into()
}

/// Create a small separator element.
pub fn separator_small() -> Element<'static, Message> {
    text("│").size(10).into()
}

/// Format a number with thousand separators.
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Truncate string to max length with ellipsis.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
