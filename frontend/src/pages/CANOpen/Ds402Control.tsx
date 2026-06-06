// DS402 Control page — enhanced with sub-components

import { useAppStore, useSelectedNode } from '@/lib/store';
import { useDs402Enable, useDs402FaultReset, useDs402SetMode, useDs402SetTarget } from '@/hooks/useCommands';
import { StateMachine } from '@/components/ds402/StateMachine';
import { ModeSelector } from '@/components/ds402/ModeSelector';
import { ControlPanel } from '@/components/ds402/ControlPanel';
import { WaveformDisplay } from '@/components/ds402/WaveformDisplay';

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

export function Ds402Control() {
  const selectedNode = useSelectedNode();
  const nodeId = selectedNode ?? 1;
  const enableMutation = useDs402Enable();
  const faultResetMutation = useDs402FaultReset();
  const modeMutation = useDs402SetMode();
  const targetMutation = useDs402SetTarget();

  const nodeState = useAppStore((s) => s.ds402.nodeStates[nodeId]);
  const posHistory = nodeState?.position_history ?? [];
  const velHistory = nodeState?.velocity_history ?? [];
  const torqueHistory = nodeState?.torque_history ?? [];

  const targetPosition = nodeState?.target_position ?? '';
  const targetVelocity = nodeState?.target_velocity ?? '';
  const targetTorque = nodeState?.target_torque ?? '';

  const currentMode = nodeState?.selected_mode ? parseInt(nodeState.selected_mode, 10) : 1;
  const currentModeName = MODE_NAMES[currentMode] ?? 'PP';

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <h2 className="text-lg font-semibold">DS402 Control</h2>

      {/* Header controls */}
      <div className="flex items-center gap-2 p-3 border rounded bg-card">
        <span className="text-sm font-medium">Node {nodeId}</span>
        <span
          className={`px-2 py-0.5 text-xs rounded ${
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
          className="px-3 py-1.5 text-sm bg-green-600 text-white rounded hover:bg-green-700"
          onClick={() => enableMutation.mutate(nodeId)}
        >
          Enable
        </button>
        <button
          className="px-3 py-1.5 text-sm bg-red-600 text-white rounded hover:bg-red-700"
          onClick={() => faultResetMutation.mutate(nodeId)}
        >
          Fault Reset
        </button>
      </div>

      {/* State machine visualization */}
      <div className="border rounded bg-card p-3">
        <StateMachine
          currentState={nodeState?.state ?? 'Unknown'}
          readOnly
        />
      </div>

      {/* Mode selector */}
      <div className="border rounded bg-card p-3">
        <ModeSelector
          selectedMode={currentMode}
          onModeChange={(mode) => {
            modeMutation.mutate({ nodeId, mode });
            useAppStore.getState().ds402.updateNodeState(nodeId, { selected_mode: mode.toString() });
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
          onSetTarget={(mode, value) =>
            targetMutation.mutate({ nodeId, mode, target: value })
          }
          actualPosition={nodeState?.actual_position}
          actualVelocity={nodeState?.actual_velocity}
          actualTorque={nodeState?.actual_torque}
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
