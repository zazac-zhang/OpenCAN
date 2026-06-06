//! Tab routing types.

/// Primary tab (protocol layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrimaryTab {
    #[default]
    CanBus,
    CanOpen,
}

impl PrimaryTab {
    /// Display name for this primary tab.
    pub fn name(&self) -> &'static str {
        match self {
            Self::CanBus => "CAN 总线",
            Self::CanOpen => "CANOpen 协议",
        }
    }

    /// All primary tabs.
    pub fn all() -> &'static [PrimaryTab] {
        &[Self::CanBus, Self::CanOpen]
    }
}

/// Secondary tab routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    // CAN Bus tabs
    FrameMonitor,
    BusStatistics,
    ErrorFrames,
    // CANOpen tabs
    NetworkManagement,
    SdoClient,
    PdoMonitor,
    Ds402Control,
    EmcyLog,
    HeartbeatMonitor,
    SyncManagement,
}

impl Default for Tab {
    fn default() -> Self {
        Self::FrameMonitor
    }
}

impl Tab {
    /// Get the primary tab for this secondary tab.
    pub fn primary(&self) -> PrimaryTab {
        match self {
            Self::FrameMonitor | Self::BusStatistics | Self::ErrorFrames => PrimaryTab::CanBus,
            Self::NetworkManagement | Self::SdoClient | Self::PdoMonitor | Self::Ds402Control
            | Self::EmcyLog | Self::HeartbeatMonitor | Self::SyncManagement => PrimaryTab::CanOpen,
        }
    }

    /// Get all tabs for a primary tab.
    pub fn for_primary(primary: PrimaryTab) -> &'static [Tab] {
        match primary {
            PrimaryTab::CanBus => &[Self::FrameMonitor, Self::BusStatistics, Self::ErrorFrames],
            PrimaryTab::CanOpen => &[
                Self::NetworkManagement,
                Self::SdoClient,
                Self::PdoMonitor,
                Self::Ds402Control,
                Self::EmcyLog,
                Self::HeartbeatMonitor,
                Self::SyncManagement,
            ],
        }
    }

    /// Get display name for this tab.
    pub fn name(&self) -> &'static str {
        match self {
            Self::FrameMonitor => "帧监控",
            Self::BusStatistics => "总线统计",
            Self::ErrorFrames => "错误帧",
            Self::NetworkManagement => "网络管理",
            Self::SdoClient => "SDO 客户端",
            Self::PdoMonitor => "PDO 监控",
            Self::Ds402Control => "DS402 控制",
            Self::EmcyLog => "EMCY 日志",
            Self::HeartbeatMonitor => "心跳监控",
            Self::SyncManagement => "同步管理",
        }
    }

    /// Get short description for this tab.
    pub fn description(&self) -> &'static str {
        match self {
            Self::FrameMonitor => "实时 CAN/CAN FD 帧列表，Wireshark 风格三面板",
            Self::BusStatistics => "总线负载、帧率、错误计数统计",
            Self::ErrorFrames => "CAN 错误帧详情和错误计数器",
            Self::NetworkManagement => "NMT 状态管理、节点扫描和控制",
            Self::SdoClient => "SDO 读写面板、传输历史",
            Self::PdoMonitor => "TPDO/RPDO 数据表格和映射解析",
            Self::Ds402Control => "DS402 状态机、操作模式、运动控制",
            Self::EmcyLog => "紧急错误记录和错误代码解析",
            Self::HeartbeatMonitor => "节点心跳状态监控",
            Self::SyncManagement => "SYNC 生产者/消费者配置",
        }
    }
}
