// Heartbeat Monitor page

import { useAppStore } from '@/lib/store';

export function HeartbeatMonitor() {
  const entries = useAppStore((s) => s.heartbeat.entries);

  return (
    <div className="p-4 space-y-4">
      <h2 className="text-lg font-semibold">Heartbeat Monitor</h2>

      {entries.length === 0 ? (
        <p className="text-sm text-muted-foreground">No heartbeat data received.</p>
      ) : (
        <div className="space-y-2">
          {entries.map((entry) => (
            <div key={entry.node_id} className="flex items-center gap-3 p-3 border rounded">
              <span className="text-lg">{entry.alive ? '🟢' : '🔴'}</span>
              <span className="font-medium">Node {entry.node_id}</span>
              <span className={`text-xs px-2 py-0.5 rounded ${
                entry.alive ? 'bg-green-500/20 text-green-500' : 'bg-red-500/20 text-red-500'
              }`}>
                {entry.alive ? 'Alive' : 'Lost'}
              </span>
              <span className="text-xs text-muted-foreground">
                Last seen: {entry.last_seen_ms}ms
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
