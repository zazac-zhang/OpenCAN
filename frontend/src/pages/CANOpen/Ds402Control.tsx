/**
 * DS402 Control page — full CiA 402 drive control with status bit display.
 *
 * Features:
 * - Enable / Fault Reset buttons
 * - State machine visualization
 * - Mode selector (8 motion modes)
 * - Motion control panel (position/velocity/torque per mode)
 * - StatusWord bit-level display
 * - ControlWord action buttons
 * - Telemetry waveform display with auto-refresh
 */
import { useEffect, useRef, useState } from 'react';
import { ControlPanel } from '@/components/ds402/ControlPanel';
import { ModeSelector } from '@/components/ds402/ModeSelector';
import { StateMachineFlow } from '@/components/ds402/StateMachineFlow';
import { WaveformDisplay } from '@/components/ds402/WaveformDisplay';
import {
  useDs402Enable,
  useDs402FaultReset,
  useDs402SetMode,
  useDs402SetTarget,
  useSdoDownload,
  useSdoUpload,
} from '@/hooks/useCommands';
import { useAppStore, useSelectedNode } from '@/lib/store';

/** Map mode number to display name */
const MODE_NAMES: Record<number, string> = {
  1: 'PP',
  2: 'VL',
  3: 'PV',
  6: 'HM',
  7: 'IP',
  8: 'CSP',
  9: 'CSV',
  10: 'CST',
};

/** StatusWord bit definitions (CiA 402) */
const STATUSWORD_BITS: [number, string, string?][] = [
  [0, 'Ready to Switch On', '🟢'],
  [1, 'Switched On', '🟢'],
  [2, 'Operation Enabled', '🟢'],
  [3, 'Fault', '🔴'],
  [4, 'Voltage Enabled', '⚡'],
  [5, 'Quick Stop Active', '🔵'],
  [6, 'Switch On Disabled', '⛔'],
  [7, 'Warning', '⚠️'],
  [8, 'Manufacturer Specific', ''],
  [9, 'Remote', '📡'],
  [10, 'Target Reached', '✅'],
  [11, 'Internal Limit Active', '⚠️'],
  [12, 'Reserved', ''],
  [13, 'Reserved', ''],
];

/** ControlWord bit definitions (CiA 402) */
const CONTROLWORD_BITS: [number, string][] = [
  [0, 'Switch On'],
  [1, 'Enable Voltage'],
  [2, 'Quick Stop'],
  [3, 'Enable Operation'],
  [4, 'Reserved (fault reset if mode=3)'],
  [5, 'Reserved (set new if mode=4)'],
  [6, 'Reserved (change set immediately)'],
  [7, 'Reserved (relative mode)'],
  [8, 'Halt'],
  [9, 'Reserved'],
  [10, 'Manufacturer Specific'],
  [11, 'Manufacturer Specific'],
];

function decodeBits(
  value: number,
  bits: [number, string, string?][],
): { bit: number; label: string; icon?: string; set: boolean }[] {
  return bits.map(([bit, label, icon]) => ({
    bit,
    label,
    icon,
    set: (value & (1 << bit)) !== 0,
  }));
}

