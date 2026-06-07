// Minimal top bar — connection status, theme toggle

import { useConnected, useAppStore } from '@/lib/store';
import { useConnectBackend, useDisconnect } from '@/hooks/useCommands';
import { Power, Settings2, Moon, Sun } from 'lucide-react';
import { useState, useEffect } from 'react';

export function TopBar() {
  const connected = useConnected();
  const backendInfo = useAppStore((s) => s.can.backendInfo);
  const showConnectionDialog = useAppStore((s) => s.connectionDialog.show);
  const dialogBitrate = useAppStore((s) => s.connectionDialog.bitrate);
  const [dark, setDark] = useState(true);

  const connectMutation = useConnectBackend();
  const disconnectMutation = useDisconnect();

  // Quick mock connect
  const handleMockConnect = () => {
    connectMutation.mutate({ backend_type: 'mock', channel: 'mock0', bitrate: dialogBitrate, node_id: 0 });
  };

  // Format bitrate display
  const formatBitrate = (bps: number) => {
    if (bps >= 1000000) return `${bps / 1000000}M`;
    return `${bps / 1000}k`;
  };

  // Sync theme class
  useEffect(() => {
    document.documentElement.classList.toggle('dark', dark);
  }, [dark]);

  return (
    <div className="flex items-center gap-3 px-4 h-10 bg-background border-b border-border shrink-0">
      {/* Logo */}
      <div className="flex items-center gap-2 shrink-0">
        <div className="w-6 h-6 rounded bg-primary flex items-center justify-center">
          <span className="text-[10px] font-bold text-primary-foreground">⚡</span>
        </div>
        <span className="text-sm font-semibold tracking-tight">OpenCAN</span>
      </div>

      <div className="w-px h-5 bg-border" />

      {/* Connection status / actions */}
      {!connected ? (
        <div className="flex items-center gap-2">
          <button
            className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium bg-primary text-primary-foreground rounded hover:bg-primary/90 transition-colors"
            onClick={handleMockConnect}
          >
            <Power className="w-3 h-3" />
            Connect
          </button>
          <button
            className="flex items-center gap-1.5 px-2.5 py-1 text-xs border border-border rounded hover:bg-muted transition-colors"
            onClick={showConnectionDialog}
          >
            <Settings2 className="w-3 h-3" />
            Configure
          </button>
        </div>
      ) : (
        <div className="flex items-center gap-2">
          <span className="flex items-center gap-1.5 text-xs">
            <span className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
            <span className="text-muted-foreground">Connected</span>
          </span>
          {backendInfo && (
            <span className="text-xs font-mono text-muted-foreground">
              {backendInfo.backend_type} @ {formatBitrate(backendInfo.bitrate)}
            </span>
          )}
          <button
            className="px-2 py-0.5 text-xs text-destructive border border-destructive/30 rounded hover:bg-destructive/10 transition-colors"
            onClick={() => disconnectMutation.mutate()}
          >
            Disconnect
          </button>
        </div>
      )}

      <div className="flex-1" />

      {/* Theme toggle */}
      <button
        className="p-1.5 rounded hover:bg-muted transition-colors text-muted-foreground"
        onClick={() => setDark(!dark)}
        title={dark ? 'Switch to light theme' : 'Switch to dark theme'}
      >
        {dark ? <Sun className="w-3.5 h-3.5" /> : <Moon className="w-3.5 h-3.5" />}
      </button>

      {/* Settings shortcut */}
      <button
        className="p-1.5 rounded hover:bg-muted transition-colors text-muted-foreground"
        onClick={() => {
          useAppStore.getState().sidebar.setActiveGroup('eds');
        }}
        title="Settings & EDS"
      >
        <Settings2 className="w-3.5 h-3.5" />
      </button>
    </div>
  );
}
