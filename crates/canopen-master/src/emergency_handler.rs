//! Emergency Handler — EMCY message processing and error tracking.
//!
//! The `EmergencyHandler` provides:
//! - EMCY message parsing and classification
//! - Error history tracking per node
//! - Error statistics
//! - Error recovery detection

use opencan_canopen_core::frame::EmergencyFrame;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// CANOpen error codes (DS301 Table 61).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Error Reset or No Error.
    NoError,
    /// Generic Error.
    GenericError,
    /// Current Error.
    CurrentError,
    /// Current, device input side.
    CurrentInput,
    /// Current, inside the device.
    CurrentInternal,
    /// Current, device output side.
    CurrentOutput,
    /// Voltage Error.
    VoltageError,
    /// Mains Voltage Error.
    MainsVoltage,
    /// Voltage, inside the device.
    VoltageInternal,
    /// Output Voltage Error.
    VoltageOutput,
    /// Temperature Error.
    TemperatureError,
    /// Ambient Temperature.
    TemperatureAmbient,
    /// Device Temperature.
    TemperatureDevice,
    /// Hardware Error.
    HardwareError,
    /// Software Error.
    SoftwareError,
    /// Software, internal error.
    SoftwareInternal,
    /// Software, user error.
    SoftwareUser,
    /// Data Set Error.
    DataSetError,
    /// Additional modules (vendor-specific).
    AdditionalModules,
    /// Communication Error.
    CommunicationError,
    /// CAN Overrun (objects lost).
    CanOverrun,
    /// CAN Passive Mode.
    CanPassiveMode,
    /// Heartbeat Error.
    HeartbeatError,
    /// Bus Off.
    BusOff,
    /// Bus Warning.
    BusWarning,
    /// Protocol Error.
    ProtocolError,
    /// PDO not received.
    PdoNotReceived,
    /// PDO Length Error.
    PdoLength,
    /// DAM MPE Error.
    DamMpe,
    /// Sync Error.
    SyncError,
    /// External Error.
    ExternalError,
    /// Additional Functions.
    AdditionalFunctions,
    /// Device Profile Specific.
    DeviceProfileSpecific,
    /// Manufacturer Specific.
    ManufacturerSpecific(u16),
    /// Unknown error code.
    Unknown(u16),
}

impl ErrorCode {
    /// Create from raw error code.
    pub fn from_u16(code: u16) -> Self {
        match code {
            0x0000 => Self::NoError,
            0x1000 => Self::GenericError,
            0x2000 => Self::CurrentError,
            0x2100 => Self::CurrentInput,
            0x2200 => Self::CurrentInternal,
            0x2300 => Self::CurrentOutput,
            0x3000 => Self::VoltageError,
            0x3100 => Self::MainsVoltage,
            0x3200 => Self::VoltageInternal,
            0x3300 => Self::VoltageOutput,
            0x4000 => Self::TemperatureError,
            0x4100 => Self::TemperatureAmbient,
            0x4200 => Self::TemperatureDevice,
            0x5000 => Self::HardwareError,
            0x6000 => Self::SoftwareError,
            0x6100 => Self::SoftwareInternal,
            0x6200 => Self::SoftwareUser,
            0x6300 => Self::DataSetError,
            0x7000 => Self::AdditionalModules,
            0x8000 => Self::CommunicationError,
            0x8100 => Self::CanOverrun,
            0x8200 => Self::CanPassiveMode,
            0x8210 => Self::HeartbeatError,
            0x8220 => Self::BusOff,
            0x8230 => Self::BusWarning,
            0x8240 => Self::ProtocolError,
            0x8250 => Self::PdoNotReceived,
            0x8260 => Self::PdoLength,
            0x8270 => Self::DamMpe,
            0x8280 => Self::SyncError,
            0x9000 => Self::ExternalError,
            0xF000 => Self::AdditionalFunctions,
            0xFF00 => Self::DeviceProfileSpecific,
            _ => {
                if code >= 0xFF01 && code <= 0xFFFF {
                    Self::ManufacturerSpecific(code)
                } else {
                    Self::Unknown(code)
                }
            }
        }
    }

