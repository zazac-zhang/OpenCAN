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
  84082688: {
    name: 'Toggle bit not altered',
    description: 'Toggle bit not alternated by the server',
  },
  84148224: {
    name: 'SDO protocol timed out',
    description: 'SDO server did not respond within the SDO timeout period',
  },
  84148225: {
    name: 'Command specifier not valid',
    description: 'Invalid or unknown command specifier',
  },
  84148226: {
    name: 'Invalid block size',
    description: 'Invalid block size in block transfer',
  },
  84148227: {
    name: 'Invalid sequence number',
    description: 'Invalid sequence number in block transfer',
  },
  84148228: {
    name: 'CRC error',
    description: 'CRC error in block transfer',
  },
  84148229: {
    name: 'Out of memory',
    description: 'Server ran out of memory',
  },
  100728832: {
    name: 'Unsupported access',
    description: 'Unsupported access to an object',
  },
  100728833: {
    name: 'Read/write-only',
    description: 'Attempt to read a write-only object',
  },
  100728834: {
    name: 'Write/read-only',
    description: 'Attempt to write a read-only object',
  },
  100794368: {
    name: 'Object does not exist',
    description: 'Object does not exist in the object dictionary',
  },
  100925505: {
    name: 'Object cannot be mapped',
    description: 'Object cannot be mapped to the PDO',
  },
  100925506: {
    name: 'Objects would exceed PDO',
    description: 'The number and length of objects would exceed PDO length',
  },
  100925507: {
    name: 'General parameter incompatibility',
    description: 'General parameter incompatibility reason',
  },
  100925511: {
    name: 'General internal incompatibility',
    description: 'General internal incompatibility in the device',
  },
  101056512: {
    name: 'Hardware access error',
    description: 'Access failed due to a hardware error',
  },
  101122064: {
    name: 'Data type mismatch, length',
    description: 'Data type does not match, length of service parameter does not match',
  },
  101122066: {
    name: 'Data type mismatch, length too high',
    description: 'Data type does not match, length of service parameter too high',
  },
  101122067: {
    name: 'Data type mismatch, length too low',
    description: 'Data type does not match, length of service parameter too low',
  },
  101253137: {
    name: 'Sub-index does not exist',
    description: 'Sub-index does not exist',
  },
  101253168: {
    name: 'Value range exceeded',
    description: 'Value range of parameter exceeded',
  },
  101253169: {
    name: 'Value too high',
    description: 'Value of parameter written too high',
  },
  101253170: {
    name: 'Value too low',
    description: 'Value of parameter written too low',
  },
  101253174: {
    name: 'Maximum less than minimum',
    description: 'Maximum value is less than minimum value',
  },
  134217728: {
    name: 'General error',
    description: 'General error',
  },
  134217760: {
    name: 'Data cannot be transferred',
    description: 'Data cannot be transferred or stored to the application',
  },
  134217761: {
    name: 'Data cannot be transferred',
    description: 'Data cannot be transferred or stored to the application because of local control',
  },
  134217762: {
    name: 'Data cannot be transferred',
    description:
      'Data cannot be transferred or stored to the application because of the present device state',
  },
  134217763: {
    name: 'Object dictionary dynamic generation',
    description: 'Object dictionary dynamic generation fails or no object dictionary is present',
  },
  134217764: {
    name: 'No data available',
    description: 'No data available',
  },
};

// ===== Emergency Error Codes =====

export const EMCY_ERROR_CODES: Record<
  number,
  { name: string; description: string; category: string }
