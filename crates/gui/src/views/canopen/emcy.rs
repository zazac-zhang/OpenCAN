//! EMCY log view.

use iced::widget::{column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length};
use crate::state::{App, Message};

/// EMCY log view.
pub fn emcy_log(app: &App) -> Element<'_, Message> {
    let mut content = column![].spacing(4).padding(10);

    // Header
    content = content.push(
        row![
            text("EMCY Log").size(16),
            text(format!("({} entries)", app.emcy_log.len())).size(12),
        ].spacing(8)
    );
    content = content.push(horizontal_rule(1));

    if app.emcy_log.is_empty() {
        content = content.push(text("No emergency errors recorded").size(12));
        content = content.push(
            text("EMCY messages will appear here when nodes report errors").size(11)
        );
    } else {
        // EMCY summary by category
        let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for emcy in &app.emcy_log {
            let category = emcy.error_category().to_string();
            *categories.entry(category).or_insert(0) += 1;
        }

        content = content.push(text("Error Summary:").size(14));
        for (category, count) in &categories {
            content = content.push(
                text(format!("  {}: {}", category, count)).size(11)
            );
        }

        content = content.push(horizontal_rule(1));

        // EMCY table
        content = content.push(text("Error Log:").size(14));

        // Table header
        content = content.push(
            row![
                text("Time").size(9).width(70),
                text("Node").size(9).width(30),
                text("Error Code").size(9).width(60),
                text("Error Reg").size(9).width(50),
                text("Description").size(9),
            ].spacing(2)
        );
        content = content.push(horizontal_rule(1));

        // Show latest EMCY entries
        for emcy in app.emcy_log.iter().rev().take(100) {
            content = content.push(
                row![
                    text(emcy.timestamp_str()).size(9).width(70),
                    text(format!("{}", emcy.node_id)).size(9).width(30),
                    text(format!("0x{:04X}", emcy.error_code)).size(9).width(60),
                    text(emcy.error_register_str()).size(9).width(50),
                    text(emcy.error_description()).size(9),
                ].spacing(2)
            );
        }
    }

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
