//! DS402 state machine (CiA 402).
//!
//! Implements the CiA 402 state machine for motion control devices.
//! State transitions are driven by control word (0x6040) and reflected
//! in status word (0x6041).

/// DS402 states (CiA 402 Figure 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ds402State {
    NotReadyToSwitchOn,
    SwitchOnDisabled,
    ReadyToSwitchOn,
    SwitchedOn,
    OperationEnabled,
    QuickStopActive,
    FaultReactionActive,
    Fault,
}

impl Ds402State {
    /// Parse state from status word (0x6041).
    ///
    /// Uses the state coding from CiA 402 Table 14.
    pub fn from_status_word(word: u16) -> Self {
        // Extract relevant bits: 0, 1, 2, 3, 5, 6
        let bits = word & 0x006F;
        match bits {
            0b0000_0000 => Self::NotReadyToSwitchOn,
            0b0100_0000 => Self::SwitchOnDisabled,
            0b0010_0001 => Self::ReadyToSwitchOn,
            0b0010_0011 => Self::SwitchedOn,
            0b0010_0111 => Self::OperationEnabled,
            0b0000_0111 => Self::QuickStopActive,
            0b0000_1111 => Self::FaultReactionActive,
            0b0000_1000 => Self::Fault,
            _ => Self::NotReadyToSwitchOn,
        }
    }

    /// Get the required control word transition command to reach target state.
    pub fn transition_to(&self, target: Ds402State) -> Option<u16> {
        match (self, target) {
            // Shutdown transitions
            (Self::SwitchOnDisabled, Self::ReadyToSwitchOn) => Some(ControlWord::SHUTDOWN),
            (Self::SwitchedOn, Self::ReadyToSwitchOn) => Some(ControlWord::SHUTDOWN),
            (Self::OperationEnabled, Self::ReadyToSwitchOn) => Some(ControlWord::SHUTDOWN),
            (Self::QuickStopActive, Self::ReadyToSwitchOn) => Some(ControlWord::SHUTDOWN),

            // Switch On transitions
            (Self::ReadyToSwitchOn, Self::SwitchedOn) => Some(ControlWord::SWITCH_ON),

            // Enable Operation transitions
            (Self::SwitchedOn, Self::OperationEnabled) => Some(ControlWord::ENABLE_OPERATION),
            (Self::ReadyToSwitchOn, Self::OperationEnabled) => Some(ControlWord::ENABLE_OPERATION),

            // Disable Voltage
            (Self::ReadyToSwitchOn, Self::SwitchOnDisabled) => Some(ControlWord::DISABLE_VOLTAGE),
            (Self::SwitchedOn, Self::SwitchOnDisabled) => Some(ControlWord::DISABLE_VOLTAGE),
            (Self::OperationEnabled, Self::SwitchOnDisabled) => Some(ControlWord::DISABLE_VOLTAGE),
            (Self::QuickStopActive, Self::SwitchOnDisabled) => Some(ControlWord::DISABLE_VOLTAGE),

            // Quick Stop
            (Self::OperationEnabled, Self::QuickStopActive) => Some(ControlWord::QUICK_STOP),

            // Fault Reset
            (Self::Fault, Self::SwitchOnDisabled) => Some(ControlWord::FAULT_RESET),

            _ => None,
        }
    }
}

/// Control word (0x6040) bit definitions.
pub struct ControlWord;

impl ControlWord {
    pub const SWITCH_ON: u16 = 0x0007;
    pub const ENABLE_OPERATION: u16 = 0x000F;
    pub const SHUTDOWN: u16 = 0x0006;
    pub const DISABLE_VOLTAGE: u16 = 0x0000;
    pub const QUICK_STOP: u16 = 0x0002;
    pub const FAULT_RESET: u16 = 0x0080;

    /// Check if state transition is acknowledged in status word.
    pub fn is_target_reached(status: u16, _control: u16) -> bool {
        status & 0x0010 != 0 // Target reached bit
    }
}

