//! DS402 Error Handling and Limit Checks.
//!
//! This module provides enhanced error handling and limit checking for DS402
//! motion control operations.

use std::fmt;

/// DS402 error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ds402Error {
    /// Invalid state transition.
    InvalidTransition { from: &'static str, to: &'static str },
    /// Value exceeds limit.
    ValueExceeded { value: i64, limit: i64, direction: LimitDirection },
    /// Invalid operation mode.
    InvalidMode(i8),
    /// Device not enabled.
    NotEnabled,
    /// Device in fault state.
    InFault,
    /// Device not ready.
    NotReady,
    /// Communication error.
    CommunicationError(String),
    /// Timeout waiting for response.
    Timeout,
}

impl fmt::Display for Ds402Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransition { from, to } => {
                write!(f, "Invalid state transition from {} to {}", from, to)
            }
            Self::ValueExceeded {
                value,
                limit,
                direction,
            } => {
                write!(
                    f,
                    "Value {} exceeded {} limit {}",
                    value, direction, limit
                )
            }
            Self::InvalidMode(mode) => write!(f, "Invalid operation mode: {}", mode),
            Self::NotEnabled => write!(f, "Device not enabled"),
            Self::InFault => write!(f, "Device in fault state"),
            Self::NotReady => write!(f, "Device not ready"),
            Self::CommunicationError(msg) => write!(f, "Communication error: {}", msg),
            Self::Timeout => write!(f, "Timeout waiting for response"),
        }
    }
}

impl std::error::Error for Ds402Error {}

/// Limit direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitDirection {
    /// Minimum limit.
    Min,
    /// Maximum limit.
    Max,
}

impl fmt::Display for LimitDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Min => write!(f, "minimum"),
            Self::Max => write!(f, "maximum"),
        }
    }
}

/// Position limits.
#[derive(Debug, Clone)]
pub struct PositionLimits {
    /// Minimum position (0x607D:1).
    pub min_position: i32,
    /// Maximum position (0x607D:2).
    pub max_position: i32,
    /// Software position limit min (0x607D:1).
    pub software_limit_min: Option<i32>,
    /// Software position limit max (0x607D:2).
    pub software_limit_max: Option<i32>,
}

impl Default for PositionLimits {
    fn default() -> Self {
        Self {
            min_position: i32::MIN,
            max_position: i32::MAX,
            software_limit_min: None,
            software_limit_max: None,
        }
    }
}

impl PositionLimits {
    /// Check if a position is within limits.
    pub fn check(&self, position: i32) -> Result<(), Ds402Error> {
        let min = self.software_limit_min.unwrap_or(self.min_position);
        let max = self.software_limit_max.unwrap_or(self.max_position);

        if position < min {
            return Err(Ds402Error::ValueExceeded {
                value: position as i64,
                limit: min as i64,
                direction: LimitDirection::Min,
            });
        }
        if position > max {
            return Err(Ds402Error::ValueExceeded {
                value: position as i64,
                limit: max as i64,
                direction: LimitDirection::Max,
            });
        }
        Ok(())
    }
}

/// Velocity limits.
#[derive(Debug, Clone)]
pub struct VelocityLimits {
    /// Maximum velocity (0x607F).
    pub max_velocity: u32,
    /// Profile velocity (0x6081).
    pub profile_velocity: Option<u32>,
    /// Quick stop deceleration (0x8500).
    pub quick_stop_deceleration: Option<u32>,
}

impl Default for VelocityLimits {
    fn default() -> Self {
        Self {
            max_velocity: u32::MAX,
            profile_velocity: None,
            quick_stop_deceleration: None,
        }
    }
}

impl VelocityLimits {
    /// Check if a velocity is within limits.
    pub fn check(&self, velocity: i32) -> Result<(), Ds402Error> {
        let abs_velocity = velocity.unsigned_abs();
        if abs_velocity > self.max_velocity {
            return Err(Ds402Error::ValueExceeded {
                value: velocity as i64,
                limit: self.max_velocity as i64,
                direction: LimitDirection::Max,
            });
        }
        Ok(())
    }
}

/// Torque limits.
#[derive(Debug, Clone)]
pub struct TorqueLimits {
    /// Max torque (0x6072) in 0.1% of rated torque.
    pub max_torque: u16,
    /// Torque slope (0x6087) in 0.1%/s.
    pub torque_slope: Option<u32>,
}

impl Default for TorqueLimits {
    fn default() -> Self {
        Self {
            max_torque: u16::MAX,
            torque_slope: None,
        }
    }
}

