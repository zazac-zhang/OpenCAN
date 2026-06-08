/**
 * NetworkOverview — CANopen network topology with live node state.
 *
 * Displays:
 * - Grid of discovered nodes with live NMT state from heartbeat data
 * - Scan Nodes button to discover new nodes
 * - Summary statistics (total nodes, operational, error states)
 * - Node health indicators based on heartbeat freshness
 */

import { Activity, Network, Power, RotateCw, Square, Zap } from 'lucide-react';
import { NodeCard } from '@/components/common/NodeCard';
import { useNmtCommand, useScanNodes } from '@/hooks/useCommands';
import { useAppStore, useNodes } from '@/lib/store';

const NMT_COMMANDS = [
  { command: 'StartNode', label: 'Start', icon: Power, color: 'bg-green-600 hover:bg-green-700' },
  { command: 'StopNode', label: 'Stop', icon: Square, color: 'bg-red-600 hover:bg-red-700' },
  {
    command: 'EnterPreOperational',
    label: 'Pre-Op',
    icon: Zap,
    color: 'bg-yellow-600 hover:bg-yellow-700',
  },
  {
    command: 'ResetNode',
    label: 'Reset',
    icon: RotateCw,
    color: 'bg-orange-600 hover:bg-orange-700',
  },
] as const;

export function NetworkOverview() {
  const nodes = useNodes();
  const selectedNode = useAppStore((s) => s.can.selectedNode);
  const heartbeatEntries = useAppStore((s) => s.heartbeat.entries);
  const scanMutation = useScanNodes();
  const nmtMutation = useNmtCommand();

  // Build a map of node_id -> heartbeat state for live NMT updates
  const heartbeatMap = new Map<number, { alive: boolean; lastSeen: number }>();
  for (const entry of heartbeatEntries) {
    heartbeatMap.set(entry.node_id, { alive: entry.alive, lastSeen: entry.last_seen_ms });
  }

  // Enrich nodes with live heartbeat state
  const enrichedNodes = nodes.map((node) => {
    const hb = heartbeatMap.get(node.node_id);
    if (hb) {
      // Override NMT state based on live heartbeat data
      const elapsed = Date.now() - hb.lastSeen;
      return {
        ...node,
        nmt_state: hb.alive && elapsed < 10000 ? 'Operational' : 'Not Responding',
        _heartbeatFresh: elapsed,
      };
    }
    return node;
  });

  // Statistics
  const operationalCount = enrichedNodes.filter((n) => n.nmt_state === 'Operational').length;
  const notRespondingCount = enrichedNodes.filter((n) => n.nmt_state === 'Not Responding').length;
  const preOpCount = enrichedNodes.filter((n) => n.nmt_state === 'PreOperational').length;

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Network className="h-5 w-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold">Network Overview</h2>
        </div>
        <div className="flex items-center gap-2">
          {selectedNode !== null && (
            <div className="flex items-center gap-1">
              <span className="text-xs text-muted-foreground mr-1">NMT → Node {selectedNode}:</span>
              {NMT_COMMANDS.map(({ command, label, icon: Icon, color }) => (
                <button
                  key={command}
                  className={`px-2 py-1 text-xs rounded text-white flex items-center gap-1 ${color}`}
                  onClick={() => nmtMutation.mutate({ nodeId: selectedNode, command })}
                  disabled={nmtMutation.isPending}
                  title={`Send NMT ${command} to node ${selectedNode}`}
                >
                  <Icon className="h-3 w-3" />
                  {label}
                </button>
              ))}
            </div>
          )}
          <button
            className="px-3 py-1 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90 disabled:opacity-50"
            onClick={() => scanMutation.mutate()}
            disabled={scanMutation.isPending}
          >
            {scanMutation.isPending ? 'Scanning...' : 'Scan Nodes'}
          </button>
        </div>
      </div>

      {/* Summary stats */}
      {enrichedNodes.length > 0 && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          <div className="p-3 border rounded-lg bg-card">
            <div className="text-xs text-muted-foreground flex items-center gap-1">
              <Activity className="h-3 w-3" /> Total Nodes
            </div>
            <div className="text-xl font-bold font-mono mt-1">{enrichedNodes.length}</div>
          </div>
          <div className="p-3 border rounded-lg bg-card">
            <div className="text-xs text-muted-foreground">Operational</div>
            <div className="text-xl font-bold font-mono mt-1 text-green-400">
              {operationalCount}
            </div>
          </div>
          <div className="p-3 border rounded-lg bg-card">
            <div className="text-xs text-muted-foreground">Pre-Operational</div>
            <div className="text-xl font-bold font-mono mt-1 text-yellow-400">{preOpCount}</div>
          </div>
          <div className="p-3 border rounded-lg bg-card">
            <div className="text-xs text-muted-foreground">Not Responding</div>
            <div className="text-xl font-bold font-mono mt-1 text-red-400">
              {notRespondingCount}
            </div>
          </div>
        </div>
      )}

      {enrichedNodes.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-12 text-center">
          <Network className="h-12 w-12 text-muted-foreground mb-4" />
          <p className="text-sm text-muted-foreground mb-2">No nodes discovered</p>
          <p className="text-xs text-muted-foreground">
            Click "Scan Nodes" to find devices on the bus
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {enrichedNodes.map((node) => (
            <NodeCard
              key={node.node_id}
              node={node}
              selected={selectedNode === node.node_id}
              onClick={(id) => {
                useAppStore.setState((s) => ({
                  can: { ...s.can, selectedNode: id },
                }));
              }}
            />
          ))}
        </div>
      )}
    </div>
  );
}
