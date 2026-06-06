/**
 * EmcyMonitor — CANopen emergency message monitoring with error code decoding.
 *
 * Displays EMCY entries with:
 * - CiA 301 standard error code descriptions
 * - Error register bit decoding
 * - Filtering by node ID and error category
 * - Per-node EMCY statistics
 * - Formatted timestamps (relative + absolute)
 * - Severity-based row highlighting
 */
import { useState, useMemo } from 'react';
import { useAppStore } from '@/lib/store';
import { AlertTriangle } from 'lucide-react';

// CiA 301 standard error codes
const ERROR_CODE_DESCRIPTIONS: Record<number, string> = {
  0x0000: 'Error Reset or No Error',
  0x1000: 'Generic Error',
  0x2000: 'Current Error',
  0x2100: 'Current: Device Input Side',
  0x2200: 'Current: Inside Device',
  0x2300: 'Current: Device Output Side',
  0x3000: 'Voltage Error',
  0x3100: 'Voltage: Mains',
  0x3200: 'Voltage: Inside Device',
  0x3300: 'Voltage: Device Output Side',
  0x4000: 'Temperature Error',
  0x4100: 'Temperature: Ambient',
  0x4200: 'Temperature: Device',
  0x5000: 'Device Hardware Error',
  0x6000: 'Device Software Error',
  0x6100: 'Internal Software',
  0x6200: 'User Software',
  0x6300: 'Data Set',
  0x7000: 'Additional Modules',
  0x8000: 'Monitoring Error',
  0x8100: 'Communication: Overrun',
  0x8110: 'Communication: Late',
  0x8120: 'Communication: Source Time',
  0x8130: 'Communication: Unexpected PDO',
  0x8140: 'Communication: Error Passive',
  0x8200: 'Protocol Error',
  0x8F00: 'Heartbeat Consumer',
  0x9000: 'External Error',
  0xF000: 'Additional Functions',
  0xFF00: 'Device Specific',
};

// Error register bit definitions (CiA 301)
const ERROR_REGISTER_BITS: [number, string][] = [
  [0, 'Generic Error'],
  [1, 'Current Error'],
  [2, 'Voltage Error'],
  [3, 'Temperature Error'],
  [4, 'Communication Error'],
  [5, 'Device Profile Specific'],
  [6, 'Reserved (always 0)'],
  [7, 'Manufacturer Specific'],
];

function getErrorDescription(code: number): string {
  // Try exact match first
  if (ERROR_CODE_DESCRIPTIONS[code]) return ERROR_CODE_DESCRIPTIONS[code];
  // Try category match (mask to 0xF000)
  const category = code & 0xF000;
  if (ERROR_CODE_DESCRIPTIONS[category]) return ERROR_CODE_DESCRIPTIONS[category];
  // Try sub-category match (mask to 0xFF00)
  const subCategory = code & 0xFF00;
  if (ERROR_CODE_DESCRIPTIONS[subCategory]) return ERROR_CODE_DESCRIPTIONS[subCategory];
  return 'Device Specific Error';
}

function decodeErrorRegister(reg: number): string[] {
  return ERROR_REGISTER_BITS
    .filter(([bit]) => reg & (1 << bit))
    .map(([, desc]) => desc);
}

function getSeverity(errorCode: number): 'critical' | 'warning' | 'info' {
  if (errorCode === 0x0000) return 'info';
  if (errorCode >= 0x8000) return 'critical';
  if (errorCode >= 0x1000) return 'warning';
  return 'info';
}

function formatTimestamp(ms: number): string {
  const sec = Math.floor(ms / 1000);
  const millis = ms % 1000;
  return `${sec}.${millis.toString().padStart(3, '0')}`;
}

function formatRelativeTime(ms: number): string {
  const elapsed = Date.now() - ms;
  if (elapsed < 1000) return `${elapsed}ms ago`;
  if (elapsed < 60000) return `${(elapsed / 1000).toFixed(1)}s ago`;
  return `${(elapsed / 60000).toFixed(1)}m ago`;
}

