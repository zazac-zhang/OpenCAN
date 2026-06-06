//! CAN log and filter types.

use std::fmt;

/// CAN log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp_ms: u64,
    pub cob_id: u16,
    pub data: [u8; 8],
    pub dlc: u8,
    pub direction: Direction,
    pub description: String,
}

/// Frame direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Rx,
    Tx,
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rx => "Rx",
            Self::Tx => "Tx",
        }
    }
}

impl LogEntry {
    pub fn new(timestamp_ms: u64, cob_id: u16, data: [u8; 8]) -> Self {
        Self {
            timestamp_ms,
            cob_id,
            data,
            dlc: 8,
            direction: Direction::Rx,
            description: String::new(),
        }
    }

    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    pub fn with_direction(mut self, dir: Direction) -> Self {
        self.direction = dir;
        self
    }

    pub fn with_dlc(mut self, dlc: u8) -> Self {
        self.dlc = dlc;
        self
    }

    /// Get hex data string.
    pub fn hex_data(&self) -> String {
        self.data[..self.dlc as usize]
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get ASCII representation of data.
    pub fn ascii_data(&self) -> String {
        self.data[..self.dlc as usize]
            .iter()
            .map(|&b| if b >= 0x20 && b < 0x7F { b as char } else { '.' })
            .collect()
    }

    /// Get hex dump string (hex + ASCII).
    pub fn hex_dump(&self) -> String {
        let hex: Vec<String> = self.data[..self.dlc as usize]
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect();
        let ascii = self.ascii_data();
        format!("{:<47}  {}", hex.join(" "), ascii)
    }

    /// Get function code from COB-ID.
    pub fn function_code(&self) -> u16 {
        (self.cob_id >> 7) & 0x0F
    }

    /// Get node ID from COB-ID.
    pub fn node_id(&self) -> u8 {
        (self.cob_id & 0x7F) as u8
    }

    /// Check if this is an SDO frame.
    pub fn is_sdo(&self) -> bool {
        (0x580..=0x67F).contains(&self.cob_id)
    }

    /// Check if this is a PDO frame.
    pub fn is_pdo(&self) -> bool {
        (0x180..=0x57F).contains(&self.cob_id)
    }

    /// Check if this is an EMCY frame.
    pub fn is_emcy(&self) -> bool {
        (0x081..=0x0FF).contains(&self.cob_id)
    }

    /// Check if this is a heartbeat frame.
    pub fn is_heartbeat(&self) -> bool {
        (0x700..=0x77F).contains(&self.cob_id)
    }

    /// Check if this is an NMT frame.
    pub fn is_nmt(&self) -> bool {
        self.cob_id == 0x000
    }

    /// Check if this is a SYNC frame.
    pub fn is_sync(&self) -> bool {
        self.cob_id == 0x080
    }

    /// Get timestamp as formatted string.
    pub fn timestamp_str(&self) -> String {
        let sec = self.timestamp_ms / 1000;
        let ms = self.timestamp_ms % 1000;
        format!("{:3}.{:03}", sec, ms)
    }
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {:03X} [{}] {}",
            self.timestamp_str(),
            self.cob_id,
            self.dlc,
            self.hex_data()
        )
    }
}

/// CAN log filter state.
#[derive(Debug, Clone)]
pub struct LogFilter {
    pub text: String,
    pub show_nmt: bool,
    pub show_sync: bool,
    pub show_sdo: bool,
    pub show_pdo: bool,
    pub show_heartbeat: bool,
    pub show_emcy: bool,
    pub show_time: bool,
    pub show_other: bool,
    /// Min CAN ID filter.
    pub min_cob_id: Option<u16>,
    /// Max CAN ID filter.
    pub max_cob_id: Option<u16>,
    /// Filter by node ID.
    pub node_id_filter: Option<u8>,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            text: String::new(),
            show_nmt: true,
            show_sync: true,
            show_sdo: true,
            show_pdo: true,
            show_heartbeat: true,
            show_emcy: true,
            show_time: true,
            show_other: true,
            min_cob_id: None,
            max_cob_id: None,
            node_id_filter: None,
        }
    }
}

impl LogFilter {
    /// Check if a log entry passes this filter.
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Text filter
        if !self.text.is_empty() {
            let query = self.text.to_lowercase();
            let haystack = format!("{:03X} {} {}",
                entry.cob_id,
                entry.hex_data(),
                entry.description
            ).to_lowercase();
            if !haystack.contains(&query) {
                return false;
            }
        }

