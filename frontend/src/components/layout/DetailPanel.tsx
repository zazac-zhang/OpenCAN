// Detail panel (right sidebar) with accordion sections

import { ChevronDown, ChevronRight, Eye, X } from 'lucide-react';
import { useCallback, useState } from 'react';
import { useNmtCommand, useSdoUpload } from '@/hooks/useCommands';
import { useAppStore, useNodes, useSelectedNode } from '@/lib/store';

// Accordion section component
function AccordionSection({
  title,
  defaultOpen = false,
  children,
}: {
  title: string;
  defaultOpen?: boolean;
  children: React.ReactNode;
}) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className="border-b border-border/50 last:border-b-0">
      <button
        className="w-full flex items-center gap-2 px-3 py-2 text-xs font-medium hover:bg-muted/30 transition-colors"
        onClick={() => setOpen(!open)}
      >
        {open ? (
          <ChevronDown className="w-3 h-3 text-muted-foreground" />
        ) : (
          <ChevronRight className="w-3 h-3 text-muted-foreground" />
        )}
        {title}
      </button>
      {open && <div className="px-3 pb-3">{children}</div>}
    </div>
  );
}

// SDO Quick Read entry
function SdoQuickRead({
  nodeId,
  index,
  subindex,
  label,
  dataType,
}: {
  nodeId: number;
  index: number;
  subindex: number;
  label: string;
  dataType: string;
}) {
  const uploadMutation = useSdoUpload();
  const [result, setResult] = useState<string | null>(null);

  const handleRead = useCallback(() => {
    uploadMutation.mutate(
      { node_id: nodeId, index, subindex, data_type: dataType },
      {
        onSuccess: (data) => {
          setResult(data?.data.map((b) => b.toString(16).padStart(2, '0')).join(' ') || '—');
        },
        onError: () => {
          setResult('Error');
        },
      },
    );
  }, [uploadMutation, nodeId, index, subindex, dataType]);

  return (
    <div className="flex items-center gap-1.5 text-[10px]">
      <span className="text-muted-foreground font-mono w-14 shrink-0">
        0x{index.toString(16).padStart(4, '0')}:{subindex.toString(16).padStart(2, '0')}
      </span>
      <span className="flex-1 truncate">{label}</span>
      <button
        className="p-0.5 text-muted-foreground hover:text-primary transition-colors"
        onClick={handleRead}
        disabled={uploadMutation.isPending}
        title="Read"
      >
        <Eye className="w-3 h-3" />
      </button>
      {result && (
        <span className="font-mono text-foreground truncate max-w-[80px]" title={result}>
          {result}
        </span>
      )}
    </div>
  );
}

// NMT action button
function NmtActionButton({
  nodeId,
  command,
  label,
  colorClass,
}: {
  nodeId: number;
  command: string;
  label: string;
  colorClass: string;
}) {
  const nmtMutation = useNmtCommand();

  return (
    <button
      className={`px-2 py-1 text-[10px] rounded border transition-colors ${colorClass}`}
      onClick={() => nmtMutation.mutate({ nodeId, command })}
    >
      {label}
    </button>
  );
}

