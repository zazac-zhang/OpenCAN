/**
 * ConnectionDialog — A modal dialog for configuring and establishing
 * a connection to a CAN hardware backend.
 *
 * Reads configuration from the Zustand store's `connectionDialog` slice,
 * populates the backend list via `useGetBackends`, and submits via
 * `useConnectBackend`.
 */

import { useEffect, useState, useCallback } from 'react';
import { useAppStore } from '@/lib/store';
import { useConnectBackend, useGetBackends } from '@/hooks/useCommands';
import { cn } from '@/lib/utils';
import { X, Loader2, Wifi } from 'lucide-react';

const BITRATE_OPTIONS = [
  { label: '10k', value: 10000 },
  { label: '20k', value: 20000 },
  { label: '50k', value: 50000 },
  { label: '125k', value: 125000 },
  { label: '250k', value: 250000 },
  { label: '500k', value: 500000 },
  { label: '800k', value: 800000 },
  { label: '1M', value: 1000000 },
];

export function ConnectionDialog() {
  const dialog = useAppStore((s) => s.connectionDialog);
  const [localBackend, setLocalBackend] = useState(dialog.selectedBackend);
  const [localChannel, setLocalChannel] = useState(dialog.channel);
  const [localBitrate, setLocalBitrate] = useState(dialog.bitrate);
  const [localNodeId, setLocalNodeId] = useState(dialog.nodeId);
  const [error, setError] = useState<string | null>(null);

  const getBackends = useGetBackends();
  const connectMutation = useConnectBackend();

  // Sync local state when dialog opens
  useEffect(() => {
    if (dialog.visible) {
      setLocalBackend(dialog.selectedBackend);
      setLocalChannel(dialog.channel);
      setLocalBitrate(dialog.bitrate);
      setLocalNodeId(dialog.nodeId);
      setError(null);
      getBackends.mutate();
    }
  }, [dialog.visible]);

  // Escape key closes dialog
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape') dialog.hide();
    },
    [dialog.hide],
  );

  useEffect(() => {
    if (dialog.visible) {
      document.addEventListener('keydown', handleKeyDown);
      return () => document.removeEventListener('keydown', handleKeyDown);
    }
  }, [dialog.visible, handleKeyDown]);

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) dialog.hide();
  };

  const handleConnect = () => {
    setError(null);
    connectMutation.mutate(
      {
        backend_type: localBackend,
        channel: localChannel,
        bitrate: localBitrate,
        node_id: parseInt(localNodeId, 10) || 0,
      },
      {
        onError: (err) => {
          setError(err instanceof Error ? err.message : String(err));
        },
      },
    );
  };

  const isConnecting = connectMutation.isPending;

  if (!dialog.visible) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={handleBackdropClick}
    >
      <div className="w-full max-w-md rounded-xl border bg-card shadow-2xl animate-in fade-in zoom-in duration-200">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b">
          <div className="flex items-center gap-2">
            <Wifi className="w-4 h-4" />
            <h2 className="text-lg font-semibold">Connect to CAN Bus</h2>
          </div>
          <button
            className="p-1 rounded hover:bg-muted transition-colors"
            onClick={() => dialog.hide()}
            disabled={isConnecting}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Form */}
        <div className="px-6 py-4 space-y-4">
          {/* Backend selector */}
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Backend</label>
            <select
              className="w-full px-3 py-2 text-sm rounded-md border bg-background"
              value={localBackend}
              onChange={(e) => setLocalBackend(e.target.value)}
              disabled={isConnecting}
            >
              {getBackends.data
                ?.filter((b) => b.available)
                .map((b) => (
                  <option key={b.backend_type} value={b.backend_type}>
                    {b.name}
                  </option>
                ))}
              {/* Fallback options when backends not yet loaded */}
              {(!getBackends.data || getBackends.data.length === 0) && (
                <>
                  <option value="mock">Mock (Simulator)</option>
                  <option value="socketcan">SocketCAN</option>
                  <option value="kvaser">Kvaser</option>
                  <option value="pcan">PCAN</option>
                  <option value="zlg">ZLG</option>
                </>
              )}
            </select>
          </div>

          {/* Channel */}
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Channel</label>
            <input
              className="w-full px-3 py-2 text-sm rounded-md border bg-background"
              type="text"
              placeholder={
                localBackend === 'socketcan' ? 'e.g. can0, vcan0' :
                localBackend === 'zlg' ? 'e.g. 4:0:0 (type:index:channel)' :
                localBackend === 'kvaser' ? 'e.g. 0 (channel number)' :
                localBackend === 'pcan' ? 'e.g. USBBUS1' :
                'e.g. can0'
              }
              value={localChannel}
              onChange={(e) => setLocalChannel(e.target.value)}
              disabled={isConnecting}
            />
          </div>

          {/* Bitrate */}
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Bitrate</label>
            <select
              className="w-full px-3 py-2 text-sm rounded-md border bg-background"
              value={localBitrate}
              onChange={(e) => setLocalBitrate(Number(e.target.value))}
              disabled={isConnecting}
            >
              {BITRATE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>

          {/* Node ID */}
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Node ID (0-127)</label>
            <input
              className="w-full px-3 py-2 text-sm rounded-md border bg-background"
              type="number"
              min={0}
              max={127}
              value={localNodeId}
              onChange={(e) => setLocalNodeId(e.target.value)}
              disabled={isConnecting}
            />
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t space-y-3">
          {error && (
            <div className="text-sm text-destructive bg-destructive/10 rounded-md px-3 py-2">
              {error}
            </div>
          )}
          <div className="flex items-center justify-end gap-2">
            <button
              className="px-4 py-2 text-sm rounded-md border hover:bg-muted transition-colors"
              onClick={() => dialog.hide()}
              disabled={isConnecting}
            >
              Cancel
            </button>
            <button
              className={cn(
                'px-4 py-2 text-sm rounded-md bg-primary text-primary-foreground transition-colors',
                'hover:bg-primary/90 disabled:opacity-50',
              )}
              onClick={handleConnect}
              disabled={isConnecting || !localChannel}
            >
              {isConnecting ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 mr-1.5 inline animate-spin" />
                  Connecting...
                </>
              ) : (
                'Connect'
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
