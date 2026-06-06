//! CAN bus statistics and error types.

/// Bus statistics.
#[derive(Debug, Clone)]
pub struct BusStatistics {
    pub bus_load_percent: f32,
    pub peak_bus_load: f32,
    pub frame_rate: u32,
    pub peak_frame_rate: u32,
    pub tx_frames: u64,
    pub rx_frames: u64,
    pub tx_errors: u32,
    pub rx_errors: u32,
    pub bitrate: u32,
    pub bus_state: BusState,
    pub frame_count: u64,
    pub error_frame_count: u32,
    pub last_error: Option<String>,
    /// Rolling frame rate samples (last 60 seconds).
    pub frame_rate_history: Vec<u32>,
    /// Rolling bus load samples (last 60 seconds).
    pub bus_load_history: Vec<f32>,
}

impl Default for BusStatistics {
    fn default() -> Self {
        Self {
            bus_load_percent: 0.0,
            peak_bus_load: 0.0,
            frame_rate: 0,
            peak_frame_rate: 0,
            tx_frames: 0,
            rx_frames: 0,
            tx_errors: 0,
            rx_errors: 0,
            bitrate: 500000,
            bus_state: BusState::Unknown,
            frame_count: 0,
            error_frame_count: 0,
            last_error: None,
            frame_rate_history: Vec::new(),
            bus_load_history: Vec::new(),
        }
    }
}

impl BusStatistics {
    /// Update frame rate sample.
    pub fn update_frame_rate(&mut self, rate: u32) {
        self.frame_rate = rate;
        if rate > self.peak_frame_rate {
            self.peak_frame_rate = rate;
        }
        self.frame_rate_history.push(rate);
        if self.frame_rate_history.len() > 60 {
            self.frame_rate_history.remove(0);
        }
    }

    /// Update bus load sample.
    pub fn update_bus_load(&mut self, load: f32) {
        self.bus_load_percent = load;
        if load > self.peak_bus_load {
            self.peak_bus_load = load;
        }
        self.bus_load_history.push(load);
        if self.bus_load_history.len() > 60 {
            self.bus_load_history.remove(0);
        }
    }

    /// Increment TX frame count.
    pub fn inc_tx(&mut self) {
        self.tx_frames += 1;
        self.frame_count += 1;
    }

    /// Increment RX frame count.
    pub fn inc_rx(&mut self) {
        self.rx_frames += 1;
        self.frame_count += 1;
    }

    /// Increment TX error count.
    pub fn inc_tx_error(&mut self) {
        self.tx_errors += 1;
    }

    /// Increment RX error count.
    pub fn inc_rx_error(&mut self) {
        self.rx_errors += 1;
    }

    /// Increment error frame count.
    pub fn inc_error_frame(&mut self) {
        self.error_frame_count += 1;
    }

    /// Record an error.
    pub fn record_error(&mut self, error: String) {
        self.last_error = Some(error);
    }

    /// Reset statistics.
    pub fn reset(&mut self) {
        self.bus_load_percent = 0.0;
        self.peak_bus_load = 0.0;
        self.frame_rate = 0;
        self.peak_frame_rate = 0;
        self.tx_frames = 0;
        self.rx_frames = 0;
        self.tx_errors = 0;
        self.rx_errors = 0;
        self.frame_count = 0;
        self.error_frame_count = 0;
        self.last_error = None;
        self.frame_rate_history.clear();
        self.bus_load_history.clear();
    }

    /// Get average frame rate.
    pub fn avg_frame_rate(&self) -> u32 {
        if self.frame_rate_history.is_empty() {
            return 0;
        }
        let sum: u32 = self.frame_rate_history.iter().sum();
        sum / self.frame_rate_history.len() as u32
    }

    /// Get average bus load.
    pub fn avg_bus_load(&self) -> f32 {
        if self.bus_load_history.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.bus_load_history.iter().sum();
        sum / self.bus_load_history.len() as f32
    }
}

/// CAN bus state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusState {
    Unknown,
    Active,
    Warning,
    Passive,
    BusOff,
}

impl BusState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Active => "Active",
            Self::Warning => "Warning",
            Self::Passive => "Passive",
            Self::BusOff => "Bus Off",
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Warning | Self::Passive | Self::BusOff)
    }
}

/// Error frame.
#[derive(Debug, Clone)]
pub struct ErrorFrame {
    pub timestamp_ms: u64,
    pub error_type: ErrorType,
    pub error_flag: u8,
    pub tec: u8,
    pub rec: u8,
    pub description: String,
}

impl ErrorFrame {
    pub fn new(timestamp_ms: u64, error_type: ErrorType) -> Self {
        Self {
            timestamp_ms,
            error_type,
            error_flag: 0,
            tec: 0,
            rec: 0,
            description: String::new(),
        }
    }

    pub fn with_counters(mut self, tec: u8, rec: u8) -> Self {
        self.tec = tec;
        self.rec = rec;
        self
    }

    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    pub fn timestamp_str(&self) -> String {
        let sec = self.timestamp_ms / 1000;
        let ms = self.timestamp_ms % 1000;
        format!("{:3}.{:03}", sec, ms)
    }
}

/// CAN error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    BitError,
    StuffError,
    CrcError,
    FormError,
    AckError,
    Other,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BitError => "Bit Error",
            Self::StuffError => "Stuff Error",
            Self::CrcError => "CRC Error",
            Self::FormError => "Form Error",
            Self::AckError => "ACK Error",
            Self::Other => "Other",
        }
    }

    pub fn short_str(&self) -> &'static str {
        match self {
            Self::BitError => "BIT",
            Self::StuffError => "STF",
            Self::CrcError => "CRC",
            Self::FormError => "FRM",
            Self::AckError => "ACK",
            Self::Other => "OTH",
        }
    }

    /// Parse error type from flags.
    pub fn from_flags(flags: u8) -> Self {
        match flags & 0x07 {
            0x00 => Self::BitError,
            0x01 => Self::StuffError,
            0x02 => Self::CrcError,
            0x03 => Self::FormError,
            0x04 => Self::AckError,
            _ => Self::Other,
        }
    }
}

/// CAN error counters.
#[derive(Debug, Clone, Default)]
pub struct ErrorCounters {
    pub tec: u8,
    pub rec: u8,
}

impl ErrorCounters {
    /// Get bus state based on error counters.
    pub fn bus_state(&self) -> BusState {
        if self.tec >= 255 || self.rec >= 255 {
            BusState::BusOff
        } else if self.tec >= 128 || self.rec >= 128 {
            BusState::Passive
        } else if self.tec >= 96 || self.rec >= 96 {
            BusState::Warning
        } else {
            BusState::Active
        }
    }
}
