/**
 * NetworkTopology — SVG-based CANopen network topology visualization.
 *
 * Features:
 * - SVG canvas with node circles/rectangles
 * - Master node at center, slaves arranged around it
 * - Connection lines with animation
 * - Node state colors (Operational/PreOp/Stopped/Offline)
 * - Click node for details + NMT commands
 * - Scan button for auto-discovery
 * - Drag-to-rearrange layout
 */

import { Network, Power, RefreshCw, RotateCw, Square, Zap } from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useNmtCommand, useScanNodes } from '@/hooks/useCommands';
import { useAppStore } from '@/lib/store';
import { cn } from '@/lib/utils';

// ===== Types =====

interface NodePosition {
  x: number;
  y: number;
}

// ===== Constants =====

const CANVAS_W = 800;
const CANVAS_H = 500;
const NODE_R = 35;
const MASTER_POS: NodePosition = { x: CANVAS_W / 2, y: 80 };

const NMT_COLORS: Record<string, string> = {
  Operational: '#22c55e',
  PreOperational: '#eab308',
  Stopped: '#ef4444',
  BootUp: '#6b7280',
  'Not Responding': '#374111',
};

const NMT_ICONS: Record<string, string> = {
  Operational: '🟢',
  PreOperational: '🟡',
  Stopped: '🔴',
  BootUp: '⚫',
  'Not Responding': '⚫',
};

// ===== Layout Algorithm =====

function computeLayout(nodeIds: number[], masterPos: NodePosition): Map<number, NodePosition> {
  const positions = new Map<number, NodePosition>();
  const slaves = nodeIds.filter((id) => id !== 1);

  if (slaves.length === 0) return positions;

  // Arrange slaves in a semicircle below master
  const radius = Math.min(180, 50 + slaves.length * 25);
  const startAngle = -Math.PI * 0.8;
  const endAngle = -Math.PI * 0.2;
  const angleStep = slaves.length > 1 ? (endAngle - startAngle) / (slaves.length - 1) : 0;

  slaves.forEach((id, i) => {
    const angle = slaves.length === 1 ? -Math.PI / 2 : startAngle + angleStep * i;
    positions.set(id, {
      x: masterPos.x + radius * Math.cos(angle),
      y: masterPos.y + 100 + radius * Math.sin(angle) * -1,
    });
  });

  return positions;
}

// ===== Main Component =====