    /// Get the raw error code.
    pub fn to_u16(&self) -> u16 {
        match self {
            Self::NoError => 0x0000,
            Self::GenericError => 0x1000,
            Self::CurrentError => 0x2000,
            Self::CurrentInput => 0x2100,
            Self::CurrentInternal => 0x2200,
            Self::CurrentOutput => 0x2300,
            Self::VoltageError => 0x3000,
            Self::MainsVoltage => 0x3100,
            Self::VoltageInternal => 0x3200,
            Self::VoltageOutput => 0x3300,
            Self::TemperatureError => 0x4000,
            Self::TemperatureAmbient => 0x4100,
            Self::TemperatureDevice => 0x4200,
            Self::HardwareError => 0x5000,
            Self::SoftwareError => 0x6000,
            Self::SoftwareInternal => 0x6100,
            Self::SoftwareUser => 0x6200,
            Self::DataSetError => 0x6300,
            Self::AdditionalModules => 0x7000,
            Self::CommunicationError => 0x8000,
            Self::CanOverrun => 0x8100,
            Self::CanPassiveMode => 0x8200,
            Self::HeartbeatError => 0x8210,
            Self::BusOff => 0x8220,
            Self::BusWarning => 0x8230,
            Self::ProtocolError => 0x8240,
            Self::PdoNotReceived => 0x8250,
            Self::PdoLength => 0x8260,
            Self::DamMpe => 0x8270,
            Self::SyncError => 0x8280,
            Self::ExternalError => 0x9000,
            Self::AdditionalFunctions => 0xF000,
            Self::DeviceProfileSpecific => 0xFF00,
            Self::ManufacturerSpecific(code) => *code,
            Self::Unknown(code) => *code,
        }
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &str {
        match self {
            Self::NoError => "Error Reset or No Error",
            Self::GenericError => "Generic Error",
            Self::CurrentError => "Current Error",
            Self::CurrentInput => "Current, device input side",
            Self::CurrentInternal => "Current, inside the device",
            Self::CurrentOutput => "Current, device output side",
            Self::VoltageError => "Voltage Error",
            Self::MainsVoltage => "Mains Voltage Error",
            Self::VoltageInternal => "Voltage, inside the device",
            Self::VoltageOutput => "Output Voltage Error",
            Self::TemperatureError => "Temperature Error",
            Self::TemperatureAmbient => "Ambient Temperature",
            Self::TemperatureDevice => "Device Temperature",
            Self::HardwareError => "Hardware Error",
            Self::SoftwareError => "Software Error",
            Self::SoftwareInternal => "Software, internal error",
            Self::SoftwareUser => "Software, user error",
            Self::DataSetError => "Data Set Error",
            Self::AdditionalModules => "Additional modules",
            Self::CommunicationError => "Communication Error",
            Self::CanOverrun => "CAN Overrun (objects lost)",
            Self::CanPassiveMode => "CAN Passive Mode",
            Self::HeartbeatError => "Heartbeat Error",
            Self::BusOff => "Bus Off",
            Self::BusWarning => "Bus Warning",
            Self::ProtocolError => "Protocol Error",
            Self::PdoNotReceived => "PDO not received",
            Self::PdoLength => "PDO Length Error",
            Self::DamMpe => "DAM MPE Error",
            Self::SyncError => "Sync Error",
            Self::ExternalError => "External Error",
            Self::AdditionalFunctions => "Additional Functions",
            Self::DeviceProfileSpecific => "Device Profile Specific",
            Self::ManufacturerSpecific(_) => "Manufacturer Specific",
            Self::Unknown(_) => "Unknown Error",
        }
    }

    /// Check if this is a critical error (requires immediate attention).
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::BusOff | Self::BusWarning | Self::HeartbeatError | Self::HardwareError
        )
    }
}

