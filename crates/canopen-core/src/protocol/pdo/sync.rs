//! SYNC-triggered PDO Processing.
//!
//! This module provides SYNC-triggered PDO processing capabilities,
//! allowing synchronous PDO exchange synchronized with SYNC messages.

use super::types::{PdoDirection, PdoMapping, TransmissionType};
use crate::frame::CanOpenFrame;
use crate::od::DataType;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// SYNC event for PDO processing.
#[derive(Debug, Clone)]
pub struct SyncEvent {
    /// SYNC counter value (0 = no counter, 1-240 = counter value).
    pub counter: Option<u8>,
    /// Timestamp of the SYNC.
    pub timestamp: Instant,
}

/// PDO SYNC configuration for a specific PDO.
#[derive(Debug, Clone)]
pub struct PdoSyncConfig {
    /// PDO number (1-4).
    pub pdo_number: u8,
    /// Direction.
    pub direction: PdoDirection,
    /// Transmission type.
    pub transmission_type: TransmissionType,
    /// PDO mappings.
    pub mappings: Vec<PdoMapping>,
    /// Data types for unpacking.
    pub data_types: Vec<DataType>,
    /// Whether this PDO is currently active.
    pub active: bool,
}

impl PdoSyncConfig {
    /// Create a new SYNC PDO configuration.
    pub fn new(
        pdo_number: u8,
        direction: PdoDirection,
        transmission_type: TransmissionType,
        mappings: Vec<PdoMapping>,
        data_types: Vec<DataType>,
    ) -> Self {
        Self {
            pdo_number,
            direction,
            transmission_type,
            mappings,
            data_types,
            active: true,
        }
    }

    /// Check if this PDO should be triggered by the given SYNC.
    pub fn should_trigger(&self, sync: &SyncEvent) -> bool {
        if !self.active {
            return false;
        }

        match self.transmission_type {
            TransmissionType::SyncAcyclic => false, // Only on RTR
            TransmissionType::SyncEvery => true,
            TransmissionType::SyncN(n) => {
                if let Some(counter) = sync.counter {
                    counter % n == 0
                } else {
                    false // No counter, can't determine
                }
            }
            _ => false, // Event-driven or RTR-only
        }
    }
}

/// SYNC-triggered PDO buffer.
///
/// Buffers PDO data for synchronous transmission.
#[derive(Debug, Clone)]
pub struct PdoBuffer {
    /// PDO number.
    pub pdo_number: u8,
    /// Direction.
    pub direction: PdoDirection,
    /// Buffered data (up to 8 bytes).
    pub data: [u8; 8],
    /// Data length (0-8).
    pub length: u8,
    /// Whether new data is available.
    pub dirty: bool,
    /// Last update timestamp.
    pub last_update: Option<Instant>,
}

impl PdoBuffer {
    /// Create a new PDO buffer.
    pub fn new(pdo_number: u8, direction: PdoDirection) -> Self {
        Self {
            pdo_number,
            direction,
            data: [0; 8],
            length: 0,
            dirty: false,
            last_update: None,
        }
    }

    /// Update the buffer with new data.
    pub fn update(&mut self, data: &[u8]) {
        let len = data.len().min(8);
        self.data[..len].copy_from_slice(&data[..len]);
        self.length = len as u8;
        self.dirty = true;
        self.last_update = Some(Instant::now());
    }

    /// Clear the dirty flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Get the data as a CAN frame.
    pub fn to_frame(&self, cob_id: u16) -> CanOpenFrame {
        CanOpenFrame::new(cob_id, self.data)
    }
}

/// SYNC-triggered PDO processor.
///
/// Manages synchronous PDO exchange triggered by SYNC messages.
pub struct SyncPdoProcessor {
    /// SYNC PDO configurations per node.
    configs: HashMap<(u8, u8, PdoDirection), PdoSyncConfig>,
    /// PDO buffers per node.
    buffers: HashMap<(u8, u8, PdoDirection), PdoBuffer>,
    /// SYNC counter.
    sync_counter: u8,
    /// Last SYNC timestamp.
    last_sync: Option<Instant>,
    /// SYNC period (for timing validation).
    sync_period: Option<Duration>,
    /// Pending TPDO frames to transmit.
    pending_tpdo: Vec<CanOpenFrame>,
    /// Pending RPDO data to process.
    pending_rpdo: Vec<(u8, u8, [u8; 8])>,
}

