//! SDO Abort Codes (DS301 Section 7.2.4.3).
//!
//! SDO abort codes indicate why an SDO transfer failed.
//! The abort code is a 32-bit value transmitted in the SDO abort message.

/// SDO abort code constants (DS301 Table 86).
///
/// These are standard abort codes defined by the CANOpen specification.
pub mod codes {
    /// Toggle bit not alternated.
    pub const TOGGLE_BIT: u32 = 0x0503_0000;

    /// SDO protocol timed out.
    pub const PROTOCOL_TIMEOUT: u32 = 0x0504_0000;

    /// Command specifier not valid or unknown.
    pub const INVALID_CS: u32 = 0x0504_0001;

    /// Invalid block size.
    pub const INVALID_BLOCK_SIZE: u32 = 0x0504_0002;

    /// Invalid sequence number.
    pub const INVALID_SEQ_NUM: u32 = 0x0504_0003;

    /// CRC error.
    pub const CRC_ERROR: u32 = 0x0504_0004;

    /// Out of memory.
    pub const OUT_OF_MEMORY: u32 = 0x0504_0005;

    /// Unsupported access to an object.
    pub const UNSUPPORTED_ACCESS: u32 = 0x0601_0000;

    /// Attempt to read a write only object.
    pub const READ_WRITE_ONLY: u32 = 0x0601_0001;

    /// Attempt to write a read only object.
    pub const WRITE_READ_ONLY: u32 = 0x0601_0002;

    /// Object does not exist in the object dictionary.
    pub const OBJECT_NOT_FOUND: u32 = 0x0602_0000;

    /// Object cannot be mapped to the PDO.
    pub const PDO_MAP_FAILED: u32 = 0x0604_0041;

    /// The number and length of the objects to be mapped would exceed the PDO length.
    pub const PDO_LENGTH_EXCEEDED: u32 = 0x0604_0042;

    /// General parameter incompatibility reason.
    pub const PARAM_INCOMPATIBLE: u32 = 0x0604_0043;

    /// Access failed due to a hardware error.
    pub const HARDWARE_ERROR: u32 = 0x0606_0000;

    /// Data type does not match, length of service parameter does not match.
    pub const TYPE_MISMATCH_LEN_HIGH: u32 = 0x0607_0010;

    /// Data type does not match, length of service parameter too high.
    pub const TYPE_MISMATCH_LEN_LOW: u32 = 0x0607_0012;

    /// Sub-index does not exist.
    pub const SUBINDEX_NOT_FOUND: u32 = 0x0609_0011;

    /// Value range of parameter exceeded.
    pub const VALUE_RANGE_EXCEEDED: u32 = 0x0609_0030;

    /// Value of parameter written too high.
    pub const VALUE_TOO_HIGH: u32 = 0x0609_0031;

    /// Value of parameter written too low.
    pub const VALUE_TOO_LOW: u32 = 0x0609_0032;

    /// Maximum value is less than minimum value.
    pub const MAX_LESS_THAN_MIN: u32 = 0x0609_0036;

    /// General error.
    pub const GENERAL_ERROR: u32 = 0x0800_0000;

    /// Data cannot be transferred or stored to the application.
    pub const DATA_NOT_TRANSFERRED: u32 = 0x0800_0020;

    /// Data cannot be transferred or stored to the application because of local control.
    pub const LOCAL_CONTROL: u32 = 0x0800_0021;

    /// Data cannot be transferred or stored to the application because of the present device state.
    pub const DEVICE_STATE: u32 = 0x0800_0022;

    /// Object dictionary dynamic generation error or no object dictionary present.
    pub const OD_ERROR: u32 = 0x0800_0023;
}

