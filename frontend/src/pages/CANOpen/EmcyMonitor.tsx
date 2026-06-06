// EMCY Monitor page

import { useAppStore } from '@/lib/store';

export function EmcyMonitor() {
  const entries = useAppStore((s) => s.emcy.entries);

  return (
    <div className="p-4 space-y-4">
      <h2 className="text-lg font-semibold">EMCY Monitor</h2>

      <div className="flex items-center gap-2 text-xs text-muted-foreground mb-2">
        <span>{entries.length} entries</span>
        <button
          className="px-2 py-0.5 bg-muted rounded"
          onClick={() => useAppStore.getState().emcy.clearEntries()}
        >
          Clear
        </button>
      </div>

      {entries.length === 0 ? (
        <p className="text-sm text-muted-foreground">No emergency messages received.</p>
      ) : (
        <div className="border rounded overflow-auto max-h-96">
          <div className="flex gap-2 px-3 py-1 bg-muted text-xs font-medium border-b">
            <span className="w-12">Node</span>
            <span className="w-20">Error Code</span>
            <span className="w-16">Register</span>
            <span className="flex-1">Data</span>
            <span className="w-24">Time</span>
          </div>
          {entries
            .slice()
            .reverse()
            .slice(0, 100)
            .map((entry, i) => (
              <div key={i} className="flex gap-2 px-3 py-0.5 text-xs font-mono border-b hover:bg-red-500/5">
                <span className="w-12">{entry.node_id}</span>
                <span className="w-20 text-red-500">0x{entry.error_code.toString(16).padStart(4, '0').toUpperCase()}</span>
                <span className="w-16">0x{entry.error_register.toString(16).padStart(2, '0').toUpperCase()}</span>
                <span className="flex-1 truncate">
                  {entry.data.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ')}
                </span>
                <span className="w-24 text-muted-foreground">{entry.timestamp_ms}ms</span>
              </div>
            ))}
        </div>
      )}
    </div>
  );
}
