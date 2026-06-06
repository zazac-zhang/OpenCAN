//! Sync management view.

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Element, Length};
use crate::state::{App, Message};

/// Sync management view.
pub fn sync_management(app: &App) -> Element<'_, Message> {
    let sync = &app.sync_status;

    let mut content = column![].spacing(8).padding(10);

    // Header
    content = content.push(text("SYNC Management").size(16));
    content = content.push(horizontal_rule(1));

    // SYNC Producer
    content = content.push(text("SYNC Producer:").size(14));
    content = content.push(
        row![
            text(format!("Status: {}", if sync.producer_enabled { "Enabled" } else { "Disabled" })).size(12),
        ].spacing(8)
    );

    // Period configuration
    content = content.push(
        row![
            text("Period (μs):").size(11),
            text_input("1000", &sync.producer_period_us.to_string())
                .on_input(Message::SyncPeriodChanged)
                .width(100),
            text(format!("({:.1} ms)", sync.period_ms())).size(10),
            text(format!("Freq: {}", sync.frequency_str())).size(10),
        ].spacing(4)
    );

    // Control buttons
    if sync.producer_enabled {
        content = content.push(
            button(text("Stop SYNC Producer").size(12))
                .on_press(Message::SyncStopProducer)
        );
    } else {
        content = content.push(
            button(text("Start SYNC Producer").size(12))
                .on_press(Message::SyncStartProducer(sync.producer_period_us.to_string()))
        );
    }

    content = content.push(horizontal_rule(1));

    // SYNC Consumer
    content = content.push(text("SYNC Consumer:").size(14));
    content = content.push(
        row![
            text(format!("Registered Consumers: {}", sync.consumer_count)).size(12),
        ].spacing(8)
    );

    let last_sync = sync.last_sync_ms
        .map(|t| format!("{}ms", t))
        .unwrap_or_else(|| "-".to_string());
    content = content.push(
        row![
            text(format!("Last SYNC: {}", last_sync)).size(12),
            text(format!("Total SYNCs: {}", sync.sync_counter)).size(12),
        ].spacing(8)
    );

    if sync.sync_overrun {
        content = content.push(
            text("⚠ SYNC overrun detected").size(12)
        );
    }

    content = content.push(horizontal_rule(1));

    // SYNC Configuration Info
    content = content.push(text("SYNC Configuration:").size(14));
    content = content.push(
        text("SYNC is used to synchronize PDO communication across nodes.").size(11)
    );
    content = content.push(
        text("The SYNC producer generates periodic SYNC messages.").size(11)
    );
    content = content.push(
        text("Nodes configured as SYNC consumers will process PDOs on SYNC.").size(11)
    );

    content = content.push(horizontal_rule(1));

    // Common SYNC COB-ID
    content = content.push(text("Standard SYNC COB-ID: 0x080").size(12));
    content = content.push(
        text("This is the default COB-ID for SYNC messages in CANOpen.").size(11)
    );

    // Common SYNC periods
    content = content.push(text("Common SYNC Periods:").size(12));
    content = content.push(
        row![
            button(text("1ms").size(10)).on_press(Message::SyncStartProducer("1000".to_string())),
            button(text("10ms").size(10)).on_press(Message::SyncStartProducer("10000".to_string())),
            button(text("100ms").size(10)).on_press(Message::SyncStartProducer("100000".to_string())),
        ].spacing(4)
    );

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
