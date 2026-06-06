//! Mock CAN driver for testing.

use crate::CanDriver;
use crate::error::CanOpenError;
use crate::frame::CanOpenFrame;
use std::collections::VecDeque;
use std::time::Duration;

/// Mock CAN driver for unit and integration testing.
///
/// Pre-load response frames via `enqueue()`, then verify sent frames via `tx_log()`.
pub struct MockCanDriver {
    rx_queue: VecDeque<CanOpenFrame>,
    tx_log: Vec<CanOpenFrame>,
    rx_delay: Duration,
    error_inject: Option<CanOpenError>,
}

impl Default for MockCanDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl MockCanDriver {
    pub fn new() -> Self {
        Self {
            rx_queue: VecDeque::new(),
            tx_log: Vec::new(),
            rx_delay: Duration::ZERO,
            error_inject: None,
        }
    }

    /// Pre-load a frame to be returned by the next `recv()` call.
    pub fn enqueue(&mut self, frame: CanOpenFrame) {
        self.rx_queue.push_back(frame);
    }

    /// Get all frames that were sent via `send()`.
    pub fn tx_log(&self) -> &[CanOpenFrame] {
        &self.tx_log
    }

    /// Clear the TX log.
    pub fn clear_tx_log(&mut self) {
        self.tx_log.clear();
    }

    /// Clear all queued RX frames.
    pub fn clear_rx(&mut self) {
        self.rx_queue.clear();
    }

    /// Set simulated receive delay.
    pub fn set_rx_delay(&mut self, delay: Duration) {
        self.rx_delay = delay;
    }

    /// Inject an error to be returned on the next recv().
    pub fn inject_error(&mut self, err: CanOpenError) {
        self.error_inject = Some(err);
    }
}

impl CanDriver for MockCanDriver {
    fn send(&mut self, frame: &CanOpenFrame) -> Result<(), CanOpenError> {
        self.tx_log.push(frame.clone());
        Ok(())
    }

    async fn recv(&mut self) -> Result<CanOpenFrame, CanOpenError> {
        if self.rx_delay > Duration::ZERO {
            tokio::time::sleep(self.rx_delay).await;
        }
        if let Some(err) = self.error_inject.take() {
            return Err(err);
        }
        self.rx_queue.pop_front().ok_or(CanOpenError::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_enqueue_recv() {
        let mut mock = MockCanDriver::new();
        let frame = CanOpenFrame::new(0x583, [0x43, 0x00, 0x10, 0x00, 0x92, 0x01, 0x02, 0x00]);
        mock.enqueue(frame.clone());

        let received = mock.recv().await.unwrap();
        assert_eq!(received, frame);
    }

    #[tokio::test]
    async fn test_mock_recv_empty() {
        let mut mock = MockCanDriver::new();
        assert!(mock.recv().await.is_err());
    }

    #[tokio::test]
    async fn test_mock_tx_log() {
        let mut mock = MockCanDriver::new();
        let frame = CanOpenFrame::new(0x603, [0x40, 0x00, 0x10, 0x00, 0, 0, 0, 0]);
        mock.send(&frame).unwrap();

        assert_eq!(mock.tx_log().len(), 1);
        assert_eq!(mock.tx_log()[0], frame);
    }

    #[tokio::test]
    async fn test_mock_error_inject() {
        let mut mock = MockCanDriver::new();
        mock.inject_error(CanOpenError::Timeout);

        let result = mock.recv().await;
        assert!(result.is_err());
    }
}
