//! Application logic.

use iced::widget::{column, container, row};
use iced::{Element, Length, Subscription, Theme, time};

use crate::state::{
    App, Message, Tab, NmtState, NodeState, LogEntry, Direction,
    EmcyEntry, HeartbeatStatus,
};
use crate::backend::{BackendCommand, BackendEvent};
use crate::views;
use crate::helpers;

impl App {
    pub fn new() -> (Self, iced::Task<Message>) {
        (Self::default(), iced::Task::none())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.backend.is_some() {
            time::every(std::time::Duration::from_millis(50))
                .map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Tick => self.handle_tick(),
            Message::ConnectMock => self.handle_connect_mock(),
            Message::ShowConnectionDialog => {
                self.connection_dialog.show();
                iced::Task::none()
            }
            Message::HideConnectionDialog => {
                self.connection_dialog.hide();
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
            Message::ConnectionConnect => self.handle_connection_connect(),
            Message::Disconnect => self.handle_disconnect(),
            Message::ScanNodes => self.handle_scan_nodes(),
            Message::SwitchTab(tab) => {
                self.current_tab = tab;
                iced::Task::none()
            }
            Message::SwitchPrimary(primary) => {
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
            Message::TogglePause => {
                self.paused = !self.paused;
                iced::Task::none()
            }
            Message::BitrateChanged(bitrate) => {
                self.toolbar_bitrate = bitrate;
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
            Message::ExportLog => self.handle_export_log(),
            Message::ImportLog => self.handle_import_log(),
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
            Message::NmtResetComm(id) => {
                if let Some(ref backend) = self.backend {
                    backend.send(BackendCommand::NmtResetComm(id));
                }
                iced::Task::none()
            }
            Message::NmtStartAll => {
                if let Some(ref backend) = self.backend {
                    backend.send(BackendCommand::NmtStart(0));
                }
                iced::Task::none()
            }
            Message::NmtStopAll => {
                if let Some(ref backend) = self.backend {
                    backend.send(BackendCommand::NmtStop(0));
                }
                iced::Task::none()
            }
            Message::NmtResetAll => {
                if let Some(ref backend) = self.backend {
                    backend.send(BackendCommand::NmtReset(0));
                }
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
            Message::SdoValueChanged(val) => {
                self.sdo_value = val;
                iced::Task::none()
            }
            Message::SdoDataTypeChanged(dtype) => {
                self.sdo_data_type = dtype;
                iced::Task::none()
            }
            Message::SdoRead => self.handle_sdo_read(),
            Message::SdoWrite => self.handle_sdo_write(),
            Message::SdoQuickRead(index, subindex) => {
                self.sdo_index = format!("{:04X}", index);
                self.sdo_subindex = subindex.to_string();
                self.handle_sdo_read()
            }
            Message::SdoClearHistory => {
                self.sdo_history.clear();
                iced::Task::none()
            }
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
            Message::Ds402Transition(id, control_word) => {
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402WriteControl { node_id: id, control_word, respond: tx });
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
            Message::Ds402TargetTorqueChanged(val) => {
                self.ds402_state.target_torque = val;
                iced::Task::none()
            }
            Message::Ds402SetPosition(id) => {
                let pos = self.ds402_state.parsed_position();
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402SetPosition { node_id: id, position: pos, respond: tx });
                }
                iced::Task::none()
            }
            Message::Ds402SetVelocity(id) => {
                let vel = self.ds402_state.parsed_velocity();
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402SetVelocity { node_id: id, velocity: vel, respond: tx });
                }
                iced::Task::none()
            }
            Message::Ds402SetTorque(id) => {
                let torque = self.ds402_state.parsed_torque();
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402SetTorque { node_id: id, torque, respond: tx });
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
            Message::Ds402ReadTorque(id) => {
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402ReadTorque { node_id: id, respond: tx });
                }
                iced::Task::none()
            }
            Message::Ds402ModeChanged(mode) => {
                self.ds402_state.selected_mode = mode;
                // Send mode to backend
                let node_id = self.selected_node.unwrap_or(1);
                if let Some(ref backend) = self.backend {
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::Ds402SetMode {
                        node_id,
                        mode: mode.mode_value(),
                        respond: tx,
                    });
                    self.status_message = format!("DS402 mode changed to {}", mode.name());
                }
                iced::Task::none()
            }
            Message::Ds402ToggleAutoRefresh => {
                self.ds402_state.toggle_auto_refresh();
                iced::Task::none()
            }
            Message::Ds402ToggleRawValues => {
                self.ds402_state.toggle_raw_values();
                iced::Task::none()
            }
            Message::FrameSelected(entry) => {
                self.selected_frame = Some(entry);
                iced::Task::none()
            }
            Message::LogFilterChanged(text) => {
                self.log_filter.text = text;
                iced::Task::none()
            }
            Message::LogFilterClear => {
                self.log_filter.reset();
                iced::Task::none()
            }
            Message::LogFilterToggleNmt => {
                self.log_filter.show_nmt = !self.log_filter.show_nmt;
                iced::Task::none()
            }
            Message::LogFilterToggleSdo => {
                self.log_filter.show_sdo = !self.log_filter.show_sdo;
                iced::Task::none()
            }
            Message::LogFilterTogglePdo => {
                self.log_filter.show_pdo = !self.log_filter.show_pdo;
                iced::Task::none()
            }
            Message::LogFilterToggleEmcy => {
                self.log_filter.show_emcy = !self.log_filter.show_emcy;
                iced::Task::none()
            }
            Message::LogFilterToggleHeartbeat => {
                self.log_filter.show_heartbeat = !self.log_filter.show_heartbeat;
                iced::Task::none()
            }
            Message::LogFilterNodeId(node_id) => {
                self.log_filter.node_id_filter = node_id;
                iced::Task::none()
            }
            Message::SyncStartProducer(period_str) => {
                let period: u32 = period_str.parse().unwrap_or(1000);
                self.sync_status.start_producer(period);
                self.status_message = format!("SYNC producer started ({} μs)", period);
                iced::Task::none()
            }
            Message::SyncStopProducer => {
                self.sync_status.stop_producer();
                self.status_message = "SYNC producer stopped".to_string();
                iced::Task::none()
            }
            Message::SyncPeriodChanged(period_str) => {
                if let Ok(period) = period_str.parse::<u32>() {
                    self.sync_status.producer_period_us = period;
                }
                iced::Task::none()
            }
            Message::ReadPdoMapping(node_id) => {
                if let Some(ref backend) = self.backend {
                    // Read TPDO1 mapping (0x1A00)
                    let (tx, _rx) = tokio::sync::oneshot::channel();
                    backend.send(BackendCommand::ReadPdoMapping {
                        node_id,
                        pdo_index: 1,
                        respond: tx,
                    });
                    self.status_message = format!("Reading PDO mapping for node {}...", node_id);
                }
                iced::Task::none()
            }
            Message::ShowAbout => {
                self.status_message = "OpenCAN v0.1.0 - CAN/CANOpen Debug Tool".to_string();
                iced::Task::none()
            }
        }
    }

    fn handle_tick(&mut self) -> iced::Task<Message> {
        let mut events = Vec::new();
        if let Some(ref mut backend) = self.backend {
            while let Some(event) = backend.try_recv() {
                events.push(event);
            }
        }

        // Count frames before processing
        let frames_before = self.can_log.len();

        for event in events {
            self.handle_backend_event(event);
        }

        // Update bus statistics
        let frames_after = self.can_log.len();
        let new_frames = frames_after - frames_before;
        if new_frames > 0 {
            // Frame rate is approximate (ticks every 50ms)
            let instant_rate = (new_frames as f32 * 20.0) as u32; // 20 ticks per second
            self.bus_stats.update_frame_rate(instant_rate);

            // Estimate bus load (rough approximation)
            // Each standard CAN frame is ~130 bits at 500kbps = ~0.26ms
            let load = (new_frames as f32 * 0.26 * 20.0).min(100.0);
            self.bus_stats.update_bus_load(load);
        } else {
            // No frames, update with 0
            self.bus_stats.update_frame_rate(0);
            self.bus_stats.update_bus_load(0.0);
        }

        iced::Task::none()
    }

    fn handle_connect_mock(&mut self) -> iced::Task<Message> {
        let backend = crate::backend::Backend::new_mock();
        self.backend = Some(backend);
        self.connected = true;
        self.status_message = "Connected (Mock)".to_string();
        iced::Task::none()
    }

    fn handle_connection_connect(&mut self) -> iced::Task<Message> {
        let backend_type = self.connection_dialog.selected_backend;
        match backend_type {
            crate::state::CanBackend::Mock => {
                let backend = crate::backend::Backend::new_mock();
                self.backend = Some(backend);
                self.connected = true;
                self.status_message = "Connected (Mock)".to_string();
            }
            crate::state::CanBackend::SocketCan => {
                self.status_message = "SocketCAN connection not available on this platform".to_string();
            }
            _ => {
                self.status_message = format!("{} backend not yet implemented", backend_type.name());
            }
        }
        self.connection_dialog.hide();
        iced::Task::none()
    }

    fn handle_disconnect(&mut self) -> iced::Task<Message> {
        if let Some(ref backend) = self.backend {
            backend.send(BackendCommand::Disconnect);
        }
        self.backend = None;
        self.connected = false;
        self.nodes.clear();
        self.status_message = "Disconnected".to_string();
        iced::Task::none()
    }

    fn handle_scan_nodes(&mut self) -> iced::Task<Message> {
        if let Some(ref backend) = self.backend {
            let (tx, _rx) = tokio::sync::oneshot::channel();
            backend.send(BackendCommand::ScanNodes { respond: tx });
            self.status_message = "Scanning nodes...".to_string();
        }
        iced::Task::none()
    }

    fn handle_sdo_read(&mut self) -> iced::Task<Message> {
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

    fn handle_sdo_write(&mut self) -> iced::Task<Message> {
        let node_id = self.selected_node.unwrap_or(1);
        let index = u16::from_str_radix(self.sdo_index.trim_start_matches("0x"), 16).unwrap_or(0);
        let subindex: u8 = self.sdo_subindex.parse().unwrap_or(0);
        let value = helpers::parse_hex_bytes(&self.sdo_value);
        self.status_message = format!("SDO write {:04X}:{:02X} to node {}...", index, subindex, node_id);
        if let Some(ref backend) = self.backend {
            let (tx, _rx) = tokio::sync::oneshot::channel();
            backend.send(BackendCommand::SdoDownload { node_id, index, subindex, value, respond: tx });
        }
        iced::Task::none()
    }

    fn handle_export_log(&mut self) -> iced::Task<Message> {
        let csv = self.log_recorder.to_csv();
        match std::fs::write("opencan_log.csv", &csv) {
            Ok(_) => self.status_message = "Log exported to opencan_log.csv".to_string(),
            Err(e) => self.status_message = format!("Export failed: {}", e),
        }
        iced::Task::none()
    }

    fn handle_import_log(&mut self) -> iced::Task<Message> {
        match std::fs::read_to_string("opencan_log.csv") {
            Ok(content) => {
                let entries = crate::state::LogRecorder::from_csv(&content);
                let count = entries.len();
                self.can_log.extend(entries);
                self.status_message = format!("Imported {} frames", count);
            }
            Err(e) => self.status_message = format!("Import failed: {}", e),
        }
        iced::Task::none()
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
                self.status_message = format!("Found {} nodes", nodes.len());

                // Auto-read node info for each discovered node
                if let Some(ref backend) = self.backend {
                    for &id in &nodes {
                        // Read Device Type (0x1000:0)
                        let (tx1, _rx1) = tokio::sync::oneshot::channel();
                        backend.send(BackendCommand::SdoUpload {
                            node_id: id, index: 0x1000, subindex: 0, respond: tx1
                        });

                        // Read Vendor ID (0x1018:1)
                        let (tx2, _rx2) = tokio::sync::oneshot::channel();
                        backend.send(BackendCommand::SdoUpload {
                            node_id: id, index: 0x1018, subindex: 1, respond: tx2
                        });

                        // Read Heartbeat Producer Period (0x1017:0)
                        let (tx3, _rx3) = tokio::sync::oneshot::channel();
                        backend.send(BackendCommand::SdoUpload {
                            node_id: id, index: 0x1017, subindex: 0, respond: tx3
                        });
                    }
                }
            }
            BackendEvent::SdoResult { node_id, index, subindex, result } => {
                match result {
                    Ok(data) => {
                        let hex = helpers::bytes_to_hex(&data);
                        self.sdo_value = hex.clone();

                        // Store in OD cache and update node info
                        if let Some(node) = self.get_node_mut(node_id) {
                            node.set_od(index, subindex, crate::state::OdEntry::new(hex.clone()));

                            // Auto-update node info fields
                            match (index, subindex) {
                                (0x1000, 0) => {
                                    // Device Type
                                    if data.len() >= 4 {
                                        node.device_type = Some(u32::from_le_bytes([
                                            data[0], data[1], data[2], data[3]
                                        ]));
                                    }
                                }
                                (0x1018, 1) => {
                                    // Vendor ID
                                    if data.len() >= 4 {
                                        node.vendor_id = Some(u32::from_le_bytes([
                                            data[0], data[1], data[2], data[3]
                                        ]));
                                    }
                                }
                                (0x1017, 0) => {
                                    // Heartbeat Producer Period
                                    if data.len() >= 2 {
                                        node.heartbeat_period = Some(u16::from_le_bytes([
                                            data[0], data[1]
                                        ]) as u32);
                                    }
                                }
                                (0x6041, 0) => {
                                    // DS402 Status Word
                                    if data.len() >= 2 {
                                        node.ds402.status_word = u16::from_le_bytes([
                                            data[0], data[1]
                                        ]);
                                    }
                                }
                                _ => {}
                            }
                        }

                        self.status_message = format!("SDO {:04X}:{:02X} = {}", index, subindex, hex);
                        self.sdo_history.push(crate::state::SdoHistoryEntry::success(
                            node_id, index, subindex, hex, true
                        ));
                    }
                    Err(e) => {
                        self.status_message = format!("SDO error: {}", e);
                        self.sdo_history.push(crate::state::SdoHistoryEntry::failure(
                            node_id, index, subindex, e.clone(), true
                        ));
                    }
                }
            }
            BackendEvent::NmtStateChanged { node_id, state } => {
                if let Some(node) = self.get_node_mut(node_id) {
                    node.nmt_state = NmtState::from_str(&state);
                }
                self.status_message = format!("Node {} → {}", node_id, state);
            }
            BackendEvent::Ds402StateResult { node_id, state, status_word } => {
                if let Some(node) = self.get_node_mut(node_id) {
                    node.ds402.state = state.clone();
                    node.ds402.status_word = status_word;
                }
                self.status_message = format!("DS402 Node {}: {} (0x{:04X})", node_id, state, status_word);
            }
            BackendEvent::Ds402PositionResult { node_id, position } => {
                if let Some(node) = self.get_node_mut(node_id) {
                    node.ds402.push_position(position);
                }
            }
            BackendEvent::Ds402VelocityResult { node_id, velocity } => {
                if let Some(node) = self.get_node_mut(node_id) {
                    node.ds402.push_velocity(velocity);
                }
            }
            BackendEvent::Ds402TorqueResult { node_id, torque } => {
                if let Some(node) = self.get_node_mut(node_id) {
                    node.ds402.push_torque(torque);
                }
            }
            BackendEvent::FrameReceived { cob_id, data, dlc, timestamp_ms } => {
                if !self.paused {
                    let entry = LogEntry {
                        timestamp_ms,
                        cob_id,
                        data,
                        dlc,
                        direction: Direction::Rx,
                        description: String::new(),
                    };
                    // Route PDOs
                    if (0x180..=0x57F).contains(&cob_id) {
                        self.pdo_log.push(entry.clone());
                        if self.pdo_log.len() > 1000 {
                            self.pdo_log.remove(0);
                        }
                    }
                    // Route EMCY
                    if (0x081..=0x0FF).contains(&cob_id) {
                        let emcy_node_id = (cob_id - 0x080) as u8;
                        if data.len() >= 2 {
                            let error_code = (data[1] as u16) << 8 | data[0] as u16;
                            let error_register = if data.len() > 2 { data[2] } else { 0 };
                            let mut emcy_data = [0u8; 5];
                            let len = data.len().min(7);
                            if len > 2 {
                                emcy_data[..len - 2].copy_from_slice(&data[2..len]);
                            }
                            self.emcy_log.push(EmcyEntry::new(
                                timestamp_ms, emcy_node_id, error_code, error_register
                            ).with_data(emcy_data));
                        }
                    }
                    // Route Heartbeat
                    if (0x700..=0x77F).contains(&cob_id) {
                        let hb_node_id = (cob_id - 0x700) as u8;
                        let alive = data[0] == 0x05; // Operational
                        if let Some(hb) = self.heartbeat_status.iter_mut().find(|h| h.node_id == hb_node_id) {
                            hb.heartbeat_received(timestamp_ms);
                            hb.alive = alive;
                        } else {
                            self.heartbeat_status.push(
                                HeartbeatStatus::new(hb_node_id)
                                    .with_producer_period(1000)
                            );
                        }
                    }
                    self.can_log.push(entry);
                    self.bus_stats.inc_rx();
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
            BackendEvent::HeartbeatChanged { node_id, alive, timestamp_ms } => {
                let state = if alive { "alive" } else { "lost" };
                self.status_message = format!("Node {} heartbeat {}", node_id, state);
                if let Some(hb) = self.heartbeat_status.iter_mut().find(|h| h.node_id == node_id) {
                    hb.alive = alive;
                    if alive {
                        hb.heartbeat_received(timestamp_ms);
                    } else {
                        hb.heartbeat_lost();
                    }
                } else {
                    self.heartbeat_status.push(
                        HeartbeatStatus::new(node_id).with_producer_period(1000)
                    );
                }
            }
            BackendEvent::EmcyReceived { node_id, error_code, error_register, data, timestamp_ms } => {
                self.emcy_log.push(
                    EmcyEntry::new(timestamp_ms, node_id, error_code, error_register)
                        .with_data(data)
                );
            }
            BackendEvent::SyncReceived { timestamp_ms } => {
                self.sync_status.sync_received(timestamp_ms);
            }
            BackendEvent::BusStatsUpdate { bus_load, frame_rate, tx_errors, rx_errors } => {
                self.bus_stats.update_bus_load(bus_load);
                self.bus_stats.update_frame_rate(frame_rate);
                self.bus_stats.tx_errors = tx_errors;
                self.bus_stats.rx_errors = rx_errors;
            }
            BackendEvent::ErrorFrameReceived { timestamp_ms, error_type, tec, rec } => {
                self.bus_stats.inc_error_frame();
                self.error_frames.push(
                    crate::state::ErrorFrame::new(
                        timestamp_ms,
                        crate::state::ErrorType::from_flags(error_type)
                    ).with_counters(tec, rec)
                );
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let toolbar = views::toolbar(self);
        let node_panel = views::node_panel(self);
        let main_content = views::main_content(self);
        let detail_panel = views::detail_panel(self);
        let statusbar = views::statusbar(self);

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
            let dialog = views::connection_dialog(self);
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

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}
