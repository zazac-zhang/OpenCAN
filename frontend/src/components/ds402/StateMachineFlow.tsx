/**
 * StateMachineFlow — Interactive SVG-based DS402 state machine flowchart.
 *
 * Features:
 * - SVG flowchart with state nodes and transition edges
 * - Edge labels showing control word values (0x6040 writes)
 * - Current state highlighting with pulse animation
 * - Clickable edges to send SDO commands
 * - StatusWord bit-level parsing panel
 * - Quick control word buttons
 */

import { useCallback, useMemo } from 'react';
import { cn } from '@/lib/utils';

// ===== Types =====

export interface StateMachineFlowProps {
  /** Current DS402 state name */
  currentState: string;
  /** Current StatusWord value */
  statusWord?: number;
  /** Current ControlWord value */
  controlWord?: number;
  /** Callback to send control word via SDO */
  onSendControlWord?: (value: number, label: string) => void;
  /** Whether in read-only mode */
  readOnly?: boolean;
}

// ===== DS402 State Definitions =====

interface Ds402State {
  key: string;
  label: string;
  short: string;
  statusWordMask: number;
  statusWordValue: number;
}

const DS402_STATES: Ds402State[] = [
  {
    key: 'not_ready',
    label: 'Not Ready to Switch On',
    short: 'Not Ready',
    statusWordMask: 0x004f,
    statusWordValue: 0x0000,
  },
  {
    key: 'switch_on_disabled',
    label: 'Switch On Disabled',
    short: 'SW Disabled',
    statusWordMask: 0x004f,
    statusWordValue: 0x0040,
  },
  {
    key: 'ready_to_switch_on',
    label: 'Ready to Switch On',
    short: 'Ready',
    statusWordMask: 0x006f,
    statusWordValue: 0x0021,
  },
  {
    key: 'switched_on',
    label: 'Switched On',
    short: 'Switched On',
    statusWordMask: 0x006f,
    statusWordValue: 0x0023,
  },
  {
    key: 'operation_enabled',
    label: 'Operation Enabled',
    short: 'Op Enabled',
    statusWordMask: 0x006f,
    statusWordValue: 0x0027,
  },
  {
    key: 'quick_stop',
    label: 'Quick Stop Active',
    short: 'Quick Stop',
    statusWordMask: 0x006f,
    statusWordValue: 0x0007,
  },
  {
    key: 'fault_reaction',
    label: 'Fault Reaction Active',
    short: 'Fault React',
    statusWordMask: 0x004f,
    statusWordValue: 0x000f,
  },
  { key: 'fault', label: 'Fault', short: 'Fault', statusWordMask: 0x004f, statusWordValue: 0x0008 },
];

// ===== Transition Definitions =====

interface Transition {
  from: string;
  to: string;
  label: string;
  controlWord: number;
  description: string;
}

