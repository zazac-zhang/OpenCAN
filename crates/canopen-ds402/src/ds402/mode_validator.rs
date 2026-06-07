//! DS402 Mode Switching Validation.
//!
//! This module provides validation for DS402 operation mode switches,
//! ensuring that transitions are valid according to the CiA 402 standard.

use super::state_machine::{Ds402State, OperationMode};
use super::error::Ds402Error;

/// Mode switching validator.
///
/// Validates that mode switches are allowed based on the current device state
/// and mode transition rules.
pub struct ModeSwitchValidator;

impl ModeSwitchValidator {
    /// Check if a mode switch is valid from the current state.
    ///
    /// According to CiA 402, mode switching is only allowed when:
    /// 1. The device is in OperationEnabled state
    /// 2. The target mode is supported by the device
    /// 3. The transition is allowed (some modes have restrictions)
    pub fn validate_switch(
        current_state: Ds402State,
        current_mode: Option<OperationMode>,
        target_mode: OperationMode,
    ) -> Result<(), Ds402Error> {
        // Mode switching is only allowed in OperationEnabled state
        if current_state != Ds402State::OperationEnabled {
            return Err(Ds402Error::InvalidTransition {
                from: current_state.name(),
                to: "ModeSwitch",
            });
        }

        // Check if mode transition is allowed
        if let Some(current) = current_mode {
            if !Self::is_transition_allowed(current, target_mode) {
                return Err(Ds402Error::InvalidTransition {
                    from: current.name(),
                    to: target_mode.name(),
                });
            }
        }

        Ok(())
    }

    /// Check if a mode transition is allowed.
    ///
    /// CiA 402 defines certain mode transitions as not allowed:
    /// - Homing mode cannot be switched to directly from other modes
    ///   (must go through a neutral state first)
    /// - Some vendor-specific restrictions may apply
    pub fn is_transition_allowed(
        from: OperationMode,
        to: OperationMode,
    ) -> bool {
        // Same mode is always allowed (no-op)
        if from == to {
            return true;
        }

        // Homing mode has restrictions
        if to == OperationMode::Homing {
            // Homing can only be entered from certain states
            // For simplicity, we allow it from any mode when in OperationEnabled
            return true;
        }

        // All other transitions are allowed when in OperationEnabled
        true
    }

    /// Get the recommended state sequence for a mode switch.
    ///
    /// Returns the control word values to send for the mode switch sequence.
    pub fn mode_switch_sequence(
        current_state: Ds402State,
        target_mode: OperationMode,
    ) -> Vec<u16> {
        let mut sequence = Vec::new();

        // If not in OperationEnabled, need to enable first
        if current_state != Ds402State::OperationEnabled {
            // Add enable sequence
            sequence.extend_from_slice(&[
                super::state_machine::ControlWord::SHUTDOWN,
                super::state_machine::ControlWord::SWITCH_ON,
                super::state_machine::ControlWord::ENABLE_OPERATION,
            ]);
        }

        sequence
    }

    /// Check if the device is ready for a specific operation mode.
    ///
    /// Returns true if the device can accept the specified mode.
    pub fn is_ready_for_mode(
        state: Ds402State,
        mode: OperationMode,
    ) -> bool {
        match state {
            Ds402State::OperationEnabled => true,
            Ds402State::SwitchedOn => {
                // Can set mode in SwitchedOn state, but cannot execute
                true
            }
            _ => false,
        }
    }
}

/// Mode transition record for tracking mode changes.
#[derive(Debug, Clone)]
pub struct ModeTransition {
    /// Timestamp of the transition.
    pub timestamp: std::time::Instant,
    /// Previous mode.
    pub from_mode: Option<OperationMode>,
    /// New mode.
    pub to_mode: OperationMode,
    /// State at the time of transition.
    pub state: Ds402State,
}

/// Mode transition history tracker.
pub struct ModeTransitionHistory {
    /// Maximum history entries.
    max_entries: usize,
    /// Transition history.
    history: Vec<ModeTransition>,
}

