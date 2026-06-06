// DS402 Control page

import { useAppStore, useSelectedNode } from '@/lib/store';
import { useDs402Enable, useDs402FaultReset, useDs402SetMode, useDs402SetTarget } from '@/hooks/useCommands';

const DS402_MODES = [
  { value: 1, label: 'PP (Profile Position)' },
  { value: 3, label: 'PV (Profile Velocity)' },
  { value: 6, label: 'Homing' },
  { value: 8, label: 'CSP (Cyclic Sync Position)' },
  { value: 9, label: 'CSV (Cyclic Sync Velocity)' },
  { value: 10, label: 'CST (Cyclic Sync Torque)' },
];

export function Ds402Control() {
  const selectedNode = useSelectedNode();
  const nodeId = selectedNode ?? 1;
  const enableMutation = useDs402Enable();
  const faultResetMutation = useDs402FaultReset();
  const modeMutation = useDs402SetMode();
  const targetMutation = useDs402SetTarget();

  const nodeState = useAppStore((s) => s.ds402.nodeStates[nodeId]);

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <h2 className="text-lg font-semibold">DS402 Control</h2>

      {/* State machine status */}
      <div className="p-3 border rounded bg-card">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">Node {nodeId}</span>
          <span className={`px-2 py-0.5 text-xs rounded ${
            nodeState?.state === 'Operation Enabled' ? 'bg-green-500/20 text-green-500' : 'bg-yellow-500/20 text-yellow-500'
          }`}>
            {nodeState?.state ?? 'Unknown'}
          </span>
        </div>
        {nodeState && (
          <div className="text-xs text-muted-foreground mt-1">
            Status Word: 0x{nodeState.status_word.toString(16).padStart(4, '0').toUpperCase()}
          </div>
        )}
      </div>

      {/* Enable sequence */}
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

      {/* Mode selection */}
      <div className="space-y-2">
        <p className="text-sm font-medium">Operation Mode:</p>
        <div className="flex flex-wrap gap-1">
          {DS402_MODES.map((mode) => (
            <button
              key={mode.value}
              onClick={() => modeMutation.mutate({ nodeId, mode: mode.value })}
              className={`px-2 py-1 text-xs rounded ${
                nodeState?.selected_mode === mode.value.toString()
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted hover:bg-muted/80'
              }`}
            >
              {mode.label}
            </button>
          ))}
        </div>
      </div>

      {/* Target inputs */}
      <div className="space-y-2">
        <p className="text-sm font-medium">Target Values:</p>
        <div className="flex items-center gap-2">
          <span className="text-xs w-20">Position:</span>
          <input
            className="flex-1 px-2 py-1 text-xs border rounded bg-background"
            value={nodeState?.target_position ?? ''}
            onChange={(e) =>
              useAppStore.getState().ds402.updateNodeState(nodeId, { target_position: e.target.value })
            }
            placeholder="0"
          />
          <button
            className="px-3 py-1 text-xs bg-primary text-primary-foreground rounded"
            onClick={() => {
              const val = parseFloat(nodeState?.target_position ?? '0');
              if (!isNaN(val)) {
                targetMutation.mutate({ nodeId, mode: 1, target: val });
              }
            }}
          >
            Set
          </button>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs w-20">Velocity:</span>
          <input
            className="flex-1 px-2 py-1 text-xs border rounded bg-background"
            value={nodeState?.target_velocity ?? ''}
            onChange={(e) =>
              useAppStore.getState().ds402.updateNodeState(nodeId, { target_velocity: e.target.value })
            }
            placeholder="0"
          />
          <button
            className="px-3 py-1 text-xs bg-primary text-primary-foreground rounded"
            onClick={() => {
              const val = parseFloat(nodeState?.target_velocity ?? '0');
              if (!isNaN(val)) {
                targetMutation.mutate({ nodeId, mode: 3, target: val });
              }
            }}
          >
            Set
          </button>
        </div>
      </div>
    </div>
  );
}
