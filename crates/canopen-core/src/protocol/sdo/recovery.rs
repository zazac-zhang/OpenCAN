//! SDO Error Recovery — handles SDO communication errors and retries.
//!
//! This module provides error recovery mechanisms for SDO communication,
//! including automatic retries, timeout handling, and error classification.

use crate::CanOpenError;
use std::time::{Duration, Instant};

/// SDO error classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdoErrorClass {
    /// Temporary error that may succeed on retry.
    Temporary,
    /// Permanent error that will not succeed on retry.
    Permanent,
    /// Timeout error.
    Timeout,
    /// Communication error (bus error, etc.).
    Communication,
    /// Protocol error (invalid response, etc.).
    Protocol,
    /// Abort from remote node.
    Abort(u32),
}

impl SdoErrorClass {
    /// Check if the error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Temporary | Self::Timeout | Self::Communication)
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &str {
        match self {
            Self::Temporary => "Temporary error",
            Self::Permanent => "Permanent error",
            Self::Timeout => "Timeout",
            Self::Communication => "Communication error",
            Self::Protocol => "Protocol error",
            Self::Abort(_) => "SDO abort",
        }
    }
}

/// Classify a CanOpenError into an SdoErrorClass.
pub fn classify_error(err: &CanOpenError) -> SdoErrorClass {
    match err {
        CanOpenError::Timeout | CanOpenError::SdoTimeout(_) => SdoErrorClass::Timeout,
        CanOpenError::Can(_) => SdoErrorClass::Communication,
        CanOpenError::Protocol(_) => SdoErrorClass::Protocol,
        CanOpenError::SdoAbort { code, .. } => {
            // Classify based on abort code
            match *code {
                // Temporary errors
                0x0503_0000 => SdoErrorClass::Temporary, // Toggle bit not altered
                0x0504_0001 => SdoErrorClass::Temporary, // SDO protocol timed out
                0x0504_0002 => SdoErrorClass::Temporary, // SDO client/server command specifier not valid
                0x0504_0003 => SdoErrorClass::Temporary, // Invalid block size
                0x0504_0004 => SdoErrorClass::Temporary, // Invalid sequence number
                0x0504_0005 => SdoErrorClass::Temporary, // CRC error
                0x0504_0006 => SdoErrorClass::Temporary, // Out of memory

                // Permanent errors
                0x0601_0000 => SdoErrorClass::Permanent, // Unsupported access
                0x0601_0001 => SdoErrorClass::Permanent, // Read write only
                0x0601_0002 => SdoErrorClass::Permanent, // Write read only
                0x0602_0000 => SdoErrorClass::Permanent, // Object does not exist
                0x0604_0041 => SdoErrorClass::Permanent, // Object cannot be mapped
                0x0604_0042 => SdoErrorClass::Permanent, // PDO length exceeded
                0x0604_0043 => SdoErrorClass::Permanent, // General parameter incompatibility
                0x0604_0047 => SdoErrorClass::Permanent, // General internal incompatibility
                0x0606_0000 => SdoErrorClass::Permanent, // Access failed due to hardware error
                0x0607_0010 => SdoErrorClass::Permanent, // Data type does not match
                0x0607_0012 => SdoErrorClass::Permanent, // Data type too long
                0x0607_0013 => SdoErrorClass::Permanent, // Data type too short
                0x0609_0011 => SdoErrorClass::Permanent, // Sub-index does not exist
                0x0609_0030 => SdoErrorClass::Permanent, // Value range exceeded
                0x0609_0031 => SdoErrorClass::Permanent, // Value too high
                0x0609_0032 => SdoErrorClass::Permanent, // Value too low
                0x0609_0036 => SdoErrorClass::Permanent, // Maximum less than minimum
                0x0800_0000 => SdoErrorClass::Permanent, // General error
                0x0800_0020 => SdoErrorClass::Permanent, // Data cannot be transferred
                0x0800_0021 => SdoErrorClass::Permanent, // Data cannot be transferred
                0x0800_0022 => SdoErrorClass::Permanent, // Data cannot be transferred

                // Default to permanent for unknown codes
                _ => SdoErrorClass::Permanent,
            }
        }
        _ => SdoErrorClass::Permanent,
    }
}

/// SDO retry configuration.
#[derive(Debug, Clone)]
pub struct SdoRetryConfig {
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Initial retry delay.
    pub initial_delay: Duration,
    /// Maximum retry delay.
    pub max_delay: Duration,
    /// Backoff multiplier.
    pub backoff_multiplier: f64,
    /// Whether to retry on timeout.
    pub retry_on_timeout: bool,
    /// Whether to retry on communication errors.
    pub retry_on_communication: bool,
}

