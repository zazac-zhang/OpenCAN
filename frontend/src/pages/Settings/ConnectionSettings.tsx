/**
 * ConnectionSettings — Connection settings management page.
 *
 * Allows selecting backend type, channel, bitrate, and node ID.
 * Settings can be saved as default (localStorage) and connection
 * history is tracked (last 5 connections, also in localStorage).
 */

import { Check, History, Save, Settings, Trash2, X } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useConnectBackend, useDisconnect, useGetBackends } from '@/hooks/useCommands';
import { useAppStore } from '@/lib/store';

const BACKENDS = ['mock', 'socketcan', 'kvaser', 'pcan', 'zlg'] as const;
const BITRATES = [10000, 20000, 50000, 125000, 250000, 500000, 800000, 1000000] as const;
const STORAGE_KEY = 'connection-defaults';
const HISTORY_KEY = 'connection-history';

interface ConnectionDefaults {
  backendType: string;
  channel: string;
  bitrate: number;
  nodeId: string;
}

interface ConnectionHistoryEntry {
  backendType: string;
  channel: string;
  bitrate: number;
  nodeId: number;
  timestamp: string;
  success: boolean;
}

function loadDefaults(): ConnectionDefaults {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw);
  } catch {
    // ignore
  }
  return { backendType: 'mock', channel: 'can0', bitrate: 500000, nodeId: '0' };
}

function saveDefaults(def: ConnectionDefaults) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(def));
}

function loadHistory(): ConnectionHistoryEntry[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY);
    if (raw) return JSON.parse(raw);
  } catch {
    // ignore
  }
  return [];
}

function saveHistory(entries: ConnectionHistoryEntry[]) {
  localStorage.setItem(HISTORY_KEY, JSON.stringify(entries));
}

function addHistoryEntry(entry: ConnectionHistoryEntry) {
  const history = loadHistory();
  history.unshift(entry);
  saveHistory(history.slice(0, 5));
}

function clearHistory() {
  localStorage.removeItem(HISTORY_KEY);
}

function formatBitrate(bps: number): string {
  if (bps >= 1000000) return `${bps / 1000000}M`;
  if (bps >= 1000) return `${bps / 1000}k`;
  return `${bps}`;
}