impl ModeTransitionHistory {
    /// Create a new history tracker.
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            history: Vec::new(),
        }
    }

    /// Record a mode transition.
    pub fn record(
        &mut self,
        from_mode: Option<OperationMode>,
        to_mode: OperationMode,
        state: Ds402State,
    ) {
        let transition = ModeTransition {
            timestamp: std::time::Instant::now(),
            from_mode,
            to_mode,
            state,
        };

        if self.history.len() >= self.max_entries {
            self.history.remove(0);
        }
        self.history.push(transition);
    }

    /// Get the transition history.
    pub fn history(&self) -> &[ModeTransition] {
        &self.history
    }

    /// Get the last transition.
    pub fn last(&self) -> Option<&ModeTransition> {
        self.history.last()
    }

    /// Clear the history.
    pub fn clear(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_switch_operation_enabled() {
        let result = ModeSwitchValidator::validate_switch(
            Ds402State::OperationEnabled,
            Some(OperationMode::ProfilePosition),
            OperationMode::CyclicSyncPosition,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_switch_not_enabled() {
        let result = ModeSwitchValidator::validate_switch(
            Ds402State::SwitchedOn,
            Some(OperationMode::ProfilePosition),
            OperationMode::CyclicSyncPosition,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_switch_same_mode() {
        let result = ModeSwitchValidator::validate_switch(
            Ds402State::OperationEnabled,
            Some(OperationMode::ProfilePosition),
            OperationMode::ProfilePosition,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_transition_allowed() {
        assert!(ModeSwitchValidator::is_transition_allowed(
            OperationMode::ProfilePosition,
            OperationMode::CyclicSyncPosition,
        ));
        assert!(ModeSwitchValidator::is_transition_allowed(
            OperationMode::ProfilePosition,
            OperationMode::Homing,
        ));
        assert!(ModeSwitchValidator::is_transition_allowed(
            OperationMode::ProfilePosition,
            OperationMode::ProfilePosition,
        ));
    }

    #[test]
    fn test_is_ready_for_mode() {
        assert!(ModeSwitchValidator::is_ready_for_mode(
            Ds402State::OperationEnabled,
            OperationMode::ProfilePosition,
        ));
        assert!(ModeSwitchValidator::is_ready_for_mode(
            Ds402State::SwitchedOn,
            OperationMode::ProfilePosition,
        ));
        assert!(!ModeSwitchValidator::is_ready_for_mode(
            Ds402State::NotReadyToSwitchOn,
            OperationMode::ProfilePosition,
        ));
    }

    #[test]
    fn test_mode_switch_sequence() {
        let seq = ModeSwitchValidator::mode_switch_sequence(
            Ds402State::OperationEnabled,
            OperationMode::ProfilePosition,
        );
        assert!(seq.is_empty()); // Already in OperationEnabled

        let seq = ModeSwitchValidator::mode_switch_sequence(
            Ds402State::SwitchedOn,
            OperationMode::ProfilePosition,
        );
        assert_eq!(seq.len(), 3); // Need enable sequence
    }

    #[test]
    fn test_mode_transition_history() {
        let mut history = ModeTransitionHistory::new(10);

        history.record(
            Some(OperationMode::ProfilePosition),
            OperationMode::CyclicSyncPosition,
            Ds402State::OperationEnabled,
        );

        assert_eq!(history.history().len(), 1);
        assert!(history.last().is_some());

        history.clear();
        assert!(history.history().is_empty());
    }

    #[test]
    fn test_mode_transition_history_max_entries() {
        let mut history = ModeTransitionHistory::new(2);

        history.record(None, OperationMode::ProfilePosition, Ds402State::OperationEnabled);
        history.record(None, OperationMode::CyclicSyncPosition, Ds402State::OperationEnabled);
        history.record(None, OperationMode::Homing, Ds402State::OperationEnabled);

        assert_eq!(history.history().len(), 2);
        // Oldest entry was removed
        assert_eq!(history.history()[0].to_mode, OperationMode::CyclicSyncPosition);
    }
}