export function NetworkTopology() {
  const nodes = useAppStore((s) => s.can.nodes);
  const selectedNode = useAppStore((s) => s.can.selectedNode);
  const heartbeatEntries = useAppStore((s) => s.heartbeat.entries);
  const scanMutation = useScanNodes(3000);
  const nmtMutation = useNmtCommand();

  const [nodePositions, setNodePositions] = useState<Map<number, NodePosition>>(new Map());
  const [dragging, setDragging] = useState<number | null>(null);
  const [dragOffset, setDragOffset] = useState<NodePosition>({ x: 0, y: 0 });
  const svgRef = useRef<SVGSVGElement>(null);

  // Build heartbeat map
  const heartbeatMap = useMemo(() => {
    const map = new Map<number, { alive: boolean; lastSeen: number }>();
    for (const entry of heartbeatEntries) {
      map.set(entry.node_id, { alive: entry.alive, lastSeen: entry.last_seen_ms });
    }
    return map;
  }, [heartbeatEntries]);

  // Compute node states
  const nodeStates = useMemo(() => {
    const states = new Map<number, string>();
    for (const node of nodes) {
      const hb = heartbeatMap.get(node.node_id);
      if (hb) {
        const elapsed = Date.now() - hb.lastSeen;
        states.set(node.node_id, hb.alive && elapsed < 10000 ? 'Operational' : 'Not Responding');
      } else {
        states.set(node.node_id, node.nmt_state || 'Unknown');
      }
    }
    return states;
  }, [nodes, heartbeatMap]);

  // Initialize layout when nodes change
  useEffect(() => {
    const nodeIds = nodes.map((n) => n.node_id);
    if (nodeIds.length === 0) {
      setNodePositions(new Map());
      return;
    }
    // Only recompute if we have new nodes
    const existingIds = new Set(nodePositions.keys());
    const newIds = nodeIds.filter((id) => !existingIds.has(id));
    if (newIds.length > 0) {
      const layout = computeLayout(nodeIds, MASTER_POS);
      setNodePositions(layout);
    }
  }, [nodes, nodePositions.keys]);

  // Handle scan
  const handleScan = useCallback(() => {
    scanMutation.mutate();
  }, [scanMutation]);

  // Handle node click
  const handleNodeClick = useCallback((nodeId: number) => {
    useAppStore.setState((s) => ({
      can: { ...s.can, selectedNode: nodeId },
    }));
  }, []);

  // Drag handlers
  const handleMouseDown = useCallback(
    (nodeId: number, e: React.MouseEvent) => {
      e.preventDefault();
      const pos = nodePositions.get(nodeId);
      if (!pos) return;
      const svg = svgRef.current;
      if (!svg) return;
      const rect = svg.getBoundingClientRect();
      setDragging(nodeId);
      setDragOffset({
        x: e.clientX - rect.left - pos.x,
        y: e.clientY - rect.top - pos.y,
      });
    },
    [nodePositions],
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (dragging === null) return;
      const svg = svgRef.current;
      if (!svg) return;
      const rect = svg.getBoundingClientRect();
      const x = Math.max(NODE_R, Math.min(CANVAS_W - NODE_R, e.clientX - rect.left - dragOffset.x));
      const y = Math.max(NODE_R, Math.min(CANVAS_H - NODE_R, e.clientY - rect.top - dragOffset.y));
      setNodePositions((prev) => {
        const next = new Map(prev);
        next.set(dragging, { x, y });
        return next;
      });
    },
    [dragging, dragOffset],
  );

  const handleMouseUp = useCallback(() => {
    setDragging(null);
  }, []);

  // Get node color
  const getNodeColor = (nodeId: number): string => {
    const state = nodeStates.get(nodeId);
    return NMT_COLORS[state || ''] || '#6b7280';
  };

  // NMT commands
  const nmtCommands = [
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

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b bg-card">
        <div className="flex items-center gap-2">
          <Network className="h-4 w-4 text-muted-foreground" />
          <h2 className="text-sm font-medium">Network Topology</h2>
          <span className="text-[10px] text-muted-foreground">
            {nodes.length} node{nodes.length !== 1 ? 's' : ''}
          </span>
        </div>
        <div className="flex items-center gap-2">
          {selectedNode !== null && (
            <div className="flex items-center gap-1">
              <span className="text-[10px] text-muted-foreground">NMT → Node {selectedNode}:</span>
              {nmtCommands.map(({ command, label, icon: Icon, color }) => (
                <button
                  key={command}
                  className={`px-1.5 py-0.5 text-[10px] rounded text-white flex items-center gap-1 ${color}`}
                  onClick={() => nmtMutation.mutate({ nodeId: selectedNode, command })}
                  disabled={nmtMutation.isPending}
                >
                  <Icon className="h-3 w-3" />
                  {label}
                </button>
              ))}
            </div>
          )}
          <button
            onClick={handleScan}
            disabled={scanMutation.isPending}
            className="flex items-center gap-1 px-2 py-1 text-xs rounded border hover:bg-muted disabled:opacity-50"
          >
            <RefreshCw className={cn('h-3 w-3', scanMutation.isPending && 'animate-spin')} />
            Scan
          </button>
        </div>
      </div>

      {/* SVG Canvas */}
      <div className="flex-1 overflow-hidden bg-background">
        {nodes.length === 0 ? (
          <div className="flex items-center justify-center h-full text-muted-foreground italic text-sm">
            No nodes discovered. Click "Scan" to find nodes on the network.
          </div>
        ) : (
          <svg
            ref={svgRef}
            width="100%"
            height="100%"
            viewBox={`0 0 ${CANVAS_W} ${CANVAS_H}`}
            className="cursor-default"
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            onMouseLeave={handleMouseUp}
          >
            {/* Connection lines (Master → Slaves) */}
            {nodes
              .filter((n) => n.node_id !== 1)
              .map((node) => {
                const pos = nodePositions.get(node.node_id);
                if (!pos) return null;
                return (
                  <line
                    key={`edge-${node.node_id}`}
                    x1={MASTER_POS.x}
                    y1={MASTER_POS.y + NODE_R}
                    x2={pos.x}
                    y2={pos.y - NODE_R}
                    stroke="#374151"
                    strokeWidth={1.5}
                    strokeDasharray="4 2"
                  />
                );
              })}

            {/* Master Node */}
            <g onClick={() => handleNodeClick(1)} className="cursor-pointer">
              <circle
                cx={MASTER_POS.x}
                cy={MASTER_POS.y}
                r={NODE_R}
                fill={`${getNodeColor(1)}20`}
                stroke={getNodeColor(1)}
                strokeWidth={selectedNode === 1 ? 3 : 2}
              />
              <text
                x={MASTER_POS.x}
                y={MASTER_POS.y - 8}
                textAnchor="middle"
                className="text-[11px] font-medium"
                fill={getNodeColor(1)}
              >
                Master
              </text>
              <text
                x={MASTER_POS.x}
                y={MASTER_POS.y + 8}
                textAnchor="middle"
                className="text-[10px]"
                fill="#9ca3af"
              >
                Node 1
              </text>
              {selectedNode === 1 && (
                <circle
                  cx={MASTER_POS.x + NODE_R - 5}
                  cy={MASTER_POS.y - NODE_R + 5}
                  r={6}
                  fill="#3b82f6"
                />
              )}
            </g>

            {/* Slave Nodes */}
            {nodes
              .filter((n) => n.node_id !== 1)
              .map((node) => {
                const pos = nodePositions.get(node.node_id);
                if (!pos) return null;
                const color = getNodeColor(node.node_id);
                const isSelected = selectedNode === node.node_id;
                const state = nodeStates.get(node.node_id) || 'Unknown';
                const icon = NMT_ICONS[state] || '⚫';

                return (
                  <g
                    key={node.node_id}
                    onClick={() => handleNodeClick(node.node_id)}
                    onMouseDown={(e) => handleMouseDown(node.node_id, e)}
                    className="cursor-pointer"
                  >
                    <circle
                      cx={pos.x}
                      cy={pos.y}
                      r={NODE_R}
                      fill={`${color}20`}
                      stroke={color}
                      strokeWidth={isSelected ? 3 : 1.5}
                    />
                    <text
                      x={pos.x}
                      y={pos.y - 10}
                      textAnchor="middle"
                      className="text-[11px] font-medium"
                      fill={color}
                    >
                      {icon} Node
                    </text>
                    <text
                      x={pos.x}
                      y={pos.y + 8}
                      textAnchor="middle"
                      className="text-[12px] font-bold"
                      fill={color}
                    >
                      {node.node_id}
                    </text>
                    <text
                      x={pos.x}
                      y={pos.y + 22}
                      textAnchor="middle"
                      className="text-[9px]"
                      fill="#9ca3af"
                    >
                      {state}
                    </text>
                    {isSelected && (
                      <circle
                        cx={pos.x + NODE_R - 5}
                        cy={pos.y - NODE_R + 5}
                        r={6}
                        fill="#3b82f6"
                      />
                    )}
                  </g>
                );
              })}
          </svg>
        )}
      </div>

      {/* Node Info Panel */}
      {selectedNode !== null && (
        <div className="border-t bg-card px-3 py-2 flex items-center gap-3">
          <div className="flex items-center gap-2">
            <span
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: getNodeColor(selectedNode) }}
            />
            <span className="text-sm font-medium">Node {selectedNode}</span>
            <span className="text-xs text-muted-foreground">
              {nodeStates.get(selectedNode) || 'Unknown'}
            </span>
          </div>
          <div className="flex items-center gap-1 ml-auto">
            {nmtCommands.map(({ command, label, icon: Icon, color }) => (
              <button
                key={command}
                className={`px-2 py-1 text-xs rounded text-white flex items-center gap-1 ${color}`}
                onClick={() => nmtMutation.mutate({ nodeId: selectedNode, command })}
                disabled={nmtMutation.isPending}
              >
                <Icon className="h-3 w-3" />
                {label}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