const TRANSITIONS: Transition[] = [
  // From Switch On Disabled
  {
    from: 'switch_on_disabled',
    to: 'ready_to_switch_on',
    label: 'Shutdown',
    controlWord: 0x0006,
    description: 'SDO Write 0x6040 = 0x0006',
  },
  // From Ready to Switch On
  {
    from: 'ready_to_switch_on',
    to: 'switched_on',
    label: 'Switch On',
    controlWord: 0x0007,
    description: 'SDO Write 0x6040 = 0x0007',
  },
  {
    from: 'ready_to_switch_on',
    to: 'switch_on_disabled',
    label: 'Disable Voltage',
    controlWord: 0x0000,
    description: 'SDO Write 0x6040 = 0x0000',
  },
  // From Switched On
  {
    from: 'switched_on',
    to: 'operation_enabled',
    label: 'Enable Operation',
    controlWord: 0x000f,
    description: 'SDO Write 0x6040 = 0x000F',
  },
  {
    from: 'switched_on',
    to: 'ready_to_switch_on',
    label: 'Shutdown',
    controlWord: 0x0006,
    description: 'SDO Write 0x6040 = 0x0006',
  },
  {
    from: 'switched_on',
    to: 'switch_on_disabled',
    label: 'Disable Voltage',
    controlWord: 0x0000,
    description: 'SDO Write 0x6040 = 0x0000',
  },
  // From Operation Enabled
  {
    from: 'operation_enabled',
    to: 'switched_on',
    label: 'Disable Operation',
    controlWord: 0x0007,
    description: 'SDO Write 0x6040 = 0x0007',
  },
  {
    from: 'operation_enabled',
    to: 'ready_to_switch_on',
    label: 'Shutdown',
    controlWord: 0x0006,
    description: 'SDO Write 0x6040 = 0x0006',
  },
  {
    from: 'operation_enabled',
    to: 'quick_stop',
    label: 'Quick Stop',
    controlWord: 0x0002,
    description: 'SDO Write 0x6040 = 0x0002',
  },
  // From Quick Stop
  {
    from: 'quick_stop',
    to: 'switch_on_disabled',
    label: 'Disable Voltage',
    controlWord: 0x0000,
    description: 'SDO Write 0x6040 = 0x0000',
  },
  // From Fault
  {
    from: 'fault',
    to: 'switch_on_disabled',
    label: 'Fault Reset',
    controlWord: 0x0080,
    description: 'SDO Write 0x6040 = 0x0080',
  },
  // Auto transitions (no user action)
  {
    from: 'not_ready',
    to: 'switch_on_disabled',
    label: 'Auto',
    controlWord: 0,
    description: 'Automatic transition',
  },
  {
    from: 'fault_reaction',
    to: 'fault',
    label: 'Auto',
    controlWord: 0,
    description: 'Automatic transition',
  },
];

// ===== StatusWord Bit Definitions =====

const STATUSWORD_BITS: [number, string, string][] = [
  [0, 'Ready to Switch On', '🟢'],
  [1, 'Switched On', '🟢'],
  [2, 'Operation Enabled', '🟢'],
  [3, 'Fault', '🔴'],
  [4, 'Voltage Enabled', '⚡'],
  [5, 'Quick Stop Active', '🔵'],
  [6, 'Switch On Disabled', '⛔'],
  [7, 'Warning', '⚠️'],
  [8, 'Manufacturer Specific', '⚙️'],
  [9, 'Remote', '📡'],
  [10, 'Target Reached', '✅'],
  [11, 'Internal Limit Active', '⚠️'],
  [12, 'Reserved', ''],
  [13, 'Reserved', ''],
];

// ===== SVG Layout Constants =====

const NODE_W = 140;
const NODE_H = 50;
const NODE_RX = 8;
const PAD = 20;

// State positions in the SVG (x, y)
const STATE_POSITIONS: Record<string, { x: number; y: number }> = {
  not_ready: { x: PAD, y: PAD },
  switch_on_disabled: { x: PAD + 200, y: PAD },
  ready_to_switch_on: { x: PAD + 200, y: PAD + 100 },
  switched_on: { x: PAD + 200, y: PAD + 200 },
  operation_enabled: { x: PAD + 200, y: PAD + 300 },
  quick_stop: { x: PAD, y: PAD + 300 },
  fault_reaction: { x: PAD + 400, y: PAD + 100 },
  fault: { x: PAD + 400, y: PAD + 200 },
};

const SVG_W = PAD * 2 + 400 + NODE_W;
const SVG_H = PAD * 2 + 300 + NODE_H;

// ===== Helper Functions =====

function detectStateFromStatusWord(sw: number): string {
  for (const state of DS402_STATES) {
    if ((sw & state.statusWordMask) === state.statusWordValue) {
      return state.key;
    }
  }
  return 'not_ready';
}

function stateNameToKey(name: string): string {
  const map: Record<string, string> = {
    'Not Ready to Switch On': 'not_ready',
    'Switch On Disabled': 'switch_on_disabled',
    'Ready to Switch On': 'ready_to_switch_on',
    'Switched On': 'switched_on',
    'Operation Enabled': 'operation_enabled',
    'Quick Stop Active': 'quick_stop',
    'Fault Reaction Active': 'fault_reaction',
    Fault: 'fault',
    Unknown: 'not_ready',
  };
  return map[name] || 'not_ready';
}

