// Sidebar with node list

import { useAppStore, useNodes, useSelectedNode, useConnected } from '@/lib/store';
import { useNmtCommand } from '@/hooks/useCommands';

export function Sidebar() {
  const connected = useConnected();
  const nodes = useNodes();
  const selectedNode = useSelectedNode();
  const setSelectedNode = (id: number | null) =>
    useAppStore.setState((s) => ({ can: { ...s.can, selectedNode: id } }));
  const nmtMutation = useNmtCommand();

  return (
    <div className="w-52 border-r bg-background flex flex-col shrink-0">
      <div className="p-3 border-b">
        <h2 className="text-sm font-semibold">Nodes</h2>
        <p className="text-xs text-muted-foreground">{nodes.length} found</p>
        <p className="text-xs text-muted-foreground mt-1">
          {connected ? '● Connected' : '○ Disconnected'}
        </p>
      </div>

      <div className="flex-1 overflow-auto p-2">
        {nodes.length === 0 ? (
          <p className="text-xs text-muted-foreground px-2">(no nodes)</p>
        ) : (
          nodes.map((node) => (
            <button
              key={node.node_id}
              onClick={() => setSelectedNode(node.node_id)}
              className={`w-full text-left p-2 rounded text-sm mb-1 ${
                selectedNode === node.node_id
                  ? 'bg-primary/10 border border-primary/30'
                  : 'hover:bg-muted'
              }`}
            >
              <div className="flex items-center gap-1">
                <span className="text-xs">{node.nmt_state === 'Operational' ? '🟢' : '🟡'}</span>
                <span>Node {node.node_id}</span>
              </div>
              <div className="text-xs text-muted-foreground ml-4">{node.nmt_state}</div>
            </button>
          ))
        )}
      </div>

      {/* Quick actions for selected node */}
      {selectedNode !== null && (
        <div className="p-2 border-t space-y-1">
          <p className="text-xs font-medium">Node {selectedNode} Actions:</p>
          <button
            className="w-full text-xs px-2 py-1 bg-primary text-primary-foreground rounded"
            onClick={() => nmtMutation.mutate({ nodeId: selectedNode, command: 'start' })}
          >
            NMT Start
          </button>
          <button
            className="w-full text-xs px-2 py-1 bg-destructive text-destructive-foreground rounded"
            onClick={() => nmtMutation.mutate({ nodeId: selectedNode, command: 'stop' })}
          >
            NMT Stop
          </button>
          <button
            className="w-full text-xs px-2 py-1 bg-muted rounded"
            onClick={() => nmtMutation.mutate({ nodeId: selectedNode, command: 'reset' })}
          >
            NMT Reset
          </button>
        </div>
      )}
    </div>
  );
}
