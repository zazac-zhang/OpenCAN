/**
 * PdoMonitor — CANopen PDO monitoring with filtering and mapping display.
 *
 * Displays PDO entries with:
 * - Formatted timestamps (relative and absolute)
 * - Filtering by node ID, PDO type (RPDO/TPDO), and COB-ID
 * - PDO rate statistics (frames per second per node)
 * - PDO mapping display toggle (shows decoded fields when mapping info available)
 * - Virtualized list for high-frequency PDO traffic
 */

import { useVirtualizer } from '@tanstack/react-virtual';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useReadPdoMapping } from '@/hooks/useCommands';
import { useAppStore } from '@/lib/store';

const PDO_TYPES = ['all', 'tpdo', 'rpdo'] as const;

// Standard CANopen PDO COB-ID base values
const PDO_COB_BASE: Record<string, number[]> = {
  TPDO1: [0x180, 0x200, 0x280, 0x300, 0x380, 0x400, 0x480, 0x500],
  TPDO2: [0x200, 0x280, 0x300, 0x380, 0x400, 0x480, 0x500, 0x580],
  TPDO3: [0x280, 0x300, 0x380, 0x400, 0x480, 0x500, 0x580, 0x600],
  TPDO4: [0x300, 0x380, 0x400, 0x480, 0x500, 0x580, 0x600, 0x680],
  RPDO1: [0x140, 0x1c0, 0x240, 0x2c0, 0x340, 0x3c0, 0x440, 0x4c0],
  RPDO2: [0x1c0, 0x240, 0x2c0, 0x340, 0x3c0, 0x440, 0x4c0, 0x540],
  RPDO3: [0x240, 0x2c0, 0x340, 0x3c0, 0x440, 0x4c0, 0x540, 0x5c0],
  RPDO4: [0x2c0, 0x340, 0x3c0, 0x440, 0x4c0, 0x540, 0x5c0, 0x640],
};

