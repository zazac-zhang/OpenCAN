/**
 * SyncManagement — CANopen SYNC producer/consumer management.
 *
 * Provides:
 * - SYNC producer start/stop with configurable period (μs)
 * - SYNC COB-ID configuration (default 0x080)
 * - SYNC counter display (total sent/received)
 * - Measured SYNC frequency and jitter
 * - Consumer statistics (receive count, measured frequency)
 */
import { useState } from 'react';
import { useAppStore } from '@/lib/store';
import { useStartSync, useStopSync } from '@/hooks/useCommands';
import { Activity, Clock, Radio } from 'lucide-react';

export function SyncManagement() {
  const syncStatus = useAppStore((s) => s.sync.status);
  const [period, setPeriod] = useState('1000');
  const [cobId, setCobId] = useState('0x080');
  const startMutation = useStartSync();
  const stopMutation = useStopSync();

  // Calculate measured SYNC rate from counter data
  const syncCount = syncStatus.sync_count || 0;
  const firstSyncTs = syncStatus.first_sync_ts;
  const lastSyncTs = syncStatus.last_sync_ts;

  let measuredFreq: number | null = null;
  if (firstSyncTs && lastSyncTs && syncCount > 1) {
    const durationSec = (lastSyncTs - firstSyncTs) / 1000;
    if (durationSec > 0) {
      measuredFreq = Math.round((syncCount / durationSec) * 100) / 100;
    }
  }

  // Calculate jitter if we have history
  const syncHistory: number[] = syncStatus.history || [];
  let jitterMs: number | null = null;
  let avgIntervalMs: number | null = null;
  if (syncHistory.length >= 3) {
    const intervals: number[] = [];
    for (let i = 1; i < syncHistory.length; i++) {
      intervals.push(syncHistory[i] - syncHistory[i - 1]);
    }
    if (intervals.length > 0) {
      const avg = intervals.reduce((a, b) => a + b, 0) / intervals.length;
      avgIntervalMs = Math.round(avg * 10) / 10;
      const variance = intervals.reduce((sum, val) => sum + Math.abs(val - avg), 0) / intervals.length;
      jitterMs = Math.round(Math.sqrt(variance) * 10) / 10;
    }
  }

  const expectedIntervalMs = (parseInt(period) || 1000) / 1000;

  // Parse COB-ID for display
  const syncCobId = parseInt(cobId, 16) || 0x080;

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <h2 className="text-lg font-semibold">SYNC Management</h2>

      {/* Summary cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground flex items-center gap-1">
            <Activity className="h-3 w-3" /> SYNC Count
          </div>
          <div className="text-xl font-bold font-mono mt-1">{syncCount}</div>
        </div>
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground flex items-center gap-1">
            <Clock className="h-3 w-3" /> Measured Freq
          </div>
          <div className="text-xl font-bold font-mono mt-1">
            {measuredFreq !== null ? `${measuredFreq} Hz` : '—'}
          </div>
        </div>
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground">Avg Interval</div>
          <div className="text-xl font-bold font-mono mt-1">
            {avgIntervalMs !== null ? `${avgIntervalMs}ms` : '—'}
          </div>
        </div>
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground">Jitter</div>
          <div className="text-xl font-bold font-mono mt-1">
            {jitterMs !== null ? `±${jitterMs}ms` : '—'}
          </div>
        </div>
      </div>

      {/* Producer configuration */}
      <div className="p-4 border rounded-lg bg-card space-y-3">
        <div className="flex items-center gap-2">
          <Radio className="h-4 w-4 text-muted-foreground" />
          <p className="text-sm font-medium">SYNC Producer</p>
          {syncStatus.is_producer && (
            <span className="ml-auto px-2 py-0.5 text-xs rounded bg-green-500/20 text-green-500 font-medium">
              Running
            </span>
          )}
        </div>

        <div className="flex items-center gap-4 flex-wrap">
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">Period (μs):</span>
            <input
              className="px-2 py-1 text-xs font-mono border rounded w-24 bg-background"
              value={period}
              onChange={(e) => setPeriod(e.target.value)}
              disabled={syncStatus.is_producer}
            />
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">COB-ID:</span>
            <input
              className="px-2 py-1 text-xs font-mono border rounded w-20 bg-background"
              value={cobId}
              onChange={(e) => setCobId(e.target.value)}
              disabled={syncStatus.is_producer}
            />
          </div>
        </div>

        <div className="flex gap-2">
          {!syncStatus.is_producer ? (
            <button
              className="px-3 py-1.5 text-sm bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
              onClick={() => startMutation.mutate(parseInt(period) || 1000)}
              disabled={startMutation.isPending}
            >
              Start Producer
            </button>
          ) : (
            <button
              className="px-3 py-1.5 text-sm bg-red-600 text-white rounded hover:bg-red-700 disabled:opacity-50"
              onClick={() => stopMutation.mutate()}
              disabled={stopMutation.isPending}
            >
              Stop Producer
            </button>
          )}
        </div>

        {syncStatus.is_producer && (
          <div className="flex gap-4 text-xs text-muted-foreground">
            <span>Configured: {syncStatus.producer_period_us || period} μs</span>
            <span>Expected: {expectedIntervalMs}ms ({Math.round(1000 / expectedIntervalMs * 100) / 100} Hz)</span>
            {measuredFreq !== null && (
              <span className={Math.abs(measuredFreq - 1000 / expectedIntervalMs) > 5 ? 'text-orange-400' : 'text-green-400'}>
                Measured: {measuredFreq} Hz
              </span>
            )}
          </div>
        )}
      </div>

      {/* Consumer status */}
      <div className="p-4 border rounded-lg bg-card space-y-3">
        <div className="flex items-center gap-2">
          <Clock className="h-4 w-4 text-muted-foreground" />
          <p className="text-sm font-medium">SYNC Consumer</p>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="p-2 border rounded bg-background">
            <div className="text-[10px] text-muted-foreground">SYNCs Received</div>
            <div className="text-lg font-bold font-mono">{syncCount}</div>
          </div>
          <div className="p-2 border rounded bg-background">
            <div className="text-[10px] text-muted-foreground">Consumer COB-ID</div>
            <div className="text-lg font-bold font-mono">0x{syncCobId.toString(16).toUpperCase().padStart(3, '0')}</div>
          </div>
        </div>

        {syncHistory.length > 0 && (
          <div className="space-y-1">
            <div className="text-xs text-muted-foreground">Recent SYNC Intervals</div>
            <div className="flex gap-1 flex-wrap">
              {syncHistory.slice(-20).map((ts: number, i: number, arr: number[]) => {
                if (i === 0) return null;
                const interval = ts - arr[i - 1];
                const deviation = avgIntervalMs !== null
                  ? Math.abs(interval - avgIntervalMs)
                  : 0;
                const color = deviation > (avgIntervalMs || 10) * 0.1
                  ? 'bg-orange-500/30 text-orange-400'
                  : 'bg-green-500/20 text-green-400';
                return (
                  <span
                    key={i}
                    className={`px-1.5 py-0.5 rounded text-[10px] font-mono ${color}`}
                    title={`Interval: ${interval}ms`}
                  >
                    {interval}ms
                  </span>
                );
              })}
            </div>
          </div>
        )}

        <p className="text-xs text-muted-foreground">
          SYNC consumption is handled automatically by the protocol stack.
          Each received SYNC triggers PDO updates and internal synchronization.
        </p>
      </div>
    </div>
  );
}