impl SyncPdoProcessor {
    /// Create a new SYNC PDO processor.
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            buffers: HashMap::new(),
            sync_counter: 0,
            last_sync: None,
            sync_period: None,
            pending_tpdo: Vec::new(),
            pending_rpdo: Vec::new(),
        }
    }

    /// Set the expected SYNC period.
    pub fn set_sync_period(&mut self, period: Duration) {
        self.sync_period = Some(period);
    }

    /// Register a SYNC PDO configuration.
    pub fn register_pdo(&mut self, node_id: u8, config: PdoSyncConfig) {
        let key = (node_id, config.pdo_number, config.direction);
        let buffer = PdoBuffer::new(config.pdo_number, config.direction);
        self.buffers.insert(key, buffer);
        self.configs.insert(key, config);
    }

    /// Update PDO data in the buffer.
    pub fn update_pdo_data(
        &mut self,
        node_id: u8,
        pdo_number: u8,
        direction: PdoDirection,
        data: &[u8],
    ) {
        let key = (node_id, pdo_number, direction);
        if let Some(buffer) = self.buffers.get_mut(&key) {
            buffer.update(data);
        }
    }

    /// Process a SYNC frame.
    ///
    /// Returns TPDO frames to transmit and RPDO data to process.
    pub fn process_sync(&mut self, sync: SyncEvent) -> SyncPdoResult {
        let now = sync.timestamp;
        self.last_sync = Some(now);

        // Increment counter
        if let Some(counter) = sync.counter {
            self.sync_counter = counter;
        } else {
            self.sync_counter = self.sync_counter.wrapping_add(1);
        }

        let mut tpdo_frames = Vec::new();
        let mut rpdo_data = Vec::new();

        // Process all registered PDOs
        for ((node_id, pdo_number, direction), config) in &self.configs {
            if !config.should_trigger(&sync) {
                continue;
            }

            let key = (*node_id, *pdo_number, *direction);
            if let Some(buffer) = self.buffers.get(&key)
                && buffer.dirty {
                    match direction {
                        PdoDirection::Tpdo => {
                            // Calculate COB-ID for TPDO
                            let cob_id = Self::tpdo_cob_id(*node_id, *pdo_number);
                            let frame = buffer.to_frame(cob_id);
                            tpdo_frames.push(frame);
                        }
                        PdoDirection::Rpdo => {
                            // Queue RPDO data for processing
                            rpdo_data.push((*node_id, *pdo_number, buffer.data));
                        }
                    }
                }
        }

        // Clear dirty flags for processed PDOs
        for buffer in self.buffers.values_mut() {
            if buffer.dirty {
                buffer.clear_dirty();
            }
        }

        SyncPdoResult {
            tpdo_frames,
            rpdo_data,
            sync_counter: self.sync_counter,
            timestamp: now,
        }
    }

    /// Get the COB-ID for a TPDO.
    fn tpdo_cob_id(node_id: u8, pdo_number: u8) -> u16 {
        match pdo_number {
            1 => 0x180 + node_id as u16,
            2 => 0x280 + node_id as u16,
            3 => 0x380 + node_id as u16,
            4 => 0x480 + node_id as u16,
            _ => 0x180 + node_id as u16, // Default
        }
    }

    /// Get the COB-ID for an RPDO.
    pub fn rpdo_cob_id(node_id: u8, pdo_number: u8) -> u16 {
        match pdo_number {
            1 => 0x200 + node_id as u16,
            2 => 0x300 + node_id as u16,
            3 => 0x400 + node_id as u16,
            4 => 0x500 + node_id as u16,
            _ => 0x200 + node_id as u16, // Default
        }
    }

    /// Get a reference to a PDO buffer.
    pub fn buffer(&self, node_id: u8, pdo_number: u8, direction: PdoDirection) -> Option<&PdoBuffer> {
        self.buffers.get(&(node_id, pdo_number, direction))
    }

    /// Get a mutable reference to a PDO buffer.
    pub fn buffer_mut(&mut self, node_id: u8, pdo_number: u8, direction: PdoDirection) -> Option<&mut PdoBuffer> {
        self.buffers.get_mut(&(node_id, pdo_number, direction))
    }

    /// Get the current SYNC counter.
    pub fn sync_counter(&self) -> u8 {
        self.sync_counter
    }

    /// Get the last SYNC timestamp.
    pub fn last_sync(&self) -> Option<Instant> {
        self.last_sync
    }

    /// Get all registered PDO configurations.
    pub fn configs(&self) -> &HashMap<(u8, u8, PdoDirection), PdoSyncConfig> {
        &self.configs
    }

    /// Clear all pending frames.
    pub fn clear_pending(&mut self) {
        self.pending_tpdo.clear();
        self.pending_rpdo.clear();
    }
}

