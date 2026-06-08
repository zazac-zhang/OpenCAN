/**
 * SdoExplorer — Interactive SDO explorer with tree-view OD browser.
 *
 * Features:
 * - Tree-view Object Dictionary browser grouped by index range
 * - Double-click to read OD entry via SDO upload
 * - Write values via SDO download
 * - SDO history with replay
 * - Multi-format value display (Hex/Bin/Dec)
 */

import { useState, useMemo, useCallback } from 'react';
import { ChevronRight, ChevronDown, Search, RefreshCw, History, Trash2, Play, Download } from 'lucide-react';
import { useAppStore } from '@/lib/store';
import { useSdoUpload, useSdoDownload } from '@/hooks/useCommands';
import { cn } from '@/lib/utils';

// ===== Types =====

interface OdEntry {
  index: number;
  subindex: number;
  name: string;
  objectType?: string;
  dataType?: string;
  access?: string;
  value?: string;
}

interface SdoHistoryEntry {
  id: number;
  timestamp: number;
  direction: 'read' | 'write';
  index: number;
  subindex: number;
  value?: number[];
  result?: string;
  success: boolean;
}

// ===== Constants =====

const AREA_LABELS: Record<string, { range: string; label: string; color: string }> = {
  communication: { range: '1000-1FFF', label: 'Communication', color: 'text-blue-400' },
  manufacturer: { range: '2000-5FFF', label: 'Manufacturer', color: 'text-purple-400' },
  device_profile: { range: '6000-9FFF', label: 'Device Profile', color: 'text-green-400' },
  reserved: { range: 'A000-BFFF', label: 'Reserved', color: 'text-gray-400' },
  profile_specific: { range: 'C000-FFFF', label: 'Profile Specific', color: 'text-orange-400' },
};

function getArea(index: number): string {
  if (index >= 0x1000 && index <= 0x1FFF) return 'communication';
  if (index >= 0x2000 && index <= 0x5FFF) return 'manufacturer';
  if (index >= 0x6000 && index <= 0x9FFF) return 'device_profile';
  if (index >= 0xA000 && index <= 0xBFFF) return 'reserved';
  if (index >= 0xC000 && index <= 0xFFFF) return 'profile_specific';
  return 'reserved';
}

function formatHex(value: number, width: number = 4): string {
  return `0x${value.toString(16).toUpperCase().padStart(width, '0')}`;
}

// Common DS301/DS402 OD entries for fallback
const COMMON_ENTRIES: OdEntry[] = [
  { index: 0x1000, subindex: 0, name: 'Device Type', objectType: 'VAR', dataType: 'UNSIGNED32', access: 'RO' },
  { index: 0x1001, subindex: 0, name: 'Error Register', objectType: 'VAR', dataType: 'UNSIGNED8', access: 'RO' },
  { index: 0x1005, subindex: 0, name: 'COB-ID SYNC', objectType: 'VAR', dataType: 'UNSIGNED32', access: 'RW' },
  { index: 0x1006, subindex: 0, name: 'Comm Cycle Period', objectType: 'VAR', dataType: 'UNSIGNED32', access: 'RW' },
  { index: 0x1008, subindex: 0, name: 'Device Name', objectType: 'VAR', dataType: 'VISIBLE_STRING', access: 'RO' },
  { index: 0x1010, subindex: 0, name: 'Store Parameters', objectType: 'RECORD', access: 'RO' },
  { index: 0x1010, subindex: 1, name: 'Save All Params', objectType: 'VAR', dataType: 'UNSIGNED32', access: 'RW' },
  { index: 0x1014, subindex: 0, name: 'COB-ID EMCY', objectType: 'VAR', dataType: 'UNSIGNED32', access: 'RW' },
  { index: 0x1017, subindex: 0, name: 'Producer Heartbeat', objectType: 'VAR', dataType: 'UNSIGNED16', access: 'RW' },
  { index: 0x1018, subindex: 0, name: 'Identity Object', objectType: 'RECORD', access: 'RO' },
  { index: 0x1018, subindex: 1, name: 'Vendor ID', objectType: 'VAR', dataType: 'UNSIGNED32', access: 'RO' },
  { index: 0x1018, subindex: 2, name: 'Product Code', objectType: 'VAR', dataType: 'UNSIGNED32', access: 'RO' },
  { index: 0x1018, subindex: 3, name: 'Revision Number', objectType: 'VAR', dataType: 'UNSIGNED32', access: 'RO' },
  { index: 0x1018, subindex: 4, name: 'Serial Number', objectType: 'VAR', dataType: 'UNSIGNED32', access: 'RO' },
  // DS402
  { index: 0x6040, subindex: 0, name: 'ControlWord', objectType: 'VAR', dataType: 'UNSIGNED16', access: 'RW' },
  { index: 0x6041, subindex: 0, name: 'StatusWord', objectType: 'VAR', dataType: 'UNSIGNED16', access: 'RO' },
  { index: 0x6060, subindex: 0, name: 'Modes of Operation', objectType: 'VAR', dataType: 'INTEGER8', access: 'RW' },
  { index: 0x6061, subindex: 0, name: 'Modes Display', objectType: 'VAR', dataType: 'INTEGER8', access: 'RO' },
  { index: 0x6064, subindex: 0, name: 'Position Actual', objectType: 'VAR', dataType: 'INTEGER32', access: 'RO' },
  { index: 0x606C, subindex: 0, name: 'Velocity Actual', objectType: 'VAR', dataType: 'INTEGER32', access: 'RO' },
  { index: 0x6077, subindex: 0, name: 'Torque Actual', objectType: 'VAR', dataType: 'INTEGER16', access: 'RO' },
  { index: 0x607A, subindex: 0, name: 'Target Position', objectType: 'VAR', dataType: 'INTEGER32', access: 'RW' },
  { index: 0x60FF, subindex: 0, name: 'Target Velocity', objectType: 'VAR', dataType: 'INTEGER32', access: 'RW' },
  { index: 0x6071, subindex: 0, name: 'Target Torque', objectType: 'VAR', dataType: 'INTEGER16', access: 'RW' },
  { index: 0x6098, subindex: 0, name: 'Homing Method', objectType: 'VAR', dataType: 'INTEGER8', access: 'RW' },
];

