//! DS402 state machine.

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
    pub fn from_status_word(word: u16) -> Self {
        let state_bits = word & 0x006F;
        match state_bits {
            0b0000_0000 => Self::NotReadyToSwitchOn,
            0b0100_0000 => Self::SwitchOnDisabled,
            0b0010_0001 => Self::ReadyToSwitchOn,
            0b0010_0011 => Self::SwitchedOn,
            0b0010_0111 => Self::OperationEnabled,
            0b0000_0111 => Self::QuickStopActive,
            0b0000_1111 => Self::FaultReactionActive,
            0b0000_1000 => Self::Fault,
            _ => Self::NotReadyToSwitchOn, // Unknown state
        }
    }

    /// Get required control word to transition to target state.
    pub fn transition_command(target: &Ds402State) -> Option<u16> {
        match target {
            Ds402State::ReadyToSwitchOn => Some(0x0006),    // Shutdown
            Ds402State::SwitchedOn => Some(0x0007),         // Switch On
            Ds402State::OperationEnabled => Some(0x000F),   // Enable Operation
            Ds402State::SwitchOnDisabled => Some(0x0000),   // Disable Voltage
            Ds402State::QuickStopActive => Some(0x0002),    // Quick Stop
            Ds402State::Fault => None,                       // Can't transition to Fault
            Ds402State::FaultReactionActive => None,
            Ds402State::NotReadyToSwitchOn => None,
        }
    }

    /// Get required control word for fault reset.
    pub fn fault_reset_command() -> u16 {
        0x0080
    }
}

/// DS402 operation modes (CiA 402 Table 27).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum OperationMode {
    ProfilePosition        = 1,
    ProfileVelocity        = 3,
    ProfileTorque          = 4,
    Homing                 = 6,
    CyclicSyncPosition     = 8,
    CyclicSyncVelocity     = 9,
    CyclicSyncTorque       = 10,
}

impl OperationMode {
    pub fn from_i8(val: i8) -> Option<Self> {
        match val {
            1 => Some(Self::ProfilePosition),
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
