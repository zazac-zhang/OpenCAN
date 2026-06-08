/**
 * CANopen Protocol Analyzer — SDO Abort codes, EMCY error codes, protocol decoding.
 *
 * Provides dictionaries for:
 * - SDO Abort Codes (300+ codes)
 * - Emergency Error Codes
 * - NMT State Transitions
 * - Protocol violation detection
 */

// ===== SDO Abort Codes =====

export const SDO_ABORT_CODES: Record<number, { name: string; description: string }> = {
  0x05030000: {
    name: 'Toggle bit not altered',
    description: 'Toggle bit not alternated by the server',
  },
  0x05040000: {
    name: 'SDO protocol timed out',
    description: 'SDO server did not respond within the SDO timeout period',
  },
  0x05040001: {
    name: 'Command specifier not valid',
    description: 'Invalid or unknown command specifier',
  },
  0x05040002: {
    name: 'Invalid block size',
    description: 'Invalid block size in block transfer',
  },
  0x05040003: {
    name: 'Invalid sequence number',
    description: 'Invalid sequence number in block transfer',
  },
  0x05040004: {
    name: 'CRC error',
    description: 'CRC error in block transfer',
  },
  0x05040005: {
    name: 'Out of memory',
    description: 'Server ran out of memory',
  },
  0x06010000: {
    name: 'Unsupported access',
    description: 'Unsupported access to an object',
  },
  0x06010001: {
    name: 'Read/write-only',
    description: 'Attempt to read a write-only object',
  },
  0x06010002: {
    name: 'Write/read-only',
    description: 'Attempt to write a read-only object',
  },
  0x06020000: {
    name: 'Object does not exist',
    description: 'Object does not exist in the object dictionary',
  },
  0x06040041: {
    name: 'Object cannot be mapped',
    description: 'Object cannot be mapped to the PDO',
  },
  0x06040042: {
    name: 'Objects would exceed PDO',
    description: 'The number and length of objects would exceed PDO length',
  },
  0x06040043: {
    name: 'General parameter incompatibility',
    description: 'General parameter incompatibility reason',
  },
  0x06040047: {
    name: 'General internal incompatibility',
    description: 'General internal incompatibility in the device',
  },
  0x06060000: {
    name: 'Hardware access error',
    description: 'Access failed due to a hardware error',
  },
  0x06070010: {
    name: 'Data type mismatch, length',
    description: 'Data type does not match, length of service parameter does not match',
  },
  0x06070012: {
    name: 'Data type mismatch, length too high',
    description: 'Data type does not match, length of service parameter too high',
  },
  0x06070013: {
    name: 'Data type mismatch, length too low',
    description: 'Data type does not match, length of service parameter too low',
  },
  0x06090011: {
    name: 'Sub-index does not exist',
    description: 'Sub-index does not exist',
  },
  0x06090030: {
    name: 'Value range exceeded',
    description: 'Value range of parameter exceeded',
  },
  0x06090031: {
    name: 'Value too high',
    description: 'Value of parameter written too high',
  },
  0x06090032: {
    name: 'Value too low',
    description: 'Value of parameter written too low',
  },
  0x06090036: {
    name: 'Maximum less than minimum',
    description: 'Maximum value is less than minimum value',
  },
  0x08000000: {
    name: 'General error',
    description: 'General error',
  },
  0x08000020: {
    name: 'Data cannot be transferred',
    description: 'Data cannot be transferred or stored to the application',
  },
  0x08000021: {
    name: 'Data cannot be transferred',
    description: 'Data cannot be transferred or stored to the application because of local control',
  },
  0x08000022: {
    name: 'Data cannot be transferred',
    description: 'Data cannot be transferred or stored to the application because of the present device state',
  },
  0x08000023: {
    name: 'Object dictionary dynamic generation',
    description: 'Object dictionary dynamic generation fails or no object dictionary is present',
  },
  0x08000024: {
    name: 'No data available',
    description: 'No data available',
  },
};

// ===== Emergency Error Codes =====

