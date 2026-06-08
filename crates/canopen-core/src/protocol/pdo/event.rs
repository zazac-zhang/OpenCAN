//! PDO Event Handler — event-driven PDO processing.
//!
//! This module provides event-driven PDO processing capabilities,
//! allowing reactive handling of PDO messages.

use super::types::{PdoData, PdoDirection, PdoMapping};
use crate::frame::CanOpenFrame;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// PDO event types.
#[derive(Debug, Clone)]
pub enum PdoEvent {
    /// PDO received from a node.
    Received {
        /// Source node ID.
        node_id: u8,
        /// PDO number (1-4).
        pdo_number: u8,
        /// Direction (TPDO from remote node = received by us).
        direction: PdoDirection,
        /// Raw PDO data.
        data: PdoData,
        /// Timestamp of receipt.
        timestamp: Instant,
    },
    /// PDO transmission requested (for RPDO).
    TransmitRequest {
        /// Target node ID.
        node_id: u8,
        /// PDO number (1-4).
        pdo_number: u8,
        /// Data to transmit.
        data: [u8; 8],
    },
    /// PDO timeout detected.
    Timeout {
        /// Node ID.
        node_id: u8,
        /// PDO number.
        pdo_number: u8,
        /// Direction.
        direction: PdoDirection,
        /// Expected period.
        expected_period: Duration,
        /// Actual elapsed time.
        elapsed: Duration,
    },
    /// PDO mapping changed.
    MappingChanged {
        /// Node ID.
        node_id: u8,
        /// PDO number.
        pdo_number: u8,
        /// Direction.
        direction: PdoDirection,
        /// New mappings.
        mappings: Vec<PdoMapping>,
    },
}

/// PDO callback function type.
pub type PdoCallback = Box<dyn Fn(&PdoEvent) + Send + Sync>;

/// PDO event handler configuration.
#[derive(Debug, Clone)]
pub struct PdoEventHandlerConfig {
    /// Enable timeout detection.
    pub enable_timeout_detection: bool,
    /// Default timeout multiplier (timeout = period * multiplier).
    pub timeout_multiplier: f64,
    /// Maximum events in queue.
    pub max_event_queue_size: usize,
}

impl Default for PdoEventHandlerConfig {
    fn default() -> Self {
        Self {
            enable_timeout_detection: true,
            timeout_multiplier: 2.0,
            max_event_queue_size: 1000,
        }
    }
}

/// PDO subscription for specific PDO messages.
#[derive(Debug, Clone)]
pub struct PdoSubscription {
    /// Node ID filter (None = any node).
    pub node_id: Option<u8>,
    /// PDO number filter (None = any PDO).
    pub pdo_number: Option<u8>,
    /// Direction filter.
    pub direction: Option<PdoDirection>,
}

impl PdoSubscription {
    /// Create a subscription for a specific PDO.
    pub fn specific(node_id: u8, pdo_number: u8, direction: PdoDirection) -> Self {
        Self {
            node_id: Some(node_id),
            pdo_number: Some(pdo_number),
            direction: Some(direction),
        }
    }

    /// Create a subscription for all PDOs from a node.
    pub fn from_node(node_id: u8) -> Self {
        Self {
            node_id: Some(node_id),
            pdo_number: None,
            direction: None,
        }
    }

    /// Create a subscription for all PDOs.
    pub fn all() -> Self {
        Self {
            node_id: None,
            pdo_number: None,
            direction: None,
        }
    }

    /// Check if a PDO event matches this subscription.
    pub fn matches(&self, node_id: u8, pdo_number: u8, direction: PdoDirection) -> bool {
        if let Some(nid) = self.node_id
            && nid != node_id {
                return false;
            }
        if let Some(pnum) = self.pdo_number
            && pnum != pdo_number {
                return false;
            }
        if let Some(dir) = self.direction
            && dir != direction {
                return false;
            }
        true
    }
}

/// PDO timeout tracker.
#[derive(Debug)]
struct PdoTimeoutTracker {
    /// Expected period.
    period: Duration,
    /// Last received timestamp.
    last_received: Option<Instant>,
    /// Whether timeout has been reported.
    timeout_reported: bool,
}

impl PdoTimeoutTracker {
    fn new(period: Duration) -> Self {
        Self {
            period,
            last_received: None,
            timeout_reported: false,
        }
    }

