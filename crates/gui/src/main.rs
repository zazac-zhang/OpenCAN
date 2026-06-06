//! OpenCAN GUI Application

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length, Subscription, Theme, time};

mod state;
mod views;
mod backend;

use state::{App, Message, View, NmtState, NodeState, LogEntry};
use backend::{BackendCommand, BackendEvent};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::application("OpenCAN", App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .run_with(App::new)
}

impl App {
    fn new() -> (Self, iced::Task<Message>) {
        (Self::default(), iced::Task::none())
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.backend.is_some() {
            // Poll backend events every 50ms
            time::every(std::time::Duration::from_millis(50))
                .map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            // === Tick: poll backend events ===
            Message::Tick => {
                let mut events = Vec::new();
                if let Some(ref mut backend) = self.backend {
                    while let Some(event) = backend.try_recv() {
                        events.push(event);
                    }
                }
                for event in events {
                    self.handle_backend_event(event);
                }
                iced::Task::none()
            }

            // === Connection ===
            Message::ConnectMock => {
                let backend = backend::Backend::new_mock();
                self.backend = Some(backend);
                self.connected = true;
                self.status_message = "Connected (Mock)".to_string();
                iced::Task::none()
            }
            Message::ShowConnectionDialog => {
                self.connection_dialog.visible = true;
                iced::Task::none()
            }
            Message::HideConnectionDialog => {
                self.connection_dialog.visible = false;
                iced::Task::none()
            }
            Message::ConnectionBackendChanged(backend) => {
                self.connection_dialog.selected_backend = backend;
                iced::Task::none()
            }
            Message::ConnectionChannelChanged(ch) => {
                self.connection_dialog.channel = ch;
                iced::Task::none()
            }
            Message::ConnectionBitrateChanged(br) => {
                self.connection_dialog.bitrate = br;
                iced::Task::none()
            }
            Message::ConnectionNodeIdChanged(id) => {
                self.connection_dialog.node_id = id;
                iced::Task::none()
            }
            Message::ConnectionConnect => {
                let dialog = &self.connection_dialog;
                let backend_type = dialog.selected_backend;
                match backend_type {
                    state::CanBackend::Mock => {
                        let backend = backend::Backend::new_mock();
                        self.backend = Some(backend);
                        self.connected = true;
                        self.status_message = "Connected (Mock)".to_string();
                    }
                    state::CanBackend::SocketCan => {
                        // Real SocketCAN connection would go here
                        self.status_message = "SocketCAN connection not available on this platform".to_string();
                    }
                    _ => {
                        self.status_message = format!("{} backend not yet implemented", backend_type.name());
                    }
                }
                self.connection_dialog.visible = false;
                iced::Task::none()
            }
            Message::Disconnect => {
                if let Some(ref backend) = self.backend {
                    backend.send(BackendCommand::Disconnect);
                }
                self.backend = None;
                self.connected = false;
                self.nodes.clear();
                self.status_message = "Disconnected".to_string();
                iced::Task::none()
            }
            Message::ScanNodes => {
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::ScanNodes { respond: tx });
                    self.status_message = "Scanning nodes...".to_string();
                }
                iced::Task::none()
            }

            // === View ===
            Message::SwitchView(view) => {
                self.current_view = view;
                iced::Task::none()
            }
            Message::NodeSelected(node_id) => {
                self.selected_node = Some(node_id);
                self.current_view = View::NodeDetail;
                iced::Task::none()
            }

            // === NMT ===
            Message::NmtStartNode(id) => {
                if let Some(ref backend) = self.backend {
                    backend.send(BackendCommand::NmtStart(id));
                }
                iced::Task::none()
            }
            Message::NmtStopNode(id) => {
                if let Some(ref backend) = self.backend {
                    backend.send(BackendCommand::NmtStop(id));
                }
                iced::Task::none()
            }
            Message::NmtResetNode(id) => {
                if let Some(ref backend) = self.backend {
                    backend.send(BackendCommand::NmtReset(id));
                }
                iced::Task::none()
            }

