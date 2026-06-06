//! View rendering functions.

mod toolbar;
mod node_panel;
mod statusbar;
mod detail_panel;
mod dialog;
pub mod can;
pub mod canopen;

// Re-export main views
pub use toolbar::toolbar;
pub use node_panel::node_panel;
pub use statusbar::statusbar;
pub use detail_panel::detail_panel;
pub use dialog::connection_dialog;

// Re-export CAN views
pub use can::frame_monitor::frame_monitor;
pub use can::bus_statistics::bus_statistics;
pub use can::error_frames::error_frames;

// Re-export CANOpen views
pub use canopen::network::network_management;
pub use canopen::sdo::sdo_client;
pub use canopen::pdo::pdo_monitor;
pub use canopen::ds402::ds402_control;
pub use canopen::emcy::emcy_log;
pub use canopen::heartbeat::heartbeat_monitor;
pub use canopen::sync::sync_management;

use iced::widget::{column, horizontal_rule, row, text};
use iced::{Element, Length};
use crate::state::{App, Message, Tab, PrimaryTab};

/// Main content view with tab switching.
pub fn main_content(app: &App) -> Element<'_, Message> {
    let primary = app.current_tab.primary();
    let tabs = Tab::for_primary(primary);

    // Primary tabs
    let mut primary_tabs = row![].spacing(4);
    for &p in PrimaryTab::all() {
        let label = p.name();
        let is_selected = primary == p;
        let btn = if is_selected {
            crate::helpers::button_primary(label)
                .on_press(Message::SwitchPrimary(p))
        } else {
            crate::helpers::button_secondary(label)
                .on_press(Message::SwitchPrimary(p))
        };
        primary_tabs = primary_tabs.push(btn);
    }

    // Secondary tabs
    let mut secondary_tabs = row![].spacing(2);
    for &tab in tabs {
        let is_selected = app.current_tab == tab;
        let btn = if is_selected {
            crate::helpers::button_primary(tab.name())
                .on_press(Message::SwitchTab(tab))
        } else {
            crate::helpers::button_secondary(tab.name())
                .on_press(Message::SwitchTab(tab))
        };
        secondary_tabs = secondary_tabs.push(btn);
    }

    // Tab description
    let description = text(app.current_tab.description())
        .size(10)
        .style(|_theme| iced::widget::text::Style {
            color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        });

    // Content
    let content = match app.current_tab {
        Tab::FrameMonitor => frame_monitor(app),
        Tab::BusStatistics => bus_statistics(app),
        Tab::ErrorFrames => error_frames(app),
        Tab::NetworkManagement => network_management(app),
        Tab::SdoClient => sdo_client(app),
        Tab::PdoMonitor => pdo_monitor(app),
        Tab::Ds402Control => ds402_control(app),
        Tab::EmcyLog => emcy_log(app),
        Tab::HeartbeatMonitor => heartbeat_monitor(app),
        Tab::SyncManagement => sync_management(app),
    };

    column![
        primary_tabs,
        secondary_tabs,
        description,
        horizontal_rule(1),
        content,
    ]
    .spacing(2)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
