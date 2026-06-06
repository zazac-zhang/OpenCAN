/**
 * NodeCard — A compact status card displaying CANOpen node information,
 * including Node ID, NMT state with a colored indicator dot, and
 * optional device type.
 */

import { cn } from '@/lib/utils';
import type { NodeInfo } from '@/types/canopen';

export interface NodeCardProps {
  node: NodeInfo;
  onClick?: (nodeId: number) => void;
  selected?: boolean;
}

const NMT_STATE_COLORS: Record<string, string> = {
  Operational: 'bg-green-500',
  PreOperational: 'bg-yellow-500',
  Stopped: 'bg-red-500',
  Unknown: 'bg-gray-500',
};

const NMT_STATE_LABELS: Record<string, string> = {
  Operational: 'Operational',
  PreOperational: 'Pre-Op',
  Stopped: 'Stopped',
  Unknown: 'Unknown',
};

export function NodeCard({ node, onClick, selected }: NodeCardProps) {
  const dotColor = NMT_STATE_COLORS[node.nmt_state] ?? 'bg-gray-500';
  const stateLabel = NMT_STATE_LABELS[node.nmt_state] ?? node.nmt_state;

  return (
    <div
      className={cn(
        'flex items-center gap-3 px-3 py-2 rounded-lg border bg-card cursor-pointer transition-all duration-150 hover:bg-muted',
        selected && 'border-primary ring-1 ring-primary',
      )}
      style={{ minHeight: 80 }}
      onClick={() => onClick?.(node.node_id)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick?.(node.node_id);
        }
      }}
    >
      {/* Node ID badge */}
      <div className="flex flex-col items-center justify-center w-12 h-12 rounded-lg bg-muted font-mono text-sm font-bold">
        {node.node_id}
      </div>

      {/* State info */}
      <div className="flex flex-col gap-0.5 flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className={cn('w-2.5 h-2.5 rounded-full shrink-0', dotColor)} />
          <span className="text-sm font-medium truncate">{stateLabel}</span>
        </div>
        <span className="text-xs text-muted-foreground">
          Node {node.node_id}
        </span>
        {node.device_type !== undefined && (
          <span className="text-xs text-muted-foreground truncate">
            Device type: 0x{node.device_type.toString(16).toUpperCase()}
          </span>
        )}
        {node.product_name && (
          <span className="text-xs text-muted-foreground truncate" title={node.product_name}>
            {node.product_name}
          </span>
        )}
      </div>
    </div>
  );
}
