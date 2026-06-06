// Top bar with connection controls, bitrate, NMT quick actions

import { useAppStore, useConnected } from '@/lib/store';
import { useConnectBackend, useDisconnect, useScanNodes, useNmtCommand } from '@/hooks/useCommands';
import { useFrameStream } from '@/hooks/useFrameStream';
import { usePdoStream } from '@/hooks/usePdoStream';
import {
  Play,
  Pause,
  ScanSearch,
  FileDown,
  FileUp,
  Eraser,
  Power,
  HardDrive,
} from 'lucide-react';

export function TopBar() {
  const connected = useConnected();
  const ui = useAppStore((s) => s.ui);
  const connectionDialog = useAppStore((s) => s.connectionDialog);
  const { startListening: startFrameStream } = useFrameStream();
  usePdoStream();

  const connectMutation = useConnectBackend();
  const disconnectMutation = useDisconnect();
  const scanMutation = useScanNodes();
  const nmtMutation = useNmtCommand();

  const handleConnect = (backendType: string, channel: string, bitrate: number, nodeId: number) => {
    connectMutation.mutate({ backend_type: backendType, channel, bitrate, node_id: nodeId });
    startFrameStream();
  };

  const handleNmtAll = (command: string) => {
    nmtMutation.mutate({ nodeId: 0, command });
  };

  return (
    <div className="flex items-center gap-2 px-4 py-2 border-b bg-background h-12 shrink-0">
      {/* Connection */}
      {!connected ? (
        <>
          <button
            className="px-3 py-1 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90"
            onClick={() => handleConnect('mock', 'mock0', 500000, 0)}
          >
            <Power className="w-3 h-3 mr-1 inline" />
            Connect (Mock)
          </button>
          <button
            className="px-3 py-1 text-sm border rounded hover:bg-muted"
            onClick={() => connectionDialog.show()}
          >
            Connect...
          </button>
        </>
      ) : (
        <>
          <button className="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700" onClick={() => disconnectMutation.mutate()}>
            Disconnect
          </button>
          <button className="px-3 py-1 text-sm border rounded hover:bg-muted" onClick={() => scanMutation.mutate()}>
            <ScanSearch className="w-3 h-3 mr-1 inline" />
            Scan Nodes
          </button>
        </>
      )}

      <span className="text-muted-foreground">│</span>

      {/* NMT quick actions */}
      {connected && (
        <>
          <button className="px-3 py-1 text-sm border rounded hover:bg-muted" onClick={() => handleNmtAll('start')}>
            Start All
          </button>
          <button className="px-3 py-1 text-sm border rounded hover:bg-muted" onClick={() => handleNmtAll('stop')}>
            Stop All
          </button>
          <button className="px-3 py-1 text-sm border rounded hover:bg-muted" onClick={() => handleNmtAll('reset')}>
            Reset All
          </button>
          <span className="text-muted-foreground">│</span>
        </>
      )}

      {/* Bitrate selector */}
      <span className="text-xs text-muted-foreground">Bitrate:</span>
      {[125000, 250000, 500000, 1000000].map((br) => (
        <button
          key={br}
          className="px-2 py-0.5 text-xs border rounded hover:bg-muted"
          onClick={() => {}}
        >
          {br / 1000}k
        </button>
      ))}

      <span className="text-muted-foreground">│</span>

      {/* Pause/Resume */}
      <button
        className="px-3 py-1 text-sm border rounded hover:bg-muted"
        onClick={() => ui.togglePause()}
      >
        {ui.paused ? <Play className="w-3 h-3 mr-1 inline" /> : <Pause className="w-3 h-3 mr-1 inline" />}
        {ui.paused ? 'Resume' : 'Pause'}
      </button>

      {/* Log actions */}
      <button className="px-3 py-1 text-sm border rounded hover:bg-muted">
        <Eraser className="w-3 h-3 mr-1 inline" />
        Clear
      </button>
      <button className="px-3 py-1 text-sm border rounded hover:bg-muted">
        <FileDown className="w-3 h-3 mr-1 inline" />
        Export
      </button>
      <button className="px-3 py-1 text-sm border rounded hover:bg-muted">
        <FileUp className="w-3 h-3 mr-1 inline" />
        Import
      </button>

      <span className="text-muted-foreground">│</span>

      {/* EDS */}
      <button className="px-3 py-1 text-sm border rounded hover:bg-muted">
        <HardDrive className="w-3 h-3 mr-1 inline" />
        Load EDS
      </button>

      <span className="text-muted-foreground">│</span>

      {/* Detail panel toggle */}
      <button className="px-3 py-1 text-sm border rounded hover:bg-muted" onClick={() => ui.toggleDetailPanel()}>
        {ui.detailPanelVisible ? 'Hide Detail' : 'Show Detail'}
      </button>

      {/* Status indicator */}
      <div className="flex-1" />
      <span className="text-xs text-muted-foreground">
        {ui.paused ? '⏸ PAUSED' : ''}
      </span>
    </div>
  );
}