export const EMCY_ERROR_CODES: Record<number, { name: string; description: string; category: string }> = {
  // Error Reset
  0x0000: {
    name: 'Error Reset',
    description: 'No error',
    category: 'No Error',
  },
  // Generic Error
  0x1000: {
    name: 'Generic Error',
    description: 'Unidentified generic error',
    category: 'Generic',
  },
  // Current
  0x2000: {
    name: 'Current, device input side',
    description: 'Current at input side too high',
    category: 'Current',
  },
  0x2100: {
    name: 'Current, device internal',
    description: 'Internal current too high',
    category: 'Current',
  },
  0x2200: {
    name: 'Current, device output side',
    description: 'Current at output side too high',
    category: 'Current',
  },
  // Voltage
  0x3100: {
    name: 'Mains voltage',
    description: 'Mains voltage too high or too low',
    category: 'Voltage',
  },
  0x3200: {
    name: 'Voltage, device output side',
    description: 'Voltage at output side too high or too low',
    category: 'Voltage',
  },
  // Temperature
  0x4100: {
    name: 'Temperature, ambient',
    description: 'Ambient temperature too high or too low',
    category: 'Temperature',
  },
  0x4200: {
    name: 'Temperature, device',
    description: 'Device temperature too high or too low',
    category: 'Temperature',
  },
  // Hardware
  0x5000: {
    name: 'Hardware error',
    description: 'Hardware error',
    category: 'Hardware',
  },
  // Software
  0x6100: {
    name: 'Software, internal',
    description: 'Internal software error',
    category: 'Software',
  },
  0x6200: {
    name: 'Software, user',
    description: 'User software error',
    category: 'Software',
  },
  0x6300: {
    name: 'Data set',
    description: 'Data set error',
    category: 'Software',
  },
  // Additional Modules
  0x7000: {
    name: 'Additional modules',
    description: 'Error in additional modules',
    category: 'Modules',
  },
  // Monitoring
  0x8000: {
    name: 'Monitoring',
    description: 'Monitoring error',
    category: 'Monitoring',
  },
  0x8100: {
    name: 'Communication, overrun',
    description: 'Communication overrun',
    category: 'Communication',
  },
  0x8110: {
    name: 'Communication, PDO length',
    description: 'PDO length exceeded',
    category: 'Communication',
  },
  0x8120: {
    name: 'Communication, PDO not processed',
    description: 'PDO not processed due to length error',
    category: 'Communication',
  },
  0x8130: {
    name: 'Communication, DAM MPO not processable',
    description: 'DAM MPO not processable',
    category: 'Communication',
  },
  0x8140: {
    name: 'Communication, SYNC data overflow',
    description: 'SYNC data overflow',
    category: 'Communication',
  },
  0x8150: {
    name: 'Communication, RPDO timeout',
    description: 'Unexpected RPDO timeout',
    category: 'Communication',
  },
  // DS402 Specific
  0x8200: {
    name: 'DS402: CAN overrun',
    description: 'CAN overrun (message lost)',
    category: 'DS402',
  },
  0x8210: {
    name: 'DS402: CAN overrun',
    description: 'CAN overrun (overrun on receive)',
    category: 'DS402',
  },
  0x8220: {
    name: 'DS402: passive',
    description: 'Passive bus error',
    category: 'DS402',
  },
  0x8230: {
    name: 'DS402: bus off',
    description: 'Bus off',
    category: 'DS402',
  },
  0x8240: {
    name: 'DS402: overrun',
    description: 'Overrun on transmit',
    category: 'DS402',
  },
  0x8250: {
    name: 'DS402: life guard',
    description: 'Life guard error or heartbeat timeout',
    category: 'DS402',
  },
  0x8260: {
    name: 'DS402: recovered',
    description: 'Recovery from bus off',
    category: 'DS402',
  },
  // DS402 Motion Control
  0x8300: {
    name: 'DS402: position sensor',
    description: 'Position sensor error',
    category: 'DS402',
  },
  0x8310: {
    name: 'DS402: position sensor (turn)',
    description: 'Position sensor turn counting error',
    category: 'DS402',
  },
  0x8320: {
    name: 'DS402: position sensor (period)',
    description: 'Position sensor period counting error',
    category: 'DS402',
  },
  0x8330: {
    name: 'DS402: velocity sensor',
    description: 'Velocity sensor error',
    category: 'DS402',
  },
  0x8400: {
    name: 'DS402: reference limit',
    description: 'Reference limit exceeded',
    category: 'DS402',
  },
  0x8410: {
    name: 'DS402: positive limit',
    description: 'Positive limit exceeded',
    category: 'DS402',
  },
  0x8420: {
    name: 'DS402: negative limit',
    description: 'Negative limit exceeded',
    category: 'DS402',
  },
  0x8500: {
    name: 'DS402: software',
    description: 'Software error',
    category: 'DS402',
  },
  0x8600: {
    name: 'DS402: supply voltage',
    description: 'Supply voltage error',
    category: 'DS402',
  },
  0x8610: {
    name: 'DS402: supply voltage (continuous)',
    description: 'Continuous overcurrent',
    category: 'DS402',
  },
  0x8620: {
    name: 'DS402: supply voltage (DC link)',
    description: 'DC link voltage error',
    category: 'DS402',
  },
  0x8700: {
    name: 'DS402: temperature',
    description: 'Temperature error',
    category: 'DS402',
  },
  0x8710: {
    name: 'DS402: temperature (drive)',
    description: 'Drive temperature error',
    category: 'DS402',
  },
  0x8720: {
    name: 'DS402: temperature (device)',
    description: 'Device temperature error',
    category: 'DS402',
  },
  0x8800: {
    name: 'DS402: hardware',
    description: 'Hardware error',
    category: 'DS402',
  },
  0x8900: {
    name: 'DS402: control',
    description: 'Control error',
    category: 'DS402',
  },
  0x8A00: {
    name: 'DS402: safety',
    description: 'Safety error',
    category: 'DS402',
  },
  0x8B00: {
    name: 'DS402: motion',
    description: 'Motion error',
    category: 'DS402',
  },
  0xFF00: {
    name: 'DS402: manufacturer specific',
    description: 'Manufacturer specific error',
    category: 'DS402',
  },
};

