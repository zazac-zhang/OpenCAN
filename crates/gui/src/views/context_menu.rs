//! Context menu component.

use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use crate::state::Message;

/// Context menu item.
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub message: Message,
    pub enabled: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, message: Message) -> Self {
        Self {
            label: label.into(),
            message,
            enabled: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Context menu state.
#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub visible: bool,
    pub items: Vec<MenuItem>,
    pub x: f32,
    pub y: f32,
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self {
            visible: false,
            items: Vec::new(),
            x: 0.0,
            y: 0.0,
        }
    }
}

impl ContextMenu {
    pub fn show(&mut self, items: Vec<MenuItem>, x: f32, y: f32) {
        self.items = items;
        self.x = x;
        self.y = y;
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.items.clear();
    }
}

/// Render a context menu.
pub fn context_menu(menu: &ContextMenu) -> Option<Element<'_, Message>> {
    if !menu.visible || menu.items.is_empty() {
        return None;
    }

    let mut content = column![].spacing(2);

    for item in &menu.items {
        let btn = if item.enabled {
            button(text(&item.label).size(12))
                .on_press(item.message.clone())
                .width(Length::Fill)
        } else {
            button(text(&item.label).size(12).style(|_theme| {
                iced::widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                }
            }))
            .width(Length::Fill)
        };
        content = content.push(btn);
    }

    Some(
        container(content)
            .padding(4)
            .width(150)
            .into()
    )
}

/// Create a node context menu.
pub fn node_context_menu(node_id: u8) -> Vec<MenuItem> {
    vec![
        MenuItem::new(
            format!("Start Node {}", node_id),
            Message::NmtStartNode(node_id),
        ),
        MenuItem::new(
            format!("Stop Node {}", node_id),
            Message::NmtStopNode(node_id),
        ),
        MenuItem::new(
            format!("Reset Node {}", node_id),
            Message::NmtResetNode(node_id),
        ),
        MenuItem::new(
            "View Details".to_string(),
            Message::NodeSelected(node_id),
        ),
        MenuItem::new(
            format!("Read PDO Mapping"),
            Message::ReadPdoMapping(node_id),
        ),
    ]
}

/// Create a frame context menu.
pub fn frame_context_menu(cob_id: u16) -> Vec<MenuItem> {
    vec![
        MenuItem::new(
            "Copy COB-ID".to_string(),
            Message::ShowAbout, // Placeholder
        ),
        MenuItem::new(
            "Copy Data".to_string(),
            Message::ShowAbout, // Placeholder
        ),
        MenuItem::new(
            format!("Filter by COB-ID {:03X}", cob_id),
            Message::ShowAbout, // Placeholder
        ),
    ]
}