export function ConnectionSettings() {
  const connected = useAppStore((s) => s.can.connected);
  const backendInfo = useAppStore((s) => s.can.backendInfo);

  const [backendType, setBackendType] = useState(loadDefaults().backendType);
  const [channel, setChannel] = useState(loadDefaults().channel);
  const [bitrate, setBitrate] = useState(loadDefaults().bitrate);
  const [nodeId, setNodeId] = useState(loadDefaults().nodeId);
  const [history, setHistory] = useState<ConnectionHistoryEntry[]>(loadHistory());

  const connectBackend = useConnectBackend();
  const getBackends = useGetBackends();
  const disconnect = useDisconnect();

  // Refresh backend availability on mount
  useEffect(() => {
    getBackends.mutate();
  }, [getBackends.mutate]);

  const handleConnect = () => {
    const nid = parseInt(nodeId, 10);
    if (Number.isNaN(nid) || nid < 0 || nid > 127) return;

    connectBackend.mutate(
      { backend_type: backendType, channel, bitrate, node_id: nid },
      {
        onSuccess: () => {
          addHistoryEntry({
            backendType,
            channel,
            bitrate,
            nodeId: nid,
            timestamp: new Date().toISOString(),
            success: true,
          });
          setHistory(loadHistory());
        },
        onError: () => {
          addHistoryEntry({
            backendType,
            channel,
            bitrate,
            nodeId: nid,
            timestamp: new Date().toISOString(),
            success: false,
          });
          setHistory(loadHistory());
        },
      },
    );
  };

  const handleSaveDefault = () => {
    saveDefaults({ backendType, channel, bitrate, nodeId });
  };

  const handleClearHistory = () => {
    clearHistory();
    setHistory([]);
  };

  const handleLoadHistoryEntry = (entry: ConnectionHistoryEntry) => {
    setBackendType(entry.backendType);
    setChannel(entry.channel);
    setBitrate(entry.bitrate);
    setNodeId(entry.nodeId.toString());
  };

  // Backend availability map
  const availableMap = new Map<string, boolean>();
  if (getBackends.data) {
    for (const b of getBackends.data) {
      availableMap.set(b.backend_type, b.available);
    }
  }

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      {/* Connection Status */}
      <section className="flex items-center gap-2 bg-card border border-border rounded-md p-3">
        <div
          className={`h-3 w-3 rounded-full ${connected ? 'bg-green-500' : 'bg-muted-foreground/30'}`}
        />
        <span className="text-sm text-foreground">
          {connected
            ? `Connected — ${backendInfo?.backend_type} (${backendInfo?.channel})`
            : 'Disconnected'}
        </span>
        {connected && (
          <button
            onClick={() => disconnect.mutate()}
            className="ml-auto text-xs text-destructive hover:underline"
          >
            Disconnect
          </button>
        )}
      </section>

      {/* Backend Configuration */}
      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <Settings className="h-4 w-4 text-muted-foreground" />
          <h2 className="text-lg font-semibold text-foreground">Backend Configuration</h2>
        </div>

        {/* Backend Type */}
        <div className="space-y-1">
          <label className="text-sm text-muted-foreground">Backend Type</label>
          <div className="flex flex-wrap gap-2">
            {BACKENDS.map((b) => {
              const available = availableMap.get(b);
              const isActive = backendType === b;
              return (
                <button
                  key={b}
                  onClick={() => setBackendType(b)}
                  disabled={available === false}
                  className={`flex items-center gap-1 px-3 py-1.5 rounded-md text-sm border transition-colors ${
                    isActive
                      ? 'bg-primary text-primary-foreground border-primary'
                      : 'bg-card border-border text-foreground hover:bg-card/80'
                  } ${available === false ? 'opacity-40 cursor-not-allowed' : ''}`}
                >
                  {b}
                  {available !== undefined &&
                    (available ? (
                      <Check className="h-3 w-3 text-green-400" />
                    ) : (
                      <X className="h-3 w-3 text-red-400" />
                    ))}
                </button>
              );
            })}
          </div>
        </div>

        {/* Channel */}
        <div className="space-y-1">
          <label className="text-sm text-muted-foreground">Channel</label>
          <input
            type="text"
            value={channel}
            onChange={(e) => setChannel(e.target.value)}
            placeholder="can0"
            className="w-full px-3 py-2 rounded-md bg-card border border-border text-sm text-foreground font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
          />
        </div>

        {/* Bitrate */}
        <div className="space-y-1">
          <label className="text-sm text-muted-foreground">Bitrate</label>
          <div className="flex flex-wrap gap-2">
            {BITRATES.map((b) => (
              <button
                key={b}
                onClick={() => setBitrate(b)}
                className={`px-3 py-1.5 rounded-md text-sm border transition-colors ${
                  bitrate === b
                    ? 'bg-primary text-primary-foreground border-primary'
                    : 'bg-card border-border text-foreground hover:bg-card/80'
                }`}
              >
                {formatBitrate(b)}
              </button>
            ))}
          </div>
        </div>

        {/* Node ID */}
        <div className="space-y-1">
          <label className="text-sm text-muted-foreground">Node ID (0–127)</label>
          <input
            type="number"
            min={0}
            max={127}
            value={nodeId}
            onChange={(e) => setNodeId(e.target.value)}
            className="w-32 px-3 py-2 rounded-md bg-card border border-border text-sm text-foreground font-mono focus:outline-none focus:ring-1 focus:ring-primary"
          />
        </div>

        {/* Actions */}
        <div className="flex gap-2 pt-1">
          <button
            onClick={handleConnect}
            disabled={connectBackend.isPending || !channel.trim()}
            className="flex items-center gap-2 px-4 py-2 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
          >
            {connected ? <Check className="h-4 w-4" /> : <Settings className="h-4 w-4" />}
            {connected ? 'Reconnect' : 'Connect'}
          </button>
          <button
            onClick={handleSaveDefault}
            className="flex items-center gap-2 px-3 py-2 rounded-md bg-card border border-border text-foreground hover:bg-card/80 text-sm"
          >
            <Save className="h-4 w-4" />
            Save as Default
          </button>
        </div>
      </section>

      {/* Connection History */}
      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <History className="h-4 w-4 text-muted-foreground" />
            <h2 className="text-lg font-semibold text-foreground">Connection History</h2>
          </div>
          {history.length > 0 && (
            <button
              onClick={handleClearHistory}
              className="flex items-center gap-1 text-xs text-destructive hover:underline"
            >
              <Trash2 className="h-3 w-3" />
              Clear
            </button>
          )}
        </div>

        {history.length === 0 ? (
          <p className="text-sm text-muted-foreground italic">No connections yet</p>
        ) : (
          <div className="bg-card border border-border rounded-md divide-y divide-border">
            {history.map((entry, i) => (
              <div
                key={i}
                className="flex items-center gap-3 px-3 py-2 text-xs cursor-pointer hover:bg-card/80"
                onClick={() => handleLoadHistoryEntry(entry)}
              >
                <span
                  className={`h-2 w-2 rounded-full flex-shrink-0 ${
                    entry.success ? 'bg-green-500' : 'bg-red-500'
                  }`}
                />
                <span className="font-mono text-foreground">{entry.backendType}</span>
                <span className="font-mono text-muted-foreground">{entry.channel}</span>
                <span className="font-mono text-muted-foreground">
                  {formatBitrate(entry.bitrate)}
                </span>
                <span className="font-mono text-muted-foreground ml-auto">Node {entry.nodeId}</span>
                <span className="text-muted-foreground">
                  {new Date(entry.timestamp).toLocaleTimeString()}
                </span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
