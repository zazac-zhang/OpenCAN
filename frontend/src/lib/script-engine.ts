/**
 * Script Engine — Simple DSL for automated CAN/CANopen testing.
 *
 * Supported commands:
 * - send <id> <data>           — Send CAN frame
 * - sdo_read <node> <idx> <sub> — Read SDO
 * - sdo_write <node> <idx> <sub> <value> — Write SDO
 * - wait <ms>                  — Wait milliseconds
 * - log <message>              — Log message
 * - repeat <n> { ... }        — Repeat block n times
 * - // comment                 — Line comment
 */

// ===== Types =====

export interface ScriptCommand {
  type: 'send' | 'sdo_read' | 'sdo_write' | 'wait' | 'log' | 'repeat' | 'nmt' | 'comment';
  line: number;
  params: Record<string, string | number>;
  body?: ScriptCommand[]; // For repeat blocks
}

export interface ScriptParseResult {
  commands: ScriptCommand[];
  errors: { line: number; message: string }[];
}

export interface ScriptExecutionLog {
  timestamp: number;
  command: string;
  status: 'running' | 'success' | 'error' | 'skipped';
  message?: string;
  duration?: number;
}

export interface ScriptExecutionState {
  running: boolean;
  currentLine: number;
  log: ScriptExecutionLog[];
  variables: Record<string, number>;
}

// ===== Parser =====

export function parseScript(script: string): ScriptParseResult {
  const lines = script.split('\n');
  const commands: ScriptCommand[] = [];
  const errors: { line: number; message: string }[] = [];

  let lineNum = 0;
  let i = 0;

  function parseLine(): ScriptCommand | null {
    while (i < lines.length) {
      lineNum = i + 1;
      const line = lines[i].trim();
      i++;

      // Skip empty lines and comments
      if (!line || line.startsWith('//')) {
        continue;
      }

      // Parse command
      const parts = line.split(/\s+/);
      const cmd = parts[0].toLowerCase();

      switch (cmd) {
        case 'send': {
          if (parts.length < 3) {
            errors.push({ line: lineNum, message: 'Usage: send <id> <data>' });
            continue;
          }
          return {
            type: 'send',
            line: lineNum,
            params: {
              id: parts[1],
              data: parts.slice(2).join(' '),
            },
          };
        }

        case 'sdo_read': {
          if (parts.length < 4) {
            errors.push({ line: lineNum, message: 'Usage: sdo_read <node> <index> <subindex>' });
            continue;
          }
          return {
            type: 'sdo_read',
            line: lineNum,
            params: {
              node: parseInt(parts[1], 10),
              index: parts[2],
              subindex: parts[3],
            },
          };
        }

        case 'sdo_write': {
          if (parts.length < 5) {
            errors.push({
              line: lineNum,
              message: 'Usage: sdo_write <node> <index> <subindex> <value>',
            });
            continue;
          }
          return {
            type: 'sdo_write',
            line: lineNum,
            params: {
              node: parseInt(parts[1], 10),
              index: parts[2],
              subindex: parts[3],
              value: parts.slice(4).join(' '),
            },
          };
        }

        case 'wait': {
          if (parts.length < 2) {
            errors.push({ line: lineNum, message: 'Usage: wait <ms>' });
            continue;
          }
          return {
            type: 'wait',
            line: lineNum,
            params: { ms: parseInt(parts[1], 10) },
          };
        }

        case 'log': {
          return {
            type: 'log',
            line: lineNum,
            params: { message: parts.slice(1).join(' ') },
          };
        }

        case 'nmt': {
          if (parts.length < 3) {
            errors.push({ line: lineNum, message: 'Usage: nmt <node> <command>' });
            continue;
          }
          return {
            type: 'nmt',
            line: lineNum,
            params: {
              node: parseInt(parts[1], 10),
              command: parts[2],
            },
          };
        }

        case 'repeat': {
          if (parts.length < 2) {
            errors.push({ line: lineNum, message: 'Usage: repeat <count> {' });
            continue;
          }
          const count = parseInt(parts[1], 10);
          if (Number.isNaN(count) || count < 1) {
            errors.push({ line: lineNum, message: 'Invalid repeat count' });
            continue;
          }

          // Find matching closing brace
          const bodyCommands: ScriptCommand[] = [];
          let braceDepth = 1;
          while (i < lines.length && braceDepth > 0) {
            const bodyLine = lines[i].trim();
            if (bodyLine === '{') {
              braceDepth++;
              i++;
            } else if (bodyLine === '}') {
              braceDepth--;
              i++;
            } else {
              const cmd = parseLine();
              if (cmd) {
                bodyCommands.push(cmd);
              }
            }
          }

          return {
            type: 'repeat',
            line: lineNum,
            params: { count },
            body: bodyCommands,
          };
        }

        case '{':
        case '}':
          // Skip braces (handled by repeat)
          continue;

        default:
          errors.push({ line: lineNum, message: `Unknown command: ${cmd}` });
          continue;
      }
    }
    return null;
  }

  while (i < lines.length) {
    const cmd = parseLine();
    if (cmd) {
      commands.push(cmd);
    }
  }

  return { commands, errors };
}

// ===== Executor =====

export type CommandExecutor = (
  command: ScriptCommand,
) => Promise<{ success: boolean; message?: string }>;

