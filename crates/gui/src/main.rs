//! OpenCAN GUI Application

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length, Theme};

mod state;
mod views;

use state::{App, Message, View};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("OpenCAN", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

impl App {
    fn new() -> (Self, iced::Task<Message>) {
        (Self::default(), iced::Task::none())
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::SwitchView(view) => {
                self.current_view = view;
                iced::Task::none()
            }
            Message::NodeSelected(node_id) => {
                self.selected_node = Some(node_id);
                self.current_view = View::NodeDetail;
                iced::Task::none()
            }
            Message::SdoIndexChanged(idx) => {
                self.sdo_index = idx;
                iced::Task::none()
            }
            Message::SdoSubindexChanged(sub) => {
                self.sdo_subindex = sub;
                iced::Task::none()
            }
            Message::SdoRead => {
                // TODO: Implement SDO read
                self.status_message = "SDO Read not yet connected".to_string();
                iced::Task::none()
            }
            Message::SdoWrite => {
                // TODO: Implement SDO write
                self.status_message = "SDO Write not yet connected".to_string();
                iced::Task::none()
            }
            Message::SdoValueChanged(val) => {
                self.sdo_value = val;
                iced::Task::none()
            }
            Message::NmtStartNode(id) => {
                self.status_message = format!("NMT Start Node {}", id);
                iced::Task::none()
            }
            Message::NmtStopNode(id) => {
                self.status_message = format!("NMT Stop Node {}", id);
                iced::Task::none()
            }
            Message::NmtResetNode(id) => {
                self.status_message = format!("NMT Reset Node {}", id);
                iced::Task::none()
            }
            Message::Ds402Enable(id) => {
                self.status_message = format!("DS402 Enable Node {}", id);
                iced::Task::none()
            }
            Message::Ds402FaultReset(id) => {
                self.status_message = format!("DS402 Fault Reset Node {}", id);
                iced::Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let content = self.view_content();
        let statusbar = self.view_statusbar();

        let main_content = row![sidebar, horizontal_rule(1), content]
            .width(Length::Fill)
            .height(Length::Fill);

        column![main_content, statusbar]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let mut sidebar = column![text("OpenCAN").size(20)].spacing(8).padding(10);

        sidebar = sidebar.push(text("Connection: Not Connected").size(12));
        sidebar = sidebar.push(horizontal_rule(1));

        sidebar = sidebar.push(text("Nodes:").size(14));
        for node in &self.nodes {
            let node_btn = button(text(format!("Node {}", node.node_id)).size(14))
                .on_press(Message::NodeSelected(node.node_id))
                .width(Length::Fill);
            sidebar = sidebar.push(node_btn);
        }

        sidebar = sidebar.push(horizontal_rule(1));

        // View buttons
        let views = [
            (View::NetworkOverview, "Network Overview"),
            (View::PdoMonitor, "PDO Monitor"),
            (View::CanLog, "CAN Log"),
        ];

        for (view, label) in views {
            let btn = button(text(label).size(14))
                .on_press(Message::SwitchView(view))
                .width(Length::Fill);
            sidebar = sidebar.push(btn);
        }

        container(scrollable(sidebar))
            .width(200)
            .height(Length::Fill)
            .into()
    }

    fn view_content(&self) -> Element<'_, Message> {
        match self.current_view {
            View::NetworkOverview => views::network_overview(self),
            View::NodeDetail => views::node_detail(self),
            View::Ds402 => views::ds402_panel(self),
            View::PdoMonitor => views::pdo_monitor(self),
            View::CanLog => views::can_log(self),
        }
    }

    fn view_statusbar(&self) -> Element<'_, Message> {
        container(
            text(&self.status_message).size(12)
        )
        .width(Length::Fill)
        .padding(4)
        .into()
    }
}