/// Get a human-readable description of an SDO abort code.
///
/// Returns a static string describing the abort reason.
///
/// # Example
/// ```
/// use opencan_canopen_core::sdo_abort::abort_reason;
///
/// assert_eq!(abort_reason(0x0602_0000), "Object does not exist");
/// assert_eq!(abort_reason(0x0504_0000), "SDO protocol timed out");
/// ```
pub fn abort_reason(code: u32) -> &'static str {
    match code {
        codes::TOGGLE_BIT => "Toggle bit not alternated",
        codes::PROTOCOL_TIMEOUT => "SDO protocol timed out",
        codes::INVALID_CS => "Command specifier not valid or unknown",
        codes::INVALID_BLOCK_SIZE => "Invalid block size",
        codes::INVALID_SEQ_NUM => "Invalid sequence number",
        codes::CRC_ERROR => "CRC error",
        codes::OUT_OF_MEMORY => "Out of memory",
        codes::UNSUPPORTED_ACCESS => "Unsupported access to an object",
        codes::READ_WRITE_ONLY => "Attempt to read a write only object",
        codes::WRITE_READ_ONLY => "Attempt to write a read only object",
        codes::OBJECT_NOT_FOUND => "Object does not exist",
        codes::PDO_MAP_FAILED => "Object cannot be mapped to the PDO",
        codes::PDO_LENGTH_EXCEEDED => "Number and length of objects exceed PDO",
        codes::PARAM_INCOMPATIBLE => "General parameter incompatibility",
        codes::HARDWARE_ERROR => "Access failed due to hardware error",
        codes::TYPE_MISMATCH_LEN_HIGH => "Data type does not match, length too high",
        codes::TYPE_MISMATCH_LEN_LOW => "Data type does not match, length too low",
        codes::SUBINDEX_NOT_FOUND => "Sub-index does not exist",
        codes::VALUE_RANGE_EXCEEDED => "Value range of parameter exceeded",
        codes::VALUE_TOO_HIGH => "Value of parameter written too high",
        codes::VALUE_TOO_LOW => "Value of parameter written too low",
        codes::MAX_LESS_THAN_MIN => "Maximum value is less than minimum value",
        codes::GENERAL_ERROR => "General error",
        codes::DATA_NOT_TRANSFERRED => "Data cannot be transferred or stored",
        codes::LOCAL_CONTROL => "Data cannot be transferred because of local control",
        codes::DEVICE_STATE => "Data cannot be transferred because of device state",
        codes::OD_ERROR => "Object dictionary dynamic generation error",
        _ => "Unknown abort code",
    }
}

/// Emergency Error Codes (DS301 Section 7.2.5 / CiA 301 Table 88).
///
/// Emergency error codes are sent in Emergency frames to indicate device errors.
pub mod emcy {
    /// Error Register (0x1001) bit definitions.
    pub mod error_register {
        /// Generic error.
        pub const GENERIC: u8 = 0x01;
        /// Current error (drive section).
        pub const CURRENT: u8 = 0x02;
        /// Voltage error.
        pub const VOLTAGE: u8 = 0x04;
        /// Temperature error.
        pub const TEMPERATURE: u8 = 0x08;
        /// Communication error (CANOverrun, heartbeat, etc.).
        pub const COMMUNICATION: u8 = 0x10;
        /// Device profile-specific error.
        pub const DEVICE_PROFILE: u8 = 0x20;
        /// Reserved (manufacturer-specific).
        pub const MANUFACTURER: u8 = 0x80;
    }

    /// Emergency error code categories (upper byte of error code).
    pub mod error_codes {
        /// Error Reset / No Error.
        pub const NO_ERROR: u16 = 0x0000;

        // === Generic Error Codes (0x10xx) ===

        /// Generic error.
        pub const GENERIC_ERROR: u16 = 0x1000;
        /// Current: current at main circuit too high.
        pub const CURRENT_MAIN_CIRCUIT_HIGH: u16 = 0x2310;
        /// Current: short circuit.
        pub const SHORT_CIRCUIT: u16 = 0x2320;
        /// Current: current at load too high.
        pub const CURRENT_LOAD_HIGH: u16 = 0x2330;

        // === Voltage Errors (0x3xxx) ===

        /// Voltage: mains voltage too high.
        pub const MAINS_VOLTAGE_HIGH: u16 = 0x3110;
        /// Voltage: mains voltage too low.
        pub const MAINS_VOLTAGE_LOW: u16 = 0x3120;
        /// Voltage: DC link voltage too high.
        pub const DC_LINK_HIGH: u16 = 0x3210;
        /// Voltage: DC link voltage too low.
        pub const DC_LINK_LOW: u16 = 0x3220;
        /// Voltage: output voltage too high.
        pub const OUTPUT_VOLTAGE_HIGH: u16 = 0x3310;
        /// Voltage: output voltage too low.
        pub const OUTPUT_VOLTAGE_LOW: u16 = 0x3320;

        // === Temperature Errors (0x4xxx) ===

