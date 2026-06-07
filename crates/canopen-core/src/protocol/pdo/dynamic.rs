//! Dynamic PDO Mapping — runtime PDO reconfiguration.
//!
//! This module provides dynamic PDO mapping capabilities, allowing PDO
//! configurations to be changed at runtime without restarting the device.
//!
//! ## Features
//!
//! - Runtime PDO mapping changes
//! - Mapping validation before applying
//! - Rollback support for failed configurations
//! - PDO configuration templates

use super::types::{PdoDirection, PdoMapping, validate_mapping};
use crate::od::DataType;
use std::collections::HashMap;

/// PDO configuration template.
///
/// Pre-defined PDO configurations that can be applied quickly.
#[derive(Debug, Clone)]
pub struct PdoTemplate {
    /// Template name.
    pub name: String,
    /// PDO number (1-4).
    pub pdo_number: u8,
    /// Direction.
    pub direction: PdoDirection,
    /// Mapping entries.
    pub mappings: Vec<PdoMapping>,
    /// Data types for unpacking.
    pub data_types: Vec<DataType>,
    /// Description.
    pub description: String,
}

impl PdoTemplate {
    /// Create a new PDO template.
    pub fn new(
        name: impl Into<String>,
        pdo_number: u8,
        direction: PdoDirection,
        mappings: Vec<PdoMapping>,
        data_types: Vec<DataType>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            pdo_number,
            direction,
            mappings,
            data_types,
            description: description.into(),
        }
    }

    /// Validate the template mappings.
    pub fn validate(&self) -> Result<(), String> {
        validate_mapping(&self.mappings)
    }
}

/// Dynamic PDO mapping manager.
///
/// Manages PDO mapping configurations and provides runtime reconfiguration.
pub struct DynamicPdoMapper {
    /// Stored PDO templates.
    templates: HashMap<String, PdoTemplate>,
    /// Current active mappings per PDO (number, direction).
    active_mappings: HashMap<(u8, PdoDirection), Vec<PdoMapping>>,
    /// Current active data types per PDO.
    active_types: HashMap<(u8, PdoDirection), Vec<DataType>>,
    /// Mapping history for rollback.
    history: Vec<MappingHistoryEntry>,
    /// Maximum history entries.
    max_history: usize,
}

/// Mapping history entry for rollback support.
#[derive(Debug, Clone)]
struct MappingHistoryEntry {
    /// PDO number.
    pdo_number: u8,
    /// Direction.
    direction: PdoDirection,
    /// Previous mappings.
    previous_mappings: Vec<PdoMapping>,
    /// Previous data types.
    previous_types: Vec<DataType>,
    /// Timestamp.
    timestamp: std::time::Instant,
}

impl DynamicPdoMapper {
    /// Create a new dynamic PDO mapper.
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            active_mappings: HashMap::new(),
            active_types: HashMap::new(),
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Register a PDO template.
    pub fn register_template(&mut self, template: PdoTemplate) -> Result<(), String> {
        template.validate()?;
        self.templates.insert(template.name.clone(), template);
        Ok(())
    }

    /// Get a template by name.
    pub fn template(&self, name: &str) -> Option<&PdoTemplate> {
        self.templates.get(name)
    }

    /// Get all template names.
    pub fn template_names(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }

    /// Apply a template to a PDO.
    pub fn apply_template(
        &mut self,
        template_name: &str,
        pdo_number: u8,
        direction: PdoDirection,
    ) -> Result<&PdoMapping, PdoMappingError> {
        let template = self.templates.get(template_name).ok_or_else(|| {
            PdoMappingError::TemplateNotFound(template_name.to_string())
        })?;

        // Validate the template
        template.validate().map_err(|e| PdoMappingError::ValidationError(e))?;

        // Save current mapping for rollback
        let key = (pdo_number, direction);
        if let Some(current) = self.active_mappings.get(&key) {
            let entry = MappingHistoryEntry {
                pdo_number,
                direction,
                previous_mappings: current.clone(),
                previous_types: self.active_types.get(&key).cloned().unwrap_or_default(),
                timestamp: std::time::Instant::now(),
            };
            if self.history.len() >= self.max_history {
                self.history.remove(0);
            }
            self.history.push(entry);
        }

        // Apply the new mapping
        self.active_mappings.insert(key, template.mappings.clone());
        self.active_types.insert(key, template.data_types.clone());

        // Return a reference to the first mapping (for SDO write)
        Ok(&template.mappings[0])
    }