export function Ds402Control() {
  const selectedNode = useSelectedNode();
  const nodeId = selectedNode ?? 1;
  const enableMutation = useDs402Enable();
  const faultResetMutation = useDs402FaultReset();
  const modeMutation = useDs402SetMode();
  const targetMutation = useDs402SetTarget();
  const sdoUpload = useSdoUpload();
  const sdoDownload = useSdoDownload();
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const nodeState = useAppStore((s) => s.ds402.nodeStates[nodeId]);
  const posHistory = nodeState?.position_history ?? [];
  const velHistory = nodeState?.velocity_history ?? [];
  const torqueHistory = nodeState?.torque_history ?? [];

  const targetPosition = nodeState?.target_position ?? '';
  const targetVelocity = nodeState?.target_velocity ?? '';
  const targetTorque = nodeState?.target_torque ?? '';

  const currentMode = nodeState?.selected_mode ? parseInt(nodeState.selected_mode, 10) : 1;
  const currentModeName = MODE_NAMES[currentMode] ?? 'PP';

  // Auto-refresh state
  const [autoRefresh, setAutoRefresh] = useState(false);

  // Periodically request status update when auto-refresh is enabled
  useEffect(() => {
    if (!autoRefresh) {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      return;
    }

    const pollStatusWord = () => {
      sdoUpload.mutate({
        node_id: nodeId,
        index: 0x6041, // StatusWord
        subindex: 0,
        data_type: 'UNS16',
      });
    };

    pollStatusWord();
    intervalRef.current = setInterval(pollStatusWord, 500);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [autoRefresh, nodeId, sdoUpload]);

  // Decode StatusWord
  const statusWordBits =
    nodeState?.status_word !== undefined ? decodeBits(nodeState.status_word, STATUSWORD_BITS) : [];

  // Decode ControlWord
  const controlWord = nodeState?.control_word ?? 0;
  const controlWordBits = decodeBits(controlWord, CONTROLWORD_BITS);

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">DS402 Control</h2>
        <div className="flex items-center gap-2">
          <label className="flex items-center gap-1 text-xs">
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
              className="accent-primary"
            />
            Auto-refresh
          </label>
          {autoRefresh && <span className="text-xs text-green-400 animate-pulse">● Polling</span>}
        </div>
      </div>

      {/* Header controls */}
      <div className="flex items-center gap-2 p-3 border rounded bg-card">
        <span className="text-sm font-medium">Node {nodeId}</span>
        <span
          className={`px-2 py-0.5 text-xs rounded font-medium ${
            nodeState?.state === 'Operation Enabled'
              ? 'bg-green-500/20 text-green-500'
              : nodeState?.state === 'Fault' || nodeState?.state === 'Fault Reaction Active'
                ? 'bg-red-500/20 text-red-500'
                : 'bg-yellow-500/20 text-yellow-500'
          }`}
        >
          {nodeState?.state ?? 'Unknown'}
        </span>
        {nodeState && (
          <span className="ml-auto text-xs text-muted-foreground font-mono">
            SW: 0x{nodeState.status_word.toString(16).padStart(4, '0').toUpperCase()}
          </span>
        )}
      </div>

      {/* Enable / Fault Reset */}
      <div className="flex gap-2">
        <button
          className="px-3 py-1.5 text-sm bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
          onClick={() => enableMutation.mutate(nodeId)}
          disabled={enableMutation.isPending}
        >
          Enable
        </button>
        <button
          className="px-3 py-1.5 text-sm bg-red-600 text-white rounded hover:bg-red-700 disabled:opacity-50"
          onClick={() => faultResetMutation.mutate(nodeId)}
          disabled={faultResetMutation.isPending}
        >
          Fault Reset
        </button>
      </div>

      {/* State machine visualization */}
      <StateMachineFlow
        currentState={nodeState?.state ?? 'Unknown'}
        statusWord={nodeState?.status_word}
        controlWord={nodeState?.control_word}
        onSendControlWord={(value, _label) => {
          // Send ControlWord via SDO write to 0x6040
          const data = [value & 0xff, (value >> 8) & 0xff];
          sdoDownload.mutate({
            node_id: nodeId,
            index: 0x6040,
            subindex: 0,
            data,
          });
        }}
        readOnly={false}
      />

      {/* StatusWord bit display */}
      {nodeState?.status_word !== undefined && (
        <div className="border rounded bg-card p-3 space-y-2">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium">StatusWord Bits</h3>
            <span className="text-xs font-mono text-muted-foreground">
              0x{nodeState.status_word.toString(16).padStart(4, '0').toUpperCase()} ={' '}
              {nodeState.status_word.toString(2).padStart(16, '0')}
            </span>
          </div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-1">
            {statusWordBits.map(({ bit, label, icon, set }) => (
              <div
                key={bit}
                className={`flex items-center gap-1.5 px-2 py-1 rounded text-xs font-mono ${
                  set ? 'bg-green-500/10 text-green-400' : 'bg-muted/30 text-muted-foreground'
                }`}
              >
                <span>{icon}</span>
                <span>Bit {bit}</span>
                <span className="truncate flex-1">{label}</span>
                <span
                  className={`w-4 h-4 rounded-full border flex items-center justify-center text-[10px] ${
                    set ? 'border-green-500 bg-green-500/20' : 'border-border'
                  }`}
                >
                  {set ? '1' : '0'}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ControlWord bit display */}
      {nodeState?.control_word !== undefined && (
        <div className="border rounded bg-card p-3 space-y-2">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium">ControlWord</h3>
            <span className="text-xs font-mono text-muted-foreground">
              0x{controlWord.toString(16).padStart(4, '0').toUpperCase()}
            </span>
          </div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-1">
            {controlWordBits.map(({ bit, label, set }) => (
              <div
                key={bit}
                className={`flex items-center gap-1.5 px-2 py-1 rounded text-xs font-mono ${
                  set ? 'bg-blue-500/10 text-blue-400' : 'bg-muted/30 text-muted-foreground'
                }`}
              >
                <span>Bit {bit}</span>
                <span className="truncate flex-1">{label}</span>
                <span
                  className={`w-4 h-4 rounded-full border flex items-center justify-center text-[10px] ${
                    set ? 'border-blue-500 bg-blue-500/20' : 'border-border'
                  }`}
                >
                  {set ? '1' : '0'}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Mode selector */}
      <div className="border rounded bg-card p-3">
        <ModeSelector
          selectedMode={currentMode}
          onModeChange={(mode) => {
            modeMutation.mutate({ nodeId, mode });
            useAppStore
              .getState()
              .ds402.updateNodeState(nodeId, { selected_mode: mode.toString() });
          }}
        />
      </div>

      {/* Motion control panel */}
      <div className="border rounded bg-card p-3">
        <ControlPanel
          nodeId={nodeId}
          currentMode={currentModeName}
          targetPosition={targetPosition}
          targetVelocity={targetVelocity}
          targetTorque={targetTorque}
          onTargetPositionChange={(v) =>
            useAppStore.getState().ds402.updateNodeState(nodeId, { target_position: v })
          }
          onTargetVelocityChange={(v) =>
            useAppStore.getState().ds402.updateNodeState(nodeId, { target_velocity: v })
          }
          onTargetTorqueChange={(v) =>
            useAppStore.getState().ds402.updateNodeState(nodeId, { target_torque: v })
          }
          onSetTarget={(mode, value) => targetMutation.mutate({ nodeId, mode, target: value })}
          actualPosition={nodeState?.actual_position}
          actualVelocity={nodeState?.actual_velocity}
          actualTorque={nodeState?.actual_torque}
          autoRefresh={autoRefresh}
          onAutoRefreshChange={setAutoRefresh}
        />
      </div>

      {/* Waveform display */}
      {(posHistory.length > 0 || velHistory.length > 0 || torqueHistory.length > 0) && (
        <div className="border rounded bg-card p-3">
          <WaveformDisplay
            nodeId={nodeId}
            positionHistory={posHistory}
            velocityHistory={velHistory}
            torqueHistory={torqueHistory}
          />
        </div>
      )}
    </div>
  );
}
