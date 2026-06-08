//! DS402 PDO Templates — pre-defined PDO configurations for DS402 devices.

use opencan_canopen_core::pdo::{PdoDirection, PdoMapping, PdoTemplate};
use opencan_canopen_core::od::DataType;

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
    pub fn register_all(mapper: &mut opencan_canopen_core::pdo::DynamicPdoMapper) {
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
    use opencan_canopen_core::pdo::DynamicPdoMapper;

    #[test]
    fn test_ds402_templates() {
        let mut mapper = DynamicPdoMapper::new();
        Ds402PdoTemplates::register_all(&mut mapper);

        assert!(mapper.template("tpdo1_status_position").is_some());
        assert!(mapper.template("rpdo1_control_position").is_some());
        assert_eq!(mapper.template_names().len(), 6);
    }

    #[test]
    fn test_tpdo1_template() {
        let template = Ds402PdoTemplates::tpdo1_status_position();
        assert_eq!(template.pdo_number, 1);
        assert_eq!(template.direction, PdoDirection::Tpdo);
        assert_eq!(template.mappings.len(), 2);
        assert!(template.validate().is_ok());
    }

    #[test]
    fn test_rpdo1_template() {
        let template = Ds402PdoTemplates::rpdo1_control_position();
        assert_eq!(template.pdo_number, 1);
        assert_eq!(template.direction, PdoDirection::Rpdo);
        assert_eq!(template.mappings.len(), 2);
        assert!(template.validate().is_ok());
    }
}