impl Default for SyncPdoProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of SYNC PDO processing.
#[derive(Debug)]
pub struct SyncPdoResult {
    /// TPDO frames to transmit.
    pub tpdo_frames: Vec<CanOpenFrame>,
    /// RPDO data to process (node_id, pdo_number, data).
    pub rpdo_data: Vec<(u8, u8, [u8; 8])>,
    /// SYNC counter value.
    pub sync_counter: u8,
    /// Timestamp of the SYNC.
    pub timestamp: Instant,
}

/// Common SYNC PDO configurations for DS402.
pub struct Ds402SyncConfigs;

impl Ds402SyncConfigs {
    /// Create a TPDO1 configuration for status word and position (every SYNC).
    pub fn tpdo1_status_position_every_sync() -> PdoSyncConfig {
        PdoSyncConfig::new(
            1,
            PdoDirection::Tpdo,
            TransmissionType::SyncEvery,
            vec![
                PdoMapping::new(0x6041, 0, 16), // Status Word
                PdoMapping::new(0x6064, 0, 32), // Position Actual
            ],
            vec![DataType::Unsigned16, DataType::Integer32],
        )
    }

    /// Create a TPDO2 configuration for status word and velocity (every SYNC).
    pub fn tpdo2_status_velocity_every_sync() -> PdoSyncConfig {
        PdoSyncConfig::new(
            2,
            PdoDirection::Tpdo,
            TransmissionType::SyncEvery,
            vec![
                PdoMapping::new(0x6041, 0, 16), // Status Word
                PdoMapping::new(0x606C, 0, 32), // Velocity Actual
            ],
            vec![DataType::Unsigned16, DataType::Integer32],
        )
    }

    /// Create an RPDO1 configuration for control word and position (every SYNC).
    pub fn rpdo1_control_position_every_sync() -> PdoSyncConfig {
        PdoSyncConfig::new(
            1,
            PdoDirection::Rpdo,
            TransmissionType::SyncEvery,
            vec![
                PdoMapping::new(0x6040, 0, 16), // Control Word
                PdoMapping::new(0x607A, 0, 32), // Target Position
            ],
            vec![DataType::Unsigned16, DataType::Integer32],
        )
    }

