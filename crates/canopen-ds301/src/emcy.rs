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

/// Emergency handler — collects and stores emergency events.
pub struct EmergencyHandler {
    events: Vec<EmergencyEvent>,
    max_events: usize,
}

impl EmergencyHandler {
    pub fn new(max_events: usize) -> Self {
        Self { events: Vec::new(), max_events }
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
        self.events.iter().filter(|e| e.node_id == node_id).collect()
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
