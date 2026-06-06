// Detail panel (right sidebar)

import { useSelectedNode, useNodes, useAppStore } from '@/lib/store';
import { NodeCard } from '@/components/common/NodeCard';

export function DetailPanel() {
  const selectedNode = useSelectedNode();
  const nodes = useNodes();
  const node = selectedNode !== null ? nodes.find((n) => n.node_id === selectedNode) : null;
  const nodeState = useAppStore((s) => s.ds402.nodeStates[selectedNode ?? -1]);

  return (
    <div className="w-64 border-l bg-background flex flex-col shrink-0 overflow-auto">
      <div className="p-3 border-b">
        <h2 className="text-sm font-semibold">Details</h2>
      </div>

      {node ? (
        <div className="p-3 space-y-3">
          {/* Node card */}
          <NodeCard
            node={node}
            selected={false}
          />

          {/* DS402 status (if available) */}
          {nodeState && (
            <div className="p-2 border rounded bg-card space-y-1 text-xs">
              <div className="font-medium text-foreground">DS402 Status</div>
              <div className="text-muted-foreground">
                State: <span className="text-foreground">{nodeState.state}</span>
              </div>
              <div className="text-muted-foreground">
                Mode: <span className="text-foreground">{nodeState.selected_mode}</span>
              </div>
              <div className="text-muted-foreground">
                Position: <span className="text-foreground font-mono">{nodeState.actual_position.toFixed(1)}</span>
              </div>
              <div className="text-muted-foreground">
                Velocity: <span className="text-foreground font-mono">{nodeState.actual_velocity.toFixed(1)}</span>
              </div>
              <div className="text-muted-foreground">
                Torque: <span className="text-foreground font-mono">{nodeState.actual_torque.toFixed(1)}%</span>
              </div>
            </div>
          )}

          {/* Node details */}
          <div className="p-2 border rounded bg-card space-y-1 text-xs">
            <div className="font-medium text-foreground">Node Information</div>
            <div className="text-muted-foreground">
              NMT: <span className="text-foreground">{node.nmt_state}</span>
            </div>
            {node.device_type && (
              <div className="text-muted-foreground">
                Device: <span className="text-foreground font-mono">0x{node.device_type.toString(16).toUpperCase()}</span>
              </div>
            )}
            {node.vendor_id && (
              <div className="text-muted-foreground">
                Vendor: <span className="text-foreground font-mono">0x{node.vendor_id.toString(16).toUpperCase()}</span>
              </div>
            )}
            {node.product_name && (
              <div className="text-muted-foreground">
                Product: <span className="text-foreground">{node.product_name}</span>
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="p-3 text-xs text-muted-foreground">Select a node</div>
      )}
    </div>
  );
}