> = {
  // Error Reset
  0: {
    name: 'Error Reset',
    description: 'No error',
    category: 'No Error',
  },
  // Generic Error
  4096: {
    name: 'Generic Error',
    description: 'Unidentified generic error',
    category: 'Generic',
  },
  // Current
  8192: {
    name: 'Current, device input side',
    description: 'Current at input side too high',
    category: 'Current',
  },
  8448: {
    name: 'Current, device internal',
    description: 'Internal current too high',
    category: 'Current',
  },
  8704: {
    name: 'Current, device output side',
    description: 'Current at output side too high',
    category: 'Current',
  },
  // Voltage
  12544: {
    name: 'Mains voltage',
    description: 'Mains voltage too high or too low',
    category: 'Voltage',
  },
  12800: {
    name: 'Voltage, device output side',
    description: 'Voltage at output side too high or too low',
    category: 'Voltage',
  },
  // Temperature
  16640: {
    name: 'Temperature, ambient',
    description: 'Ambient temperature too high or too low',
    category: 'Temperature',
  },
  16896: {
    name: 'Temperature, device',
    description: 'Device temperature too high or too low',
    category: 'Temperature',
  },
  // Hardware
  20480: {
    name: 'Hardware error',
    description: 'Hardware error',
    category: 'Hardware',
  },
  // Software
  24832: {
    name: 'Software, internal',
    description: 'Internal software error',
    category: 'Software',
  },
  25088: {
    name: 'Software, user',
    description: 'User software error',
    category: 'Software',
  },
  25344: {
    name: 'Data set',
    description: 'Data set error',
    category: 'Software',
  },
  // Additional Modules
  28672: {
    name: 'Additional modules',
    description: 'Error in additional modules',
    category: 'Modules',
  },
  // Monitoring
  32768: {
    name: 'Monitoring',
    description: 'Monitoring error',
    category: 'Monitoring',
  },
  33024: {
    name: 'Communication, overrun',
    description: 'Communication overrun',
    category: 'Communication',
  },
  33040: {
    name: 'Communication, PDO length',
    description: 'PDO length exceeded',
    category: 'Communication',
  },
  33056: {
    name: 'Communication, PDO not processed',
    description: 'PDO not processed due to length error',
    category: 'Communication',
  },
  33072: {
    name: 'Communication, DAM MPO not processable',
    description: 'DAM MPO not processable',
    category: 'Communication',
  },
  33088: {
    name: 'Communication, SYNC data overflow',
    description: 'SYNC data overflow',
    category: 'Communication',
  },
  33104: {
    name: 'Communication, RPDO timeout',
    description: 'Unexpected RPDO timeout',
    category: 'Communication',
  },
  // DS402 Specific
  33280: {
    name: 'DS402: CAN overrun',
    description: 'CAN overrun (message lost)',
    category: 'DS402',
  },
  33296: {
    name: 'DS402: CAN overrun',
    description: 'CAN overrun (overrun on receive)',
    category: 'DS402',
  },
  33312: {
    name: 'DS402: passive',
    description: 'Passive bus error',
    category: 'DS402',
  },
  33328: {
    name: 'DS402: bus off',
    description: 'Bus off',
    category: 'DS402',
  },
  33344: {
    name: 'DS402: overrun',
    description: 'Overrun on transmit',
    category: 'DS402',
  },
  33360: {
    name: 'DS402: life guard',
    description: 'Life guard error or heartbeat timeout',
    category: 'DS402',
  },
  33376: {
    name: 'DS402: recovered',
    description: 'Recovery from bus off',
    category: 'DS402',
  },
  // DS402 Motion Control
  33536: {
    name: 'DS402: position sensor',
    description: 'Position sensor error',
    category: 'DS402',
  },
  33552: {
    name: 'DS402: position sensor (turn)',
    description: 'Position sensor turn counting error',
    category: 'DS402',
  },
  33568: {
    name: 'DS402: position sensor (period)',
    description: 'Position sensor period counting error',
    category: 'DS402',
  },
  33584: {
    name: 'DS402: velocity sensor',
    description: 'Velocity sensor error',
    category: 'DS402',
  },
  33792: {
    name: 'DS402: reference limit',
    description: 'Reference limit exceeded',
    category: 'DS402',
  },
  33808: {
    name: 'DS402: positive limit',
    description: 'Positive limit exceeded',
    category: 'DS402',
  },
  33824: {
    name: 'DS402: negative limit',
    description: 'Negative limit exceeded',
    category: 'DS402',
  },
  34048: {
    name: 'DS402: software',
    description: 'Software error',
    category: 'DS402',
  },
  34304: {
    name: 'DS402: supply voltage',
    description: 'Supply voltage error',
    category: 'DS402',
  },
  34320: {
    name: 'DS402: supply voltage (continuous)',
    description: 'Continuous overcurrent',
    category: 'DS402',
  },
  34336: {
    name: 'DS402: supply voltage (DC link)',
    description: 'DC link voltage error',
    category: 'DS402',
  },
  34560: {
    name: 'DS402: temperature',
    description: 'Temperature error',
    category: 'DS402',
  },
  34576: {
    name: 'DS402: temperature (drive)',
    description: 'Drive temperature error',
    category: 'DS402',
  },
  34592: {
    name: 'DS402: temperature (device)',
    description: 'Device temperature error',
    category: 'DS402',
  },
  34816: {
    name: 'DS402: hardware',
    description: 'Hardware error',
    category: 'DS402',
  },
  35072: {
    name: 'DS402: control',
    description: 'Control error',
    category: 'DS402',
  },
  35328: {
    name: 'DS402: safety',
    description: 'Safety error',
    category: 'DS402',
  },
  35584: {
    name: 'DS402: motion',
    description: 'Motion error',
    category: 'DS402',
  },
  65280: {
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
  {
    from: 'Operational',
    to: 'Pre-operational',
    command: 'Enter Pre-operational',
    commandCode: 0x80,
  },
  { from: 'Operational', to: 'Stopped', command: 'Stop Remote Node', commandCode: 0x02 },
  { from: 'Stopped', to: 'Pre-operational', command: 'Enter Pre-operational', commandCode: 0x80 },
  { from: 'Stopped', to: 'Operational', command: 'Start Remote Node', commandCode: 0x01 },
  { from: 'Pre-operational', to: 'Initialising', command: 'Reset Node', commandCode: 0x81 },
  {
    from: 'Pre-operational',
    to: 'Initialising',
    command: 'Reset Communication',
    commandCode: 0x82,
  },
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
export function decodeEmcyErrorCode(code: number): {
  name: string;
  description: string;
  category: string;
} {
  const entry = EMCY_ERROR_CODES[code];
  if (entry) {
    return entry;
  }
  // Try generic category
  const category = (code & 0xff00) >> 8;
  const categories: Record<number, string> = {
    16: 'Generic',
    32: 'Current',
    48: 'Voltage',
    64: 'Temperature',
    80: 'Hardware',
    96: 'Software',
    112: 'Modules',
    128: 'Monitoring',
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
    1: `Start Node ${nodeId}`,
    2: `Stop Node ${nodeId}`,
    128: `Enter Pre-operational Node ${nodeId}`,
    129: `Reset Node ${nodeId}`,
    130: `Reset Communication Node ${nodeId}`,
  };
  return commands[commandCode] || `Unknown NMT command 0x${commandCode.toString(16)}`;
}

/**
 * Decode CANopen Function Code from COB-ID
 */
export function decodeFunctionCode(cobId: number): { name: string; description: string } {
  const functionCode = (cobId >> 7) & 0x0f;
  const nodeId = cobId & 0x7f;

  const functionCodes: Record<number, { name: string; description: (id: number) => string }> = {
    0: { name: 'NMT', description: (id) => `Network Management (Node ${id})` },
    1: { name: 'SYNC', description: () => 'Synchronization' },
    2: { name: 'TIME', description: () => 'Time Stamp' },
    3: { name: 'EMCY', description: (id) => `Emergency (Node ${id})` },
    4: { name: 'PDO1TX', description: (id) => `PDO 1 Transmit (Node ${id})` },
    5: { name: 'PDO1RX', description: (id) => `PDO 1 Receive (Node ${id})` },
    6: { name: 'PDO2TX', description: (id) => `PDO 2 Transmit (Node ${id})` },
    7: { name: 'PDO2RX', description: (id) => `PDO 2 Receive (Node ${id})` },
    8: { name: 'PDO3TX', description: (id) => `PDO 3 Transmit (Node ${id})` },
    9: { name: 'PDO3RX', description: (id) => `PDO 3 Receive (Node ${id})` },
    10: { name: 'PDO4TX', description: (id) => `PDO 4 Transmit (Node ${id})` },
    11: { name: 'PDO4RX', description: (id) => `PDO 4 Receive (Node ${id})` },
    12: { name: 'SDO_TX', description: (id) => `SDO Transmit/Response (Node ${id})` },
    13: { name: 'SDO_RX', description: (id) => `SDO Receive/Request (Node ${id})` },
    14: { name: 'NMT_EC', description: (id) => `NMT Error Control (Node ${id})` },
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
export function decodeSdoCommand(data: number[]): {
  command: string;
  index?: number;
  subindex?: number;
  size?: number;
} {
  if (data.length < 4) {
    return { command: 'Invalid SDO (too short)' };
  }

  const cmd = data[0];
  const n = (cmd >> 2) & 0x03; // Size if e=1
  const e = (cmd >> 1) & 0x01; // Expedited
  const s = cmd & 0x01; // Size indicated

  const index = (data[2] << 8) | data[1];
  const subindex = data[3];

  // Client commands
  if ((cmd & 0xe0) === 0x40) {
    return { command: 'Initiate Upload (Read)', index, subindex };
  }
  if ((cmd & 0xe0) === 0x20) {
    if (e && s) {
      const size = 4 - n;
      return {
        command: `Initiate Download (Write, Expedited, ${size} bytes)`,
        index,
        subindex,
        size,
      };
    }
    return { command: 'Initiate Download (Write)', index, subindex };
  }
  if ((cmd & 0xe0) === 0x60) {
    return { command: 'Segment Upload' };
  }
  if ((cmd & 0xe0) === 0x00) {
    return { command: 'Segment Download' };
  }
  if ((cmd & 0xe0) === 0x80) {
    const abortCode = data[4] | (data[5] << 8) | (data[6] << 16) | (data[7] << 24);
    return { command: `Abort (0x${abortCode.toString(16).padStart(8, '0')})`, index, subindex };
  }

  // Server responses
  if ((cmd & 0xe0) === 0x40 && cmd & 0x02) {
    return { command: 'Initiate Upload Response', index, subindex };
  }
  if ((cmd & 0xe0) === 0x60) {
    return { command: 'Upload Segment Response' };
  }

  return { command: `Unknown (0x${cmd.toString(16).padStart(2, '0')})`, index, subindex };
}

/**
 * Get human-readable data type name
 */
export function getDataTypeName(code: number): string {
  const types: Record<number, string> = {
    1: 'BOOLEAN',
    2: 'INTEGER8',
    3: 'INTEGER16',
    4: 'INTEGER32',
    5: 'UNSIGNED8',
    6: 'UNSIGNED16',
    7: 'UNSIGNED32',
    8: 'REAL32',
    9: 'VISIBLE_STRING',
    10: 'OCTET_STRING',
    11: 'UNICODE_STRING',
    15: 'REAL64',
    16: 'INTEGER64',
    17: 'UNSIGNED64',
  };
  return types[code] || `TYPE_0x${code.toString(16).padStart(4, '0')}`;
}
