//! Backend integration — connects GUI to CAN/CANOpen protocol stack.

mod command;
mod event;
mod mock;
mod real;

pub use command::BackendCommand;
pub use event::BackendEvent;

use tokio::sync::mpsc;
use opencan_canopen_core::CanDriver;

/// Backend manages the async CAN/CANOpen protocol stack.
pub struct Backend {
    cmd_tx: mpsc::Sender<BackendCommand>,
    evt_rx: mpsc::Receiver<BackendEvent>,
}

impl std::fmt::Debug for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Backend").finish()
    }
}

impl Backend {
    /// Create a new backend with a mock CAN driver (for testing/development).
    pub fn new_mock() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel(64);
        let (evt_tx, evt_rx) = mpsc::channel(256);

        tokio::spawn(mock::mock_backend_task(cmd_rx, evt_tx));

        Self { cmd_tx, evt_rx }
    }

    /// Create a new backend with a real CAN driver.
    #[allow(dead_code)]
    pub fn new_with_driver<D: CanDriver + 'static>(driver: D, node_id: u8) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel(64);
        let (evt_tx, evt_rx) = mpsc::channel(256);

        tokio::spawn(real::real_backend_task(driver, node_id, cmd_rx, evt_tx));

        Self { cmd_tx, evt_rx }
    }

    /// Send a command to the backend (non-blocking).
    pub fn send(&self, cmd: BackendCommand) {
        let _ = self.cmd_tx.try_send(cmd);
    }

    /// Try to receive an event (non-blocking).
    pub fn try_recv(&mut self) -> Option<BackendEvent> {
        self.evt_rx.try_recv().ok()
    }

    /// Check if backend is connected (channel not closed).
    pub fn is_connected(&self) -> bool {
        !self.cmd_tx.is_closed()
    }
}