export function EmcyMonitor() {
  const entries = useAppStore((s) => s.emcy.entries);
  const [nodeFilter, setNodeFilter] = useState('');
  const [severityFilter, setSeverityFilter] = useState<'all' | 'critical' | 'warning' | 'info'>('all');
  const [expandedIdx, setExpandedIdx] = useState<number | null>(null);

  // Per-node EMCY statistics
  const nodeStats = useMemo(() => {
    const stats: Record<number, number> = {};
    for (const e of entries) {
      stats[e.node_id] = (stats[e.node_id] || 0) + 1;
    }
    return stats;
  }, [entries]);

  // Unique nodes for filter
  const uniqueNodes = useMemo(() => {
    const ids = new Set<number>();
    for (const e of entries) ids.add(e.node_id);
    return Array.from(ids).sort((a, b) => a - b);
  }, [entries]);

  // Apply filters
  const filteredEntries = useMemo(() => {
    let result = entries;

    if (nodeFilter !== '') {
      const nid = parseInt(nodeFilter);
      if (!isNaN(nid)) {
        result = result.filter((e) => e.node_id === nid);
      }
    }

    if (severityFilter !== 'all') {
      result = result.filter((e) => getSeverity(e.error_code) === severityFilter);
    }

    // Newest first
    return result.slice().reverse();
  }, [entries, nodeFilter, severityFilter]);

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">EMCY Monitor</h2>
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">
            {filteredEntries.length} / {entries.length} entries
          </span>
          <button
            onClick={() => useAppStore.getState().emcy.clearEntries()}
            className="px-2 py-1 text-xs rounded border border-red-500/30 text-red-400 hover:bg-red-500/10"
          >
            Clear
          </button>
        </div>
      </div>

      {/* Summary stats */}
      {Object.keys(nodeStats).length > 0 && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          <div className="p-3 border rounded-lg bg-card">
            <div className="text-xs text-muted-foreground">Total EMCY</div>
            <div className="text-xl font-bold font-mono mt-1">{entries.length}</div>
          </div>
          <div className="p-3 border rounded-lg bg-card">
            <div className="text-xs text-muted-foreground">Affected Nodes</div>
            <div className="text-xl font-bold font-mono mt-1">{Object.keys(nodeStats).length}</div>
          </div>
          <div className="p-3 border rounded-lg bg-card">
            <div className="text-xs text-muted-foreground">Critical</div>
            <div className="text-xl font-bold font-mono mt-1 text-red-400">
              {entries.filter((e) => getSeverity(e.error_code) === 'critical').length}
            </div>
          </div>
          <div className="p-3 border rounded-lg bg-card">
            <div className="text-xs text-muted-foreground">Most Affected</div>
            <div className="text-lg font-bold mt-1">
              {Object.entries(nodeStats).sort((a, b) => b[1] - a[1])[0]
                ? `Node ${Object.entries(nodeStats).sort((a, b) => b[1] - a[1])[0][0]}`
                : '—'}
            </div>
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="flex items-center gap-2">
        <select
          value={nodeFilter}
          onChange={(e) => setNodeFilter(e.target.value)}
          className="px-2 py-1 text-xs rounded border border-border bg-card"
        >
          <option value="">All Nodes</option>
          {uniqueNodes.map((nid) => (
            <option key={nid} value={nid}>Node {nid}</option>
          ))}
        </select>
        <select
          value={severityFilter}
          onChange={(e) => setSeverityFilter(e.target.value as 'all' | 'critical' | 'warning' | 'info')}
          className="px-2 py-1 text-xs rounded border border-border bg-card"
        >
          <option value="all">All Severity</option>
          <option value="critical">Critical</option>
          <option value="warning">Warning</option>
          <option value="info">Info</option>
        </select>
      </div>

      {/* EMCY entries */}
      {filteredEntries.length === 0 ? (
        <div className="bg-card border border-border rounded-md p-6 text-center">
          <AlertTriangle className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
          <p className="text-sm text-muted-foreground">
            {entries.length === 0
              ? 'No emergency messages yet'
              : 'No entries match the current filter'}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            EMCY frames will appear here when devices report errors
          </p>
        </div>
      ) : (
        <div className="space-y-1">
          {filteredEntries.map((entry, i) => {
            const severity = getSeverity(entry.error_code);
            const description = getErrorDescription(entry.error_code);
            const regBits = decodeErrorRegister(entry.error_register);
            const rowBg = severity === 'critical'
              ? 'bg-red-500/5 hover:bg-red-500/10'
              : severity === 'warning'
                ? 'bg-orange-500/5 hover:bg-orange-500/10'
                : 'bg-card hover:bg-muted/50';
            return (
              <div key={i} className={`rounded-md border overflow-hidden transition-colors ${rowBg}`}>
                {/* Main row */}
                <div
                  className="flex items-center gap-2 px-3 py-1.5 text-xs font-mono cursor-pointer"
                  onClick={() => setExpandedIdx(expandedIdx === i ? null : i)}
                >
                  <span className="w-12 font-semibold">{entry.node_id}</span>
                  <span className={`w-20 ${severity === 'critical' ? 'text-red-500 font-bold' : severity === 'warning' ? 'text-orange-500' : 'text-muted-foreground'}`}>
                    0x{entry.error_code.toString(16).padStart(4, '0').toUpperCase()}
                  </span>
                  <span className="w-16 text-muted-foreground">
                    0x{entry.error_register.toString(16).padStart(2, '0').toUpperCase()}
                  </span>
                  <span className="flex-1 truncate">
                    {entry.data.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ')}
                  </span>
                  <span className="w-20 text-muted-foreground text-[10px]">
                    {formatRelativeTime(entry.timestamp_ms)}
                  </span>
                  <span className="w-20 text-muted-foreground text-[10px]">
                    {formatTimestamp(entry.timestamp_ms)}
                  </span>
                </div>
                {/* Expanded detail */}
                {expandedIdx === i && (
                  <div className="px-3 py-2 bg-muted/50 border-t text-xs space-y-1">
                    <div className="flex items-start gap-2">
                      <span className="text-muted-foreground w-24 shrink-0">Description</span>
                      <span className="text-foreground">{description}</span>
                    </div>
                    <div className="flex items-start gap-2">
                      <span className="text-muted-foreground w-24 shrink-0">Severity</span>
                      <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium ${
                        severity === 'critical' ? 'bg-red-500/20 text-red-400' :
                        severity === 'warning' ? 'bg-orange-500/20 text-orange-400' :
                        'bg-blue-500/20 text-blue-400'
                      }`}>
                        {severity.toUpperCase()}
                      </span>
                    </div>
                    {regBits.length > 0 && (
                      <div className="flex items-start gap-2">
                        <span className="text-muted-foreground w-24 shrink-0">Error Reg</span>
                        <div className="flex flex-wrap gap-1">
                          {regBits.map((bit, bi) => (
                            <span key={bi} className="px-1.5 py-0.5 rounded bg-muted text-muted-foreground text-[10px]">
                              {bit}
                            </span>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