    /// Set custom mappings for a PDO.
    pub fn set_mappings(
        &mut self,
        pdo_number: u8,
        direction: PdoDirection,
        mappings: Vec<PdoMapping>,
        data_types: Vec<DataType>,
    ) -> Result<(), PdoMappingError> {
        // Validate mappings
        validate_mapping(&mappings).map_err(|e| PdoMappingError::ValidationError(e))?;

        // Check data types length matches mappings
        if mappings.len() != data_types.len() {
            return Err(PdoMappingError::ValidationError(
                format!("Mappings and data types length mismatch: {} vs {}", mappings.len(), data_types.len())
            ));
        }

        // Save current mapping for rollback
        let key = (pdo_number, direction);
        if let Some(current) = self.active_mappings.get(&key) {
            let entry = MappingHistoryEntry {
                pdo_number,
                direction,
                previous_mappings: current.clone(),
                previous_types: self.active_types.get(&key).cloned().unwrap_or_default(),
                timestamp: std::time::Instant::now(),
            };
            if self.history.len() >= self.max_history {
                self.history.remove(0);
            }
            self.history.push(entry);
        }

        // Apply the new mapping
        self.active_mappings.insert(key, mappings);
        self.active_types.insert(key, data_types);

        Ok(())
    }

    /// Get current mappings for a PDO.
    pub fn mappings(&self, pdo_number: u8, direction: PdoDirection) -> Option<&[PdoMapping]> {
        self.active_mappings
            .get(&(pdo_number, direction))
            .map(|v| v.as_slice())
    }

    /// Get current data types for a PDO.
    pub fn data_types(&self, pdo_number: u8, direction: PdoDirection) -> Option<&[DataType]> {
        self.active_types
            .get(&(pdo_number, direction))
            .map(|v| v.as_slice())
    }

    /// Rollback to the previous mapping for a PDO.
    pub fn rollback(
        &mut self,
        pdo_number: u8,
        direction: PdoDirection,
    ) -> Result<(), PdoMappingError> {
        let key = (pdo_number, direction);

        // Find the most recent history entry for this PDO
        let entry_idx = self.history
            .iter()
            .rposition(|e| e.pdo_number == pdo_number && e.direction == direction)
            .ok_or(PdoMappingError::NoHistory)?;

        let entry = self.history.swap_remove(entry_idx);

        // Restore previous mappings
        self.active_mappings.insert(key, entry.previous_mappings);
        self.active_types.insert(key, entry.previous_types);

        Ok(())
    }

    /// Get mapping history.
    pub fn history(&self) -> &[MappingHistoryEntry] {
        &self.history
    }

    /// Clear all mappings for a PDO.
    pub fn clear(&mut self, pdo_number: u8, direction: PdoDirection) {
        let key = (pdo_number, direction);
        self.active_mappings.remove(&key);
        self.active_types.remove(&key);
    }

    /// Clear all mappings.
    pub fn clear_all(&mut self) {
        self.active_mappings.clear();
        self.active_types.clear();
    }
}

impl Default for DynamicPdoMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// PDO mapping error types.
#[derive(Debug, Clone)]
pub enum PdoMappingError {
    /// Template not found.
    TemplateNotFound(String),
    /// Validation failed.
    ValidationError(String),
    /// No history for rollback.
    NoHistory,
    /// PDO number out of range.
    InvalidPdoNumber(u8),
    /// Mapping exceeds 64 bits.
    MappingExceeds64Bits,
}

