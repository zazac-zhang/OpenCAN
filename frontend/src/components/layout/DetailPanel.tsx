// Detail panel (right sidebar)

import { useSelectedNode, useNodes } from '@/lib/store';

export function DetailPanel() {
  const selectedNode = useSelectedNode();
  const nodes = useNodes();
  const node = selectedNode !== null ? nodes.find((n) => n.node_id === selectedNode) : null;

  return (
    <div className="w-56 border-l bg-background flex flex-col shrink-0 overflow-auto">
      <div className="p-3 border-b">
        <h2 className="text-sm font-semibold">Details</h2>
      </div>

      {node ? (
        <div className="p-3 space-y-2 text-xs">
          <div>
            <span className="font-medium">Node {node.node_id}</span>
          </div>
          <div className="text-muted-foreground">{node.nmt_state}</div>

          {node.device_type && (
            <div>Device Type: 0x{node.device_type.toString(16).padStart(8, '0')}</div>
          )}
          {node.vendor_id && (
            <div>Vendor ID: 0x{node.vendor_id.toString(16).padStart(8, '0')}</div>
          )}
          {node.product_name && <div>Product: {node.product_name}</div>}
        </div>
      ) : (
        <div className="p-3 text-xs text-muted-foreground">Select a node</div>
      )}
    </div>
  );
}