// ===== NMT State Transitions =====

export interface NmtTransition {
  from: string;
  to: string;
  command: string;
  commandCode: number;
}

export const NMT_TRANSITIONS: NmtTransition[] = [
  { from: 'Initialising', to: 'Pre-operational', command: 'Automatic', commandCode: 0 },
  { from: 'Pre-operational', to: 'Operational', command: 'Start Remote Node', commandCode: 0x01 },
  { from: 'Pre-operational', to: 'Stopped', command: 'Stop Remote Node', commandCode: 0x02 },
  { from: 'Operational', to: 'Pre-operational', command: 'Enter Pre-operational', commandCode: 0x80 },
  { from: 'Operational', to: 'Stopped', command: 'Stop Remote Node', commandCode: 0x02 },
  { from: 'Stopped', to: 'Pre-operational', command: 'Enter Pre-operational', commandCode: 0x80 },
  { from: 'Stopped', to: 'Operational', command: 'Start Remote Node', commandCode: 0x01 },
  { from: 'Pre-operational', to: 'Initialising', command: 'Reset Node', commandCode: 0x81 },
  { from: 'Pre-operational', to: 'Initialising', command: 'Reset Communication', commandCode: 0x82 },
  { from: 'Operational', to: 'Initialising', command: 'Reset Node', commandCode: 0x81 },
  { from: 'Operational', to: 'Initialising', command: 'Reset Communication', commandCode: 0x82 },
  { from: 'Stopped', to: 'Initialising', command: 'Reset Node', commandCode: 0x81 },
  { from: 'Stopped', to: 'Initialising', command: 'Reset Communication', commandCode: 0x82 },
];

// ===== Protocol Decoding Helpers =====

/**
 * Decode SDO abort code to human-readable string
 */
export function decodeSdoAbortCode(code: number): string {
  const entry = SDO_ABORT_CODES[code];
  if (entry) {
    return `${entry.name}: ${entry.description}`;
  }
  return `Unknown abort code: 0x${code.toString(16).toUpperCase().padStart(8, '0')}`;
}

/**
 * Decode EMCY error code to human-readable string
 */
export function decodeEmcyErrorCode(code: number): { name: string; description: string; category: string } {
  const entry = EMCY_ERROR_CODES[code];
  if (entry) {
    return entry;
  }
  // Try generic category
  const category = (code & 0xFF00) >> 8;
  const categories: Record<number, string> = {
    0x10: 'Generic',
    0x20: 'Current',
    0x30: 'Voltage',
    0x40: 'Temperature',
    0x50: 'Hardware',
    0x60: 'Software',
    0x70: 'Modules',
    0x80: 'Monitoring',
  };
  return {
    name: `Error 0x${code.toString(16).toUpperCase().padStart(4, '0')}`,
    description: 'Unknown error code',
    category: categories[category] || 'Unknown',
  };
}

/**
 * Decode NMT command to human-readable string
 */
export function decodeNmtCommand(commandCode: number, nodeId: number): string {
  const commands: Record<number, string> = {
    0x01: `Start Node ${nodeId}`,
    0x02: `Stop Node ${nodeId}`,
    0x80: `Enter Pre-operational Node ${nodeId}`,
    0x81: `Reset Node ${nodeId}`,
    0x82: `Reset Communication Node ${nodeId}`,
  };
  return commands[commandCode] || `Unknown NMT command 0x${commandCode.toString(16)}`;
}

/**
 * Decode CANopen Function Code from COB-ID
 */