impl std::fmt::Display for PdoMappingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TemplateNotFound(name) => write!(f, "Template not found: {}", name),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::NoHistory => write!(f, "No history available for rollback"),
            Self::InvalidPdoNumber(n) => write!(f, "Invalid PDO number: {}", n),
            Self::MappingExceeds64Bits => write!(f, "Mapping exceeds 64 bits"),
        }
    }
}

impl std::error::Error for PdoMappingError {}

/// Common PDO templates for DS402 devices.
pub struct Ds402PdoTemplates;

impl Ds402PdoTemplates {
    /// Create a TPDO1 template for status word and actual position.
    pub fn tpdo1_status_position() -> PdoTemplate {
        PdoTemplate::new(
            "tpdo1_status_position",
            1,
            PdoDirection::Tpdo,
            vec![
                PdoMapping::new(0x6041, 0, 16), // Status Word
                PdoMapping::new(0x6064, 0, 32), // Position Actual
            ],
            vec![DataType::Unsigned16, DataType::Integer32],
            "TPDO1: Status Word + Actual Position",
        )
    }

    /// Create a TPDO2 template for status word and actual velocity.
    pub fn tpdo2_status_velocity() -> PdoTemplate {
        PdoTemplate::new(
            "tpdo2_status_velocity",
            2,
            PdoDirection::Tpdo,
            vec![
                PdoMapping::new(0x6041, 0, 16), // Status Word
                PdoMapping::new(0x606C, 0, 32), // Velocity Actual
            ],
            vec![DataType::Unsigned16, DataType::Integer32],
            "TPDO2: Status Word + Actual Velocity",
        )
    }

    /// Create a TPDO3 template for status word and actual torque.
    pub fn tpdo3_status_torque() -> PdoTemplate {
        PdoTemplate::new(
            "tpdo3_status_torque",
            3,
            PdoDirection::Tpdo,
            vec![
                PdoMapping::new(0x6041, 0, 16), // Status Word
                PdoMapping::new(0x6077, 0, 16), // Torque Actual
            ],
            vec![DataType::Unsigned16, DataType::Integer16],
            "TPDO3: Status Word + Actual Torque",
        )
    }

    /// Create a RPDO1 template for control word and target position.
    pub fn rpdo1_control_position() -> PdoTemplate {
        PdoTemplate::new(
            "rpdo1_control_position",
            1,
            PdoDirection::Rpdo,
            vec![
                PdoMapping::new(0x6040, 0, 16), // Control Word
                PdoMapping::new(0x607A, 0, 32), // Target Position
            ],
            vec![DataType::Unsigned16, DataType::Integer32],
            "RPDO1: Control Word + Target Position",
        )
    }

    /// Create a RPDO2 template for control word and target velocity.
    pub fn rpdo2_control_velocity() -> PdoTemplate {
        PdoTemplate::new(
            "rpdo2_control_velocity",
            2,
            PdoDirection::Rpdo,
            vec![
                PdoMapping::new(0x6040, 0, 16), // Control Word
                PdoMapping::new(0x60FF, 0, 32), // Target Velocity
            ],
            vec![DataType::Unsigned16, DataType::Integer32],
            "RPDO2: Control Word + Target Velocity",
        )
    }

    /// Create a RPDO3 template for control word and target torque.
    pub fn rpdo3_control_torque() -> PdoTemplate {
        PdoTemplate::new(
            "rpdo3_control_torque",
            3,
            PdoDirection::Rpdo,
            vec![
                PdoMapping::new(0x6040, 0, 16), // Control Word
                PdoMapping::new(0x6071, 0, 16), // Target Torque
            ],
            vec![DataType::Unsigned16, DataType::Integer16],
            "RPDO3: Control Word + Target Torque",
        )
    }