impl Default for SdoRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            retry_on_timeout: true,
            retry_on_communication: true,
        }
    }
}

/// SDO retry state.
#[derive(Debug)]
pub struct SdoRetryState {
    /// Current retry count.
    pub retry_count: u32,
    /// Current delay.
    pub current_delay: Duration,
    /// Last error.
    pub last_error: Option<CanOpenError>,
    /// Last attempt timestamp.
    pub last_attempt: Option<Instant>,
    /// Total time spent retrying.
    pub total_retry_time: Duration,
}

impl SdoRetryState {
    /// Create a new retry state.
    pub fn new(config: &SdoRetryConfig) -> Self {
        Self {
            retry_count: 0,
            current_delay: config.initial_delay,
            last_error: None,
            last_attempt: None,
            total_retry_time: Duration::ZERO,
        }
    }

    /// Record a failed attempt.
    pub fn record_failure(&mut self, err: CanOpenError, config: &SdoRetryConfig) {
        let now = Instant::now();
        if let Some(last) = self.last_attempt {
            self.total_retry_time += now.duration_since(last);
        }
        self.last_attempt = Some(now);
        self.last_error = Some(err);
        self.retry_count += 1;

        // Exponential backoff
        self.current_delay = Duration::from_secs_f64(
            self.current_delay.as_secs_f64() * config.backoff_multiplier,
        );
        if self.current_delay > config.max_delay {
            self.current_delay = config.max_delay;
        }
    }

    /// Check if we should retry.
    pub fn should_retry(&self, config: &SdoRetryConfig, err: &CanOpenError) -> bool {
        if self.retry_count >= config.max_retries {
            return false;
        }

        let class = classify_error(err);
        match class {
            SdoErrorClass::Temporary => true,
            SdoErrorClass::Timeout => config.retry_on_timeout,
            SdoErrorClass::Communication => config.retry_on_communication,
            _ => false,
        }
    }

    /// Get the delay before the next retry.
    pub fn next_delay(&self) -> Duration {
        self.current_delay
    }

    /// Reset the retry state.
    pub fn reset(&mut self) {
        self.retry_count = 0;
        self.current_delay = Duration::from_millis(100);
        self.last_error = None;
        self.last_attempt = None;
        self.total_retry_time = Duration::ZERO;
    }
}

/// SDO error recovery manager.
pub struct SdoErrorRecovery {
    /// Retry configuration.
    config: SdoRetryConfig,
    /// Retry states per operation.
    states: std::collections::HashMap<String, SdoRetryState>,
}

impl SdoErrorRecovery {
    /// Create a new error recovery manager.
    pub fn new() -> Self {
        Self {
            config: SdoRetryConfig::default(),
            states: std::collections::HashMap::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: SdoRetryConfig) -> Self {
        Self {
            config,
            states: std::collections::HashMap::new(),
        }
    }

    /// Get or create a retry state for an operation.
    pub fn state(&mut self, operation_id: &str) -> &mut SdoRetryState {
        self.states
            .entry(operation_id.to_string())
            .or_insert_with(|| SdoRetryState::new(&self.config))
    }

    /// Check if an operation should be retried.
    pub fn should_retry(&mut self, operation_id: &str, err: &CanOpenError) -> bool {
        let config = &self.config;
        let state = self.states
            .entry(operation_id.to_string())
            .or_insert_with(|| SdoRetryState::new(config));
        state.should_retry(config, err)
    }

    /// Record a failed attempt for an operation.
    pub fn record_failure(&mut self, operation_id: &str, err: CanOpenError) {
        let config = &self.config;
        let state = self.states
            .entry(operation_id.to_string())
            .or_insert_with(|| SdoRetryState::new(config));
        state.record_failure(err, config);
    }

    /// Get the retry configuration.
    pub fn config(&self) -> &SdoRetryConfig {
        &self.config
    }

    /// Get the retry configuration mutably.
    pub fn config_mut(&mut self) -> &mut SdoRetryConfig {
        &mut self.config
    }

    /// Reset all retry states.
    pub fn reset_all(&mut self) {
        for state in self.states.values_mut() {
            state.reset();
        }
    }

    /// Reset a specific operation's retry state.
    pub fn reset(&mut self, operation_id: &str) {
        if let Some(state) = self.states.get_mut(operation_id) {
            state.reset();
        }
    }

    /// Clear all states.
    pub fn clear(&mut self) {
        self.states.clear();
    }
}

impl Default for SdoErrorRecovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_class_retryable() {
        assert!(SdoErrorClass::Temporary.is_retryable());
        assert!(SdoErrorClass::Timeout.is_retryable());
        assert!(SdoErrorClass::Communication.is_retryable());
        assert!(!SdoErrorClass::Permanent.is_retryable());
        assert!(!SdoErrorClass::Protocol.is_retryable());
        assert!(!SdoErrorClass::Abort(0x0602_0000).is_retryable());
    }