export async function executeScript(
  commands: ScriptCommand[],
  executor: CommandExecutor,
  onLog: (log: ScriptExecutionLog) => void,
  signal?: AbortSignal,
): Promise<void> {
  for (const cmd of commands) {
    if (signal?.aborted) {
      onLog({
        timestamp: Date.now(),
        command: formatCommand(cmd),
        status: 'skipped',
        message: 'Cancelled',
      });
      break;
    }

    const startTime = Date.now();

    switch (cmd.type) {
      case 'repeat': {
        const count = cmd.params.count as number;
        for (let i = 0; i < count; i++) {
          if (signal?.aborted) break;
          onLog({
            timestamp: Date.now(),
            command: `repeat ${i + 1}/${count}`,
            status: 'running',
          });
          if (cmd.body) {
            await executeScript(cmd.body, executor, onLog, signal);
          }
        }
        break;
      }

      default: {
        onLog({
          timestamp: Date.now(),
          command: formatCommand(cmd),
          status: 'running',
        });

        try {
          const result = await executor(cmd);
          onLog({
            timestamp: Date.now(),
            command: formatCommand(cmd),
            status: result.success ? 'success' : 'error',
            message: result.message,
            duration: Date.now() - startTime,
          });
        } catch (error) {
          onLog({
            timestamp: Date.now(),
            command: formatCommand(cmd),
            status: 'error',
            message: String(error),
            duration: Date.now() - startTime,
          });
        }
        break;
      }
    }
  }
}

function formatCommand(cmd: ScriptCommand): string {
  switch (cmd.type) {
    case 'send':
      return `send ${cmd.params.id} ${cmd.params.data}`;
    case 'sdo_read':
      return `sdo_read ${cmd.params.node} ${cmd.params.index} ${cmd.params.subindex}`;
    case 'sdo_write':
      return `sdo_write ${cmd.params.node} ${cmd.params.index} ${cmd.params.subindex} ${cmd.params.value}`;
    case 'wait':
      return `wait ${cmd.params.ms}ms`;
    case 'log':
      return `log "${cmd.params.message}"`;
    case 'nmt':
      return `nmt ${cmd.params.node} ${cmd.params.command}`;
    case 'repeat':
      return `repeat ${cmd.params.count}`;
    default:
      return cmd.type;
  }
}

// ===== Predefined Scripts =====

export const PREDEFINED_SCRIPTS: Record<
  string,
  { name: string; description: string; script: string }
> = {
  quick_health_check: {
    name: 'Quick Health Check',
    description: 'Read basic device info from a node',
    script: `// Quick Health Check
// Reads device type, error register, and status word

sdo_read 1 1000 0
wait 100
sdo_read 1 1001 0
wait 100
sdo_read 1 6041 0
log "Health check complete"`,
  },
  ds402_enable_sequence: {
    name: 'DS402 Enable Sequence',
    description: 'Standard DS402 enable sequence (Shutdown → Switch On → Enable Operation)',
    script: `// DS402 Enable Sequence
// Step 1: Shutdown (0x0006)
sdo_write 1 6040 0 0006
wait 100

// Step 2: Switch On (0x0007)
sdo_write 1 6040 0 0007
wait 100

// Step 3: Enable Operation (0x000F)
sdo_write 1 6040 0 000F
wait 100

// Read StatusWord to verify
sdo_read 1 6041 0
log "DS402 enable sequence complete"`,
  },
  scan_all_nodes: {
    name: 'Scan All Nodes',
    description: 'Scan nodes 1-127 with SDO read',
    script: `// Scan All Nodes
// Try to read Device Type from each node

repeat 127 {
  // Will timeout for non-existent nodes
  sdo_read 1 1000 0
  wait 50
}
log "Scan complete"`,
  },
  pdo_test: {
    name: 'PDO Configuration Test',
    description: 'Configure and test PDO communication',
    script: `// PDO Configuration Test

// Read TPDO1 communication parameters
sdo_read 1 1800 1
wait 100
sdo_read 1 1800 2
wait 100

// Read TPDO1 mapping
sdo_read 1 1A00 0
wait 100
sdo_read 1 1A00 1
wait 100

log "PDO configuration read complete"`,
  },
};

// ===== Script Templates =====

export function getScriptTemplate(type: 'basic' | 'sdo' | 'ds402' | 'test'): string {
  switch (type) {
    case 'basic':
      return `// Basic CAN Frame Test
send 123 01 02 03 04 05 06 07 08
wait 100
send 456 AA BB CC DD
log "Basic test complete"`;
    case 'sdo':
      return `// SDO Read/Write Test
sdo_read 1 1000 0
wait 100
sdo_write 1 6040 0 0006
wait 100
sdo_read 1 6041 0
log "SDO test complete"`;
    case 'ds402':
      return `// DS402 Control Test
// Enable sequence
sdo_write 1 6040 0 0006
wait 100
sdo_write 1 6040 0 0007
wait 100
sdo_write 1 6040 0 000F
wait 100

// Set target position
sdo_write 1 607A 0 00001000
wait 1000

// Read actual position
sdo_read 1 6064 0
log "DS402 test complete"`;
    case 'test':
      return `// Automated Test Suite
log "Starting test suite..."

// Test 1: Device Type
sdo_read 1 1000 0
wait 100

// Test 2: Error Register
sdo_read 1 1001 0
wait 100

// Test 3: Heartbeat
sdo_read 1 1017 0
wait 100

log "Test suite complete"`;
  }
}
