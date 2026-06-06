//! OpenCAN GUI Application

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length, Subscription, Theme, time};

mod state;
mod views;
mod backend;

use state::{
    App, Message, Tab, PrimaryTab, NmtState, NodeState, LogEntry,
    EmcyEntry, HeartbeatStatus,
};
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
            Message::SwitchTab(tab) => {
                self.current_tab = tab;
                iced::Task::none()
            }
            Message::SwitchPrimary(primary) => {
                // Switch to the first tab of the primary
                self.current_tab = Tab::for_primary(primary)[0];
                iced::Task::none()
            }
            Message::NodeSelected(node_id) => {
                self.selected_node = Some(node_id);
                self.current_tab = Tab::NetworkManagement;
                iced::Task::none()
            }
            Message::ToggleDetailPanel => {
                self.detail_collapsed = !self.detail_collapsed;
                iced::Task::none()
            }

            // === Toolbar ===
            Message::TogglePause => {
                self.toolbar.paused = !self.toolbar.paused;
                iced::Task::none()
            }
            Message::BitrateChanged(bitrate) => {
                self.toolbar.bitrate = bitrate;
                iced::Task::none()
            }
            Message::ClearLog => {
                self.can_log.clear();
                self.pdo_log.clear();
                self.error_frames.clear();
                self.emcy_log.clear();
                self.selected_frame = None;
                iced::Task::none()
            }
            Message::ExportLog => {
                // Export CAN log to CSV
                let path = std::path::PathBuf::from("opencan_log.csv");
                let mut csv = String::from("timestamp_ms,cob_id,dlc,data\n");
                for entry in &self.can_log {
                    let hex_data: String = entry.data.iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    csv.push_str(&format!("{},{:03X},{}\"{}\"\n",
                        entry.timestamp_ms, entry.cob_id, entry.data.len(), hex_data));
                }
                match std::fs::write(&path, &csv) {
                    Ok(_) => self.status_message = format!("Log exported to {:?}", path),
                    Err(e) => self.status_message = format!("Export failed: {}", e),
                }
                iced::Task::none()
            }
            Message::ImportLog => {
                // Import CAN log from CSV
                let path = std::path::PathBuf::from("opencan_log.csv");
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let mut count = 0;
                        for line in content.lines().skip(1) {
                            let parts: Vec<&str> = line.splitn(4, ',').collect();
                            if parts.len() >= 3 {
                                if let (Ok(ts), Ok(cob_id)) = (parts[0].parse::<u64>(), u16::from_str_radix(parts[1], 16)) {
                                    let data_str = parts.get(2).unwrap_or(&"");
                                    let mut data = [0u8; 8];
                                    for (i, byte) in data_str.split_whitespace().enumerate() {
                                        if i < 8 {
                                            if let Ok(b) = u8::from_str_radix(byte, 16) {
                                                data[i] = b;
                                            }
                                        }
                                    }
                                    self.can_log.push(LogEntry {
                                        timestamp_ms: ts,
                                        cob_id,
                                        data,
                                        description: String::new(),
                                    });
                                    count += 1;
                                }
                            }
                        }
                        self.status_message = format!("Imported {} frames from {:?}", count, path);
                    }
                    Err(e) => self.status_message = format!("Import failed: {}", e),
                }
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
            Message::NmtStartAll => {
                if let Some(ref backend) = self.backend {
                    backend.send(BackendCommand::NmtStart(0)); // 0 = all nodes
                }
                iced::Task::none()
            }
            Message::NmtStopAll => {
                if let Some(ref backend) = self.backend {
                    backend.send(BackendCommand::NmtStop(0)); // 0 = all nodes
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
            Message::Ds402ModeChanged(mode) => {
                self.ds402_state.selected_mode = mode;
                iced::Task::none()
            }

            // === Frame detail ===
            Message::FrameSelected(entry) => {
                self.selected_frame = Some(entry);
                iced::Task::none()
            }

            // === CAN log filter ===
            Message::LogFilterChanged(text) => {
                self.log_filter.text = text;
                iced::Task::none()
            }
            Message::LogClear => {
                self.can_log.clear();
                self.pdo_log.clear();
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
                            node.od_cache.insert((index, subindex), state::OdEntry::new(hex.clone()));
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
                    node.ds402.position_history.push(position);
                    if node.ds402.position_history.len() > 100 {
                        node.ds402.position_history.remove(0);
                    }
                }
            }
            BackendEvent::Ds402VelocityResult { node_id, velocity } => {
                if let Some(node) = self.nodes.iter_mut().find(|n| n.node_id == node_id) {
                    node.ds402.actual_velocity = velocity;
                    node.ds402.velocity_history.push(velocity);
                    if node.ds402.velocity_history.len() > 100 {
                        node.ds402.velocity_history.remove(0);
                    }
                }
            }
            BackendEvent::FrameReceived { cob_id, data, timestamp_ms } => {
                if !self.toolbar.paused {
                    let entry = LogEntry {
                        timestamp_ms,
                        cob_id,
                        data,
                        description: String::new(),
                    };
                    // Route PDOs to dedicated pdo_log
                    if (0x180..=0x57F).contains(&cob_id) {
                        self.pdo_log.push(entry.clone());
                        if self.pdo_log.len() > 1000 {
                            self.pdo_log.remove(0);
                        }
                    }
                    // Route EMCY to emcy_log
                    if (0x081..=0x0FF).contains(&cob_id) {
                        let node_id = (cob_id - 0x080) as u8;
                        if data.len() >= 2 {
                            let error_code = (data[1] as u16) << 8 | data[0] as u16;
                            let error_register = if data.len() > 2 { data[2] } else { 0 };
                            let mut emcy_data = [0u8; 5];
                            let len = data.len().min(7);
                            if len > 2 {
                                emcy_data[..len - 2].copy_from_slice(&data[2..len]);
                            }
                            self.emcy_log.push(EmcyEntry {
                                timestamp_ms,
                                node_id,
                                error_code,
                                error_register,
                                data: emcy_data,
                            });
                        }
                    }
                    self.can_log.push(entry);
                    self.bus_stats.frame_count += 1;
                }
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
                // Update heartbeat status
                if let Some(hb) = self.heartbeat_status.iter_mut().find(|h| h.node_id == node_id) {
                    hb.alive = alive;
                    hb.last_heartbeat_ms = Some(0); // TODO: use actual timestamp
                } else {
                    self.heartbeat_status.push(HeartbeatStatus {
                        node_id,
                        producer_period_ms: None,
                        consumer_config: None,
                        last_heartbeat_ms: Some(0),
                        alive,
                    });
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let toolbar = self.view_toolbar();
        let node_panel = self.view_node_panel();
        let main_content = self.view_main_content();
        let detail_panel = self.view_detail_panel();
        let statusbar = self.view_statusbar();

        // Build main layout
        let workspace = if self.detail_collapsed {
            row![main_content].width(Length::Fill).height(Length::Fill)
        } else {
            row![main_content, detail_panel].width(Length::Fill).height(Length::Fill)
        };

        let content = row![node_panel, workspace]
            .width(Length::Fill)
            .height(Length::Fill);

        let base = column![toolbar, content, statusbar]
            .width(Length::Fill)
            .height(Length::Fill);

        // Show connection dialog as an overlay
        if self.connection_dialog.visible {
            let dialog = views::connection_dialog(&self.connection_dialog);
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

    fn view_toolbar(&self) -> Element<'_, Message> {
        let mut toolbar = row![].spacing(8).padding(8);

        // Connection buttons
        if !self.connected {
            toolbar = toolbar.push(
                button(text("Connect (Mock)").size(12))
                    .on_press(Message::ConnectMock)
            );
            toolbar = toolbar.push(
                button(text("Connect...").size(12))
                    .on_press(Message::ShowConnectionDialog)
            );
        } else {
            toolbar = toolbar.push(
                button(text("Disconnect").size(12))
                    .on_press(Message::Disconnect)
            );
            toolbar = toolbar.push(
                button(text("Scan Nodes").size(12))
                    .on_press(Message::ScanNodes)
            );
        }

        toolbar = toolbar.push(text("│").size(12));

        // NMT quick actions
        if self.connected {
            toolbar = toolbar.push(
                button(text("Start All").size(12))
                    .on_press(Message::NmtStartAll)
            );
            toolbar = toolbar.push(
                button(text("Stop All").size(12))
                    .on_press(Message::NmtStopAll)
            );
        }

        toolbar = toolbar.push(text("│").size(12));

        // Bitrate selector
        toolbar = toolbar.push(text("Bitrate:").size(12));
        for br in [250000u32, 500000, 1000000] {
            let label = format!("{}k", br / 1000);
            let is_selected = self.toolbar.bitrate == br;
            let btn = if is_selected {
                button(text(format!("[{}]", label)).size(11))
                    .on_press(Message::BitrateChanged(br))
            } else {
                button(text(label).size(11))
                    .on_press(Message::BitrateChanged(br))
            };
            toolbar = toolbar.push(btn);
        }

        toolbar = toolbar.push(text("│").size(12));

        // Pause/Resume
        let pause_text = if self.toolbar.paused { "Resume" } else { "Pause" };
        toolbar = toolbar.push(
            button(text(pause_text).size(12))
                .on_press(Message::TogglePause)
        );

        // Log actions
        toolbar = toolbar.push(
            button(text("Clear").size(12))
                .on_press(Message::ClearLog)
        );
        toolbar = toolbar.push(
            button(text("Export").size(12))
                .on_press(Message::ExportLog)
        );
        toolbar = toolbar.push(
            button(text("Import").size(12))
                .on_press(Message::ImportLog)
        );

        // Detail panel toggle
        let detail_text = if self.detail_collapsed { "Show Detail" } else { "Hide Detail" };
        toolbar = toolbar.push(
            button(text(detail_text).size(12))
                .on_press(Message::ToggleDetailPanel)
        );

        container(toolbar)
            .width(Length::Fill)
            .height(48)
            .into()
    }

    fn view_node_panel(&self) -> Element<'_, Message> {
        let mut panel = column![text("Nodes").size(14)].spacing(4).padding(8);

        // Connection status
        let conn_text = if self.connected { "● Connected" } else { "○ Disconnected" };
        panel = panel.push(text(conn_text).size(11));

        panel = panel.push(horizontal_rule(1));

        // Node list
        if self.nodes.is_empty() {
            panel = panel.push(text("  (none)").size(11));
        }
        for node in &self.nodes {
            let state_str = node.nmt_state.as_str();
            let label = format!("Node {} [{}]", node.node_id, state_str);
            let is_selected = self.selected_node == Some(node.node_id);
            let btn = if is_selected {
                button(text(format!("► {}", label)).size(11))
                    .on_press(Message::NodeSelected(node.node_id))
                    .width(Length::Fill)
            } else {
                button(text(label).size(11))
                    .on_press(Message::NodeSelected(node.node_id))
                    .width(Length::Fill)
            };
            panel = panel.push(btn);
        }

        container(scrollable(panel))
            .width(200)
            .height(Length::Fill)
            .into()
    }

    fn view_main_content(&self) -> Element<'_, Message> {
        let primary = self.current_tab.primary();
        let tabs = Tab::for_primary(primary);

        // Primary tabs
        let mut primary_tabs = row![].spacing(4);
        for &p in &[PrimaryTab::CanBus, PrimaryTab::CanOpen] {
            let label = match p {
                PrimaryTab::CanBus => "CAN 总线",
                PrimaryTab::CanOpen => "CANOpen 协议",
            };
            let is_selected = primary == p;
            let btn = if is_selected {
                button(text(format!("[{}]", label)).size(12))
                    .on_press(Message::SwitchPrimary(p))
            } else {
                button(text(label).size(12))
                    .on_press(Message::SwitchPrimary(p))
            };
            primary_tabs = primary_tabs.push(btn);
        }

        // Secondary tabs
        let mut secondary_tabs = row![].spacing(2);
        for &tab in tabs {
            let is_selected = self.current_tab == tab;
            let btn = if is_selected {
                button(text(format!("[{}]", tab.name())).size(11))
                    .on_press(Message::SwitchTab(tab))
            } else {
                button(text(tab.name()).size(11))
                    .on_press(Message::SwitchTab(tab))
            };
            secondary_tabs = secondary_tabs.push(btn);
        }

        // Content
        let content = match self.current_tab {
            Tab::FrameMonitor => views::frame_monitor(self),
            Tab::BusStatistics => views::bus_statistics(self),
            Tab::ErrorFrames => views::error_frames(self),
            Tab::NetworkManagement => views::network_management(self),
            Tab::SdoClient => views::sdo_client(self),
            Tab::PdoMonitor => views::pdo_monitor(self),
            Tab::Ds402Control => views::ds402_control(self),
            Tab::EmcyLog => views::emcy_log(self),
            Tab::HeartbeatMonitor => views::heartbeat_monitor(self),
            Tab::SyncManagement => views::sync_management(self),
        };

        column![
            primary_tabs,
            secondary_tabs,
            horizontal_rule(1),
            content,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_detail_panel(&self) -> Element<'_, Message> {
        views::detail_panel(self)
    }

    fn view_statusbar(&self) -> Element<'_, Message> {
        let mut status = row![].spacing(8).padding(4);

        // Connection status
        let conn_text = if self.connected {
            format!("Connected ({})", self.backend.as_ref().map(|_| "Active").unwrap_or("Unknown"))
        } else {
            "Disconnected".to_string()
        };
        status = status.push(text(conn_text).size(10));

        status = status.push(text("│").size(10));

        // Bus stats
        status = status.push(text(format!("Load: {:.1}%", self.bus_stats.bus_load_percent)).size(10));
        status = status.push(text(format!("Rate: {} fps", self.bus_stats.frame_rate)).size(10));
        status = status.push(text(format!("Frames: {}", self.bus_stats.frame_count)).size(10));
        status = status.push(text(format!("Errors: {}/{}", self.bus_stats.tx_errors, self.bus_stats.rx_errors)).size(10));

        status = status.push(text("│").size(10));

        // Last operation
        status = status.push(text(&self.status_message).size(10));

        container(status)
            .width(Length::Fill)
            .height(32)
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
