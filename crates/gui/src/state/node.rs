//! Node state types.

use std::collections::BTreeMap;

/// Node state.
#[derive(Debug, Clone)]
pub struct NodeState {
    pub node_id: u8,
    pub nmt_state: NmtState,
    pub device_type: Option<u32>,
    pub vendor_id: Option<u32>,
    pub product_name: Option<String>,
    pub od_cache: BTreeMap<(u16, u8), OdEntry>,
    pub ds402: Ds402NodeState,
    pub heartbeat_period: Option<u32>,
    pub last_heartbeat: Option<u64>,
    /// Error register (object 0x1001).
    pub error_register: Option<u8>,
    /// Manufacturer status register.
    pub manufacturer_status: Option<u32>,
}

impl NodeState {
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
            nmt_state: NmtState::Unknown,
            device_type: None,
            vendor_id: None,
            product_name: None,
            od_cache: BTreeMap::new(),
            ds402: Ds402NodeState::default(),
            heartbeat_period: None,
            last_heartbeat: None,
            error_register: None,
            manufacturer_status: None,
        }
    }

    /// Get display label for this node.
    pub fn label(&self) -> String {
        let name = self.product_name.as_deref().unwrap_or("");
        if name.is_empty() {
            format!("Node {} [{}]", self.node_id, self.nmt_state.as_str())
        } else {
            format!("Node {} ({}) [{}]", self.node_id, name, self.nmt_state.as_str())
        }
    }

    /// Check if node is operational.
    pub fn is_operational(&self) -> bool {
        self.nmt_state == NmtState::Operational
    }

    /// Check if node has DS402 support.
    pub fn has_ds402(&self) -> bool {
        // Check device type for DS402 flag (bit 14 set indicates drive profile)
        self.device_type.map(|dt| dt & 0x4000 != 0).unwrap_or(false)
            || !self.ds402.state.is_empty()
    }

    /// Get OD entry by index and subindex.
    pub fn get_od(&self, index: u16, subindex: u8) -> Option<&OdEntry> {
        self.od_cache.get(&(index, subindex))
    }

    /// Set OD entry.
    pub fn set_od(&mut self, index: u16, subindex: u8, entry: OdEntry) {
        self.od_cache.insert((index, subindex), entry);
    }

    /// Get cached OD value as string.
    pub fn get_od_value(&self, index: u16, subindex: u8) -> Option<&str> {
        self.od_cache.get(&(index, subindex)).map(|e| e.value.as_str())
    }
}

/// NMT state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NmtState {
    Unknown,
    BootUp,
    PreOperational,
    Operational,
    Stopped,
}

impl NmtState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::BootUp => "BootUp",
            Self::PreOperational => "Pre-Operational",
            Self::Operational => "Operational",
            Self::Stopped => "Stopped",
        }
    }

    /// Parse NMT state from string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "Operational" => Self::Operational,
            "Stopped" => Self::Stopped,
            "PreOperational" | "Pre-Operational" => Self::PreOperational,
            "BootUp" => Self::BootUp,
            _ => Self::Unknown,
        }
    }

    /// Get color indicator for this state.
    pub fn color_indicator(&self) -> &'static str {
        match self {
            Self::Unknown => "○",
            Self::BootUp => "◐",
            Self::PreOperational => "◑",
            Self::Operational => "●",
            Self::Stopped => "⊘",
        }
    }
}

/// OD cache entry.
#[derive(Debug, Clone)]
pub struct OdEntry {
    pub value: String,
    pub data_type: Option<String>,
    pub name: Option<String>,
    pub access: Option<OdAccess>,
    pub low_limit: Option<String>,
    pub high_limit: Option<String>,
    pub default_value: Option<String>,
}

impl OdEntry {
    pub fn new(value: String) -> Self {
        Self {
            value,
            data_type: None,
            name: None,
            access: None,
            low_limit: None,
            high_limit: None,
            default_value: None,
        }
    }

    pub fn with_type(value: String, data_type: String) -> Self {
        Self {
            value,
            data_type: Some(data_type),
            name: None,
            access: None,
            low_limit: None,
            high_limit: None,
            default_value: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_access(mut self, access: OdAccess) -> Self {
        self.access = Some(access);
        self
    }

    pub fn with_limits(mut self, low: String, high: String) -> Self {
        self.low_limit = Some(low);
        self.high_limit = Some(high);
        self
    }

    pub fn with_default(mut self, default: String) -> Self {
        self.default_value = Some(default);
        self
    }

    /// Get display string for this entry.
    pub fn display(&self) -> String {
        let mut parts = vec![self.value.clone()];
        if let Some(ref name) = self.name {
            parts.insert(0, format!("({})", name));
        }
        if let Some(ref dtype) = self.data_type {
            parts.push(format!("[{}]", dtype));
        }
        parts.join(" ")
    }
}

/// OD access type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OdAccess {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Constant,
}

impl OdAccess {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadOnly => "ro",
            Self::WriteOnly => "wo",
            Self::ReadWrite => "rw",
            Self::Constant => "const",
        }
    }
}

/// DS402 node state.
#[derive(Debug, Clone, Default)]
pub struct Ds402NodeState {
    pub state: String,
    pub status_word: u16,
    pub control_word: u16,
    pub actual_position: i32,
    pub actual_velocity: i32,
    pub actual_torque: i16,
    pub target_position: i32,
    pub target_velocity: i32,
    pub target_torque: i16,
    pub position_history: Vec<i32>,
    pub velocity_history: Vec<i32>,
    pub torque_history: Vec<i16>,
    pub modes_of_operation: Option<i8>,
    pub supported_modes: Vec<Ds402Mode>,
}

