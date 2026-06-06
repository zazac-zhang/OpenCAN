//! Application messages.

use super::tab::{Tab, PrimaryTab};
use super::node::Ds402Mode;
use super::connection::CanBackend;
use super::log::LogEntry;
use super::sdo::SdoDataType;
use super::DragTarget;

/// Application messages.
#[derive(Debug, Clone)]
pub enum Message {
    // === View Navigation ===
    /// Switch to a specific tab.
    SwitchTab(Tab),
    /// Switch primary tab (auto-selects first secondary tab).
    SwitchPrimary(PrimaryTab),
    /// Select a node.
    NodeSelected(u8),
    /// Toggle detail panel visibility.
    ToggleDetailPanel,

    // === Connection ===
    /// Quick connect with mock backend.
    ConnectMock,
    /// Show connection dialog.
    ShowConnectionDialog,
    /// Hide connection dialog.
    HideConnectionDialog,
    /// Change selected backend in dialog.
    ConnectionBackendChanged(CanBackend),
    /// Change channel in dialog.
    ConnectionChannelChanged(String),
    /// Change bitrate in dialog.
    ConnectionBitrateChanged(String),
    /// Change node ID in dialog.
    ConnectionNodeIdChanged(String),
    /// Connect with current dialog settings.
    ConnectionConnect,
    /// Disconnect from backend.
    Disconnect,
    /// Scan for nodes on the bus.
    ScanNodes,

    // === Toolbar ===
    /// Toggle pause/resume.
    TogglePause,
    /// Change bitrate.
    BitrateChanged(u32),
    /// Clear all logs.
    ClearLog,
    /// Export log to CSV.
    ExportLog,
    /// Import log from CSV.
    ImportLog,

    // === NMT ===
    /// Start a specific node.
    NmtStartNode(u8),
    /// Stop a specific node.
    NmtStopNode(u8),
    /// Reset a specific node.
    NmtResetNode(u8),
    /// Reset communication for a specific node.
    NmtResetComm(u8),
    /// Start all nodes (NMT broadcast).
    NmtStartAll,
    /// Stop all nodes (NMT broadcast).
    NmtStopAll,
    /// Reset all nodes (NMT broadcast).
    NmtResetAll,

    // === SDO ===
    /// SDO index changed.
    SdoIndexChanged(String),
    /// SDO subindex changed.
    SdoSubindexChanged(String),
    /// SDO value changed.
    SdoValueChanged(String),
    /// SDO data type changed.
    SdoDataTypeChanged(SdoDataType),
    /// Execute SDO read.
    SdoRead,
    /// Execute SDO write.
    SdoWrite,
    /// Quick SDO read (preset index/subindex).
    SdoQuickRead(u16, u8),
    /// Clear SDO history.
    SdoClearHistory,

    // === DS402 ===
    /// Enable DS402 operation.
    Ds402Enable(u8),
    /// Reset DS402 fault.
    Ds402FaultReset(u8),
    /// Read DS402 state.
    Ds402ReadState(u8),
    /// Execute DS402 state transition.
    Ds402Transition(u8, u16),
    /// Target position changed.
    Ds402TargetPositionChanged(String),
    /// Target velocity changed.
    Ds402TargetVelocityChanged(String),
    /// Target torque changed.
    Ds402TargetTorqueChanged(String),
    /// Set target position.
    Ds402SetPosition(u8),
    /// Set target velocity.
    Ds402SetVelocity(u8),
    /// Set target torque.
    Ds402SetTorque(u8),
    /// Read actual position.
    Ds402ReadPosition(u8),
    /// Read actual velocity.
    Ds402ReadVelocity(u8),
    /// Read actual torque.
    Ds402ReadTorque(u8),
    /// Change DS402 operation mode.
    Ds402ModeChanged(Ds402Mode),
    /// Toggle DS402 auto refresh.
    Ds402ToggleAutoRefresh,
    /// Toggle raw value display.
    Ds402ToggleRawValues,

    // === Frame Detail ===
    /// Select a frame for detail view.
    FrameSelected(LogEntry),

    // === CAN Log Filter ===
    /// Filter text changed.
    LogFilterChanged(String),
    /// Clear all filters.
    LogFilterClear,
    /// Toggle NMT filter.
    LogFilterToggleNmt,
    /// Toggle SDO filter.
    LogFilterToggleSdo,
    /// Toggle PDO filter.
    LogFilterTogglePdo,
    /// Toggle EMCY filter.
    LogFilterToggleEmcy,
    /// Toggle heartbeat filter.
    LogFilterToggleHeartbeat,
    /// Set node ID filter.
    LogFilterNodeId(Option<u8>),

    // === SYNC ===
    /// Start SYNC producer.
    SyncStartProducer(String),
    /// Stop SYNC producer.
    SyncStopProducer,
    /// SYNC period changed.
    SyncPeriodChanged(String),

    // === PDO ===
    /// Read PDO mapping for a node.
    ReadPdoMapping(u8),

    // === Panel Layout ===
    /// Start dragging a panel divider.
    PanelDragStart(DragTarget),
    /// Update panel width during drag.
    PanelDragUpdate(f32),
    /// Stop dragging.
    PanelDragEnd,

    // === System ===
    /// Tick (polls backend events).
    Tick,
    /// Show about dialog.
    ShowAbout,
}