        // COB-ID range filter
        if let Some(min) = self.min_cob_id {
            if entry.cob_id < min {
                return false;
            }
        }
        if let Some(max) = self.max_cob_id {
            if entry.cob_id > max {
                return false;
            }
        }

        // Node ID filter
        if let Some(filter_node) = self.node_id_filter {
            let node_id = entry.node_id();
            if node_id != filter_node && node_id != 0 {
                return false;
            }
        }

        // Protocol type filter
        let cob = entry.cob_id;
        match cob {
            0x000 => self.show_nmt,
            0x080 => self.show_sync,
            0x081..=0x0FF => self.show_emcy,
            0x100..=0x17F => self.show_time,
            0x180..=0x57F => self.show_pdo,
            0x580..=0x67F => self.show_sdo,
            0x700..=0x77F => self.show_heartbeat,
            _ => self.show_other,
        }
    }

    /// Get count of active filters.
    pub fn active_filter_count(&self) -> usize {
        let mut count = 0;
        if !self.text.is_empty() { count += 1; }
        if self.min_cob_id.is_some() { count += 1; }
        if self.max_cob_id.is_some() { count += 1; }
        if self.node_id_filter.is_some() { count += 1; }
        count
    }

    /// Get summary of active filters.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.text.is_empty() {
            parts.push(format!("text:{}", self.text));
        }
        if let Some(min) = self.min_cob_id {
            parts.push(format!(">={:03X}", min));
        }
        if let Some(max) = self.max_cob_id {
            parts.push(format!("<={:03X}", max));
        }
        if let Some(node) = self.node_id_filter {
            parts.push(format!("node:{}", node));
        }
        if parts.is_empty() {
            "No filters".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Reset all filters.
    pub fn reset(&mut self) {
        self.text.clear();
        self.show_nmt = true;
        self.show_sync = true;
        self.show_sdo = true;
        self.show_pdo = true;
        self.show_heartbeat = true;
        self.show_emcy = true;
        self.show_time = true;
        self.show_other = true;
        self.min_cob_id = None;
        self.max_cob_id = None;
        self.node_id_filter = None;
    }
}

/// Log recording state.
#[derive(Debug, Clone)]
pub struct LogRecorder {
    pub recording: bool,
    pub entries: Vec<LogEntry>,
    pub max_entries: usize,
    pub started_at: Option<u64>,
}

impl Default for LogRecorder {
    fn default() -> Self {
        Self {
            recording: false,
            entries: Vec::new(),
            max_entries: 100000,
            started_at: None,
        }
    }
}

impl LogRecorder {
    /// Start recording.
    pub fn start(&mut self, timestamp: u64) {
        self.recording = true;
        self.entries.clear();
        self.started_at = Some(timestamp);
    }

    /// Stop recording.
    pub fn stop(&mut self) {
        self.recording = false;
    }

    /// Add entry to recording.
    pub fn push(&mut self, entry: LogEntry) {
        if self.recording {
            if self.entries.len() >= self.max_entries {
                self.entries.remove(0);
            }
            self.entries.push(entry);
        }
    }

    /// Get recorded entries count.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Export to CSV string.
    pub fn to_csv(&self) -> String {
        let mut csv = String::from("timestamp_ms,cob_id,dlc,data,direction,description\n");
        for entry in &self.entries {
            csv.push_str(&format!("{},{:03X},{},\"{}\",{},\"{}\"\n",
                entry.timestamp_ms,
                entry.cob_id,
                entry.dlc,
                entry.hex_data(),
                entry.direction.as_str(),
                entry.description
            ));
        }
        csv
    }

    /// Import from CSV string.
    pub fn from_csv(csv: &str) -> Vec<LogEntry> {
        let mut entries = Vec::new();
        for line in csv.lines().skip(1) {
            let parts: Vec<&str> = line.splitn(5, ',').collect();
            if parts.len() >= 4 {
                if let (Ok(ts), Ok(cob_id)) = (
                    parts[0].parse::<u64>(),
                    u16::from_str_radix(parts[1].trim_start_matches("0x"), 16),
                ) {
                    let dlc: u8 = parts[2].parse().unwrap_or(8);
                    let data_str = parts[3].trim_matches('"');
                    let mut data = [0u8; 8];
                    for (i, byte) in data_str.split_whitespace().enumerate() {
                        if i < 8 {
                            if let Ok(b) = u8::from_str_radix(byte, 16) {
                                data[i] = b;
                            }
                        }
                    }
                    entries.push(LogEntry {
                        timestamp_ms: ts,
                        cob_id,
                        data,
                        dlc,
                        direction: Direction::Rx,
                        description: String::new(),
                    });
                }
            }
        }
        entries
    }
}
