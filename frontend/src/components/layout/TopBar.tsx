// Top bar with connection controls, bitrate, NMT quick actions

import { useAppStore, useConnected } from '@/lib/store';
import { useConnectBackend, useDisconnect, useScanNodes, useNmtCommand, useLoadEdsFile } from '@/hooks/useCommands';
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
  Settings,
} from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { save } from '@tauri-apps/plugin-dialog';

const BITRATES = [10000, 20000, 50000, 125000, 250000, 500000, 800000, 1000000] as const;

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
  const edsMutation = useLoadEdsFile();

  const handleConnect = (backendType: string, channel: string, bitrate: number, nodeId: number) => {
    connectMutation.mutate({ backend_type: backendType, channel, bitrate, node_id: nodeId });
    startFrameStream();
  };

  const handleNmtAll = (command: string) => {
    nmtMutation.mutate({ nodeId: 0, command });
  };

  const handleBitrateChange = (bitrate: number) => {
    useAppStore.getState().connectionDialog.bitrate = bitrate;
    useAppStore.getState().ui.setStatusMessage(`Bitrate set to ${bitrate / 1000}k`);
  };

  const handleLoadEds = async () => {
    const path = await open({
      title: 'Load EDS File',
      filters: [{ name: 'EDS Files', extensions: ['eds'] }],
    });
    if (path && typeof path === 'string') {
      edsMutation.mutate(path);
    }
  };

  const handleClear = () => {
    useAppStore.getState().frames.clearFrames();
    useAppStore.getState().pdo.clearEntries();
    useAppStore.getState().emcy.clearEntries();
    useAppStore.getState().ui.setStatusMessage('Cleared all data');
  };

  const handleExport = async () => {
    const frames = useAppStore.getState().frames.frames;
    if (frames.length === 0) {
      useAppStore.getState().ui.setStatusMessage('No frames to export');
      return;
    }
    const path = await save({
      title: 'Export Frames',
      defaultPath: 'opencan-export.json',
      filters: [{ name: 'JSON Files', extensions: ['json'] }],
    });
    if (path) {
      // Simple export: write JSON to file via Tauri fs
      const { writeFile } = await import('@tauri-apps/plugin-fs');
      const content = JSON.stringify(frames, null, 2);
      await writeFile(path, new TextEncoder().encode(content));
      useAppStore.getState().ui.setStatusMessage(`Exported ${frames.length} frames to ${path}`);
    }
  };

  const handleImport = async () => {
    const path = await open({
      title: 'Import Frames',
      filters: [{ name: 'JSON Files', extensions: ['json'] }],
    });
    if (path && typeof path === 'string') {
      const { readFile } = await import('@tauri-apps/plugin-fs');
      const content = await readFile(path);
      const frames = JSON.parse(new TextDecoder().decode(content));
      if (Array.isArray(frames)) {
        useAppStore.getState().frames.addFrames(frames);
        useAppStore.getState().ui.setStatusMessage(`Imported ${frames.length} frames`);
      }
    }
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
            <Settings className="w-3 h-3 mr-1 inline" />
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
      {BITRATES.map((br) => (
        <button
          key={br}
          className={`px-2 py-0.5 text-xs border rounded transition-colors ${
            useAppStore.getState().connectionDialog.bitrate === br
              ? 'bg-primary text-primary-foreground'
              : 'hover:bg-muted'
          }`}
          onClick={() => handleBitrateChange(br)}
        >
          {br >= 1000000 ? `${br / 1000000}M` : `${br / 1000}k`}
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
      <button className="px-3 py-1 text-sm border rounded hover:bg-muted" onClick={handleClear}>
        <Eraser className="w-3 h-3 mr-1 inline" />
        Clear
      </button>
      <button className="px-3 py-1 text-sm border rounded hover:bg-muted" onClick={handleExport}>
        <FileDown className="w-3 h-3 mr-1 inline" />
        Export
      </button>
      <button className="px-3 py-1 text-sm border rounded hover:bg-muted" onClick={handleImport}>
        <FileUp className="w-3 h-3 mr-1 inline" />
        Import
      </button>

      <span className="text-muted-foreground">│</span>

      {/* EDS */}
      <button className="px-3 py-1 text-sm border rounded hover:bg-muted" onClick={handleLoadEds}>
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