impl TorqueLimits {
    /// Check if a torque value is within limits.
    pub fn check(&self, torque: i16) -> Result<(), Ds402Error> {
        let abs_torque = torque.unsigned_abs();
        if abs_torque > self.max_torque {
            return Err(Ds402Error::ValueExceeded {
                value: torque as i64,
                limit: self.max_torque as i64,
                direction: LimitDirection::Max,
            });
        }
        Ok(())
    }
}

/// Comprehensive limit checker for DS402 operations.
#[derive(Debug, Clone, Default)]
pub struct LimitChecker {
    /// Position limits.
    pub position: PositionLimits,
    /// Velocity limits.
    pub velocity: VelocityLimits,
    /// Torque limits.
    pub torque: TorqueLimits,
}

impl LimitChecker {
    /// Create a new limit checker with default limits.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check position limit.
    pub fn check_position(&self, position: i32) -> Result<(), Ds402Error> {
        self.position.check(position)
    }

    /// Check velocity limit.
    pub fn check_velocity(&self, velocity: i32) -> Result<(), Ds402Error> {
        self.velocity.check(velocity)
    }

    /// Check torque limit.
    pub fn check_torque(&self, torque: i16) -> Result<(), Ds402Error> {
        self.torque.check(torque)
    }

    /// Check all limits for a position move.
    pub fn check_position_move(
        &self,
        target: i32,
        velocity: u32,
    ) -> Result<(), Ds402Error> {
        self.check_position(target)?;
        self.check_velocity(velocity as i32)?;
        Ok(())
    }

    /// Check all limits for a velocity move.
    pub fn check_velocity_move(&self, velocity: i32) -> Result<(), Ds402Error> {
        self.check_velocity(velocity)
    }

    /// Check all limits for a torque move.
    pub fn check_torque_move(&self, torque: i16) -> Result<(), Ds402Error> {
        self.check_torque(torque)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_limits_default() {
        let limits = PositionLimits::default();
        assert!(limits.check(0).is_ok());
        assert!(limits.check(i32::MIN).is_ok());
        assert!(limits.check(i32::MAX).is_ok());
    }

    #[test]
    fn test_position_limits_with_software_limits() {
        let limits = PositionLimits {
            software_limit_min: Some(-1000),
            software_limit_max: Some(1000),
            ..Default::default()
        };

        assert!(limits.check(0).is_ok());
        assert!(limits.check(-1000).is_ok());
        assert!(limits.check(1000).is_ok());
        assert!(limits.check(-1001).is_err());
        assert!(limits.check(1001).is_err());
    }

    #[test]
    fn test_velocity_limits() {
        let limits = VelocityLimits {
            max_velocity: 1000,
            ..Default::default()
        };

        assert!(limits.check(500).is_ok());
        assert!(limits.check(-500).is_ok());
        assert!(limits.check(1000).is_ok());
        assert!(limits.check(1001).is_err());
        assert!(limits.check(-1001).is_err());
    }

    #[test]
    fn test_torque_limits() {
        let limits = TorqueLimits {
            max_torque: 500,
            ..Default::default()
        };

        assert!(limits.check(250).is_ok());
        assert!(limits.check(-250).is_ok());
        assert!(limits.check(500).is_ok());
        assert!(limits.check(501).is_err());
        assert!(limits.check(-501).is_err());
    }

    #[test]
    fn test_limit_checker_position_move() {
        let checker = LimitChecker {
            position: PositionLimits {
                software_limit_min: Some(-1000),
                software_limit_max: Some(1000),
                ..Default::default()
            },
            velocity: VelocityLimits {
                max_velocity: 5000,
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(checker.check_position_move(500, 1000).is_ok());
        assert!(checker.check_position_move(500, 6000).is_err()); // velocity exceeded
        assert!(checker.check_position_move(2000, 1000).is_err()); // position exceeded
    }

    #[test]
    fn test_ds402_error_display() {
        let err = Ds402Error::InvalidTransition {
            from: "NotReadyToSwitchOn",
            to: "Operational",
        };
        assert!(!err.to_string().is_empty());

        let err = Ds402Error::ValueExceeded {
            value: 2000,
            limit: 1000,
            direction: LimitDirection::Max,
        };
        assert!(err.to_string().contains("2000"));
        assert!(err.to_string().contains("1000"));

        let err = Ds402Error::InvalidMode(99);
        assert!(err.to_string().contains("99"));

        let err = Ds402Error::NotEnabled;
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_limit_direction_display() {
        assert_eq!(LimitDirection::Min.to_string(), "minimum");
        assert_eq!(LimitDirection::Max.to_string(), "maximum");
    }
}