// ===== Main Component =====

export function SdoExplorer() {
  const selectedNode = useAppStore((s) => s.can.selectedNode) ?? 1;
  const sdoUpload = useSdoUpload();
  const sdoDownload = useSdoDownload();

  // State
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedIndexes, setExpandedIndexes] = useState<Set<number>>(new Set());
  const [selectedEntry, setSelectedEntry] = useState<OdEntry | null>(null);
  const [readValue, setReadValue] = useState<number[] | null>(null);
  const [writeHex, setWriteHex] = useState('');
  const [history, setHistory] = useState<SdoHistoryEntry[]>([]);
  const [historyId, setHistoryId] = useState(0);

  // Use common entries as OD entries
  const odEntries = COMMON_ENTRIES;

  // Group entries by index
  const groupedByIndex = useMemo(() => {
    const groups = new Map<number, OdEntry[]>();
    for (const entry of odEntries) {
      const existing = groups.get(entry.index);
      if (existing) {
        existing.push(entry);
      } else {
        groups.set(entry.index, [entry]);
      }
    }
    return groups;
  }, [odEntries]);

  // Filter indexes by search term
  const filteredIndexes = useMemo(() => {
    const indexes = Array.from(groupedByIndex.keys());
    if (!searchTerm) return indexes;
    const term = searchTerm.toLowerCase();
    return indexes.filter((idx) => {
      if (formatHex(idx).toLowerCase().includes(term)) return true;
      const entries = groupedByIndex.get(idx) || [];
      return entries.some((e) => e.name.toLowerCase().includes(term));
    });
  }, [groupedByIndex, searchTerm]);

  // Toggle index expansion
  const toggleIndex = useCallback((index: number) => {
    setExpandedIndexes((prev) => {
      const next = new Set(prev);
      if (next.has(index)) {
        next.delete(index);
      } else {
        next.add(index);
      }
      return next;
    });
  }, []);

  // Add to history
  const addToHistory = useCallback((entry: Omit<SdoHistoryEntry, 'id' | 'timestamp'>) => {
    setHistoryId((prev) => prev + 1);
    setHistory((prev) => [
      { ...entry, id: historyId + 1, timestamp: Date.now() },
      ...prev.slice(0, 99), // Keep last 100
    ]);
  }, [historyId]);

  // Read entry via SDO upload
  const handleRead = useCallback((entry: OdEntry) => {
    setSelectedEntry(entry);
    setReadValue(null);
    sdoUpload.mutate(
      {
        node_id: selectedNode,
        index: entry.index,
        subindex: entry.subindex,
        data_type: entry.dataType || 'UNS16',
      },
      {
        onSuccess: (result) => {
          const bytes = result.data || [];
          setReadValue(bytes);
          addToHistory({
            direction: 'read',
            index: entry.index,
            subindex: entry.subindex,
            value: bytes,
            result: bytes.map((b: number) => b.toString(16).padStart(2, '0')).join(' '),
            success: true,
          });
        },
        onError: (err) => {
          addToHistory({
            direction: 'read',
            index: entry.index,
            subindex: entry.subindex,
            result: String(err),
            success: false,
          });
        },
      },
    );
  }, [selectedNode, sdoUpload, addToHistory]);

  // Write entry via SDO download
  const handleWrite = useCallback(() => {
    if (!selectedEntry || !writeHex.trim()) return;
    // Parse hex string to bytes
    const hex = writeHex.replace(/\s/g, '').replace(/^0x/i, '');
    const bytes: number[] = [];
    for (let i = 0; i < hex.length; i += 2) {
      bytes.push(parseInt(hex.substring(i, i + 2), 16));
    }
    if (bytes.length === 0) return;

    sdoDownload.mutate(
      {
        node_id: selectedNode,
        index: selectedEntry.index,
        subindex: selectedEntry.subindex,
        data: bytes,
      },
      {
        onSuccess: () => {
          addToHistory({
            direction: 'write',
            index: selectedEntry.index,
            subindex: selectedEntry.subindex,
            value: bytes,
            result: 'OK',
            success: true,
          });
          setWriteHex('');
        },
        onError: (err) => {
          addToHistory({
            direction: 'write',
            index: selectedEntry.index,
            subindex: selectedEntry.subindex,
            value: bytes,
            result: String(err),
            success: false,
          });
        },
      },
    );
  }, [selectedNode, selectedEntry, writeHex, sdoDownload, addToHistory]);

  // Format bytes for display
  const formatBytes = (bytes: number[] | null): string => {
    if (!bytes || bytes.length === 0) return '—';
    return bytes.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');
  };

  // Export history as CSV
  const handleExportHistory = () => {
    const header = 'Time,Direction,Index,SubIndex,Value,Result,Success\n';
    const rows = history.map((h) => {
      const time = new Date(h.timestamp).toISOString();
      const value = h.value?.map((b) => b.toString(16).padStart(2, '0')).join(' ') || '';
      return `${time},${h.direction},${formatHex(h.index)},0x${h.subindex.toString(16).padStart(2, '0')},"${value}","${h.result || ''}",${h.success}`;
    }).join('\n');
    const blob = new Blob([header + rows], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'sdo_history.csv';
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="flex h-full overflow-hidden">
      {/* Left: OD Tree View */}
      <div className="w-1/2 border-r flex flex-col overflow-hidden">
        <div className="px-3 py-2 border-b flex items-center gap-2">
          <Search className="h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search OD entries..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="flex-1 px-2 py-1 text-xs bg-transparent border-none outline-none"
          />
          <span className="text-[10px] text-muted-foreground">
            {filteredIndexes.length} indexes
          </span>
        </div>
        <div className="flex-1 overflow-auto">
          {filteredIndexes.map((index) => {
            const isExpanded = expandedIndexes.has(index);
            const area = getArea(index);
            const areaInfo = AREA_LABELS[area];
            const entries = groupedByIndex.get(index) || [];

            return (
              <div key={index}>
                <button
                  onClick={() => toggleIndex(index)}
                  className="flex items-center gap-1.5 w-full px-3 py-1 text-xs hover:bg-muted/50 text-left"
                >
                  {isExpanded ? (
                    <ChevronDown className="h-3 w-3 text-muted-foreground" />
                  ) : (
                    <ChevronRight className="h-3 w-3 text-muted-foreground" />
                  )}
                  <span className="font-mono text-primary w-16">{formatHex(index)}</span>
                  <span className={cn('text-[10px]', areaInfo?.color)}>{areaInfo?.label}</span>
                  <span className="text-muted-foreground ml-auto">{entries.length} sub</span>
                </button>
                {isExpanded &&
                  entries.map((entry, i) => (
                    <button
                      key={i}
                      onClick={() => handleRead(entry)}
                      className={cn(
                        'flex items-center gap-2 w-full pl-8 pr-3 py-1 text-[11px] text-left transition-colors',
                        selectedEntry === entry
                          ? 'bg-primary/10 text-primary'
                          : 'text-muted-foreground hover:bg-muted/30',
                      )}
                    >
                      <span className="font-mono w-8">
                        .{entry.subindex.toString(16).padStart(2, '0')}
                      </span>
                      <span className="truncate flex-1">{entry.name}</span>
                      <span className="text-[10px] opacity-60">{entry.access}</span>
                      <span className="text-[10px] opacity-60">{entry.dataType}</span>
                    </button>
                  ))}
              </div>
            );
          })}
        </div>
      </div>

      {/* Right: Detail Panel */}
      <div className="w-1/2 flex flex-col overflow-hidden">
        {selectedEntry ? (
          <>
            {/* Entry Info */}
            <div className="px-3 py-2 border-b">
              <div className="flex items-center gap-2">
                <span className="font-mono text-sm text-primary">
                  {formatHex(selectedEntry.index)}:{selectedEntry.subindex.toString(16).padStart(2, '0')}
                </span>
                <span className="text-sm font-medium">{selectedEntry.name}</span>
              </div>
              <div className="flex items-center gap-3 mt-1 text-[10px] text-muted-foreground">
                <span>Type: {selectedEntry.objectType}</span>
                <span>Data: {selectedEntry.dataType}</span>
                <span>Access: {selectedEntry.access}</span>
              </div>
            </div>

            {/* Read Value */}
            <div className="px-3 py-2 border-b space-y-2">
              <div className="flex items-center gap-2">
                <button
                  onClick={() => handleRead(selectedEntry)}
                  disabled={sdoUpload.isPending}
                  className="flex items-center gap-1 px-2 py-1 text-xs rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                  <RefreshCw className={cn('h-3 w-3', sdoUpload.isPending && 'animate-spin')} />
                  Read
                </button>
                <span className="text-xs text-muted-foreground">0x6041:0 StatusWord</span>
              </div>
              {readValue && (
                <div className="bg-muted/30 rounded p-2 font-mono text-xs space-y-1">
                  <div>Hex: {formatBytes(readValue)}</div>
                  {readValue.length <= 4 && (
                    <>
                      <div>
                        Dec: {readValue.reduce((acc, b, i) => acc + (b << (i * 8)), 0 >>> 0)}
                      </div>
                      <div>
                        Bin: {readValue.map((b) => b.toString(2).padStart(8, '0')).join(' ')}
                      </div>
                    </>
                  )}
                </div>
              )}
            </div>

            {/* Write Value */}
            {selectedEntry.access?.includes('W') && (
              <div className="px-3 py-2 border-b space-y-2">
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    placeholder="Hex value (e.g. 0F 00)"
                    value={writeHex}
                    onChange={(e) => setWriteHex(e.target.value)}
                    className="flex-1 px-2 py-1 text-xs font-mono bg-muted/30 rounded border border-border"
                  />
                  <button
                    onClick={handleWrite}
                    disabled={sdoDownload.isPending || !writeHex.trim()}
                    className="flex items-center gap-1 px-2 py-1 text-xs rounded bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50"
                  >
                    Write
                  </button>
                </div>
              </div>
            )}

            {/* SDO History */}
            <div className="flex-1 overflow-hidden flex flex-col">
              <div className="px-3 py-2 border-b flex items-center justify-between">
                <div className="flex items-center gap-1.5">
                  <History className="h-3.5 w-3.5 text-muted-foreground" />
                  <span className="text-xs font-medium">SDO History</span>
                  <span className="text-[10px] text-muted-foreground">({history.length})</span>
                </div>
                <div className="flex items-center gap-1">
                  <button
                    onClick={handleExportHistory}
                    disabled={history.length === 0}
                    className="p-1 rounded hover:bg-muted disabled:opacity-50"
                    title="Export history"
                  >
                    <Download className="h-3 w-3" />
                  </button>
                  <button
                    onClick={() => setHistory([])}
                    disabled={history.length === 0}
                    className="p-1 rounded hover:bg-muted disabled:opacity-50"
                    title="Clear history"
                  >
                    <Trash2 className="h-3 w-3" />
                  </button>
                </div>
              </div>
              <div className="flex-1 overflow-auto">
                {history.length === 0 ? (
                  <div className="p-4 text-center text-xs text-muted-foreground italic">
                    No SDO operations yet. Click an entry to read it.
                  </div>
                ) : (
                  <div className="divide-y divide-border">
                    {history.map((h) => (
                      <div
                        key={h.id}
                        className={cn(
                          'px-3 py-1.5 text-[11px] flex items-center gap-2',
                          !h.success && 'bg-red-500/5',
                        )}
                      >
                        <span className="text-muted-foreground w-16">
                          {new Date(h.timestamp).toLocaleTimeString()}
                        </span>
                        <span
                          className={cn(
                            'w-8 font-medium',
                            h.direction === 'read' ? 'text-blue-400' : 'text-green-400',
                          )}
                        >
                          {h.direction === 'read' ? 'R' : 'W'}
                        </span>
                        <span className="font-mono">
                          {formatHex(h.index)}:{h.subindex.toString(16).padStart(2, '0')}
                        </span>
                        {h.value && (
                          <span className="font-mono text-muted-foreground">
                            {h.value.map((b) => b.toString(16).padStart(2, '0')).join(' ')}
                          </span>
                        )}
                        <span className={cn(
                          'ml-auto',
                          h.success ? 'text-green-400' : 'text-red-400',
                        )}>
                          {h.result}
                        </span>
                        {/* Replay button */}
                        <button
                          onClick={() => {
                            if (h.direction === 'read') {
                              const entry = odEntries.find(
                                (e) => e.index === h.index && e.subindex === h.subindex,
                              );
                              if (entry) handleRead(entry);
                            }
                          }}
                          className="p-0.5 rounded hover:bg-muted"
                          title="Replay"
                        >
                          <Play className="h-3 w-3" />
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-sm text-muted-foreground italic">
            Select an OD entry from the tree to read or write
          </div>
        )}
      </div>
    </div>
  );
}