    /// Register all DS402 templates with a mapper.
    pub fn register_all(mapper: &mut DynamicPdoMapper) {
        let templates = vec![
            Self::tpdo1_status_position(),
            Self::tpdo2_status_velocity(),
            Self::tpdo3_status_torque(),
            Self::rpdo1_control_position(),
            Self::rpdo2_control_velocity(),
            Self::rpdo3_control_torque(),
        ];

        for template in templates {
            let _ = mapper.register_template(template);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdo_template_validation() {
        let template = PdoTemplate::new(
            "test",
            1,
            PdoDirection::Tpdo,
            vec![PdoMapping::new(0x6041, 0, 16)],
            vec![DataType::Unsigned16],
            "Test template",
        );

        assert!(template.validate().is_ok());
    }

    #[test]
    fn test_dynamic_mapper_register_template() {
        let mut mapper = DynamicPdoMapper::new();
        let template = PdoTemplate::new(
            "test",
            1,
            PdoDirection::Tpdo,
            vec![PdoMapping::new(0x6041, 0, 16)],
            vec![DataType::Unsigned16],
            "Test template",
        );

        mapper.register_template(template).unwrap();
        assert!(mapper.template("test").is_some());
        assert_eq!(mapper.template_names().len(), 1);
    }

    #[test]
    fn test_dynamic_mapper_apply_template() {
        let mut mapper = DynamicPdoMapper::new();
        let template = PdoTemplate::new(
            "test",
            1,
            PdoDirection::Tpdo,
            vec![PdoMapping::new(0x6041, 0, 16)],
            vec![DataType::Unsigned16],
            "Test template",
        );

        mapper.register_template(template).unwrap();
        mapper.apply_template("test", 1, PdoDirection::Tpdo).unwrap();

        let mappings = mapper.mappings(1, PdoDirection::Tpdo).unwrap();
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].index, 0x6041);
    }

    #[test]
    fn test_dynamic_mapper_set_custom_mappings() {
        let mut mapper = DynamicPdoMapper::new();

        let mappings = vec![
            PdoMapping::new(0x6041, 0, 16),
            PdoMapping::new(0x6064, 0, 32),
        ];
        let types = vec![DataType::Unsigned16, DataType::Integer32];

        mapper.set_mappings(1, PdoDirection::Tpdo, mappings, types).unwrap();

        let result = mapper.mappings(1, PdoDirection::Tpdo).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_dynamic_mapper_rollback() {
        let mut mapper = DynamicPdoMapper::new();

        // Set initial mappings
        let mappings1 = vec![PdoMapping::new(0x6041, 0, 16)];
        let types1 = vec![DataType::Unsigned16];
        mapper.set_mappings(1, PdoDirection::Tpdo, mappings1, types1).unwrap();

        // Set new mappings
        let mappings2 = vec![PdoMapping::new(0x6064, 0, 32)];
        let types2 = vec![DataType::Integer32];
        mapper.set_mappings(1, PdoDirection::Tpdo, mappings2, types2).unwrap();

        // Rollback
        mapper.rollback(1, PdoDirection::Tpdo).unwrap();

        let result = mapper.mappings(1, PdoDirection::Tpdo).unwrap();
        assert_eq!(result[0].index, 0x6041);
    }

    #[test]
    fn test_dynamic_mapper_no_history_rollback() {
        let mut mapper = DynamicPdoMapper::new();
        assert!(mapper.rollback(1, PdoDirection::Tpdo).is_err());
    }

    #[test]
    fn test_ds402_templates() {
        let mut mapper = DynamicPdoMapper::new();
        Ds402PdoTemplates::register_all(&mut mapper);

        assert!(mapper.template("tpdo1_status_position").is_some());
        assert!(mapper.template("rpdo1_control_position").is_some());
        assert_eq!(mapper.template_names().len(), 6);
    }

    #[test]
    fn test_mapping_error_display() {
        let err = PdoMappingError::TemplateNotFound("test".to_string());
        assert!(!err.to_string().is_empty());

        let err = PdoMappingError::ValidationError("invalid".to_string());
        assert!(!err.to_string().is_empty());

        let err = PdoMappingError::NoHistory;
        assert!(!err.to_string().is_empty());
    }
}