function getStateColor(key: string, isActive: boolean): string {
  if (!isActive) return '#374151'; // gray-700
  switch (key) {
    case 'operation_enabled':
      return '#22c55e'; // green-500
    case 'ready_to_switch_on':
    case 'switched_on':
      return '#eab308'; // yellow-500
    case 'fault':
    case 'fault_reaction':
      return '#ef4444'; // red-500
    case 'quick_stop':
      return '#f97316'; // orange-500
    default:
      return '#6b7280'; // gray-500
  }
}

// ===== Edge Path Calculation =====

function getEdgePath(from: string, to: string): string {
  const f = STATE_POSITIONS[from];
  const t = STATE_POSITIONS[to];
  if (!f || !t) return '';

  const fx = f.x + NODE_W / 2;
  const fy = f.y + NODE_H / 2;
  const tx = t.x + NODE_W / 2;
  const ty = t.y + NODE_H / 2;

  // Calculate edge start/end points (on the border of the node)
  const angle = Math.atan2(ty - fy, tx - fx);
  const startX = fx + (NODE_W / 2) * Math.cos(angle);
  const startY = fy + (NODE_H / 2) * Math.sin(angle);
  const endX = tx - (NODE_W / 2) * Math.cos(angle);
  const endY = ty - (NODE_H / 2) * Math.sin(angle);

  // Use a curved path for better visibility
  const midX = (startX + endX) / 2;
  const midY = (startY + endY) / 2;
  const dx = endX - startX;
  const dy = endY - startY;
  const len = Math.sqrt(dx * dx + dy * dy);
  const offset = len * 0.15;

  // Perpendicular offset for curve
  const nx = (-dy / len) * offset;
  const ny = (dx / len) * offset;

  return `M ${startX} ${startY} Q ${midX + nx} ${midY + ny} ${endX} ${endY}`;
}

function getEdgeLabelPos(from: string, to: string): { x: number; y: number } {
  const f = STATE_POSITIONS[from];
  const t = STATE_POSITIONS[to];
  if (!f || !t) return { x: 0, y: 0 };
  const fx = f.x + NODE_W / 2;
  const fy = f.y + NODE_H / 2;
  const tx = t.x + NODE_W / 2;
  const ty = t.y + NODE_H / 2;
  const dx = tx - fx;
  const dy = ty - fy;
  const len = Math.sqrt(dx * dx + dy * dy);
  const offset = len * 0.15;
  const nx = (-dy / len) * offset;
  const ny = (dx / len) * offset;
  return {
    x: (fx + tx) / 2 + nx,
    y: (fy + ty) / 2 + ny,
  };
}

// ===== Main Component =====