        /// Temperature: ambient temperature too high.
        pub const AMBIENT_TEMP_HIGH: u16 = 0x4110;
        /// Temperature: ambient temperature too low.
        pub const AMBIENT_TEMP_LOW: u16 = 0x4120;
        /// Temperature: drive temperature too high.
        pub const DRIVE_TEMP_HIGH: u16 = 0x4210;
        /// Temperature: drive temperature too low.
        pub const DRIVE_TEMP_LOW: u16 = 0x4220;
        /// Temperature: device temperature too high.
        pub const DEVICE_TEMP_HIGH: u16 = 0x4310;
        /// Temperature: device temperature too low.
        pub const DEVICE_TEMP_LOW: u16 = 0x4320;

        // === Device Hardware Errors (0x5xxx) ===

        /// Device hardware: supply voltage too low.
        pub const SUPPLY_VOLTAGE_LOW: u16 = 0x5110;
        /// Device hardware: supply voltage too high.
        pub const SUPPLY_VOLTAGE_HIGH: u16 = 0x5120;
        /// Device hardware: internal supply voltage too low.
        pub const INTERNAL_SUPPLY_LOW: u16 = 0x5210;
        /// Device hardware: internal supply voltage too high.
        pub const INTERNAL_SUPPLY_HIGH: u16 = 0x5220;

        // === Device Software Errors (0x6xxx) ===

        /// Device software: software reset.
        pub const SOFTWARE_RESET: u16 = 0x6100;
        /// Device software: watchdog.
        pub const WATCHDOG: u16 = 0x6110;
        /// Device software: internal software error.
        pub const INTERNAL_SOFTWARE: u16 = 0x6120;
        /// Device software: user software error.
        pub const USER_SOFTWARE: u16 = 0x6130;

        // === Additional Errors (0x7xxx - 0x8xxx) ===

        /// Additional modules: overvoltage.
        pub const OVERVOLTAGE: u16 = 0x7110;
        /// Additional modules: undervoltage.
        pub const UNDERVOLTAGE: u16 = 0x7120;

        // === Communication Errors (0x8xxx) ===

        /// Communication: CAN overrun (objects lost).
        pub const CAN_OVERRUN: u16 = 0x8110;
        /// Communication: CAN in error passive mode.
        pub const CAN_ERROR_PASSIVE: u16 = 0x8120;
        /// Communication: life guard error / heartbeat error.
        pub const HEARTBEAT_ERROR: u16 = 0x8130;
        /// Communication: recovered from bus off.
        pub const BUS_OFF_RECOVERED: u16 = 0x8140;
        /// Communication: TX COB-ID collision.
        pub const TX_COBID_COLLISION: u16 = 0x8150;

        // === Protocol Errors (0x82xx) ===

        /// Protocol: PDO length exceeded.
        pub const PDO_LENGTH: u16 = 0x8210;
        /// Protocol: PDO length exceeded (device specific).
        pub const PDO_LENGTH_DEV: u16 = 0x8220;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abort_reason_known_codes() {
        assert_eq!(
            abort_reason(codes::OBJECT_NOT_FOUND),
            "Object does not exist"
        );
        assert_eq!(
            abort_reason(codes::SUBINDEX_NOT_FOUND),
            "Sub-index does not exist"
        );
        assert_eq!(
            abort_reason(codes::UNSUPPORTED_ACCESS),
            "Unsupported access to an object"
        );
        assert_eq!(
            abort_reason(codes::PROTOCOL_TIMEOUT),
            "SDO protocol timed out"
        );
    }

    #[test]
    fn test_abort_reason_unknown() {
        assert_eq!(abort_reason(0xFFFF_FFFF), "Unknown abort code");
        assert_eq!(abort_reason(0x0000_0000), "Unknown abort code");
    }

    #[test]
    fn test_emcy_error_codes() {
        assert_eq!(emcy::error_codes::NO_ERROR, 0x0000);
        assert_eq!(emcy::error_codes::GENERIC_ERROR, 0x1000);
        assert_eq!(emcy::error_codes::HEARTBEAT_ERROR, 0x8130);
    }

    #[test]
    fn test_error_register_bits() {
        assert_eq!(emcy::error_register::GENERIC, 0x01);
        assert_eq!(emcy::error_register::CURRENT, 0x02);
        assert_eq!(emcy::error_register::VOLTAGE, 0x04);
        assert_eq!(emcy::error_register::TEMPERATURE, 0x08);
        assert_eq!(emcy::error_register::COMMUNICATION, 0x10);
    }
}
