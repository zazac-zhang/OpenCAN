//! NMT (Network Management) master implementation.

use opencan_canopen_core::CanDriver;
use opencan_canopen_core::error::CanOpenError;
use opencan_canopen_core::frame::{NmtCommand, NmtCommandSpecifier};

/// NMT master for controlling CANOpen network nodes.
pub struct NmtMaster;

impl Default for NmtMaster {
    fn default() -> Self {
        Self::new()
    }
}

impl NmtMaster {
    pub fn new() -> Self {
        Self
    }

    /// Send NMT command to start a remote node.
    pub fn start_remote_node<C: CanDriver>(
        &self,
        can: &mut C,
        node_id: u8,
    ) -> Result<(), CanOpenError> {
        let cmd = NmtCommand {
            command: NmtCommandSpecifier::EnterOperational,
            node_id,
        };
        can.send(&cmd.encode())
    }

    /// Send NMT command to stop a remote node.
    pub fn stop_remote_node<C: CanDriver>(
        &self,
        can: &mut C,
        node_id: u8,
    ) -> Result<(), CanOpenError> {
        let cmd = NmtCommand {
            command: NmtCommandSpecifier::EnterStopped,
            node_id,
        };
        can.send(&cmd.encode())
    }

    /// Send NMT command to reset a remote node.
    pub fn reset_node<C: CanDriver>(&self, can: &mut C, node_id: u8) -> Result<(), CanOpenError> {
        let cmd = NmtCommand {
            command: NmtCommandSpecifier::ResetNode,
            node_id,
        };
        can.send(&cmd.encode())
    }

    /// Send NMT command to reset communication on a remote node.
    pub fn reset_communication<C: CanDriver>(
        &self,
        can: &mut C,
        node_id: u8,
    ) -> Result<(), CanOpenError> {
        let cmd = NmtCommand {
            command: NmtCommandSpecifier::ResetCommunication,
            node_id,
        };
        can.send(&cmd.encode())
    }

    /// Broadcast NMT command to all nodes (node_id = 0).
    pub fn broadcast<C: CanDriver>(
        &self,
        can: &mut C,
        command: NmtCommandSpecifier,
    ) -> Result<(), CanOpenError> {
        let cmd = NmtCommand {
            command,
            node_id: 0,
        };
        can.send(&cmd.encode())
    }
}
