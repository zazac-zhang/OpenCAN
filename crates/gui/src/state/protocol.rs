//! CANOpen protocol state types (EMCY, Heartbeat, Sync).

/// EMCY log entry.
#[derive(Debug, Clone)]
pub struct EmcyEntry {
    pub timestamp_ms: u64,
    pub node_id: u8,
    pub error_code: u16,
    pub error_register: u8,
    pub data: [u8; 5],
}

impl EmcyEntry {
    pub fn new(timestamp_ms: u64, node_id: u8, error_code: u16, error_register: u8) -> Self {
        Self {
            timestamp_ms,
            node_id,
            error_code,
            error_register,
            data: [0u8; 5],
        }
    }

    pub fn with_data(mut self, data: [u8; 5]) -> Self {
        self.data = data;
        self
    }

    pub fn timestamp_str(&self) -> String {
        let sec = self.timestamp_ms / 1000;
        let ms = self.timestamp_ms % 1000;
        format!("{:3}.{:03}", sec, ms)
    }

    /// Get error description based on error code.
    pub fn error_description(&self) -> String {
        match self.error_code {
            0x0000 => "Error Reset / No Error".to_string(),
            0x1000 => "Generic Error".to_string(),
            0x2000 => "Current Error".to_string(),
            0x2100 => "Current, device input side".to_string(),
            0x2200 => "Current inside the device".to_string(),
            0x2300 => "Current, device output side".to_string(),
            0x3000 => "Voltage Error".to_string(),
            0x3100 => "Mains Voltage Error".to_string(),
            0x3200 => "Voltage inside the device".to_string(),
            0x3300 => "Output Voltage Error".to_string(),
            0x4000 => "Temperature Error".to_string(),
            0x4100 => "Ambient Temperature".to_string(),
            0x4200 => "Device Temperature".to_string(),
            0x5000 => "Hardware Error".to_string(),
            0x6000 => "Software Error".to_string(),
            0x6100 => "Internal Software Error".to_string(),
            0x6200 => "User Software Error".to_string(),
            0x6300 => "Data Set Error".to_string(),
            0x7000 => "Additional Modules Error".to_string(),
            0x8000 => "Monitoring Error".to_string(),
            0x8100 => "Communication Error".to_string(),
            0x8110 => "CAN Overrun (objects lost)".to_string(),
            0x8120 => "CAN Error Passive".to_string(),
            0x8130 => "Life Guard Error / Heartbeat Error".to_string(),
            0x8140 => "Recovered from Bus Off".to_string(),
            0x8150 => "CAN-ID Collision".to_string(),
            0x8200 => "Protocol Error".to_string(),
            0x8210 => "PDO not processed due to length error".to_string(),
            0x8220 => "PDO length exceeded".to_string(),
            0x8230 => "DAM MPDO not processed, destination object not available".to_string(),
            0x8240 => "Unexpected SYNC data length".to_string(),
            0x8250 => "RPDO timeout".to_string(),
            0x9000 => "External Error".to_string(),
            0xF000 => "Additional Functions Error".to_string(),
            0xFF00 => "Device Specific Error".to_string(),
            _ => format!("Unknown Error (0x{:04X})", self.error_code),
        }
    }

