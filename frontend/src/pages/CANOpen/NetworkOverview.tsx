// Network Overview page

import { useNodes, useSelectedNode, useAppStore } from '@/lib/store';
import { useScanNodes } from '@/hooks/useCommands';

export function NetworkOverview() {
  const nodes = useNodes();
  const selectedNode = useSelectedNode();
  const scanMutation = useScanNodes();

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Network Overview</h2>
        <button
          className="px-3 py-1 text-sm bg-primary text-primary-foreground rounded"
          onClick={() => scanMutation.mutate()}
        >
          Scan Nodes
        </button>
      </div>

      {nodes.length === 0 ? (
        <p className="text-sm text-muted-foreground">No nodes discovered. Click "Scan Nodes" to find devices.</p>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {nodes.map((node) => (
            <div
              key={node.node_id}
              className={`p-4 border rounded-lg cursor-pointer transition ${
                selectedNode === node.node_id
                  ? 'border-primary bg-primary/5'
                  : 'hover:border-muted-foreground/50'
              }`}
              onClick={() => {
                useAppStore.setState((s) => ({
                  can: { ...s.can, selectedNode: node.node_id },
                }));
              }}
            >
              <div className="flex items-center gap-2">
                <span className="text-lg">{node.nmt_state === 'Operational' ? '🟢' : '🟡'}</span>
                <span className="font-medium">Node {node.node_id}</span>
              </div>
              <div className="text-sm text-muted-foreground mt-1">{node.nmt_state}</div>
              {node.device_type && (
                <div className="text-xs text-muted-foreground mt-2">
                  Device: 0x{node.device_type.toString(16).padStart(8, '0')}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