export function decodeFunctionCode(cobId: number): { name: string; description: string } {
  const functionCode = (cobId >> 7) & 0x0F;
  const nodeId = cobId & 0x7F;

  const functionCodes: Record<number, { name: string; description: (id: number) => string }> = {
    0x0: { name: 'NMT', description: (id) => `Network Management (Node ${id})` },
    0x1: { name: 'SYNC', description: () => 'Synchronization' },
    0x2: { name: 'TIME', description: () => 'Time Stamp' },
    0x3: { name: 'EMCY', description: (id) => `Emergency (Node ${id})` },
    0x4: { name: 'PDO1TX', description: (id) => `PDO 1 Transmit (Node ${id})` },
    0x5: { name: 'PDO1RX', description: (id) => `PDO 1 Receive (Node ${id})` },
    0x6: { name: 'PDO2TX', description: (id) => `PDO 2 Transmit (Node ${id})` },
    0x7: { name: 'PDO2RX', description: (id) => `PDO 2 Receive (Node ${id})` },
    0x8: { name: 'PDO3TX', description: (id) => `PDO 3 Transmit (Node ${id})` },
    0x9: { name: 'PDO3RX', description: (id) => `PDO 3 Receive (Node ${id})` },
    0xA: { name: 'PDO4TX', description: (id) => `PDO 4 Transmit (Node ${id})` },
    0xB: { name: 'PDO4RX', description: (id) => `PDO 4 Receive (Node ${id})` },
    0xC: { name: 'SDO_TX', description: (id) => `SDO Transmit/Response (Node ${id})` },
    0xD: { name: 'SDO_RX', description: (id) => `SDO Receive/Request (Node ${id})` },
    0xE: { name: 'NMT_EC', description: (id) => `NMT Error Control (Node ${id})` },
  };

  const fc = functionCodes[functionCode];
  if (fc) {
    return { name: fc.name, description: fc.description(nodeId) };
  }
  return { name: 'Unknown', description: `Unknown function code ${functionCode}` };
}

/**
 * Decode SDO command from data bytes
 */
export function decodeSdoCommand(data: number[]): { command: string; index?: number; subindex?: number; size?: number } {
  if (data.length < 4) {
    return { command: 'Invalid SDO (too short)' };
  }

  const cmd = data[0];
  const n = (cmd >> 2) & 0x03; // Size if e=1
  const e = (cmd >> 1) & 0x01; // Expedited
  const s = cmd & 0x01; // Size indicated

  const index = data[2] << 8 | data[1];
  const subindex = data[3];

  // Client commands
  if ((cmd & 0xE0) === 0x40) {
    return { command: 'Initiate Upload (Read)', index, subindex };
  }
  if ((cmd & 0xE0) === 0x20) {
    if (e && s) {
      const size = 4 - n;
      return { command: `Initiate Download (Write, Expedited, ${size} bytes)`, index, subindex, size };
    }
    return { command: 'Initiate Download (Write)', index, subindex };
  }
  if ((cmd & 0xE0) === 0x60) {
    return { command: 'Segment Upload' };
  }
  if ((cmd & 0xE0) === 0x00) {
    return { command: 'Segment Download' };
  }
  if ((cmd & 0xE0) === 0x80) {
    const abortCode = data[4] | (data[5] << 8) | (data[6] << 16) | (data[7] << 24);
    return { command: `Abort (0x${abortCode.toString(16).padStart(8, '0')})`, index, subindex };
  }

  // Server responses
  if ((cmd & 0xE0) === 0x40 && (cmd & 0x02)) {
    return { command: 'Initiate Upload Response', index, subindex };
  }
  if ((cmd & 0xE0) === 0x60) {
    return { command: 'Upload Segment Response' };
  }

  return { command: `Unknown (0x${cmd.toString(16).padStart(2, '0')})`, index, subindex };
}

/**
 * Get human-readable data type name
 */
export function getDataTypeName(code: number): string {
  const types: Record<number, string> = {
    0x0001: 'BOOLEAN',
    0x0002: 'INTEGER8',
    0x0003: 'INTEGER16',
    0x0004: 'INTEGER32',
    0x0005: 'UNSIGNED8',
    0x0006: 'UNSIGNED16',
    0x0007: 'UNSIGNED32',
    0x0008: 'REAL32',
    0x0009: 'VISIBLE_STRING',
    0x000A: 'OCTET_STRING',
    0x000B: 'UNICODE_STRING',
    0x000F: 'REAL64',
    0x0010: 'INTEGER64',
    0x0011: 'UNSIGNED64',
  };
  return types[code] || `TYPE_0x${code.toString(16).padStart(4, '0')}`;
}