export function DetailPanel() {
  const selectedNode = useSelectedNode();
  const nodes = useNodes();
  const node = selectedNode !== null ? nodes.find((n) => n.node_id === selectedNode) : null;
  const nodeState = useAppStore((s) => s.ds402.nodeStates[selectedNode ?? -1]);
  const { toggleDetailPanel } = useAppStore((s) => s.ui);

  const QUICK_READS = [
    { index: 0x1000, subindex: 0, label: 'Device Type', type: 'UNS32' },
    { index: 0x1001, subindex: 0, label: 'Error Reg', type: 'UNS8' },
    { index: 0x1018, subindex: 1, label: 'Vendor ID', type: 'UNS32' },
    { index: 0x1008, subindex: 0, label: 'Mfr Name', type: 'VISIBLE_STRING' },
    { index: 0x6041, subindex: 0, label: 'StatusWord', type: 'UNS16' },
    { index: 0x6064, subindex: 0, label: 'Actual Pos', type: 'INTEGER32' },
  ];

  if (!node) return null;

  return (
    <div className="w-72 border-l bg-background flex flex-col shrink-0 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b bg-card/50">
        <div>
          <h2 className="text-xs font-semibold">Node {node.node_id}</h2>
          {node.product_name && (
            <p className="text-[10px] text-muted-foreground truncate max-w-[160px]">
              {node.product_name}
            </p>
          )}
        </div>
        <button
          className="p-1 text-muted-foreground hover:text-foreground transition-colors"
          onClick={toggleDetailPanel}
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      <div className="flex-1 overflow-auto">
        {/* Overview */}
        <AccordionSection title="Overview" defaultOpen>
          <div className="space-y-1.5">
            {/* NMT state */}
            <div className="flex items-center gap-2">
              <span
                className={`w-2 h-2 rounded-full ${
                  node.nmt_state === 'Operational'
                    ? 'bg-green-500'
                    : node.nmt_state === 'PreOperational'
                      ? 'bg-yellow-500'
                      : 'bg-red-500'
                }`}
              />
              <span className="text-xs">{node.nmt_state}</span>
            </div>

            {/* NMT quick actions */}
            <div className="grid grid-cols-3 gap-1">
              <NmtActionButton
                nodeId={node.node_id}
                command="start"
                label="Start"
                colorClass="bg-green-500/10 text-green-500 border-green-500/20 hover:bg-green-500/20"
              />
              <NmtActionButton
                nodeId={node.node_id}
                command="stop"
                label="Stop"
                colorClass="bg-red-500/10 text-red-500 border-red-500/20 hover:bg-red-500/20"
              />
              <NmtActionButton
                nodeId={node.node_id}
                command="reset"
                label="Reset"
                colorClass="bg-yellow-500/10 text-yellow-500 border-yellow-500/20 hover:bg-yellow-500/20"
              />
            </div>

            {/* Device info */}
            <div className="text-[10px] space-y-0.5">
              {node.device_type !== undefined && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Device</span>
                  <span className="font-mono">0x{node.device_type.toString(16).toUpperCase()}</span>
                </div>
              )}
              {node.vendor_id !== undefined && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Vendor</span>
                  <span className="font-mono">0x{node.vendor_id.toString(16).toUpperCase()}</span>
                </div>
              )}
            </div>
          </div>
        </AccordionSection>

        {/* SDO Quick Read */}
        <AccordionSection title="SDO Quick Read" defaultOpen>
          <div className="space-y-0.5">
            {QUICK_READS.map((qr) => (
              <SdoQuickRead
                key={`${qr.index}-${qr.subindex}`}
                nodeId={node.node_id}
                index={qr.index}
                subindex={qr.subindex}
                label={qr.label}
                dataType={qr.type}
              />
            ))}
          </div>
        </AccordionSection>

        {/* DS402 Control */}
        {nodeState && (
          <AccordionSection title="DS402 Control" defaultOpen>
            <div className="space-y-2">
              {/* State badge */}
              <div
                className={`px-2 py-1 rounded text-[10px] font-medium text-center ${
                  nodeState.state === 'Operation Enabled'
                    ? 'bg-green-500/20 text-green-400'
                    : nodeState.state === 'Fault'
                      ? 'bg-red-500/20 text-red-400'
                      : 'bg-yellow-500/20 text-yellow-400'
                }`}
              >
                {nodeState.state}
              </div>

              {/* Status/Control Word */}
              <div className="grid grid-cols-2 gap-1 text-[10px] font-mono">
                <div className="text-muted-foreground">SW</div>
                <div>0x{nodeState.status_word?.toString(16).padStart(4, '0').toUpperCase()}</div>
                <div className="text-muted-foreground">CW</div>
                <div>0x{nodeState.control_word?.toString(16).padStart(4, '0').toUpperCase()}</div>
              </div>

              {/* Actual values */}
              <div className="space-y-0.5 text-[10px]">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Position</span>
                  <span className="font-mono">{nodeState.actual_position?.toFixed(1)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Velocity</span>
                  <span className="font-mono">{nodeState.actual_velocity?.toFixed(1)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Torque</span>
                  <span className="font-mono">{nodeState.actual_torque?.toFixed(1)}%</span>
                </div>
              </div>

              {/* Quick actions */}
              <div className="grid grid-cols-2 gap-1">
                <button className="px-2 py-1 text-[10px] rounded bg-red-500/10 text-red-500 border border-red-500/20 hover:bg-red-500/20 transition-colors">
                  Quick Stop
                </button>
                <button className="px-2 py-1 text-[10px] rounded bg-yellow-500/10 text-yellow-500 border border-yellow-500/20 hover:bg-yellow-500/20 transition-colors">
                  Fault Reset
                </button>
              </div>
            </div>
          </AccordionSection>
        )}
      </div>
    </div>
  );
}