function inferPdoLabel(cobId: number): string {
  for (const [label, bases] of Object.entries(PDO_COB_BASE)) {
    // Check if COB-ID matches node 1 (base) or calculate node offset
    for (let nodeId = 1; nodeId <= 127; nodeId++) {
      const expectedBase = bases[0] + (nodeId - 1) * 0x80;
      if (cobId === expectedBase) return `${label} (Node ${nodeId})`;
    }
  }
  return '';
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

export function PdoMonitor() {
  const entries = useAppStore((s) => s.pdo.entries);
  const parentRef = useRef<HTMLDivElement>(null);

  const [nodeFilter, setNodeFilter] = useState('');
  const [typeFilter, setTypeFilter] = useState<'all' | 'tpdo' | 'rpdo'>('all');
  const [cobFilter, setCobFilter] = useState('');
  const [showMapping, setShowMapping] = useState(false);
  const [pdoMappings, setPdoMappings] = useState<
    Record<
      string,
      { cob_id: number; entries: { index: number; subindex: number; bit_length: number }[] }
    >
  >({});
  const readPdoMapping = useReadPdoMapping();

  // Unique node IDs for filter dropdown
  const uniqueNodes = useMemo(() => {
    const ids = new Set<number>();
    for (const e of entries) ids.add(e.node_id);
    return Array.from(ids).sort((a, b) => a - b);
  }, [entries]);

  // Fetch PDO mappings when showMapping is enabled
  useEffect(() => {
    if (!showMapping) return;
    const nodes = uniqueNodes;
    for (const node of nodes) {
      for (let pdoIdx = 1; pdoIdx <= 4; pdoIdx++) {
        const key = `${node}-${pdoIdx}`;
        if (!pdoMappings[key]) {
          readPdoMapping.mutate(
            { nodeId: node, pdoIndex: pdoIdx },
            {
              onSuccess: (data) => {
                if (data) {
                  setPdoMappings((prev) => ({ ...prev, [key]: data }));
                }
              },
            },
          );
        }
      }
    }
  }, [showMapping, uniqueNodes, readPdoMapping.mutate, pdoMappings]);

  // Calculate PDO rates per node
  const pdoStats = useMemo(() => {
    const stats: Record<number, { count: number; lastTs: number; firstTs: number }> = {};
    for (const entry of entries) {
      const nid = entry.node_id;
      if (!stats[nid]) {
        stats[nid] = { count: 0, lastTs: entry.timestamp_ms, firstTs: entry.timestamp_ms };
      }
      stats[nid].count++;
      stats[nid].lastTs = Math.max(stats[nid].lastTs, entry.timestamp_ms);
      stats[nid].firstTs = Math.min(stats[nid].firstTs, entry.timestamp_ms);
    }
    // Calculate rate (frames/sec)
    const rates: Record<number, number> = {};
    for (const [nid, s] of Object.entries(stats)) {
      const durationSec = (s.lastTs - s.firstTs) / 1000;
      rates[Number(nid)] = durationSec > 0 ? Math.round((s.count / durationSec) * 10) / 10 : 0;
    }
    return { stats, rates };
  }, [entries]);

  // Apply filters
  const filteredEntries = useMemo(() => {
    let result = entries;

    if (nodeFilter !== '') {
      const nid = parseInt(nodeFilter, 10);
      if (!Number.isNaN(nid)) {
        result = result.filter((e) => e.node_id === nid);
      }
    }

    if (typeFilter !== 'all') {
      result = result.filter((e) => e.pdo_type === typeFilter);
    }

    if (cobFilter !== '') {
      const cob = parseInt(cobFilter, 16);
      if (!Number.isNaN(cob)) {
        result = result.filter((e) => e.cob_id === cob);
      }
    }

    // Return newest first, limit to last 500 for performance
    return result.slice().reverse().slice(0, 500);
  }, [entries, nodeFilter, typeFilter, cobFilter]);

  const rowVirtualizer = useVirtualizer({
    count: filteredEntries.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => (showMapping ? 48 : 28),
    overscan: 10,
  });

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">PDO Monitor</h2>
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">
            {filteredEntries.length} / {entries.length} entries
          </span>
          <button
            onClick={() => useAppStore.getState().pdo.clearEntries()}
            className="px-2 py-1 text-xs rounded border border-red-500/30 text-red-400 hover:bg-red-500/10"
          >
            Clear
          </button>
        </div>
      </div>

      {/* PDO rate stats */}
      {Object.keys(pdoStats.rates).length > 0 && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {Object.entries(pdoStats.rates).map(([nid, rate]) => (
            <div key={nid} className="p-3 border rounded-lg bg-card">
              <div className="text-xs text-muted-foreground">Node {nid} Rate</div>
              <div className="text-xl font-bold font-mono mt-1">
                {rate} <span className="text-xs text-muted-foreground">fps</span>
              </div>
              <div className="text-xs text-muted-foreground">
                {pdoStats.stats[Number(nid)]?.count || 0} total
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Filters */}
      <div className="flex items-center gap-2 flex-wrap">
        <select
          value={nodeFilter}
          onChange={(e) => setNodeFilter(e.target.value)}
          className="px-2 py-1 text-xs rounded border border-border bg-card"
        >
          <option value="">All Nodes</option>
          {uniqueNodes.map((nid) => (
            <option key={nid} value={nid}>
              Node {nid}
            </option>
          ))}
        </select>
        <select
          value={typeFilter}
          onChange={(e) => setTypeFilter(e.target.value as 'all' | 'tpdo' | 'rpdo')}
          className="px-2 py-1 text-xs rounded border border-border bg-card"
        >
          {PDO_TYPES.map((t) => (
            <option key={t} value={t}>
              {t === 'all' ? 'All Types' : t.toUpperCase()}
            </option>
          ))}
        </select>
        <input
          type="text"
          placeholder="COB-ID (hex)"
          value={cobFilter}
          onChange={(e) => setCobFilter(e.target.value)}
          className="w-24 px-2 py-1 text-xs font-mono rounded border border-border bg-card"
        />
        <label className="flex items-center gap-1 text-xs ml-auto">
          <input
            type="checkbox"
            checked={showMapping}
            onChange={(e) => setShowMapping(e.target.checked)}
            className="accent-primary"
          />
          Show mapping info
        </label>
      </div>

      {/* PDO entries */}
      {filteredEntries.length === 0 ? (
        <div className="bg-card border border-border rounded-md p-6 text-center">
          <p className="text-sm text-muted-foreground">
            {entries.length === 0 ? 'No PDO data yet' : 'No entries match the current filter'}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            PDO frames will appear here when the protocol stack processes them
          </p>
        </div>
      ) : (
        <div className="border rounded-lg bg-card overflow-hidden" style={{ minHeight: '300px' }}>
          {/* Table header */}
          <div className="flex items-center gap-2 px-3 py-1 bg-muted text-xs font-medium border-b shrink-0">
            <span className="w-12">Node</span>
            <span className="w-16">Type</span>
            <span className="w-16">COB-ID</span>
            <span className="flex-1">Data</span>
            <span className="w-20">Relative</span>
            <span className="w-20">Time</span>
          </div>

          {/* Virtualized rows */}
          <div ref={parentRef} className="overflow-auto" style={{ maxHeight: '500px' }}>
            <div
              style={{
                height: `${rowVirtualizer.getTotalSize()}px`,
                position: 'relative',
              }}
            >
              {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                const entry = filteredEntries[virtualRow.index];
                const pdoLabel = inferPdoLabel(entry.cob_id);
                return (
                  <div
                    key={virtualRow.index}
                    style={{
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      width: '100%',
                      height: `${virtualRow.size}px`,
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                    className="border-b hover:bg-muted/50"
                  >
                    <div className="flex items-center gap-2 px-3 text-xs font-mono">
                      <span className="w-12 font-semibold">{entry.node_id}</span>
                      <span
                        className={`w-16 ${entry.pdo_type === 'tpdo' ? 'text-blue-500' : 'text-orange-500'}`}
                      >
                        {entry.pdo_type.toUpperCase()}
                      </span>
                      <span className="w-16">
                        0x{entry.cob_id.toString(16).padStart(3, '0').toUpperCase()}
                      </span>
                      <span className="flex-1 truncate">
                        {entry.data
                          .map((b) => b.toString(16).padStart(2, '0').toUpperCase())
                          .join(' ')}
                      </span>
                      <span className="w-20 text-muted-foreground text-[10px]">
                        {formatRelativeTime(entry.timestamp_ms)}
                      </span>
                      <span className="w-20 text-muted-foreground text-[10px]">
                        {formatTimestamp(entry.timestamp_ms)}
                      </span>
                    </div>
                    {showMapping && (
                      <div className="px-3 pb-1 text-[10px] text-muted-foreground font-sans">
                        {(() => {
                          // Try to find mapping for this COB-ID
                          for (const [key, mapping] of Object.entries(pdoMappings)) {
                            if (mapping.cob_id === entry.cob_id) {
                              return `PDO${key.split('-')[1]}: ${mapping.entries.map((e) => `0x${e.index.toString(16).toUpperCase()}:${e.subindex.toString(16)} (${e.bit_length}b)`).join(', ')}`;
                            }
                          }
                          return pdoLabel || '';
                        })()}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
