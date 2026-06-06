// PDO Monitor page

import { useAppStore } from '@/lib/store';

export function PdoMonitor() {
  const entries = useAppStore((s) => s.pdo.entries);

  return (
    <div className="p-4 space-y-4">
      <h2 className="text-lg font-semibold">PDO Monitor</h2>

      <div className="flex items-center gap-2 text-xs text-muted-foreground mb-2">
        <span>{entries.length} entries</span>
        <button
          className="px-2 py-0.5 bg-muted rounded"
          onClick={() => useAppStore.getState().pdo.clearEntries()}
        >
          Clear
        </button>
      </div>

      {entries.length === 0 ? (
        <p className="text-sm text-muted-foreground">No PDO data received yet.</p>
      ) : (
        <div className="border rounded overflow-auto max-h-96">
          {/* Header */}
          <div className="flex gap-2 px-3 py-1 bg-muted text-xs font-medium border-b sticky top-0">
            <span className="w-12">Node</span>
            <span className="w-12">Type</span>
            <span className="w-16">COB-ID</span>
            <span className="flex-1">Data</span>
            <span className="w-24">Time</span>
          </div>
          {/* Rows */}
          {entries
            .slice()
            .reverse()
            .slice(0, 200)
            .map((entry, i) => (
              <div key={i} className="flex gap-2 px-3 py-0.5 text-xs font-mono border-b hover:bg-muted/50">
                <span className="w-12">{entry.node_id}</span>
                <span className={`w-12 ${entry.pdo_type === 'tpdo' ? 'text-blue-500' : 'text-orange-500'}`}>
                  {entry.pdo_type}
                </span>
                <span className="w-16">0x{entry.cob_id.toString(16).padStart(3, '0').toUpperCase()}</span>
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