    /// Get error category.
    pub fn error_category(&self) -> &'static str {
        match self.error_code {
            0x0000 => "No Error",
            0x1000..=0x1FFF => "Generic",
            0x2000..=0x2FFF => "Current",
            0x3000..=0x3FFF => "Voltage",
            0x4000..=0x4FFF => "Temperature",
            0x5000..=0x5FFF => "Hardware",
            0x6000..=0x6FFF => "Software",
            0x7000..=0x7FFF => "Modules",
            0x8000..=0x8FFF => "Monitoring",
            0x9000..=0x9FFF => "External",
            0xF000..=0xFFFF => "Device Specific",
            _ => "Unknown",
        }
    }

    /// Get error register bits as description.
    pub fn error_register_str(&self) -> String {
        let mut parts = Vec::new();
        if self.error_register & 0x01 != 0 { parts.push("Generic"); }
        if self.error_register & 0x02 != 0 { parts.push("Current"); }
        if self.error_register & 0x04 != 0 { parts.push("Voltage"); }
        if self.error_register & 0x08 != 0 { parts.push("Temperature"); }
        if self.error_register & 0x10 != 0 { parts.push("Communication"); }
        if self.error_register & 0x20 != 0 { parts.push("Device Profile"); }
        if self.error_register & 0x40 != 0 { parts.push("Reserved"); }
        if self.error_register & 0x80 != 0 { parts.push("Manufacturer"); }
        if parts.is_empty() {
            "None".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Heartbeat status.
#[derive(Debug, Clone)]
pub struct HeartbeatStatus {
    pub node_id: u8,
    pub producer_period_ms: Option<u32>,
    pub consumer_id: Option<u8>,
    pub consumer_timeout_ms: Option<u32>,
    pub last_heartbeat_ms: Option<u64>,
    pub alive: bool,
    pub missed_count: u32,
}

impl HeartbeatStatus {
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
            producer_period_ms: None,
            consumer_id: None,
            consumer_timeout_ms: None,
            last_heartbeat_ms: None,
            alive: false,
            missed_count: 0,
        }
    }

    pub fn with_producer_period(mut self, period_ms: u32) -> Self {
        self.producer_period_ms = Some(period_ms);
        self
    }

    pub fn status_text(&self) -> &'static str {
        if self.alive {
            "Online"
        } else {
            "Offline"
        }
    }

    pub fn status_indicator(&self) -> &'static str {
        if self.alive {
            "●"
        } else {
            "○"
        }
    }

    /// Check if heartbeat is overdue.
    pub fn is_overdue(&self, current_time_ms: u64) -> bool {
        if let (Some(last), Some(period)) = (self.last_heartbeat_ms, self.producer_period_ms) {
            current_time_ms - last > period as u64 * 2
        } else {
            false
        }
    }

    /// Get time since last heartbeat.
    pub fn time_since_last(&self, current_time_ms: u64) -> Option<u64> {
        self.last_heartbeat_ms.map(|last| current_time_ms - last)
    }

    /// Update heartbeat received.
    pub fn heartbeat_received(&mut self, timestamp_ms: u64) {
        self.last_heartbeat_ms = Some(timestamp_ms);
        self.alive = true;
    }

    /// Mark heartbeat lost.
    pub fn heartbeat_lost(&mut self) {
        self.alive = false;
        self.missed_count += 1;
    }
}

/// Sync status.
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub producer_enabled: bool,
    pub producer_period_us: u32,
    pub consumer_count: u32,
    pub last_sync_ms: Option<u64>,
    pub sync_counter: u64,
    pub sync_overrun: bool,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self {
            producer_enabled: false,
            producer_period_us: 0,
            consumer_count: 0,
            last_sync_ms: None,
            sync_counter: 0,
            sync_overrun: false,
        }
    }
}

impl SyncStatus {
    /// Start SYNC producer.
    pub fn start_producer(&mut self, period_us: u32) {
        self.producer_enabled = true;
        self.producer_period_us = period_us;
    }

    /// Stop SYNC producer.
    pub fn stop_producer(&mut self) {
        self.producer_enabled = false;
    }

    /// Record SYNC received.
    pub fn sync_received(&mut self, timestamp_ms: u64) {
        self.last_sync_ms = Some(timestamp_ms);
        self.sync_counter += 1;
    }

    /// Get SYNC frequency in Hz.
    pub fn frequency_hz(&self) -> f32 {
        if self.producer_period_us == 0 {
            0.0
        } else {
            1_000_000.0 / self.producer_period_us as f32
        }
    }

    /// Get SYNC period in ms.
    pub fn period_ms(&self) -> f32 {
        self.producer_period_us as f32 / 1000.0
    }

    /// Get formatted frequency string.
    pub fn frequency_str(&self) -> String {
        let hz = self.frequency_hz();
        if hz >= 1000.0 {
            format!("{:.1} kHz", hz / 1000.0)
        } else if hz >= 1.0 {
            format!("{:.1} Hz", hz)
        } else {
            format!("{:.3} Hz", hz)
        }
    }
}
