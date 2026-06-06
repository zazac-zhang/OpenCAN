//! DS402 panel state types.

use super::node::Ds402Mode;

/// DS402 panel state (UI inputs).
#[derive(Debug, Clone)]
pub struct Ds402PanelState {
    pub target_position: String,
    pub target_velocity: String,
    pub target_torque: String,
    pub selected_mode: Ds402Mode,
    pub position_offset: i32,
    pub velocity_factor: f32,
    pub torque_factor: f32,
    pub auto_refresh: bool,
    pub refresh_interval_ms: u32,
    pub show_raw_values: bool,
}

impl Default for Ds402PanelState {
    fn default() -> Self {
        Self {
            target_position: "0".to_string(),
            target_velocity: "0".to_string(),
            target_torque: "0".to_string(),
            selected_mode: Ds402Mode::default(),
            position_offset: 0,
            velocity_factor: 1.0,
            torque_factor: 1.0,
            auto_refresh: false,
            refresh_interval_ms: 100,
            show_raw_values: false,
        }
    }
}

impl Ds402PanelState {
    /// Get parsed target position.
    pub fn parsed_position(&self) -> i32 {
        self.target_position.parse().unwrap_or(0)
    }

    /// Get parsed target velocity.
    pub fn parsed_velocity(&self) -> i32 {
        self.target_velocity.parse().unwrap_or(0)
    }

    /// Get parsed target torque.
    pub fn parsed_torque(&self) -> i16 {
        self.target_torque.parse().unwrap_or(0)
    }

    /// Set target position from value.
    pub fn set_position(&mut self, pos: i32) {
        self.target_position = pos.to_string();
    }

    /// Set target velocity from value.
    pub fn set_velocity(&mut self, vel: i32) {
        self.target_velocity = vel.to_string();
    }

    /// Set target torque from value.
    pub fn set_torque(&mut self, torque: i16) {
        self.target_torque = torque.to_string();
    }

    /// Toggle auto refresh.
    pub fn toggle_auto_refresh(&mut self) {
        self.auto_refresh = !self.auto_refresh;
    }

    /// Toggle raw value display.
    pub fn toggle_raw_values(&mut self) {
        self.show_raw_values = !self.show_raw_values;
    }
}

/// DS402 state machine states.
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
    pub fn name(&self) -> &'static str {
        match self {
            Self::NotReadyToSwitchOn => "Not Ready To Switch On",
            Self::SwitchOnDisabled => "Switch On Disabled",
            Self::ReadyToSwitchOn => "Ready To Switch On",
            Self::SwitchedOn => "Switched On",
            Self::OperationEnabled => "Operation Enabled",
            Self::QuickStopActive => "Quick Stop Active",
            Self::FaultReactionActive => "Fault Reaction Active",
            Self::Fault => "Fault",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::NotReadyToSwitchOn => "NotReady",
            Self::SwitchOnDisabled => "Disabled",
            Self::ReadyToSwitchOn => "Ready",
            Self::SwitchedOn => "SwitchedOn",
            Self::OperationEnabled => "Enabled",
            Self::QuickStopActive => "QuickStop",
            Self::FaultReactionActive => "FaultReaction",
            Self::Fault => "Fault",
        }
    }

    /// Parse state from status word.
    pub fn from_status_word(status: u16) -> Self {
        match status & 0x006F {
            0x0000 => Self::NotReadyToSwitchOn,
            0x0040 => Self::SwitchOnDisabled,
            0x0021 => Self::ReadyToSwitchOn,
            0x0023 => Self::SwitchedOn,
            0x0027 => Self::OperationEnabled,
            0x0007 => Self::QuickStopActive,
            0x000F => Self::FaultReactionActive,
            0x0008 => Self::Fault,
            _ => Self::NotReadyToSwitchOn,
        }
    }

    /// Get valid transitions from this state.
    pub fn valid_transitions(&self) -> &'static [Ds402Transition] {
        match self {
            Self::SwitchOnDisabled => &[Ds402Transition::Shutdown],
            Self::ReadyToSwitchOn => &[
                Ds402Transition::SwitchOn,
                Ds402Transition::DisableVoltage,
            ],
            Self::SwitchedOn => &[
                Ds402Transition::EnableOperation,
                Ds402Transition::Shutdown,
                Ds402Transition::DisableVoltage,
            ],
            Self::OperationEnabled => &[
                Ds402Transition::DisableOperation,
                Ds402Transition::QuickStop,
                Ds402Transition::Shutdown,
                Ds402Transition::DisableVoltage,
            ],
            Self::QuickStopActive => &[
                Ds402Transition::EnableOperation,
                Ds402Transition::DisableVoltage,
            ],
            Self::Fault => &[Ds402Transition::FaultReset],
            _ => &[],
        }
    }
}

/// DS402 state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ds402Transition {
    Shutdown,
    SwitchOn,
    EnableOperation,
    DisableOperation,
    DisableVoltage,
    QuickStop,
    FaultReset,
}

impl Ds402Transition {
    pub fn control_word(&self) -> u16 {
        match self {
            Self::Shutdown => 0x0006,
            Self::SwitchOn => 0x0007,
            Self::EnableOperation => 0x000F,
            Self::DisableOperation => 0x0007,
            Self::DisableVoltage => 0x0000,
            Self::QuickStop => 0x0002,
            Self::FaultReset => 0x0080,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Shutdown => "Shutdown",
            Self::SwitchOn => "Switch On",
            Self::EnableOperation => "Enable Operation",
            Self::DisableOperation => "Disable Operation",
            Self::DisableVoltage => "Disable Voltage",
            Self::QuickStop => "Quick Stop",
            Self::FaultReset => "Fault Reset",
        }
    }
}

/// DS402 operation data for display.
#[derive(Debug, Clone, Default)]
pub struct Ds402OperationData {
    pub actual_position: i32,
    pub actual_velocity: i32,
    pub actual_torque: i16,
    pub target_position: i32,
    pub target_velocity: i32,
    pub target_torque: i16,
    pub position_error: i32,
    pub velocity_error: i32,
    pub torque_error: i16,
}

impl Ds402OperationData {
    /// Calculate position error.
    pub fn update_errors(&mut self) {
        self.position_error = self.target_position - self.actual_position;
        self.velocity_error = self.target_velocity - self.actual_velocity;
        self.torque_error = self.target_torque - self.actual_torque;
    }
}