impl Ds402NodeState {
    /// Check if node is in fault state.
    pub fn is_fault(&self) -> bool {
        self.status_word & 0x0008 != 0
    }

    /// Check if operation is enabled.
    pub fn is_operation_enabled(&self) -> bool {
        self.status_word & 0x0004 != 0
    }

    /// Check if target is reached.
    pub fn is_target_reached(&self) -> bool {
        self.status_word & 0x0400 != 0
    }

    /// Get status word bits as a vector of (mask, name, active).
    pub fn status_bits(&self) -> Vec<(u16, &'static str, bool)> {
        vec![
            (0x0001, "Ready To Switch On", self.status_word & 0x0001 != 0),
            (0x0002, "Switched On", self.status_word & 0x0002 != 0),
            (0x0004, "Operation Enabled", self.status_word & 0x0004 != 0),
            (0x0008, "Fault", self.status_word & 0x0008 != 0),
            (0x0010, "Voltage Enabled", self.status_word & 0x0010 != 0),
            (0x0020, "Quick Stop", self.status_word & 0x0020 != 0),
            (0x0040, "Switch On Disabled", self.status_word & 0x0040 != 0),
            (0x0080, "Warning", self.status_word & 0x0080 != 0),
            (0x0100, "STO (Safe Torque Off)", self.status_word & 0x0100 != 0),
            (0x0200, "Remote", self.status_word & 0x0200 != 0),
            (0x0400, "Target Reached", self.status_word & 0x0400 != 0),
            (0x0800, "Internal Limit Active", self.status_word & 0x0800 != 0),
            (0x1000, "Set-Point Acknowledge", self.status_word & 0x1000 != 0),
            (0x2000, "Following Error", self.status_word & 0x2000 != 0),
        ]
    }

    /// Add position sample to history.
    pub fn push_position(&mut self, pos: i32) {
        self.actual_position = pos;
        self.position_history.push(pos);
        if self.position_history.len() > 200 {
            self.position_history.remove(0);
        }
    }

    /// Add velocity sample to history.
    pub fn push_velocity(&mut self, vel: i32) {
        self.actual_velocity = vel;
        self.velocity_history.push(vel);
        if self.velocity_history.len() > 200 {
            self.velocity_history.remove(0);
        }
    }

    /// Add torque sample to history.
    pub fn push_torque(&mut self, torque: i16) {
        self.actual_torque = torque;
        self.torque_history.push(torque);
        if self.torque_history.len() > 200 {
            self.torque_history.remove(0);
        }
    }
}

/// DS402 operation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ds402Mode {
    #[default]
    ProfilePosition,
    InterpolatedPosition,
    CyclicSyncPosition,
    CyclicSyncVelocity,
    CyclicSyncTorque,
    ProfileVelocity,
    ProfileTorque,
}

impl Ds402Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ProfilePosition => "PP (Profile Position)",
            Self::InterpolatedPosition => "IP (Interpolated Position)",
            Self::CyclicSyncPosition => "CSP (Cyclic Sync Position)",
            Self::CyclicSyncVelocity => "CSV (Cyclic Sync Velocity)",
            Self::CyclicSyncTorque => "CST (Cyclic Sync Torque)",
            Self::ProfileVelocity => "PV (Profile Velocity)",
            Self::ProfileTorque => "PT (Profile Torque)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::ProfilePosition => "PP",
            Self::InterpolatedPosition => "IP",
            Self::CyclicSyncPosition => "CSP",
            Self::CyclicSyncVelocity => "CSV",
            Self::CyclicSyncTorque => "CST",
            Self::ProfileVelocity => "PV",
            Self::ProfileTorque => "PT",
        }
    }

    pub fn mode_value(&self) -> i8 {
        match self {
            Self::ProfilePosition => 1,
            Self::InterpolatedPosition => 7,
            Self::CyclicSyncPosition => 8,
            Self::CyclicSyncVelocity => 9,
            Self::CyclicSyncTorque => 10,
            Self::ProfileVelocity => 3,
            Self::ProfileTorque => 4,
        }
    }

    pub fn all() -> &'static [Ds402Mode] {
        &[
            Self::ProfilePosition,
            Self::InterpolatedPosition,
            Self::CyclicSyncPosition,
            Self::CyclicSyncVelocity,
            Self::CyclicSyncTorque,
            Self::ProfileVelocity,
            Self::ProfileTorque,
        ]
    }

    /// Parse mode from value.
    pub fn from_value(value: i8) -> Option<Self> {
        match value {
            1 => Some(Self::ProfilePosition),
            3 => Some(Self::ProfileVelocity),
            4 => Some(Self::ProfileTorque),
            7 => Some(Self::InterpolatedPosition),
            8 => Some(Self::CyclicSyncPosition),
            9 => Some(Self::CyclicSyncVelocity),
            10 => Some(Self::CyclicSyncTorque),
            _ => None,
        }
    }
}

/// DS402 control word commands.
#[derive(Debug, Clone, Copy)]
pub enum Ds402Command {
    Shutdown,
    SwitchOn,
    EnableOperation,
    DisableOperation,
    DisableVoltage,
    QuickStop,
    FaultReset,
}

impl Ds402Command {
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
