//! View rendering functions.

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::state::{App, Message, CanBackend, ConnectionDialog, Ds402Mode};

/// Frame monitor view (three-pane layout).
pub fn frame_monitor(app: &App) -> Element<'_, Message> {
    let filtered: Vec<_> = app.can_log.iter()
        .filter(|e| app.log_filter.matches(e))
        .collect();

    // Frame list (top pane)
    let mut frame_list = column![
        row![
            text("帧列表").size(14),
            text(format!("({} 帧)", filtered.len())).size(11),
        ].spacing(8),
        horizontal_rule(1),
    ].spacing(2);

    // Table header
    frame_list = frame_list.push(
        row![
            text("时间").size(10).width(80),
            text("CAN ID").size(10).width(60),
            text("DLC").size(10).width(30),
            text("数据").size(10).width(160),
            text("解码").size(10),
        ].spacing(4)
    );
    frame_list = frame_list.push(horizontal_rule(1));

    // Show latest frames (newest first)
    for entry in filtered.iter().rev().take(100) {
        let hex_data: String = entry.data.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let desc = if entry.description.is_empty() {
            classify_cob_id(entry.cob_id)
        } else {
            entry.description.clone()
        };

        let _is_selected = app.selected_frame.as_ref()
            .map(|f| f.cob_id == entry.cob_id && f.timestamp_ms == entry.timestamp_ms)
            .unwrap_or(false);

        let row_content = row![
            text(format!("{}ms", entry.timestamp_ms)).size(10).width(80),
            text(format!("{:03X}", entry.cob_id)).size(10).width(60),
            text(format!("{}", entry.data.len())).size(10).width(30),
            text(hex_data).size(10).width(160),
            text(desc).size(10),
        ].spacing(4);

        let entry_clone = (*entry).clone();
        let btn = button(row_content)
            .on_press(Message::FrameSelected(entry_clone))
            .width(Length::Fill);

        frame_list = frame_list.push(btn);
    }

    // Frame detail (middle pane)
    let frame_detail = if let Some(ref frame) = app.selected_frame {
        let mut detail = column![
            text("帧详情").size(14),
            horizontal_rule(1),
        ].spacing(2);

        detail = detail.push(text(format!("CAN ID: 0x{:03X} ({})", frame.cob_id, classify_cob_id(frame.cob_id))).size(11));
        detail = detail.push(text(format!("时间: {}ms", frame.timestamp_ms)).size(11));
        detail = detail.push(text(format!("DLC: {}", frame.data.len())).size(11));

        // Decode COB-ID
        let (func, node_id) = decode_cob_id(frame.cob_id);
        detail = detail.push(text(format!("Function Code: {}", func)).size(11));
        detail = detail.push(text(format!("Node ID: {}", node_id)).size(11));

        // SDO decode
        if (0x580..=0x67F).contains(&frame.cob_id) && frame.data.len() >= 4 {
            detail = detail.push(horizontal_rule(1));
            detail = detail.push(text("SDO 解码:").size(11));
            let cmd = frame.data[0];
            let index = (frame.data[2] as u16) << 8 | frame.data[1] as u16;
            let subindex = frame.data[3];
            detail = detail.push(text(format!("  命令: 0x{:02X}", cmd)).size(10));
            detail = detail.push(text(format!("  Index: 0x{:04X}", index)).size(10));
            detail = detail.push(text(format!("  Subindex: 0x{:02X}", subindex)).size(10));
            if cmd & 0xE0 == 0x40 {
                // Upload response
                let expedited = cmd & 0x02 != 0;
                let size = if expedited { 4 - ((cmd >> 2) & 0x03) as usize } else { 0 };
                detail = detail.push(text(format!("  类型: Upload Response")).size(10));
                detail = detail.push(text(format!("  Expedited: {}", expedited)).size(10));
                if expedited && size > 0 {
                    let data_start = 4;
                    let data_end = (data_start + size).min(frame.data.len());
                    let data: Vec<String> = frame.data[data_start..data_end].iter()
                        .map(|b| format!("{:02X}", b))
                        .collect();
                    detail = detail.push(text(format!("  数据: {}", data.join(" "))).size(10));
                }
            } else if cmd & 0xE0 == 0x20 {
                detail = detail.push(text("  类型: Download Response").size(10));
            }
        }

        // PDO decode
        if (0x180..=0x57F).contains(&frame.cob_id) {
            detail = detail.push(horizontal_rule(1));
            detail = detail.push(text("PDO 数据:").size(11));
            let hex_data: Vec<String> = frame.data.iter()
                .map(|b| format!("{:02X}", b))
                .collect();
            detail = detail.push(text(format!("  原始数据: {}", hex_data.join(" "))).size(10));
        }

        container(scrollable(detail)).height(Length::Fill)
    } else {
        container(
            text("选择一帧查看详情").size(11)
        ).height(Length::Fill)
    };

    // Hex dump (bottom pane)
    let hex_dump = if let Some(ref frame) = app.selected_frame {
        let mut dump = column![
            text("原始字节").size(14),
            horizontal_rule(1),
        ].spacing(2);

        let hex: Vec<String> = frame.data.iter()
            .map(|b| format!("{:02X}", b))
            .collect();
        let ascii: String = frame.data.iter()
            .map(|&b| if b >= 0x20 && b < 0x7F { b as char } else { '.' })
            .collect();

        dump = dump.push(text(format!("00000000  {:<47}  {}", hex.join(" "), ascii)).size(10));

        container(dump)
    } else {
        container(text("").size(10))
    };

    // Filter bar
    let filter_bar = row![
        text("过滤:").size(11),
        text_input("搜索 COB-ID、数据或描述...", &app.log_filter.text)
            .on_input(Message::LogFilterChanged)
            .width(Length::Fill),
    ].spacing(4);

    column![
        filter_bar,
        horizontal_rule(1),
        container(scrollable(frame_list)).height(Length::FillPortion(3)),
        horizontal_rule(1),
        frame_detail,
        horizontal_rule(1),
        hex_dump,
    ]
    .spacing(2)
    .padding(8)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Bus statistics view.
