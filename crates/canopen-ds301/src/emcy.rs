//! Emergency handler implementation.

use opencan_canopen_core::frame::EmergencyFrame;

/// Emergency event record.
#[derive(Debug, Clone)]
pub struct EmergencyEvent {
    pub node_id: u8,
    pub error_code: u16,
    pub error_register: u8,
    pub data: [u8; 5],
    pub timestamp: std::time::Instant,
}

impl EmergencyEvent {
    /// Get human-readable description of the error code.
    /// Based on CiA 301 emergency error codes.
    pub fn error_description(&self) -> &'static str {
        emcy_description(self.error_code)
    }

    /// Get a formatted string with error code and description.
    pub fn display_code(&self) -> String {
        format!("0x{:04X}: {}", self.error_code, self.error_description())
    }
}

/// Get human-readable description for an EMCY error code.
/// Based on CiA 301 Table 34 — Emergency error codes.
pub fn emcy_description(code: u16) -> &'static str {
    match code {
        0x0000 => "Error Reset / No Error",
        0x1000 => "Generic Error",
        0x2000 => "Current Error",
        0x2100 => "Current, device input side",
        0x2200 => "Current inside the device",
        0x2300 => "Current, device output side",
        0x3000 => "Voltage Error",
        0x3100 => "Mains Voltage",
        0x3200 => "Voltage inside the device",
        0x3300 => "Output Voltage",
        0x4000 => "Temperature Error",
        0x4100 => "Ambient Temperature",
        0x4200 => "Device Temperature",
        0x5000 => "Device Hardware Error",
        0x6000 => "Device Software Error",
        0x6100 => "Internal Software Error",
        0x6200 => "User Software Error",
        0x6300 => "Data Set Error",
        0x7000 => "Additional Modules Error",
        0x8000 => "Monitoring Error",
        0x8100 => "Communication Error",
        0x8110 => "CAN Overrun (objects lost)",
        0x8120 => "CAN in Error Passive Mode",
        0x8130 => "CAN Heartbeat Error",
        0x8140 => "CAN Transmit COB-ID collision",
        0x8150 => "Bus Off",
        0x8200 => "Protocol Error",
        0x8210 => "PDO not processed",
        0x8220 => "PDO length exceeded",
        0x8230 => "DAM MPDO not processed",
        0x8240 => "Unexpected SYNC data length",
        0x8250 => "RPDO timeout",
        0x9000 => "External Error",
        0xF000 => "Additional Functions Error",
        0xFF00 => "Device Specific Error",
        _ => "Unknown Error Code",
    }
}

/// Emergency handler — collects and stores emergency events.
pub struct EmergencyHandler {
    events: Vec<EmergencyEvent>,
    max_events: usize,
}

impl EmergencyHandler {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }

    /// Record a new emergency event.
    pub fn record(&mut self, frame: &EmergencyFrame) {
        let event = EmergencyEvent {
            node_id: frame.node_id,
            error_code: frame.error_code,
            error_register: frame.error_register,
            data: frame.data,
            timestamp: std::time::Instant::now(),
        };

        self.events.push(event);

        // Trim if exceeds max
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }
    }

    /// Get all recorded events.
    pub fn events(&self) -> &[EmergencyEvent] {
        &self.events
    }

    /// Get events for a specific node.
    pub fn events_for_node(&self, node_id: u8) -> Vec<&EmergencyEvent> {
        self.events
            .iter()
            .filter(|e| e.node_id == node_id)
            .collect()
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emcy_description_known_codes() {
        assert_eq!(emcy_description(0x0000), "Error Reset / No Error");
        assert_eq!(emcy_description(0x1000), "Generic Error");
        assert_eq!(emcy_description(0x2000), "Current Error");
        assert_eq!(emcy_description(0x8100), "Communication Error");
        assert_eq!(emcy_description(0x8130), "CAN Heartbeat Error");
        assert_eq!(emcy_description(0xFF00), "Device Specific Error");
    }

    #[test]
    fn test_emcy_description_unknown_code() {
        assert_eq!(emcy_description(0x1234), "Unknown Error Code");
        assert_eq!(emcy_description(0x0001), "Unknown Error Code");
    }

    #[test]
    fn test_emergency_event_display() {
        let event = EmergencyEvent {
            node_id: 3,
            error_code: 0x8130,
            error_register: 0x01,
            data: [0; 5],
            timestamp: std::time::Instant::now(),
        };
        assert_eq!(event.error_description(), "CAN Heartbeat Error");
        assert_eq!(event.display_code(), "0x8130: CAN Heartbeat Error");
    }

    #[test]
    fn test_emergency_handler_record() {
        let mut handler = EmergencyHandler::new(10);
        let frame = EmergencyFrame {
            node_id: 3,
            error_code: 0x8130,
            error_register: 0x01,
            data: [0, 0, 0, 0, 0],
        };
        handler.record(&frame);

        assert_eq!(handler.events().len(), 1);
        assert_eq!(handler.events()[0].error_code, 0x8130);
    }

    #[test]
    fn test_emergency_handler_max_events() {
        let mut handler = EmergencyHandler::new(3);
        for i in 0..5 {
            let frame = EmergencyFrame {
                node_id: 3,
                error_code: i,
                error_register: 0,
                data: [0; 5],
            };
            handler.record(&frame);
        }
        assert_eq!(handler.events().len(), 3);
        // Oldest events should be removed
        assert_eq!(handler.events()[0].error_code, 2);
    }

    #[test]
    fn test_emergency_handler_events_for_node() {
        let mut handler = EmergencyHandler::new(10);
        handler.record(&EmergencyFrame { node_id: 1, error_code: 0x1000, error_register: 0, data: [0; 5] });
        handler.record(&EmergencyFrame { node_id: 2, error_code: 0x2000, error_register: 0, data: [0; 5] });
        handler.record(&EmergencyFrame { node_id: 1, error_code: 0x3000, error_register: 0, data: [0; 5] });

        let node1_events = handler.events_for_node(1);
        assert_eq!(node1_events.len(), 2);
        assert_eq!(node1_events[0].error_code, 0x1000);
        assert_eq!(node1_events[1].error_code, 0x3000);
    }
}
