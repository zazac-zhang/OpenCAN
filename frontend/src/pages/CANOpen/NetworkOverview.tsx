// Network Overview page

import { useNodes, useAppStore } from '@/lib/store';
import { useScanNodes } from '@/hooks/useCommands';
import { NodeCard } from '@/components/common/NodeCard';

export function NetworkOverview() {
  const nodes = useNodes();
  const selectedNode = useAppStore((s) => s.can.selectedNode);
  const scanMutation = useScanNodes();

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Network Overview</h2>
        <button
          className="px-3 py-1 text-sm bg-primary text-primary-foreground rounded"
          onClick={() => scanMutation.mutate()}
          disabled={scanMutation.isPending}
        >
          {scanMutation.isPending ? 'Scanning...' : 'Scan Nodes'}
        </button>
      </div>

      {nodes.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-12 text-center">
          <p className="text-sm text-muted-foreground mb-2">No nodes discovered</p>
          <p className="text-xs text-muted-foreground">Click "Scan Nodes" to find devices on the bus</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {nodes.map((node) => (
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
