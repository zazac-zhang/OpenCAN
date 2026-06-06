/**
 * HeartbeatMonitor — CANopen node heartbeat monitoring with timeout detection.
 *
 * Displays per-node heartbeat status with:
 * - Alive/dead indicator with color coding
 * - Relative time since last heartbeat ("2.3s ago")
 * - Heartbeat interval estimation (from consecutive heartbeats)
 * - Timeout detection — nodes that haven't sent a heartbeat within 2× their
 *   expected interval are highlighted as "Timed Out" with alarm styling
 * - Historical timeline of heartbeat events per node
 */
import { useState, useEffect, useCallback } from 'react';
import { useAppStore } from '@/lib/store';
import { AlertTriangle, Clock } from 'lucide-react';

// Default heartbeat timeout in ms (used when we don't know the interval)
const DEFAULT_HEARTBEAT_TIMEOUT = 5000;

interface NodeHeartbeatInfo {
  nodeId: number;
  alive: boolean;
  lastSeenMs: number;
  estimatedInterval: number | null;
  timedOut: boolean;
  consecutiveCount: number;
  missCount: number;
}

function formatRelativeTime(ms: number): string {
  if (ms < 1000) return `${ms}ms ago`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s ago`;
  return `${(ms / 60000).toFixed(1)}m ago`;
}

export function HeartbeatMonitor() {
  const entries = useAppStore((s) => s.heartbeat.entries);
  const nodes = useAppStore((s) => s.can.nodes);
  const [now, setNow] = useState(Date.now());

  // Update "now" every second for relative time display
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, []);

  // Build heartbeat info with timeout detection
  const heartbeatInfo = useCallback((): NodeHeartbeatInfo[] => {
    return entries.map((entry) => {
      // Estimate heartbeat interval from the entry data
      // The entry tracks timestamps of recent heartbeats
      const timestamps = entry.timestamps ?? [];
      let estimatedInterval: number | null = null;
      if (timestamps.length >= 2) {
        const recent = timestamps.slice(-5);
        let totalDiff = 0;
        for (let i = 1; i < recent.length; i++) {
          totalDiff += recent[i] - recent[i - 1];
        }
        estimatedInterval = Math.round(totalDiff / (recent.length - 1));
      }

      // Determine timeout threshold
      const timeout = estimatedInterval
        ? estimatedInterval * 2
        : DEFAULT_HEARTBEAT_TIMEOUT;

      const elapsedSinceLast = now - entry.last_seen_ms;
      const timedOut = entry.alive && elapsedSinceLast > timeout;

      // Count missed heartbeats
      const missCount = estimatedInterval
        ? Math.max(0, Math.floor(elapsedSinceLast / estimatedInterval) - 1)
        : 0;

      return {
        nodeId: entry.node_id,
        alive: entry.alive,
        lastSeenMs: entry.last_seen_ms,
        estimatedInterval,
        timedOut,
        consecutiveCount: timestamps.length,
        missCount,
      };
    });
  }, [entries, now]);

  const info = heartbeatInfo();

  // Check if any node has timed out
  const anyTimedOut = info.some((i) => i.timedOut);

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Heartbeat Monitor</h2>
        {anyTimedOut && (
          <div className="flex items-center gap-1 px-2 py-1 rounded bg-red-500/10 text-red-400 text-xs">
            <AlertTriangle className="h-3 w-3" />
            {info.filter((i) => i.timedOut).length} node(s) timed out
          </div>
        )}
      </div>

      {/* Summary */}
      <div className="grid grid-cols-3 gap-3">
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground">Monitored Nodes</div>
          <div className="text-xl font-bold font-mono mt-1">{info.length}</div>
        </div>
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground">Alive</div>
          <div className="text-xl font-bold font-mono mt-1 text-green-400">
            {info.filter((i) => i.alive && !i.timedOut).length}
          </div>
        </div>
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground">Timed Out</div>
          <div className="text-xl font-bold font-mono mt-1 text-red-400">
            {info.filter((i) => i.timedOut).length}
          </div>
        </div>
      </div>

      {info.length === 0 ? (
        <div className="bg-card border border-border rounded-md p-6 text-center">
          <Clock className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
          <p className="text-sm text-muted-foreground">
            No heartbeat data yet
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            Heartbeats will appear here once the protocol stack receives them
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          {info.map((hb) => {
            const node = nodes.find((n) => n.node_id === hb.nodeId);
            const elapsed = now - hb.lastSeenMs;
            return (
              <div
                key={hb.nodeId}
                className={`flex items-center gap-3 p-3 rounded-md border transition-colors ${
                  hb.timedOut
                    ? 'border-red-500/50 bg-red-500/5'
                    : !hb.alive
                      ? 'border-red-500/30 bg-red-500/5'
                      : 'border-border bg-card hover:bg-card/80'
                }`}
              >
                {/* Status indicator */}
                <div className="flex flex-col items-center gap-1">
                  {hb.timedOut ? (
                    <AlertTriangle className="h-5 w-5 text-red-400" />
                  ) : hb.alive ? (
                    <div className="h-3 w-3 rounded-full bg-green-500 animate-pulse" />
                  ) : (
                    <div className="h-3 w-3 rounded-full bg-red-500" />
                  )}
                </div>

                {/* Node info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">Node {hb.nodeId}</span>
                    {node?.product_name && (
                      <span className="text-xs text-muted-foreground truncate">
                        {node.product_name}
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-3 mt-1">
                    <span className={`text-xs px-2 py-0.5 rounded font-medium ${
                      hb.timedOut
                        ? 'bg-red-500/20 text-red-400'
                        : hb.alive
                          ? 'bg-green-500/20 text-green-500'
                          : 'bg-red-500/20 text-red-500'
                    }`}>
                      {hb.timedOut ? 'Timed Out' : hb.alive ? 'Alive' : 'Lost'}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {formatRelativeTime(elapsed)}
                    </span>
                    {hb.missCount > 0 && (
                      <span className="text-xs text-orange-400">
                        ~{hb.missCount} missed
                      </span>
                    )}
                  </div>
                </div>

                {/* Interval info */}
                <div className="text-right text-xs font-mono">
                  {hb.estimatedInterval !== null ? (
                    <>
                      <div className="text-muted-foreground">Interval</div>
                      <div>{hb.estimatedInterval}ms</div>
                    </>
                  ) : (
                    <div className="text-muted-foreground">—</div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