/// Error register bits (DS301 Table 62).
#[derive(Debug, Clone, Copy, Default)]
pub struct ErrorRegister {
    /// Generic Error.
    pub generic: bool,
    /// Current Error.
    pub current: bool,
    /// Voltage Error.
    pub voltage: bool,
    /// Temperature Error.
    pub temperature: bool,
    /// Communication Error.
    pub communication: bool,
    /// Device Profile Specific.
    pub device_profile: bool,
    /// Manufacturer Specific.
    pub manufacturer: bool,
}

impl ErrorRegister {
    /// Create from raw byte.
    pub fn from_u8(val: u8) -> Self {
        Self {
            generic: val & 0x01 != 0,
            current: val & 0x02 != 0,
            voltage: val & 0x04 != 0,
            temperature: val & 0x08 != 0,
            communication: val & 0x10 != 0,
            device_profile: val & 0x20 != 0,
            manufacturer: val & 0x80 != 0,
        }
    }

    /// Encode to raw byte.
    pub fn to_u8(&self) -> u8 {
        let mut val = 0u8;
        if self.generic { val |= 0x01; }
        if self.current { val |= 0x02; }
        if self.voltage { val |= 0x04; }
        if self.temperature { val |= 0x08; }
        if self.communication { val |= 0x10; }
        if self.device_profile { val |= 0x20; }
        if self.manufacturer { val |= 0x80; }
        val
    }

    /// Check if any error is set.
    pub fn has_error(&self) -> bool {
        self.generic
            || self.current
            || self.voltage
            || self.temperature
            || self.communication
            || self.device_profile
            || self.manufacturer
    }
}

/// Parsed emergency event.
#[derive(Debug, Clone)]
pub struct EmergencyEvent {
    /// Timestamp of the event.
    pub timestamp: Instant,
    /// Node ID that sent the emergency.
    pub node_id: u8,
    /// Error code.
    pub error_code: ErrorCode,
    /// Error register.
    pub error_register: ErrorRegister,
    /// Vendor-specific data (5 bytes).
    pub vendor_data: [u8; 5],
}

impl EmergencyEvent {
    /// Create from an EmergencyFrame.
    pub fn from_frame(frame: &EmergencyFrame) -> Self {
        Self {
            timestamp: Instant::now(),
            node_id: frame.node_id,
            error_code: ErrorCode::from_u16(frame.error_code),
            error_register: ErrorRegister::from_u8(frame.error_register),
            vendor_data: frame.data,
        }
    }

    /// Check if this is an error reset (no error).
    pub fn is_error_reset(&self) -> bool {
        self.error_code == ErrorCode::NoError
    }

    /// Check if this is a critical error.
    pub fn is_critical(&self) -> bool {
        self.error_code.is_critical()
    }
}

/// Node emergency statistics.
#[derive(Debug, Clone, Default)]
pub struct EmergencyStats {
    /// Total emergency messages received.
    pub total_count: u64,
    /// Number of error resets received.
    pub reset_count: u64,
    /// Number of critical errors.
    pub critical_count: u64,
    /// Last emergency timestamp.
    pub last_emergency: Option<Instant>,
    /// Last error reset timestamp.
    pub last_reset: Option<Instant>,
    /// Current error code (None if no error).
    pub current_error: Option<ErrorCode>,
    /// Current error register.
    pub error_register: ErrorRegister,
}

impl EmergencyStats {
    /// Update with a new emergency event.
    fn update(&mut self, event: &EmergencyEvent) {
        self.total_count += 1;
        self.last_emergency = Some(event.timestamp);

        if event.is_error_reset() {
            self.reset_count += 1;
            self.last_reset = Some(event.timestamp);
            self.current_error = None;
        } else {
            self.current_error = Some(event.error_code);
            if event.is_critical() {
                self.critical_count += 1;
            }
        }

        self.error_register = event.error_register;
    }
}

