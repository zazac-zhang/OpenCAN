//! Connection state types.

/// Available CAN backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanBackend {
    Mock,
    SocketCan,
    Kvaser,
    Pcan,
    Zlg,
}

impl CanBackend {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mock => "Mock (Testing)",
            Self::SocketCan => "SocketCAN (Linux)",
            Self::Kvaser => "Kvaser",
            Self::Pcan => "PCAN",
            Self::Zlg => "ZLG",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Mock => "模拟后端，用于测试和开发",
            Self::SocketCan => "Linux SocketCAN 接口",
            Self::Kvaser => "Kvaser CAN 适配器",
            Self::Pcan => "PEAK PCAN 适配器",
            Self::Zlg => "ZLG USBCAN 适配器",
        }
    }

    /// All available backends.
    pub fn all() -> &'static [CanBackend] {
        &[Self::Mock, Self::SocketCan, Self::Kvaser, Self::Pcan, Self::Zlg]
    }

    /// Check if this backend requires a channel.
    pub fn requires_channel(&self) -> bool {
        match self {
            Self::Mock => false,
            _ => true,
        }
    }

    /// Get default channel for this backend.
    pub fn default_channel(&self) -> &'static str {
        match self {
            Self::Mock => "",
            Self::SocketCan => "can0",
            Self::Kvaser => "0",
            Self::Pcan => "PCAN_USBBUS1",
            Self::Zlg => "0",
        }
    }

    /// Get supported bitrates for this backend.
    pub fn supported_bitrates(&self) -> &'static [u32] {
        match self {
            Self::Mock => &[125000, 250000, 500000, 1000000],
            Self::SocketCan => &[125000, 250000, 500000, 1000000],
            Self::Kvaser => &[125000, 250000, 500000, 1000000],
            Self::Pcan => &[125000, 250000, 500000, 1000000],
            Self::Zlg => &[125000, 250000, 500000, 1000000],
        }
    }
}

/// Connection dialog state.
#[derive(Debug)]
pub struct ConnectionDialog {
    pub visible: bool,
    pub selected_backend: CanBackend,
    pub channel: String,
    pub bitrate: String,
    pub node_id: String,
    /// Connection error message.
    pub error: Option<String>,
    /// Whether connection is in progress.
    pub connecting: bool,
}

impl Default for ConnectionDialog {
    fn default() -> Self {
        Self {
            visible: false,
            selected_backend: CanBackend::Mock,
            channel: "can0".to_string(),
            bitrate: "500000".to_string(),
            node_id: "0".to_string(),
            error: None,
            connecting: false,
        }
    }
}

impl ConnectionDialog {
    /// Show the dialog.
    pub fn show(&mut self) {
        self.visible = true;
        self.error = None;
        self.connecting = false;
    }

    /// Hide the dialog.
    pub fn hide(&mut self) {
        self.visible = false;
        self.error = None;
        self.connecting = false;
    }

    /// Set error message.
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.connecting = false;
    }

    /// Clear error message.
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Set connecting state.
    pub fn set_connecting(&mut self) {
        self.connecting = true;
        self.error = None;
    }

    /// Get parsed bitrate.
    pub fn parsed_bitrate(&self) -> u32 {
        self.bitrate.parse().unwrap_or(500000)
    }

    /// Get parsed node ID.
    pub fn parsed_node_id(&self) -> u8 {
        self.node_id.parse().unwrap_or(0)
    }

    /// Validate dialog inputs.
    pub fn validate(&self) -> Result<(), String> {
        if self.selected_backend.requires_channel() && self.channel.is_empty() {
            return Err("Channel is required".to_string());
        }

        let bitrate = self.parsed_bitrate();
        if !self.selected_backend.supported_bitrates().contains(&bitrate) {
            return Err(format!("Unsupported bitrate: {}", bitrate));
        }

        let node_id = self.parsed_node_id();
        if node_id > 127 {
            return Err("Node ID must be 0-127".to_string());
        }

        Ok(())
    }
}

/// Connection info.
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub backend: CanBackend,
    pub channel: String,
    pub bitrate: u32,
    pub node_id: u8,
    pub connected_at: Option<u64>,
}

impl ConnectionInfo {
    pub fn new(backend: CanBackend, channel: String, bitrate: u32, node_id: u8) -> Self {
        Self {
            backend,
            channel,
            bitrate,
            node_id,
            connected_at: None,
        }
    }

    pub fn summary(&self) -> String {
        format!("{} @ {}kbps (Node {})",
            self.backend.name(),
            self.bitrate / 1000,
            self.node_id
        )
    }
}