    /// Update with a new received timestamp.
    fn update(&mut self, now: Instant) {
        self.last_received = Some(now);
        self.timeout_reported = false;
    }

    /// Check if timeout has occurred.
    fn check_timeout(&self, now: Instant, multiplier: f64) -> Option<(Duration, Duration)> {
        if let Some(last) = self.last_received {
            let elapsed = now.duration_since(last);
            let timeout = self.period.mul_f64(multiplier);
            if elapsed > timeout && !self.timeout_reported {
                Some((timeout, elapsed))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// PDO event handler for processing PDO messages.
///
/// Provides event-driven PDO processing with:
/// - Subscription-based event filtering
/// - Callback registration
/// - Timeout detection
/// - Event queue
pub struct PdoEventHandler {
    /// Configuration.
    config: PdoEventHandlerConfig,
    /// Registered callbacks with subscriptions.
    callbacks: Vec<(PdoSubscription, PdoCallback)>,
    /// Event queue.
    events: Vec<PdoEvent>,
    /// Timeout trackers per (node_id, pdo_number, direction).
    timeout_trackers: HashMap<(u8, u8, PdoDirection), PdoTimeoutTracker>,
    /// PDO periods for timeout detection.
    pdo_periods: HashMap<(u8, u8, PdoDirection), Duration>,
}

impl PdoEventHandler {
    /// Create a new PDO event handler.
    pub fn new() -> Self {
        Self {
            config: PdoEventHandlerConfig::default(),
            callbacks: Vec::new(),
            events: Vec::new(),
            timeout_trackers: HashMap::new(),
            pdo_periods: HashMap::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: PdoEventHandlerConfig) -> Self {
        Self {
            config,
            callbacks: Vec::new(),
            events: Vec::new(),
            timeout_trackers: HashMap::new(),
            pdo_periods: HashMap::new(),
        }
    }

    /// Register a callback for PDO events.
    pub fn subscribe(&mut self, subscription: PdoSubscription, callback: PdoCallback) {
        self.callbacks.push((subscription, callback));
    }

    /// Set the expected period for a PDO (for timeout detection).
    pub fn set_pdo_period(
        &mut self,
        node_id: u8,
        pdo_number: u8,
        direction: PdoDirection,
        period: Duration,
    ) {
        let key = (node_id, pdo_number, direction);
        self.pdo_periods.insert(key, period);
        self.timeout_trackers
            .entry(key)
            .or_insert_with(|| PdoTimeoutTracker::new(period));
    }

    /// Process a received PDO frame.
    pub fn process_pdo(&mut self, frame: &CanOpenFrame) -> Vec<PdoEvent> {
        let pdo_data = match super::types::parse_pdo(frame) {
            Some(data) => data,
            None => return Vec::new(),
        };

        let now = Instant::now();
        let mut events = Vec::new();

        // Update timeout tracker
        let key = (pdo_data.node_id, pdo_data.pdo_number, pdo_data.direction);
        if let Some(tracker) = self.timeout_trackers.get_mut(&key) {
            tracker.update(now);
        }

        // Create received event
        let event = PdoEvent::Received {
            node_id: pdo_data.node_id,
            pdo_number: pdo_data.pdo_number,
            direction: pdo_data.direction,
            data: pdo_data.clone(),
            timestamp: now,
        };

        // Notify matching callbacks
        for (subscription, callback) in &self.callbacks {
            if subscription.matches(pdo_data.node_id, pdo_data.pdo_number, pdo_data.direction) {
                callback(&event);
            }
        }

        events.push(event);

        // Add to event queue
        if self.events.len() < self.config.max_event_queue_size {
            self.events.extend(events.clone());
        }

        events
    }

    /// Check for PDO timeouts.
    pub fn check_timeouts(&mut self) -> Vec<PdoEvent> {
        if !self.config.enable_timeout_detection {
            return Vec::new();
        }

        let now = Instant::now();
        let mut events = Vec::new();

        for ((node_id, pdo_number, direction), tracker) in &self.timeout_trackers {
            if let Some((expected, elapsed)) =
                tracker.check_timeout(now, self.config.timeout_multiplier)
            {
                let event = PdoEvent::Timeout {
                    node_id: *node_id,
                    pdo_number: *pdo_number,
                    direction: *direction,
                    expected_period: expected,
                    elapsed,
                };

                // Notify matching callbacks
                for (subscription, callback) in &self.callbacks {
                    if subscription.matches(*node_id, *pdo_number, *direction) {
                        callback(&event);
                    }
                }

                events.push(event);
            }
        }

        events
    }

    /// Drain all pending events from the queue.
    pub fn drain_events(&mut self) -> Vec<PdoEvent> {
        std::mem::take(&mut self.events)
    }

    /// Get the number of registered callbacks.
    pub fn callback_count(&self) -> usize {
        self.callbacks.len()
    }

    /// Get the number of pending events.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

impl Default for PdoEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdo_subscription_specific() {
        let sub = PdoSubscription::specific(5, 1, PdoDirection::Tpdo);
        assert!(sub.matches(5, 1, PdoDirection::Tpdo));
        assert!(!sub.matches(5, 2, PdoDirection::Tpdo));
        assert!(!sub.matches(6, 1, PdoDirection::Tpdo));
        assert!(!sub.matches(5, 1, PdoDirection::Rpdo));
    }

    #[test]
    fn test_pdo_subscription_from_node() {
        let sub = PdoSubscription::from_node(5);
        assert!(sub.matches(5, 1, PdoDirection::Tpdo));
        assert!(sub.matches(5, 2, PdoDirection::Rpdo));
        assert!(!sub.matches(6, 1, PdoDirection::Tpdo));
    }

    #[test]
    fn test_pdo_subscription_all() {
        let sub = PdoSubscription::all();
        assert!(sub.matches(5, 1, PdoDirection::Tpdo));
        assert!(sub.matches(6, 2, PdoDirection::Rpdo));
    }

    #[test]
    fn test_pdo_event_handler_process() {
        let mut handler = PdoEventHandler::new();
        let received = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let received_clone = received.clone();

        handler.subscribe(
            PdoSubscription::all(),
            Box::new(move |_event| {
                received_clone.store(true, std::sync::atomic::Ordering::Relaxed);
            }),
        );

        let frame = CanOpenFrame::new(0x185, [1, 2, 3, 4, 5, 6, 7, 8]);
        let events = handler.process_pdo(&frame);

        assert_eq!(events.len(), 1);
        assert!(received.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_pdo_event_handler_specific_subscription() {
        let mut handler = PdoEventHandler::new();
        let count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let count_clone = count.clone();

        handler.subscribe(
            PdoSubscription::specific(5, 1, PdoDirection::Tpdo),
            Box::new(move |_| {
                count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }),
        );

        // Matching PDO
        let frame1 = CanOpenFrame::new(0x185, [1, 2, 3, 4, 5, 6, 7, 8]);
        handler.process_pdo(&frame1);

        // Non-matching PDO (different node)
        let frame2 = CanOpenFrame::new(0x186, [1, 2, 3, 4, 5, 6, 7, 8]);
        handler.process_pdo(&frame2);

        assert_eq!(count.load(std::sync::atomic::Ordering::Relaxed), 1);
    }

    #[test]
    fn test_pdo_event_handler_drain() {
        let mut handler = PdoEventHandler::new();

        let frame = CanOpenFrame::new(0x185, [1, 2, 3, 4, 5, 6, 7, 8]);
        handler.process_pdo(&frame);
        handler.process_pdo(&frame);

        assert_eq!(handler.event_count(), 2);

        let events = handler.drain_events();
        assert_eq!(events.len(), 2);
        assert_eq!(handler.event_count(), 0);
    }

    #[test]
    fn test_pdo_timeout_tracker() {
        let mut tracker = PdoTimeoutTracker::new(Duration::from_millis(100));
        let now = Instant::now();

        // No timeout initially
        assert!(tracker.check_timeout(now, 2.0).is_none());

        // Update with receipt
        tracker.update(now);

        // No timeout right after receipt
        assert!(tracker.check_timeout(now, 2.0).is_none());
    }

    #[test]
    fn test_pdo_event_handler_config_default() {
        let config = PdoEventHandlerConfig::default();
        assert!(config.enable_timeout_detection);
        assert_eq!(config.timeout_multiplier, 2.0);
        assert_eq!(config.max_event_queue_size, 1000);
    }
}