/// Emergency handler event.
#[derive(Debug, Clone)]
pub enum EmergencyHandlerEvent {
    /// New emergency received.
    Emergency(EmergencyEvent),
    /// Error reset received.
    ErrorReset {
        node_id: u8,
        timestamp: Instant,
    },
    /// Critical error received.
    CriticalError(EmergencyEvent),
}

/// Configuration for the emergency handler.
#[derive(Debug, Clone)]
pub struct EmergencyHandlerConfig {
    /// Maximum history entries per node (0 = unlimited).
    pub max_history_per_node: usize,
    /// Maximum total history entries (0 = unlimited).
    pub max_total_history: usize,
    /// Whether to track statistics.
    pub track_stats: bool,
}

impl Default for EmergencyHandlerConfig {
    fn default() -> Self {
        Self {
            max_history_per_node: 100,
            max_total_history: 1000,
            track_stats: true,
        }
    }
}

/// Emergency handler for processing EMCY messages.
///
/// Provides:
/// - EMCY message parsing and classification
/// - Error history tracking per node
/// - Error statistics
/// - Error recovery detection
pub struct EmergencyHandler {
    /// Emergency history per node.
    history: HashMap<u8, Vec<EmergencyEvent>>,
    /// Total emergency history (all nodes).
    total_history: Vec<EmergencyEvent>,
    /// Statistics per node.
    stats: HashMap<u8, EmergencyStats>,
    /// Pending handler events.
    events: Vec<EmergencyHandlerEvent>,
    /// Configuration.
    config: EmergencyHandlerConfig,
}