            // === SDO ===
            Message::SdoIndexChanged(idx) => {
                self.sdo_index = idx;
                iced::Task::none()
            }
            Message::SdoSubindexChanged(sub) => {
                self.sdo_subindex = sub;
                iced::Task::none()
            }
            Message::SdoValueChanged(val) => {
                self.sdo_value = val;
                iced::Task::none()
            }
            Message::SdoRead => {
                let node_id = self.selected_node.unwrap_or(1);
                let index = u16::from_str_radix(self.sdo_index.trim_start_matches("0x"), 16).unwrap_or(0);
                let subindex: u8 = self.sdo_subindex.parse().unwrap_or(0);
                self.status_message = format!("SDO read {:04X}:{:02X} from node {}...", index, subindex, node_id);
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::SdoUpload { node_id, index, subindex, respond: tx });
                }
                iced::Task::none()
            }
            Message::SdoWrite => {
                let node_id = self.selected_node.unwrap_or(1);
                let index = u16::from_str_radix(self.sdo_index.trim_start_matches("0x"), 16).unwrap_or(0);
                let subindex: u8 = self.sdo_subindex.parse().unwrap_or(0);
                let value = parse_hex_bytes(&self.sdo_value);
                self.status_message = format!("SDO write {:04X}:{:02X} to node {}...", index, subindex, node_id);
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::SdoDownload { node_id, index, subindex, value, respond: tx });
                }
                iced::Task::none()
            }

            // === DS402 ===
            Message::Ds402Enable(id) => {
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402Enable { node_id: id, respond: tx });
                    self.status_message = format!("DS402 enabling node {}...", id);
                }
                iced::Task::none()
            }
            Message::Ds402FaultReset(id) => {
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402FaultReset { node_id: id, respond: tx });
                }
                iced::Task::none()
            }
            Message::Ds402ReadState(id) => {
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402ReadState { node_id: id, respond: tx });
                }
                iced::Task::none()
            }
            Message::Ds402TargetPositionChanged(val) => {
                self.ds402_state.target_position = val;
                iced::Task::none()
            }
            Message::Ds402TargetVelocityChanged(val) => {
                self.ds402_state.target_velocity = val;
                iced::Task::none()
            }
            Message::Ds402SetPosition(id) => {
                let pos: i32 = self.ds402_state.target_position.parse().unwrap_or(0);
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402SetPosition { node_id: id, position: pos, respond: tx });
                }
                iced::Task::none()
            }
            Message::Ds402SetVelocity(id) => {
                let vel: i32 = self.ds402_state.target_velocity.parse().unwrap_or(0);
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402SetVelocity { node_id: id, velocity: vel, respond: tx });
                }
                iced::Task::none()
            }
            Message::Ds402ReadPosition(id) => {
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402ReadPosition { node_id: id, respond: tx });
                }
                iced::Task::none()
            }
            Message::Ds402ReadVelocity(id) => {
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402ReadVelocity { node_id: id, respond: tx });
                }
                iced::Task::none()
            }
        }
    }

    fn handle_backend_event(&mut self, event: BackendEvent) {
        match event {
            BackendEvent::ScanResult(nodes) => {
                self.nodes.clear();
                for &id in &nodes {
                    let mut node = NodeState::new(id);
                    node.nmt_state = NmtState::PreOperational;
                    self.nodes.push(node);
                }
                self.status_message = format!("Found {} nodes: {:?}", nodes.len(), nodes);
            }
            BackendEvent::SdoResult { node_id, index, subindex, result } => {
                match result {
                    Ok(data) => {
                        let hex = bytes_to_hex(&data);
                        self.sdo_value = hex.clone();
                        // Cache in node
                        if let Some(node) = self.nodes.iter_mut().find(|n| n.node_id == node_id) {
                            node.od_cache.insert((index, subindex), hex.clone());
                        }
                        self.status_message = format!("SDO {:04X}:{:02X} = {}", index, subindex, hex);
                        self.can_log.push(LogEntry {
                            timestamp_ms: 0,
                            cob_id: 0x580 + node_id as u16,
                            data: {
                                let mut d = [0u8; 8];
                                let len = data.len().min(8);
                                d[..len].copy_from_slice(&data[..len]);
                                d
                            },
                            description: format!("SDO upload response {:04X}:{:02X}", index, subindex),
                        });
                    }
                    Err(e) => {
                        self.status_message = format!("SDO error: {}", e);
                    }
                }
            }
            BackendEvent::NmtStateChanged { node_id, state } => {
                if let Some(node) = self.nodes.iter_mut().find(|n| n.node_id == node_id) {
                    node.nmt_state = match state.as_str() {
                        "Operational" => NmtState::Operational,
                        "Stopped" => NmtState::Stopped,
                        "PreOperational" => NmtState::PreOperational,
                        "BootUp" => NmtState::BootUp,
                        _ => NmtState::Unknown,
                    };
                }
                self.status_message = format!("Node {} → {}", node_id, state);
            }
            BackendEvent::Ds402StateResult { node_id, state, status_word } => {
                if let Some(node) = self.nodes.iter_mut().find(|n| n.node_id == node_id) {
                    node.ds402.state = state.clone();
                    node.ds402.status_word = status_word;
                }
                self.status_message = format!("DS402 Node {}: {} (0x{:04X})", node_id, state, status_word);
            }
            BackendEvent::Ds402PositionResult { node_id, position } => {
                if let Some(node) = self.nodes.iter_mut().find(|n| n.node_id == node_id) {
                    node.ds402.actual_position = position;
                }
            }
            BackendEvent::Ds402VelocityResult { node_id, velocity } => {
                if let Some(node) = self.nodes.iter_mut().find(|n| n.node_id == node_id) {
                    node.ds402.actual_velocity = velocity;
                }
            }
            BackendEvent::FrameReceived { cob_id, data, timestamp_ms } => {
                self.can_log.push(LogEntry {
                    timestamp_ms,
                    cob_id,
                    data,
                    description: String::new(),
                });
            }
            BackendEvent::Connected(info) => {
                self.connected = true;
                self.status_message = format!("Connected: {}", info);
            }
            BackendEvent::Disconnected => {
                self.connected = false;
                self.status_message = "Disconnected".to_string();
            }
            BackendEvent::Error(e) => {
                self.status_message = format!("Error: {}", e);
            }
            BackendEvent::HeartbeatChanged { node_id, alive } => {
                let state = if alive { "alive" } else { "lost" };
                self.status_message = format!("Node {} heartbeat {}", node_id, state);
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

        let base = column![main_content, statusbar]
            .width(Length::Fill)
            .height(Length::Fill);

        // Show connection dialog as an overlay
        if self.connection_dialog.visible {
            let dialog = views::connection_dialog(&self.connection_dialog);
            // Stack the dialog on top of the main content
            iced::widget::stack![
                base,
                container(dialog)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .width(Length::Fill)
                    .height(Length::Fill)
            ].into()
        } else {
            base.into()
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let mut sidebar = column![text("OpenCAN").size(20)].spacing(6).padding(8);

        // Connection status
        let conn_text = if self.connected { "Connected" } else { "Disconnected" };
        sidebar = sidebar.push(text(conn_text).size(12));

        // Connection buttons
        if !self.connected {
            sidebar = sidebar.push(
                button(text("Connect (Mock)").size(12))
                    .on_press(Message::ConnectMock)
                    .width(Length::Fill)
            );
            sidebar = sidebar.push(
                button(text("Connect...").size(12))
                    .on_press(Message::ShowConnectionDialog)
                    .width(Length::Fill)
            );
        } else {
            sidebar = sidebar.push(
                button(text("Scan Nodes").size(12))
                    .on_press(Message::ScanNodes)
                    .width(Length::Fill)
            );
            sidebar = sidebar.push(
                button(text("Disconnect").size(12))
                    .on_press(Message::Disconnect)
                    .width(Length::Fill)
            );
        }

        sidebar = sidebar.push(horizontal_rule(1));

        // Node list
        sidebar = sidebar.push(text("Nodes:").size(14));
        if self.nodes.is_empty() {
            sidebar = sidebar.push(text("  (none)").size(11));
        }
        for node in &self.nodes {
            let label = format!("Node {} [{}]", node.node_id, node.nmt_state.as_str());
            let btn = button(text(label).size(12))
                .on_press(Message::NodeSelected(node.node_id))
                .width(Length::Fill);
            sidebar = sidebar.push(btn);
        }

        sidebar = sidebar.push(horizontal_rule(1));

        // View navigation
        let views = [
            (View::NetworkOverview, "Network"),
            (View::PdoMonitor, "PDO Monitor"),
            (View::CanLog, "CAN Log"),
        ];
        for (view, label) in views {
            let btn = button(text(label).size(12))
                .on_press(Message::SwitchView(view))
                .width(Length::Fill);
            sidebar = sidebar.push(btn);
        }

        container(scrollable(sidebar))
            .width(180)
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
        container(text(&self.status_message).size(11))
            .width(Length::Fill)
            .padding(4)
            .into()
    }
}

fn bytes_to_hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
}

fn parse_hex_bytes(s: &str) -> Vec<u8> {
    s.split_whitespace()
        .filter_map(|b| u8::from_str_radix(b, 16).ok())
        .collect()
}