    /// Create a TPDO1 configuration for status word and position (every Nth SYNC).
    pub fn tpdo1_status_position_sync_n(n: u8) -> PdoSyncConfig {
        PdoSyncConfig::new(
            1,
            PdoDirection::Tpdo,
            TransmissionType::SyncN(n),
            vec![
                PdoMapping::new(0x6041, 0, 16), // Status Word
                PdoMapping::new(0x6064, 0, 32), // Position Actual
            ],
            vec![DataType::Unsigned16, DataType::Integer32],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_event_counter() {
        let sync = SyncEvent {
            counter: Some(5),
            timestamp: Instant::now(),
        };
        assert_eq!(sync.counter, Some(5));
    }

    #[test]
    fn test_pdo_sync_config_should_trigger() {
        let config = PdoSyncConfig::new(
            1,
            PdoDirection::Tpdo,
            TransmissionType::SyncEvery,
            vec![],
            vec![],
        );

        let sync = SyncEvent {
            counter: None,
            timestamp: Instant::now(),
        };

        assert!(config.should_trigger(&sync));
    }

    #[test]
    fn test_pdo_sync_config_sync_n() {
        let config = PdoSyncConfig::new(
            1,
            PdoDirection::Tpdo,
            TransmissionType::SyncN(5),
            vec![],
            vec![],
        );

        // Counter 10 is divisible by 5
        let sync1 = SyncEvent {
            counter: Some(10),
            timestamp: Instant::now(),
        };
        assert!(config.should_trigger(&sync1));

        // Counter 11 is not divisible by 5
        let sync2 = SyncEvent {
            counter: Some(11),
            timestamp: Instant::now(),
        };
        assert!(!config.should_trigger(&sync2));
    }

    #[test]
    fn test_pdo_sync_config_inactive() {
        let mut config = PdoSyncConfig::new(
            1,
            PdoDirection::Tpdo,
            TransmissionType::SyncEvery,
            vec![],
            vec![],
        );
        config.active = false;

        let sync = SyncEvent {
            counter: None,
            timestamp: Instant::now(),
        };

        assert!(!config.should_trigger(&sync));
    }

    #[test]
    fn test_pdo_buffer_update() {
        let mut buffer = PdoBuffer::new(1, PdoDirection::Tpdo);
        assert!(!buffer.dirty);

        buffer.update(&[1, 2, 3, 4]);
        assert!(buffer.dirty);
        assert_eq!(buffer.length, 4);
        assert_eq!(&buffer.data[..4], &[1, 2, 3, 4]);

        buffer.clear_dirty();
        assert!(!buffer.dirty);
    }

    #[test]
    fn test_sync_pdo_processor_register() {
        let mut processor = SyncPdoProcessor::new();
        let config = Ds402SyncConfigs::tpdo1_status_position_every_sync();

        processor.register_pdo(5, config);
        assert!(processor.buffer(5, 1, PdoDirection::Tpdo).is_some());
    }

    #[test]
    fn test_sync_pdo_processor_update_data() {
        let mut processor = SyncPdoProcessor::new();
        let config = Ds402SyncConfigs::tpdo1_status_position_every_sync();

        processor.register_pdo(5, config);
        processor.update_pdo_data(5, 1, PdoDirection::Tpdo, &[0x27, 0x00, 0x39, 0x30, 0x00, 0x00]);

        let buffer = processor.buffer(5, 1, PdoDirection::Tpdo).unwrap();
        assert!(buffer.dirty);
    }

    #[test]
    fn test_sync_pdo_processor_process_sync() {
        let mut processor = SyncPdoProcessor::new();
        let config = Ds402SyncConfigs::tpdo1_status_position_every_sync();

        processor.register_pdo(5, config);
        processor.update_pdo_data(5, 1, PdoDirection::Tpdo, &[0x27, 0x00, 0x39, 0x30, 0x00, 0x00]);

        let sync = SyncEvent {
            counter: None,
            timestamp: Instant::now(),
        };

        let result = processor.process_sync(sync);
        assert_eq!(result.tpdo_frames.len(), 1);
        assert_eq!(result.tpdo_frames[0].cob_id, 0x185);
    }

    #[test]
    fn test_sync_pdo_processor_cob_ids() {
        assert_eq!(SyncPdoProcessor::tpdo_cob_id(5, 1), 0x185);
        assert_eq!(SyncPdoProcessor::tpdo_cob_id(5, 2), 0x285);
        assert_eq!(SyncPdoProcessor::rpdo_cob_id(5, 1), 0x205);
        assert_eq!(SyncPdoProcessor::rpdo_cob_id(5, 2), 0x305);
    }

    #[test]
    fn test_ds402_sync_configs() {
        let config = Ds402SyncConfigs::tpdo1_status_position_every_sync();
        assert_eq!(config.pdo_number, 1);
        assert_eq!(config.direction, PdoDirection::Tpdo);
        assert_eq!(config.transmission_type, TransmissionType::SyncEvery);

        let config = Ds402SyncConfigs::tpdo1_status_position_sync_n(5);
        assert_eq!(config.transmission_type, TransmissionType::SyncN(5));
    }
}