export function StateMachineFlow({
  currentState,
  statusWord,
  controlWord,
  onSendControlWord,
  readOnly = false,
}: StateMachineFlowProps) {
  const currentKey = stateNameToKey(currentState);
  const detectedKey = statusWord !== undefined ? detectStateFromStatusWord(statusWord) : currentKey;
  const activeKey = detectedKey || currentKey;

  // Available transitions from current state
  const availableTransitions = useMemo(
    () => TRANSITIONS.filter((t) => t.from === activeKey && t.controlWord !== 0),
    [activeKey],
  );

  // Handle edge click
  const handleEdgeClick = useCallback(
    (transition: Transition) => {
      if (readOnly || !onSendControlWord || transition.controlWord === 0) return;
      onSendControlWord(transition.controlWord, transition.label);
    },
    [readOnly, onSendControlWord],
  );

  // StatusWord bit decoding
  const statusBits = useMemo(() => {
    if (statusWord === undefined) return [];
    return STATUSWORD_BITS.map(([bit, label, icon]) => ({
      bit,
      label,
      icon,
      set: (statusWord & (1 << bit)) !== 0,
    }));
  }, [statusWord]);

  return (
    <div className="space-y-3">
      {/* SVG Flowchart */}
      <div className="border rounded-lg bg-card overflow-hidden">
        <div className="px-3 py-2 border-b flex items-center justify-between">
          <h3 className="text-sm font-medium">State Machine Flowchart</h3>
          <span className="text-xs text-muted-foreground font-mono">
            {activeKey.replace(/_/g, ' ')}
          </span>
        </div>
        <div className="overflow-auto p-2">
          <svg width={SVG_W} height={SVG_H} viewBox={`0 0 ${SVG_W} ${SVG_H}`} className="mx-auto">
            {/* Edges */}
            {TRANSITIONS.map((t, i) => {
              const path = getEdgePath(t.from, t.to);
              const labelPos = getEdgeLabelPos(t.from, t.to);
              const isActive = t.from === activeKey;
              const isAuto = t.controlWord === 0;
              const edgeColor = isActive ? '#3b82f6' : '#4b5563';

              return (
                <g key={i}>
                  {/* Edge path */}
                  <path
                    d={path}
                    fill="none"
                    stroke={edgeColor}
                    strokeWidth={isActive ? 2.5 : 1.5}
                    strokeDasharray={isAuto ? '4 2' : 'none'}
                    className={isActive && !readOnly ? 'cursor-pointer hover:stroke-blue-400' : ''}
                    onClick={() => handleEdgeClick(t)}
                    markerEnd={isActive ? 'url(#arrow-blue)' : 'url(#arrow-gray)'}
                  />
                  {/* Edge label */}
                  {!isAuto && (
                    <g
                      className={isActive && !readOnly ? 'cursor-pointer' : ''}
                      onClick={() => handleEdgeClick(t)}
                    >
                      <rect
                        x={labelPos.x - 28}
                        y={labelPos.y - 10}
                        width={56}
                        height={20}
                        rx={4}
                        fill={isActive ? '#1e3a5f' : '#1f2937'}
                        stroke={isActive ? '#3b82f6' : '#374151'}
                        strokeWidth={1}
                      />
                      <text
                        x={labelPos.x}
                        y={labelPos.y + 4}
                        textAnchor="middle"
                        className="text-[10px] font-mono"
                        fill={isActive ? '#93c5fd' : '#9ca3af'}
                      >
                        0x{t.controlWord.toString(16).padStart(4, '0')}
                      </text>
                    </g>
                  )}
                </g>
              );
            })}

            {/* Arrow markers */}
            <defs>
              <marker
                id="arrow-blue"
                viewBox="0 0 10 10"
                refX="10"
                refY="5"
                markerWidth="6"
                markerHeight="6"
                orient="auto-start-reverse"
              >
                <path d="M 0 0 L 10 5 L 0 10 z" fill="#3b82f6" />
              </marker>
              <marker
                id="arrow-gray"
                viewBox="0 0 10 10"
                refX="10"
                refY="5"
                markerWidth="6"
                markerHeight="6"
                orient="auto-start-reverse"
              >
                <path d="M 0 0 L 10 5 L 0 10 z" fill="#4b5563" />
              </marker>
            </defs>

            {/* State nodes */}
            {DS402_STATES.map((state) => {
              const pos = STATE_POSITIONS[state.key];
              if (!pos) return null;
              const isActive = state.key === activeKey;
              const color = getStateColor(state.key, isActive);

              return (
                <g key={state.key}>
                  {/* Node background */}
                  <rect
                    x={pos.x}
                    y={pos.y}
                    width={NODE_W}
                    height={NODE_H}
                    rx={NODE_RX}
                    fill={isActive ? `${color}20` : '#1f2937'}
                    stroke={isActive ? color : '#374151'}
                    strokeWidth={isActive ? 2 : 1}
                    className={isActive ? 'animate-pulse-slow' : ''}
                  />
                  {/* State label */}
                  <text
                    x={pos.x + NODE_W / 2}
                    y={pos.y + NODE_H / 2 - 6}
                    textAnchor="middle"
                    className="text-[11px] font-medium"
                    fill={isActive ? color : '#9ca3af'}
                  >
                    {state.short}
                  </text>
                  {/* StatusWord info */}
                  <text
                    x={pos.x + NODE_W / 2}
                    y={pos.y + NODE_H / 2 + 10}
                    textAnchor="middle"
                    className="text-[9px] font-mono"
                    fill={isActive ? `${color}cc` : '#6b7280'}
                  >
                    (SW & 0x{state.statusWordMask.toString(16).padStart(4, '0')}) = 0x
                    {state.statusWordValue.toString(16).padStart(4, '0')}
                  </text>
                  {/* Active indicator */}
                  {isActive && (
                    <circle
                      cx={pos.x + NODE_W - 8}
                      cy={pos.y + 8}
                      r={5}
                      fill={color}
                      className="animate-ping-slow"
                    />
                  )}
                </g>
              );
            })}
          </svg>
        </div>
      </div>

      {/* Quick Control Buttons */}
      {availableTransitions.length > 0 && (
        <div className="border rounded-lg bg-card p-3">
          <h3 className="text-sm font-medium mb-2">Available Transitions</h3>
          <div className="flex flex-wrap gap-2">
            {availableTransitions.map((t, i) => (
              <button
                key={i}
                onClick={() => handleEdgeClick(t)}
                disabled={readOnly}
                className={cn(
                  'px-3 py-1.5 text-xs rounded-md border transition-colors',
                  readOnly
                    ? 'border-border text-muted-foreground cursor-not-allowed'
                    : 'border-blue-500/30 text-blue-400 hover:bg-blue-500/10 hover:border-blue-500/50',
                )}
              >
                {t.label}
                <span className="ml-1.5 font-mono text-[10px] opacity-70">
                  0x{t.controlWord.toString(16).padStart(4, '0')}
                </span>
              </button>
            ))}
            {/* Fault Reset (always available) */}
            <button
              onClick={() => onSendControlWord?.(0x0080, 'Fault Reset')}
              disabled={readOnly}
              className={cn(
                'px-3 py-1.5 text-xs rounded-md border transition-colors',
                readOnly
                  ? 'border-border text-muted-foreground cursor-not-allowed'
                  : 'border-red-500/30 text-red-400 hover:bg-red-500/10 hover:border-red-500/50',
              )}
            >
              Fault Reset
              <span className="ml-1.5 font-mono text-[10px] opacity-70">0x0080</span>
            </button>
          </div>
        </div>
      )}

      {/* StatusWord Bit Display */}
      {statusWord !== undefined && (
        <div className="border rounded-lg bg-card p-3">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-medium">StatusWord (0x6041)</h3>
            <span className="text-xs font-mono text-muted-foreground">
              0x{statusWord.toString(16).padStart(4, '0').toUpperCase()} ={' '}
              {statusWord.toString(2).padStart(16, '0')}
            </span>
          </div>
          <div className="grid grid-cols-2 gap-1">
            {statusBits.map(({ bit, label, icon, set }) => (
              <div
                key={bit}
                className={cn(
                  'flex items-center gap-1.5 px-2 py-1 rounded text-xs font-mono',
                  set ? 'bg-green-500/10 text-green-400' : 'bg-muted/30 text-muted-foreground',
                )}
              >
                <span className="w-4">{icon}</span>
                <span className="w-6">b{bit}</span>
                <span className="truncate flex-1">{label}</span>
                <span
                  className={cn(
                    'w-4 h-4 rounded-full border flex items-center justify-center text-[10px]',
                    set ? 'border-green-500 bg-green-500/20' : 'border-border',
                  )}
                >
                  {set ? '1' : '0'}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ControlWord Display */}
      {controlWord !== undefined && (
        <div className="border rounded-lg bg-card p-3">
          <h3 className="text-sm font-medium mb-1">ControlWord (0x6040)</h3>
          <span className="text-xs font-mono text-muted-foreground">
            0x{controlWord.toString(16).padStart(4, '0').toUpperCase()} ={' '}
            {controlWord.toString(2).padStart(16, '0')}
          </span>
        </div>
      )}
    </div>
  );
}