/// Status word (0x6041) bit definitions.
pub struct StatusWord;

impl StatusWord {
    pub const READY_TO_SWITCH_ON: u16 = 0x0001;
    pub const SWITCHED_ON: u16 = 0x0002;
    pub const OPERATION_ENABLED: u16 = 0x0004;
    pub const FAULT: u16 = 0x0008;
    pub const VOLTAGE_ENABLED: u16 = 0x0010;
    pub const QUICK_STOP: u16 = 0x0020;
    pub const SWITCH_ON_DISABLED: u16 = 0x0040;
    pub const WARNING: u16 = 0x0080;
    pub const REMOTE: u16 = 0x0200;
    pub const TARGET_REACHED: u16 = 0x0400;
    pub const INTERNAL_LIMIT: u16 = 0x0800;
}

/// DS402 operation modes (CiA 402 Table 27).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum OperationMode {
    ProfilePosition = 1,
    Velocity = 2,
    ProfileVelocity = 3,
    ProfileTorque = 4,
    Homing = 6,
    CyclicSyncPosition = 8,
    CyclicSyncVelocity = 9,
    CyclicSyncTorque = 10,
}

impl OperationMode {
    pub fn from_i8(val: i8) -> Option<Self> {
        match val {
            1 => Some(Self::ProfilePosition),
            2 => Some(Self::Velocity),
            3 => Some(Self::ProfileVelocity),
            4 => Some(Self::ProfileTorque),
            6 => Some(Self::Homing),
            8 => Some(Self::CyclicSyncPosition),
            9 => Some(Self::CyclicSyncVelocity),
            10 => Some(Self::CyclicSyncTorque),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_from_status_word() {
        // OperationEnabled: bits 0,1,2,5 = 0b0010_0111 = 0x27
        assert_eq!(
            Ds402State::from_status_word(0x0027),
            Ds402State::OperationEnabled
        );

        // SwitchOnDisabled: bit 6 = 0b0100_0000 = 0x40
        assert_eq!(
            Ds402State::from_status_word(0x0040),
            Ds402State::SwitchOnDisabled
        );

        // Fault: bit 3 = 0b0000_1000 = 0x08
        assert_eq!(Ds402State::from_status_word(0x0008), Ds402State::Fault);
    }

    #[test]
    fn test_state_transitions() {
        // SwitchOnDisabled → ReadyToSwitchOn: Shutdown
        assert_eq!(
            Ds402State::SwitchOnDisabled.transition_to(Ds402State::ReadyToSwitchOn),
            Some(ControlWord::SHUTDOWN)
        );

        // ReadyToSwitchOn → SwitchedOn: Switch On
        assert_eq!(
            Ds402State::ReadyToSwitchOn.transition_to(Ds402State::SwitchedOn),
            Some(ControlWord::SWITCH_ON)
        );

        // SwitchedOn → OperationEnabled: Enable Operation
        assert_eq!(
            Ds402State::SwitchedOn.transition_to(Ds402State::OperationEnabled),
            Some(ControlWord::ENABLE_OPERATION)
        );

        // OperationEnabled → SwitchOnDisabled: Disable Voltage
        assert_eq!(
            Ds402State::OperationEnabled.transition_to(Ds402State::SwitchOnDisabled),
            Some(ControlWord::DISABLE_VOLTAGE)
        );

        // Fault → SwitchOnDisabled: Fault Reset
        assert_eq!(
            Ds402State::Fault.transition_to(Ds402State::SwitchOnDisabled),
            Some(ControlWord::FAULT_RESET)
        );
    }

    #[test]
    fn test_operation_mode_from_i8() {
        assert_eq!(
            OperationMode::from_i8(8),
            Some(OperationMode::CyclicSyncPosition)
        );
        assert_eq!(
            OperationMode::from_i8(1),
            Some(OperationMode::ProfilePosition)
        );
        assert_eq!(OperationMode::from_i8(99), None);
    }
}