pub fn bus_statistics(app: &App) -> Element<'_, Message> {
    let stats = &app.bus_stats;

    let mut content = column![
        text("总线统计").size(16),
        horizontal_rule(1),
    ].spacing(8).padding(10);

    // Bus load
    content = content.push(text(format!("总线负载: {:.1}%", stats.bus_load_percent)).size(14));
    // Simple bar visualization
    let load_bar = format!("[{}{}]",
        "█".repeat((stats.bus_load_percent / 5.0) as usize),
        "░".repeat(20 - (stats.bus_load_percent / 5.0) as usize)
    );
    content = content.push(text(load_bar).size(12));

    content = content.push(horizontal_rule(1));

    // Frame rate
    content = content.push(text(format!("帧率: {} 帧/秒", stats.frame_rate)).size(14));
    content = content.push(text(format!("总帧数: {}", stats.frame_count)).size(12));

    content = content.push(horizontal_rule(1));

    // Error counts
    content = content.push(text("错误统计:").size(14));
    content = content.push(text(format!("  发送错误: {}", stats.tx_errors)).size(12));
    content = content.push(text(format!("  接收错误: {}", stats.rx_errors)).size(12));

    content = content.push(horizontal_rule(1));

    // Bus state
    content = content.push(text(format!("总线状态: {}", stats.bus_state.as_str())).size(14));

    // Bitrate
    content = content.push(text(format!("波特率: {} kbps", stats.bitrate / 1000)).size(12));

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// Error frames view.
pub fn error_frames(app: &App) -> Element<'_, Message> {
    let mut content = column![
        row![
            text("错误帧").size(16),
            text(format!("({} 帧)", app.error_frames.len())).size(12),
        ].spacing(8),
        horizontal_rule(1),
    ].spacing(4).padding(10);

    if app.error_frames.is_empty() {
        content = content.push(text("无错误帧").size(12));
    } else {
        // Table header
        content = content.push(
            row![
                text("时间").size(10).width(80),
                text("类型").size(10).width(80),
                text("标志").size(10).width(40),
                text("TEC").size(10).width(40),
                text("REC").size(10).width(40),
            ].spacing(4)
        );
        content = content.push(horizontal_rule(1));

        for err in app.error_frames.iter().rev().take(100) {
            content = content.push(
                row![
                    text(format!("{}ms", err.timestamp_ms)).size(10).width(80),
                    text(err.error_type.as_str()).size(10).width(80),
                    text(format!("0x{:02X}", err.error_flag)).size(10).width(40),
                    text(format!("{}", err.tec)).size(10).width(40),
                    text(format!("{}", err.rec)).size(10).width(40),
                ].spacing(4)
            );
        }
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// Network management view.
pub fn network_management(app: &App) -> Element<'_, Message> {
    let mut content = column![
        text("网络管理").size(16),
        horizontal_rule(1),
    ].spacing(8).padding(10);

    if !app.connected {
        content = content.push(text("未连接。请先连接到 CAN 总线。").size(12));
        return container(scrollable(content)).width(Length::Fill).height(Length::Fill).into();
    }

    // Batch operations
    content = content.push(
        row![
            button(text("扫描节点").size(11)).on_press(Message::ScanNodes),
            button(text("全部启动").size(11)).on_press(Message::NmtStartAll),
            button(text("全部停止").size(11)).on_press(Message::NmtStopAll),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // Node table header
    content = content.push(
        row![
            text("Node ID").size(10).width(60),
            text("NMT 状态").size(10).width(100),
            text("设备类型").size(10).width(80),
            text("心跳").size(10).width(60),
            text("操作").size(10),
        ].spacing(4)
    );
    content = content.push(horizontal_rule(1));

    // Node list
    for node in &app.nodes {
        let state_str = node.nmt_state.as_str();
        let device_type = node.device_type
            .map(|dt| format!("0x{:08X}", dt))
            .unwrap_or_else(|| "-".to_string());
        let heartbeat = node.heartbeat_period
            .map(|p| format!("{}ms", p))
            .unwrap_or_else(|| "-".to_string());

        content = content.push(
            row![
                text(format!("{}", node.node_id)).size(11).width(60),
                text(state_str).size(11).width(100),
                text(device_type).size(11).width(80),
                text(heartbeat).size(11).width(60),
                row![
                    button(text("Start").size(10)).on_press(Message::NmtStartNode(node.node_id)),
                    button(text("Stop").size(10)).on_press(Message::NmtStopNode(node.node_id)),
                    button(text("Reset").size(10)).on_press(Message::NmtResetNode(node.node_id)),
                ].spacing(2),
            ].spacing(4)
        );
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// SDO client view.
pub fn sdo_client(app: &App) -> Element<'_, Message> {
    let mut content = column![
        text("SDO 客户端").size(16),
        horizontal_rule(1),
    ].spacing(8).padding(10);

    // SDO read/write panel
    content = content.push(text("SDO 读写:").size(14));
    content = content.push(
        row![
            text("节点:").size(11),
            text_input("1", &app.selected_node.map(|n| n.to_string()).unwrap_or_default())
                .width(40),
            text("Index:").size(11),
            text_input("0x1000", &app.sdo_index)
                .on_input(Message::SdoIndexChanged)
                .width(80),
            text("Sub:").size(11),
            text_input("0", &app.sdo_subindex)
                .on_input(Message::SdoSubindexChanged)
                .width(40),
        ].spacing(4)
    );
    content = content.push(
        row![
            text("数据类型:").size(11),
            text("UNS32").size(11), // TODO: make this a dropdown
            text("值:").size(11),
            text_input("hex bytes", &app.sdo_value)
                .on_input(Message::SdoValueChanged)
                .width(200),
            button(text("读取").size(11)).on_press(Message::SdoRead),
            button(text("写入").size(11)).on_press(Message::SdoWrite),
        ].spacing(4)
    );

    // Quick read buttons
    content = content.push(
        row![
            text("快速读取:").size(11),
            button(text("Device Type (1000)").size(10))
                .on_press(Message::SdoIndexChanged("1000".to_string())),
            button(text("Error Reg (1001)").size(10))
                .on_press(Message::SdoIndexChanged("1001".to_string())),
            button(text("Status Word (6041)").size(10))
                .on_press(Message::SdoIndexChanged("6041".to_string())),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // SDO history
    content = content.push(text("传输历史:").size(14));
    // TODO: add SDO history tracking

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// PDO monitor view.
pub fn pdo_monitor(app: &App) -> Element<'_, Message> {
    let mut content = column![
        row![
            text("PDO 监控").size(16),
            text(format!("({} 帧)", app.pdo_log.len())).size(12),
        ].spacing(8),
        horizontal_rule(1),
    ].spacing(4).padding(10);

    if !app.connected {
        content = content.push(text("未连接。PDO 数据将在连接后显示。").size(12));
    } else if app.pdo_log.is_empty() {
        content = content.push(text("未收到 PDO 帧").size(12));
    } else {
        // Table header
        content = content.push(
            row![
                text("时间").size(10).width(80),
                text("COB-ID").size(10).width(60),
                text("类型").size(10).width(60),
                text("节点").size(10).width(40),
                text("数据").size(10),
            ].spacing(4)
        );
        content = content.push(horizontal_rule(1));

        // Show latest PDOs
        for entry in app.pdo_log.iter().rev().take(200) {
            let hex_data: String = entry.data.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            let (pdo_type, node_id) = classify_pdo(entry.cob_id);

            content = content.push(
                row![
                    text(format!("{}ms", entry.timestamp_ms)).size(10).width(80),
                    text(format!("{:03X}", entry.cob_id)).size(10).width(60),
                    text(pdo_type).size(10).width(60),
                    text(format!("{}", node_id)).size(10).width(40),
                    text(hex_data).size(10),
                ].spacing(4)
            );
        }
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// DS402 control view.
pub fn ds402_control(app: &App) -> Element<'_, Message> {
    let node_id = app.selected_node.unwrap_or(1);
    let node = app.nodes.iter().find(|n| n.node_id == node_id);
    let ds402 = node.map(|n| &n.ds402);

    let mut content = column![
        text(format!("DS402 控制 — 节点 {}", node_id)).size(16),
        horizontal_rule(1),
    ].spacing(8).padding(10);

    // State machine
    content = content.push(text("状态机:").size(14));
    let state_str = ds402.map(|d| d.state.as_str()).unwrap_or("--");
    let status_word = ds402.map(|d| d.status_word).unwrap_or(0);
    content = content.push(text(format!("  当前状态: {}", state_str)).size(12));
    content = content.push(text(format!("  Status Word: 0x{:04X}", status_word)).size(11));

    // Status word bits
    let bits = [
        (0x0001, "Ready To Switch On"),
        (0x0002, "Switched On"),
        (0x0004, "Operation Enabled"),
        (0x0008, "Fault"),
        (0x0010, "Voltage Enabled"),
        (0x0020, "Quick Stop"),
        (0x0040, "Switch On Disabled"),
        (0x0080, "Warning"),
        (0x0200, "Remote"),
        (0x0400, "Target Reached"),
        (0x0800, "Internal Limit Active"),
    ];
    for (mask, name) in bits {
        let active = status_word & mask != 0;
        let marker = if active { "●" } else { "○" };
        content = content.push(text(format!("  {} {} ({:04X})", marker, name, mask)).size(10));
    }

    content = content.push(
        row![
            button(text("Read State").size(11)).on_press(Message::Ds402ReadState(node_id)),
            button(text("Enable").size(11)).on_press(Message::Ds402Enable(node_id)),
            button(text("Fault Reset").size(11)).on_press(Message::Ds402FaultReset(node_id)),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // Operation mode
    content = content.push(text("操作模式:").size(14));
    let mut mode_row = row![].spacing(4);
    for &mode in Ds402Mode::all() {
        let is_selected = app.ds402_state.selected_mode == mode;
        let btn = if is_selected {
            button(text(format!("[{}]", mode.name())).size(10))
                .on_press(Message::Ds402ModeChanged(mode))
        } else {
            button(text(mode.name()).size(10))
                .on_press(Message::Ds402ModeChanged(mode))
        };
        mode_row = mode_row.push(btn);
    }
    content = content.push(mode_row);

    content = content.push(horizontal_rule(1));

    // Position control
    content = content.push(text("位置控制:").size(14));
    content = content.push(
        row![
            text("目标:").size(11),
            text_input("0", &app.ds402_state.target_position)
                .on_input(Message::Ds402TargetPositionChanged)
                .width(100),
            button(text("Set").size(11)).on_press(Message::Ds402SetPosition(node_id)),
            button(text("Read Actual").size(11)).on_press(Message::Ds402ReadPosition(node_id)),
        ].spacing(4)
    );
    let actual_pos = ds402.map(|d| d.actual_position).unwrap_or(0);
    content = content.push(text(format!("  实际位置: {}", actual_pos)).size(11));

    // Mini sparkline for position
    if let Some(ds402) = ds402 {
        if !ds402.position_history.is_empty() {
            let sparkline = render_sparkline(&ds402.position_history, 200, 30);
            content = content.push(sparkline);
        }
    }

    content = content.push(horizontal_rule(1));

    // Velocity control
    content = content.push(text("速度控制:").size(14));
    content = content.push(
        row![
            text("目标:").size(11),
            text_input("0", &app.ds402_state.target_velocity)
                .on_input(Message::Ds402TargetVelocityChanged)
                .width(100),
            button(text("Set").size(11)).on_press(Message::Ds402SetVelocity(node_id)),
            button(text("Read Actual").size(11)).on_press(Message::Ds402ReadVelocity(node_id)),
        ].spacing(4)
    );
    let actual_vel = ds402.map(|d| d.actual_velocity).unwrap_or(0);
    content = content.push(text(format!("  实际速度: {}", actual_vel)).size(11));

    // Mini sparkline for velocity
    if let Some(ds402) = ds402 {
        if !ds402.velocity_history.is_empty() {
            let sparkline = render_sparkline(&ds402.velocity_history, 200, 30);
            content = content.push(sparkline);
        }
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// EMCY log view.
pub fn emcy_log(app: &App) -> Element<'_, Message> {
    let mut content = column![
        row![
            text("EMCY 日志").size(16),
            text(format!("({} 条)", app.emcy_log.len())).size(12),
        ].spacing(8),
        horizontal_rule(1),
    ].spacing(4).padding(10);

    if app.emcy_log.is_empty() {
        content = content.push(text("无紧急错误记录").size(12));
    } else {
        // Table header
        content = content.push(
            row![
                text("时间").size(10).width(80),
                text("节点").size(10).width(40),
                text("错误代码").size(10).width(60),
                text("错误寄存器").size(10).width(60),
                text("描述").size(10),
            ].spacing(4)
        );
        content = content.push(horizontal_rule(1));

        for emcy in app.emcy_log.iter().rev().take(100) {
            content = content.push(
                row![
                    text(format!("{}ms", emcy.timestamp_ms)).size(10).width(80),
                    text(format!("{}", emcy.node_id)).size(10).width(40),
                    text(format!("0x{:04X}", emcy.error_code)).size(10).width(60),
                    text(format!("0x{:02X}", emcy.error_register)).size(10).width(60),
                    text(emcy.error_description()).size(10),
                ].spacing(4)
            );
        }
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// Heartbeat monitor view.
pub fn heartbeat_monitor(app: &App) -> Element<'_, Message> {
    let mut content = column![
        text("心跳监控").size(16),
        horizontal_rule(1),
    ].spacing(8).padding(10);

    if app.heartbeat_status.is_empty() {
        content = content.push(text("无心跳数据").size(12));
    } else {
        // Table header
        content = content.push(
            row![
                text("节点").size(10).width(40),
                text("生产者周期").size(10).width(80),
                text("最后心跳").size(10).width(80),
                text("状态").size(10),
            ].spacing(4)
        );
        content = content.push(horizontal_rule(1));

        for hb in &app.heartbeat_status {
            let period = hb.producer_period_ms
                .map(|p| format!("{}ms", p))
                .unwrap_or_else(|| "-".to_string());
            let last = hb.last_heartbeat_ms
                .map(|t| format!("{}ms", t))
                .unwrap_or_else(|| "-".to_string());

            content = content.push(
                row![
                    text(format!("{}", hb.node_id)).size(11).width(40),
                    text(period).size(11).width(80),
                    text(last).size(11).width(80),
                    text(hb.status_text()).size(11),
                ].spacing(4)
            );
        }
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// Sync management view.
pub fn sync_management(app: &App) -> Element<'_, Message> {
    let sync = &app.sync_status;

    let mut content = column![
        text("同步管理").size(16),
        horizontal_rule(1),
    ].spacing(8).padding(10);

    // SYNC producer
    content = content.push(text("SYNC 生产者:").size(14));
    content = content.push(text(format!("  状态: {}", if sync.producer_enabled { "已启用" } else { "未启用" })).size(12));
    content = content.push(text(format!("  周期: {} μs", sync.producer_period_us)).size(12));

    content = content.push(horizontal_rule(1));

    // SYNC consumer
    content = content.push(text("SYNC 消费者:").size(14));
    content = content.push(text(format!("  消费者数量: {}", sync.consumer_count)).size(12));
    let last_sync = sync.last_sync_ms
        .map(|t| format!("{}ms", t))
        .unwrap_or_else(|| "-".to_string());
    content = content.push(text(format!("  最后同步: {}", last_sync)).size(12));

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

/// Detail panel (right side).
pub fn detail_panel(app: &App) -> Element<'_, Message> {
    let mut content = column![
        row![
            text("详情").size(14),
            button(text("◀").size(10)).on_press(Message::ToggleDetailPanel),
        ].spacing(8),
        horizontal_rule(1),
    ].spacing(4).padding(8);

    if let Some(node_id) = app.selected_node {
        let node = app.nodes.iter().find(|n| n.node_id == node_id);

        if let Some(node) = node {
            // Node info
            content = content.push(text(format!("节点 {}", node.node_id)).size(13));
            content = content.push(text(format!("NMT: {}", node.nmt_state.as_str())).size(11));

            if let Some(dt) = node.device_type {
                content = content.push(text(format!("Device Type: 0x{:08X}", dt)).size(10));
            }
            if let Some(vid) = node.vendor_id {
                content = content.push(text(format!("Vendor ID: 0x{:08X}", vid)).size(10));
            }

            content = content.push(horizontal_rule(1));

            // OD cache
            content = content.push(text("OD 缓存:").size(12));
            if node.od_cache.is_empty() {
                content = content.push(text("  (空)").size(10));
            }
            for ((idx, sub), entry) in &node.od_cache {
                let name = entry.name.as_deref().unwrap_or("");
                let type_str = entry.data_type.as_deref().unwrap_or("");
                let line = if name.is_empty() {
                    format!("  {:04X}:{:02X} = {} {}", idx, sub, entry.value, type_str)
                } else {
                    format!("  {:04X}:{:02X} ({}) = {} {}", idx, sub, name, entry.value, type_str)
                };
                content = content.push(text(line).size(10));
            }

            content = content.push(horizontal_rule(1));

            // DS402 quick status
            if !node.ds402.state.is_empty() {
                content = content.push(text("DS402:").size(12));
                content = content.push(text(format!("  状态: {}", node.ds402.state)).size(10));
                content = content.push(text(format!("  Status: 0x{:04X}", node.ds402.status_word)).size(10));
                content = content.push(text(format!("  位置: {}", node.ds402.actual_position)).size(10));
                content = content.push(text(format!("  速度: {}", node.ds402.actual_velocity)).size(10));
            }
        } else {
            content = content.push(text(format!("节点 {} (未发现)", node_id)).size(11));
        }
    } else {
        content = content.push(text("选择一个节点查看详情").size(11));
    }

    container(scrollable(content))
        .width(200)
        .height(Length::Fill)
        .into()
}

/// Connection dialog overlay.
pub fn connection_dialog(dialog: &ConnectionDialog) -> Element<'_, Message> {
    let mut content = column![
        text("连接到 CAN 总线").size(16),
        horizontal_rule(1),
    ].spacing(8).padding(16);

    // Backend selection
    content = content.push(text("后端:").size(12));
    let mut backend_row = row![].spacing(4);
    for b in CanBackend::all() {
        let label = b.name();
        let is_selected = *b == dialog.selected_backend;
        let btn: Element<'_, Message> = if is_selected {
            button(text(format!("[{}]", label)).size(11))
                .on_press(Message::ConnectionBackendChanged(*b))
                .into()
        } else {
            button(text(label).size(11))
                .on_press(Message::ConnectionBackendChanged(*b))
                .into()
        };
        backend_row = backend_row.push(btn);
    }
    content = content.push(backend_row);

    // Channel input
    content = content.push(
        row![
            text("Channel:").size(11),
            text_input("can0", &dialog.channel)
                .on_input(Message::ConnectionChannelChanged)
                .width(120),
        ].spacing(4)
    );

    // Bitrate input
    content = content.push(
        row![
            text("Bitrate:").size(11),
            text_input("500000", &dialog.bitrate)
                .on_input(Message::ConnectionBitrateChanged)
                .width(120),
        ].spacing(4)
    );

    // Node ID input
    content = content.push(
        row![
            text("Node ID:").size(11),
            text_input("0", &dialog.node_id)
                .on_input(Message::ConnectionNodeIdChanged)
                .width(60),
            text("(0 = master)").size(9),
        ].spacing(4)
    );

    content = content.push(horizontal_rule(1));

    // Action buttons
    content = content.push(
        row![
            button(text("连接").size(12))
                .on_press(Message::ConnectionConnect),
            button(text("取消").size(12))
                .on_press(Message::HideConnectionDialog),
        ].spacing(8)
    );

    container(content)
        .width(400)
        .padding(4)
        .into()
}

/// Classify a PDO COB-ID to type and node ID.
fn classify_pdo(cob_id: u16) -> (String, u8) {
    match cob_id {
        0x180..=0x1FF => ("TPDO1".to_string(), (cob_id - 0x180) as u8),
        0x200..=0x27F => ("RPDO1".to_string(), (cob_id - 0x200) as u8),
        0x280..=0x2FF => ("TPDO2".to_string(), (cob_id - 0x280) as u8),
        0x300..=0x37F => ("RPDO2".to_string(), (cob_id - 0x300) as u8),
        0x380..=0x3FF => ("TPDO3".to_string(), (cob_id - 0x380) as u8),
        0x400..=0x47F => ("RPDO3".to_string(), (cob_id - 0x400) as u8),
        0x480..=0x4FF => ("TPDO4".to_string(), (cob_id - 0x480) as u8),
        0x500..=0x57F => ("RPDO4".to_string(), (cob_id - 0x500) as u8),
        _ => ("?".to_string(), 0),
    }
}

/// Classify COB-ID to human-readable description.
fn classify_cob_id(cob_id: u16) -> String {
    match cob_id {
        0x000 => "NMT".to_string(),
        0x080 => "SYNC".to_string(),
        0x081..=0x0FF => format!("EMCY node {}", cob_id - 0x080),
        0x100..=0x17F => "TIME".to_string(),
        0x180..=0x1FF => format!("TPDO1 node {}", cob_id - 0x180),
        0x200..=0x27F => format!("RPDO1 node {}", cob_id - 0x200),
        0x280..=0x2FF => format!("TPDO2 node {}", cob_id - 0x280),
        0x300..=0x37F => format!("RPDO2 node {}", cob_id - 0x300),
        0x380..=0x3FF => format!("TPDO3 node {}", cob_id - 0x380),
        0x400..=0x47F => format!("RPDO3 node {}", cob_id - 0x400),
        0x480..=0x4FF => format!("TPDO4 node {}", cob_id - 0x480),
        0x500..=0x57F => format!("RPDO4 node {}", cob_id - 0x500),
        0x580..=0x5FF => format!("SDO server node {}", cob_id - 0x580),
        0x600..=0x67F => format!("SDO client node {}", cob_id - 0x600),
        0x700..=0x77F => format!("HEARTBEAT node {}", cob_id - 0x700),
        _ => "Unknown".to_string(),
    }
}

/// Decode COB-ID to function and node ID.
fn decode_cob_id(cob_id: u16) -> (String, u8) {
    match cob_id {
        0x000 => ("NMT".to_string(), 0),
        0x080 => ("SYNC".to_string(), 0),
        0x081..=0x0FF => ("EMCY".to_string(), (cob_id - 0x080) as u8),
        0x100..=0x17F => ("TIME".to_string(), 0),
        0x180..=0x1FF => ("TPDO1".to_string(), (cob_id - 0x180) as u8),
        0x200..=0x27F => ("RPDO1".to_string(), (cob_id - 0x200) as u8),
        0x280..=0x2FF => ("TPDO2".to_string(), (cob_id - 0x280) as u8),
        0x300..=0x37F => ("RPDO2".to_string(), (cob_id - 0x300) as u8),
        0x380..=0x3FF => ("TPDO3".to_string(), (cob_id - 0x380) as u8),
        0x400..=0x47F => ("RPDO3".to_string(), (cob_id - 0x400) as u8),
        0x480..=0x4FF => ("TPDO4".to_string(), (cob_id - 0x480) as u8),
        0x500..=0x57F => ("RPDO4".to_string(), (cob_id - 0x500) as u8),
        0x580..=0x5FF => ("SDO Server".to_string(), (cob_id - 0x580) as u8),
        0x600..=0x67F => ("SDO Client".to_string(), (cob_id - 0x600) as u8),
        0x700..=0x77F => ("Heartbeat".to_string(), (cob_id - 0x700) as u8),
        _ => ("Unknown".to_string(), 0),
    }
}

/// Render a simple sparkline using text.
fn render_sparkline(data: &[i32], _width: u32, _height: u32) -> Element<'_, Message> {
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

    // Normalize to 0-10
    let normalized: Vec<u8> = if range == 0 {
        vec![5; display_data.len()]
    } else {
        display_data.iter()
            .map(|&v| ((v - min) * 10 / range) as u8)
            .collect()
    };

    // Build sparkline string
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let sparkline: String = normalized.iter()
        .map(|&n| chars[n.min(7) as usize])
        .collect();

    column![
        text(format!("{} - {}", min, max)).size(9),
        text(sparkline).size(12),
    ]
    .into()
}