impl EmergencyHandler {
    /// Create a new emergency handler.
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
            total_history: Vec::new(),
            stats: HashMap::new(),
            events: Vec::new(),
            config: EmergencyHandlerConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: EmergencyHandlerConfig) -> Self {
        Self {
            history: HashMap::new(),
            total_history: Vec::new(),
            stats: HashMap::new(),
            events: Vec::new(),
            config,
        }
    }

    /// Process an emergency frame.
    ///
    /// Returns a list of handler events generated.
    pub fn process_emergency(&mut self, frame: &EmergencyFrame) -> Vec<EmergencyHandlerEvent> {
        let event = EmergencyEvent::from_frame(frame);
        let mut handler_events = Vec::new();

        // Update statistics
        if self.config.track_stats {
            let stats = self.stats.entry(frame.node_id).or_default();
            stats.update(&event);
        }

        // Generate handler events
        if event.is_error_reset() {
            handler_events.push(EmergencyHandlerEvent::ErrorReset {
                node_id: frame.node_id,
                timestamp: event.timestamp,
            });
        } else if event.is_critical() {
            handler_events.push(EmergencyHandlerEvent::CriticalError(event.clone()));
        } else {
            handler_events.push(EmergencyHandlerEvent::Emergency(event.clone()));
        }

        // Add to node history
        let node_history = self.history.entry(frame.node_id).or_default();
        if self.config.max_history_per_node > 0
            && node_history.len() >= self.config.max_history_per_node
        {
            node_history.remove(0);
        }
        node_history.push(event.clone());

        // Add to total history
        if self.config.max_total_history > 0
            && self.total_history.len() >= self.config.max_total_history
        {
            self.total_history.remove(0);
        }
        self.total_history.push(event);

        // Store pending events
        self.events.extend(handler_events.clone());

        handler_events
    }

    /// Get emergency history for a node.
    pub fn node_history(&self, node_id: u8) -> Option<&[EmergencyEvent]> {
        self.history.get(&node_id).map(|h| h.as_slice())
    }

    /// Get total emergency history.
    pub fn total_history(&self) -> &[EmergencyEvent] {
        &self.total_history
    }

    /// Get statistics for a node.
    pub fn node_stats(&self, node_id: u8) -> Option<&EmergencyStats> {
        self.stats.get(&node_id)
    }

    /// Get all nodes with active errors.
    pub fn nodes_with_errors(&self) -> Vec<u8> {
        self.stats
            .iter()
            .filter(|(_, stats)| stats.current_error.is_some())
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get all nodes with critical errors.
    pub fn nodes_with_critical_errors(&self) -> Vec<u8> {
        self.stats
            .iter()
            .filter(|(_, stats)| {
                stats.current_error.as_ref().map_or(false, |e| e.is_critical())
            })
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get the current error for a node.
    pub fn current_error(&self, node_id: u8) -> Option<&ErrorCode> {
        self.stats
            .get(&node_id)
            .and_then(|stats| stats.current_error.as_ref())
    }

    /// Check if a node has an active error.
    pub fn has_error(&self, node_id: u8) -> bool {
        self.stats
            .get(&node_id)
            .map_or(false, |stats| stats.current_error.is_some())
    }

    /// Drain all pending handler events.
    pub fn drain_events(&mut self) -> Vec<EmergencyHandlerEvent> {
        std::mem::take(&mut self.events)
    }

    /// Get a summary of the emergency handler state.
    pub fn summary(&self) -> EmergencySummary {
        let total_nodes = self.stats.len();
        let nodes_with_errors = self.nodes_with_errors().len();
        let nodes_with_critical = self.nodes_with_critical_errors().len();
        let total_emergencies = self.total_history.len();

        EmergencySummary {
            total_nodes,
            nodes_with_errors,
            nodes_with_critical,
            total_emergencies,
        }
    }
}

impl Default for EmergencyHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of emergency handler state.
#[derive(Debug, Clone)]
pub struct EmergencySummary {
    /// Total number of nodes with emergency history.
    pub total_nodes: usize,
    /// Number of nodes with active errors.
    pub nodes_with_errors: usize,
    /// Number of nodes with critical errors.
    pub nodes_with_critical: usize,
    /// Total emergency messages processed.
    pub total_emergencies: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_from_u16() {
        assert_eq!(ErrorCode::from_u16(0x0000), ErrorCode::NoError);
        assert_eq!(ErrorCode::from_u16(0x1000), ErrorCode::GenericError);
        assert_eq!(ErrorCode::from_u16(0x8220), ErrorCode::BusOff);
        assert_eq!(ErrorCode::from_u16(0x8210), ErrorCode::HeartbeatError);
        assert_eq!(
            ErrorCode::from_u16(0xFF01),
            ErrorCode::ManufacturerSpecific(0xFF01)
        );
        assert_eq!(ErrorCode::from_u16(0x1234), ErrorCode::Unknown(0x1234));
    }

    #[test]
    fn test_error_code_to_u16() {
        assert_eq!(ErrorCode::NoError.to_u16(), 0x0000);
        assert_eq!(ErrorCode::GenericError.to_u16(), 0x1000);
        assert_eq!(ErrorCode::BusOff.to_u16(), 0x8220);
        assert_eq!(
            ErrorCode::ManufacturerSpecific(0xFF01).to_u16(),
            0xFF01
        );
    }

    #[test]
    fn test_error_code_description() {
        assert!(!ErrorCode::NoError.description().is_empty());
        assert!(!ErrorCode::BusOff.description().is_empty());
        assert!(!ErrorCode::HeartbeatError.description().is_empty());
    }

    #[test]
    fn test_error_code_is_critical() {
        assert!(!ErrorCode::NoError.is_critical());
        assert!(!ErrorCode::GenericError.is_critical());
        assert!(ErrorCode::BusOff.is_critical());
        assert!(ErrorCode::HeartbeatError.is_critical());
        assert!(ErrorCode::HardwareError.is_critical());
    }

    #[test]
    fn test_error_register_from_u8() {
        let reg = ErrorRegister::from_u8(0x00);
        assert!(!reg.has_error());

        let reg = ErrorRegister::from_u8(0x11);
        assert!(reg.generic);
        assert!(reg.communication);
        assert!(!reg.current);

        let reg = ErrorRegister::from_u8(0xFF);
        assert!(reg.has_error());
    }

    #[test]
    fn test_error_register_to_u8() {
        let reg = ErrorRegister::default();
        assert_eq!(reg.to_u8(), 0x00);

        let mut reg = ErrorRegister::default();
        reg.generic = true;
        reg.communication = true;
        assert_eq!(reg.to_u8(), 0x11);
    }

    #[test]
    fn test_emergency_event_from_frame() {
        let frame = EmergencyFrame {
            node_id: 5,
            error_code: 0x8220,
            error_register: 0x11,
            data: [0x01, 0x02, 0x03, 0x04, 0x05],
        };

        let event = EmergencyEvent::from_frame(&frame);
        assert_eq!(event.node_id, 5);
        assert_eq!(event.error_code, ErrorCode::BusOff);
        assert!(event.error_register.generic);
        assert!(event.error_register.communication);
        assert!(!event.is_error_reset());
        assert!(event.is_critical());
    }

    #[test]
    fn test_emergency_handler_process() {
        let mut handler = EmergencyHandler::new();

        // Process an error
        let frame = EmergencyFrame {
            node_id: 1,
            error_code: 0x8220,
            error_register: 0x10,
            data: [0; 5],
        };

        let events = handler.process_emergency(&frame);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], EmergencyHandlerEvent::CriticalError(_)));

        // Check stats
        let stats = handler.node_stats(1).unwrap();
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.critical_count, 1);
        assert!(stats.current_error.is_some());
    }

    #[test]
    fn test_emergency_handler_error_reset() {
        let mut handler = EmergencyHandler::new();

        // First, create an error
        let frame = EmergencyFrame {
            node_id: 1,
            error_code: 0x8220,
            error_register: 0x10,
            data: [0; 5],
        };
        handler.process_emergency(&frame);

        // Then reset
        let reset_frame = EmergencyFrame {
            node_id: 1,
            error_code: 0x0000,
            error_register: 0x00,
            data: [0; 5],
        };

        let events = handler.process_emergency(&reset_frame);
        assert!(events
            .iter()
            .any(|e| matches!(e, EmergencyHandlerEvent::ErrorReset { .. })));

        // Check stats
        let stats = handler.node_stats(1).unwrap();
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.reset_count, 1);
        assert!(stats.current_error.is_none());
    }

    #[test]
    fn test_emergency_handler_history() {
        let mut handler = EmergencyHandler::new();

        // Process multiple emergencies
        for i in 0..5 {
            let frame = EmergencyFrame {
                node_id: 1,
                error_code: 0x1000 + i,
                error_register: 0x01,
                data: [0; 5],
            };
            handler.process_emergency(&frame);
        }

        let history = handler.node_history(1).unwrap();
        assert_eq!(history.len(), 5);

        let total = handler.total_history();
        assert_eq!(total.len(), 5);
    }

    #[test]
    fn test_emergency_handler_nodes_with_errors() {
        let mut handler = EmergencyHandler::new();

        // Node 1: error
        let frame1 = EmergencyFrame {
            node_id: 1,
            error_code: 0x1000,
            error_register: 0x01,
            data: [0; 5],
        };
        handler.process_emergency(&frame1);

        // Node 2: error reset (no error)
        let frame2 = EmergencyFrame {
            node_id: 2,
            error_code: 0x0000,
            error_register: 0x00,
            data: [0; 5],
        };
        handler.process_emergency(&frame2);

        let nodes = handler.nodes_with_errors();
        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains(&1));
    }

    #[test]
    fn test_emergency_handler_summary() {
        let mut handler = EmergencyHandler::new();

        // Process some emergencies
        for i in 1..=3 {
            let frame = EmergencyFrame {
                node_id: i,
                error_code: 0x1000,
                error_register: 0x01,
                data: [0; 5],
            };
            handler.process_emergency(&frame);
        }

        let summary = handler.summary();
        assert_eq!(summary.total_nodes, 3);
        assert_eq!(summary.nodes_with_errors, 3);
        assert_eq!(summary.total_emergencies, 3);
    }
}