    #[test]
    fn test_classify_error() {
        assert_eq!(classify_error(&CanOpenError::Timeout), SdoErrorClass::Timeout);
        assert_eq!(classify_error(&CanOpenError::SdoTimeout(std::time::Duration::from_secs(1))), SdoErrorClass::Timeout);
        assert_eq!(
            classify_error(&CanOpenError::Protocol("test".to_string())),
            SdoErrorClass::Protocol
        );
        assert_eq!(
            classify_error(&CanOpenError::SdoAbort {
                code: 0x0503_0000,
                reason: "test",
            }),
            SdoErrorClass::Temporary
        );
        assert_eq!(
            classify_error(&CanOpenError::SdoAbort {
                code: 0x0602_0000,
                reason: "test",
            }),
            SdoErrorClass::Permanent
        );
    }

    #[test]
    fn test_retry_state_new() {
        let config = SdoRetryConfig::default();
        let state = SdoRetryState::new(&config);

        assert_eq!(state.retry_count, 0);
        assert_eq!(state.current_delay, config.initial_delay);
        assert!(state.last_error.is_none());
    }

    #[test]
    fn test_retry_state_record_failure() {
        let config = SdoRetryConfig::default();
        let mut state = SdoRetryState::new(&config);

        let err = CanOpenError::Timeout;
        state.record_failure(err, &config);

        assert_eq!(state.retry_count, 1);
        assert!(state.last_error.is_some());
    }

    #[test]
    fn test_retry_state_should_retry() {
        let config = SdoRetryConfig::default();
        let state = SdoRetryState::new(&config);

        // Temporary error should be retried
        let err = CanOpenError::SdoAbort {
            code: 0x0503_0000,
            reason: "test",
        };
        assert!(state.should_retry(&config, &err));

        // Permanent error should not be retried
        let err = CanOpenError::SdoAbort {
            code: 0x0602_0000,
            reason: "test",
        };
        assert!(!state.should_retry(&config, &err));
    }

    #[test]
    fn test_retry_state_max_retries() {
        let config = SdoRetryConfig {
            max_retries: 2,
            ..Default::default()
        };
        let mut state = SdoRetryState::new(&config);

        // First retry
        let err1 = CanOpenError::Timeout;
        state.record_failure(err1, &config);
        let err2 = CanOpenError::Timeout;
        assert!(state.should_retry(&config, &err2));

        // Second retry
        let err3 = CanOpenError::Timeout;
        state.record_failure(err3, &config);
        let err4 = CanOpenError::Timeout;
        assert!(!state.should_retry(&config, &err4)); // Max retries reached
    }

    #[test]
    fn test_error_recovery_manager() {
        let mut manager = SdoErrorRecovery::new();

        // First attempt
        let err1 = CanOpenError::Timeout;
        assert!(manager.should_retry("op1", &err1));
        manager.record_failure("op1", CanOpenError::Timeout);

        // Second attempt
        let err2 = CanOpenError::Timeout;
        assert!(manager.should_retry("op1", &err2));
        manager.record_failure("op1", CanOpenError::Timeout);

        // Third attempt
        let err3 = CanOpenError::Timeout;
        assert!(manager.should_retry("op1", &err3));
        manager.record_failure("op1", CanOpenError::Timeout);

        // Fourth attempt - should not retry (max retries = 3)
        let err4 = CanOpenError::Timeout;
        assert!(!manager.should_retry("op1", &err4));
    }

    #[test]
    fn test_error_recovery_manager_reset() {
        let mut manager = SdoErrorRecovery::new();

        // Record some failures
        manager.record_failure("op1", CanOpenError::Timeout);
        manager.record_failure("op1", CanOpenError::Timeout);

        // Reset
        manager.reset("op1");

        // Should be able to retry again
        let err = CanOpenError::Timeout;
        assert!(manager.should_retry("op1", &err));
    }

    #[test]
    fn test_sdo_retry_config_default() {
        let config = SdoRetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert!(config.retry_on_timeout);
        assert!(config.retry_on_communication);
    }

    #[test]
    fn test_error_class_description() {
        assert!(!SdoErrorClass::Temporary.description().is_empty());
        assert!(!SdoErrorClass::Permanent.description().is_empty());
        assert!(!SdoErrorClass::Timeout.description().is_empty());
    }
}
