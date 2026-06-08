/**
 * NodeDetail — SDO client with live node state and EDS-aware display.
 *
 * Provides:
 * - SDO upload (read) / download (write) with data type selection
 * - Quick-read shortcuts for common object dictionary indexes
 * - Live node state display (NMT state from heartbeat, StatusWord from DS402)
 * - Value formatting based on data type (signed/unsigned/hex/float/string)
 * - SDO history with filtering and expandable details
 */

import { Activity } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useSdoDownload, useSdoUpload } from '@/hooks/useCommands';
import { useAppStore, useSdoHistory, useSelectedNode } from '@/lib/store';

const DATA_TYPES = [
  { key: 'UNS8', name: 'UNS8', bytes: 1 },
  { key: 'UNS16', name: 'UNS16', bytes: 2 },
  { key: 'UNS32', name: 'UNS32', bytes: 4 },
  { key: 'UNS64', name: 'UNS64', bytes: 8 },
  { key: 'INT8', name: 'INT8', bytes: 1 },
  { key: 'INT16', name: 'INT16', bytes: 2 },
  { key: 'INT32', name: 'INT32', bytes: 4 },
  { key: 'INT64', name: 'INT64', bytes: 8 },
  { key: 'BOOLEAN', name: 'BOOLEAN', bytes: 1 },
  { key: 'REAL32', name: 'REAL32', bytes: 4 },
  { key: 'REAL64', name: 'REAL64', bytes: 8 },
  { key: 'VISIBLE_STRING', name: 'VISIBLE_STRING', bytes: null as null },
  { key: 'OCTET_STRING', name: 'OCTET_STRING', bytes: null as null },
  { key: 'DOMAIN', name: 'DOMAIN', bytes: null as null },
];

type DataType = (typeof DATA_TYPES)[number];

const QUICK_READS = [
  { label: 'Device Type', index: 0x1000, subindex: 0, type: 'UNS32' },
  { label: 'Error Reg', index: 0x1001, subindex: 0, type: 'UNS8' },
  { label: 'Vendor ID', index: 0x1018, subindex: 1, type: 'UNS32' },
  { label: 'Mfr Name', index: 0x1008, subindex: 0, type: 'VISIBLE_STRING' },
  { label: 'HW Version', index: 0x1009, subindex: 0, type: 'VISIBLE_STRING' },
  { label: 'SW Version', index: 0x100a, subindex: 0, type: 'VISIBLE_STRING' },
  { label: 'Heartbeat', index: 0x1017, subindex: 0, type: 'UNS16' },
  { label: 'Status Word', index: 0x6041, subindex: 0, type: 'UNS16' },
  { label: 'ControlWord', index: 0x6040, subindex: 0, type: 'UNS16' },
  { label: 'Actual Pos', index: 0x6064, subindex: 0, type: 'INTEGER32' },
  { label: 'Actual Vel', index: 0x606c, subindex: 0, type: 'INTEGER32' },
  { label: 'Actual Torque', index: 0x6077, subindex: 0, type: 'INTEGER16' },
  { label: 'Mode', index: 0x6060, subindex: 0, type: 'INTEGER8' },
];

// Known object names for common indexes
const OBJECT_NAMES: Record<number, string> = {
  4096: 'Device Type',
  4097: 'Error Register',
  4099: 'Predefined Error Field',
  4101: 'COB-ID SYNC Message',
  4102: 'Communication Cycle Period',
  4104: 'Manufacturer Device Name',
  4105: 'Manufacturer Hardware Version',
  4106: 'Manufacturer Software Version',
  4112: 'Store Parameters',
  4113: 'Restore Default Parameters',
  4116: 'COB-ID EMCY Message',
  4117: 'Inhibit Time EMCY',
  4118: 'Consumer Heartbeat Time',
  4119: 'Producer Heartbeat Time',
  4120: 'Identity Object',
  4121: 'Synchronous Counter Overflow Value',
  24640: 'ControlWord',
  24641: 'StatusWord',
  24672: 'Modes of Operation',
  24673: 'Modes of Operation Display',
  24676: 'Position Actual Value',
  24684: 'Velocity Actual Value',
  24695: 'Torque Actual Value',
  24698: 'Target Position',
  24831: 'Target Velocity',
  24689: 'Target Torque',
  24728: 'Homing Method',
};

function formatValueByType(rawBytes: number[], dataType: string): string {
  if (!rawBytes.length) return '';

  switch (dataType) {
    case 'UNS8':
      return rawBytes[0].toString();
    case 'UNS16': {
      const v = rawBytes[0] | (rawBytes[1] << 8);
      return `${v} (0x${v.toString(16).toUpperCase().padStart(4, '0')})`;
    }
    case 'UNS32': {
      const v =
        (rawBytes[0] | (rawBytes[1] << 8) | (rawBytes[2] << 16) | (rawBytes[3] << 24)) >>> 0;
      return `${v} (0x${v.toString(16).toUpperCase().padStart(8, '0')})`;
    }
    case 'UNS64':
      return rawBytes.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');
    case 'INT8': {
      const v = rawBytes[0] > 127 ? rawBytes[0] - 256 : rawBytes[0];
      return v.toString();
    }
    case 'INT16': {
      const v = rawBytes[0] | (rawBytes[1] << 8);
      return v > 32767 ? (v - 65536).toString() : v.toString();
    }
    case 'INT32': {
      const v = rawBytes[0] | (rawBytes[1] << 8) | (rawBytes[2] << 16) | (rawBytes[3] << 24);
      return `${v}`;
    }
    case 'INT64':
      return rawBytes.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');
    case 'BOOLEAN':
      return rawBytes[0] !== 0 ? 'true' : 'false';
    case 'REAL32': {
      const buf = new ArrayBuffer(4);
      const view = new DataView(buf);
      rawBytes.forEach((b, i) => view.setUint8(i, b));
      return new Float32Array(buf)[0].toString();
    }
    case 'REAL64': {
      const buf = new ArrayBuffer(8);
      const view = new DataView(buf);
      rawBytes.forEach((b, i) => view.setUint8(i, b));
      return new Float64Array(buf)[0].toString();
    }
    case 'VISIBLE_STRING':
      return String.fromCharCode(...rawBytes.filter((b) => b > 0));
    case 'OCTET_STRING':
      return rawBytes.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');
    default:
      return rawBytes.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');
  }
}

function getObjectName(index: number): string {
  return OBJECT_NAMES[index] || '';
}

export function NodeDetail() {
  const selectedNode = useSelectedNode();
  const sdoHistoryList = useSdoHistory();
  const heartbeatEntries = useAppStore((s) => s.heartbeat.entries);
  const nodeId = selectedNode ?? 1;

  const [index, setIndex] = useState('0x1000');
  const [subindex, setSubindex] = useState('0');
  const [value, setValue] = useState('');
  const [dataType, setDataType] = useState<DataType>(DATA_TYPES[2]); // UNS32 default
  const [historyFilter, setHistoryFilter] = useState<'all' | 'read' | 'write'>('all');
  const [expandedHistoryIdx, setExpandedHistoryIdx] = useState<number | null>(null);

  const uploadMutation = useSdoUpload();
  const downloadMutation = useSdoDownload();

  // Live node state from heartbeat
  const heartbeatEntry = heartbeatEntries.find((e) => e.node_id === nodeId);
  const nmtState = heartbeatEntry
    ? heartbeatEntry.alive
      ? 'Operational'
      : 'Not Responding'
    : 'Unknown';

  // Parse current index for object name lookup
  const currentIndexNum = useMemo(() => {
    const parsed = parseInt(index.replace('0x', ''), 16);
    return Number.isNaN(parsed) ? 0 : parsed;
  }, [index]);
  const objectName = getObjectName(currentIndexNum);

  const handleRead = () => {
    const idx = parseInt(index.replace('0x', ''), 16) || 0;
    const sub = parseInt(subindex, 10) || 0;
    uploadMutation.mutate({
      node_id: nodeId,
      index: idx,
      subindex: sub,
      data_type: dataType.key,
    });
  };

  const handleWrite = () => {
    const idx = parseInt(index.replace('0x', ''), 16) || 0;
    const sub = parseInt(subindex, 10) || 0;
    const bytes = value
      .split(' ')
      .filter(Boolean)
      .map((b) => parseInt(b, 16))
      .filter((b) => !Number.isNaN(b));
    downloadMutation.mutate({
      node_id: nodeId,
      index: idx,
      subindex: sub,
      data: bytes,
    });
  };

  const handleQuickRead = (qrIndex: number, qrSubindex: number, qrType: string) => {
    setIndex(`0x${qrIndex.toString(16).toUpperCase().padStart(4, '0')}`);
    setSubindex(qrSubindex.toString());
    const type = DATA_TYPES.find((dt) => dt.key === qrType);
    if (type) setDataType(type);
    uploadMutation.mutate({
      node_id: nodeId,
      index: qrIndex,
      subindex: qrSubindex,
      data_type: qrType,
    });
  };

  // Filtered history
  const filteredHistory = useMemo(() => {
    let result = sdoHistoryList;
    if (historyFilter === 'read') result = result.filter((e) => e.is_read);
    if (historyFilter === 'write') result = result.filter((e) => !e.is_read);
    return result.slice().reverse().slice(0, 100);
  }, [sdoHistoryList, historyFilter]);

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">SDO Client</h2>
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">Node {nodeId}</span>
          <span
            className={`flex items-center gap-1 px-2 py-0.5 text-xs rounded font-medium ${
              nmtState === 'Operational'
                ? 'bg-green-500/20 text-green-500'
                : nmtState === 'Not Responding'
                  ? 'bg-red-500/20 text-red-500'
                  : 'bg-yellow-500/20 text-yellow-500'
            }`}
          >
            <Activity className="h-3 w-3" />
            {nmtState}
          </span>
        </div>
      </div>

      {/* SDO Access */}
      <div className="p-3 border rounded-lg bg-card space-y-3">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">Index:</span>
            <input
              className="px-2 py-1 text-xs font-mono border rounded w-24 bg-background"
              value={index}
              onChange={(e) => setIndex(e.target.value)}
            />
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">Subindex:</span>
            <input
              className="px-2 py-1 text-xs font-mono border rounded w-14 bg-background"
              value={subindex}
              onChange={(e) => setSubindex(e.target.value)}
            />
          </div>
          {objectName && <span className="text-xs text-primary font-medium">{objectName}</span>}
        </div>

        {/* Data type selection */}
        <div>
          <span className="text-xs text-muted-foreground">Data Type:</span>
          <div className="flex flex-wrap gap-1 mt-1">
            {DATA_TYPES.map((dt) => (
              <button
                key={dt.key}
                onClick={() => setDataType(dt)}
                className={`px-2 py-0.5 text-xs rounded font-mono transition-colors ${
                  dataType.key === dt.key
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted hover:bg-muted/80'
                }`}
              >
                {dt.name}
              </button>
            ))}
          </div>
        </div>

        {/* Value input for write */}
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">Value:</span>
          <input
            className="flex-1 px-2 py-1 text-xs font-mono border rounded bg-background"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="hex bytes (e.g. FF 00 01)"
          />
        </div>

        {/* Action buttons */}
        <div className="flex gap-2">
          <button
            className="px-4 py-1.5 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90 disabled:opacity-50"
            onClick={handleRead}
            disabled={uploadMutation.isPending}
          >
            {uploadMutation.isPending ? 'Reading...' : 'Read'}
          </button>
          <button
            className="px-4 py-1.5 text-sm bg-card border border-border rounded hover:bg-muted disabled:opacity-50"
            onClick={handleWrite}
            disabled={downloadMutation.isPending}
          >
            {downloadMutation.isPending ? 'Writing...' : 'Write'}
          </button>
        </div>
      </div>

      {/* Quick read */}
      <div className="p-3 border rounded-lg bg-card space-y-2">
        <p className="text-sm font-medium">Quick Read:</p>
        <div className="flex flex-wrap gap-1">
          {QUICK_READS.map((qr) => (
            <button
              key={`${qr.index}-${qr.subindex}`}
              onClick={() => handleQuickRead(qr.index, qr.subindex, qr.type)}
              className="px-2 py-1 text-xs bg-muted rounded hover:bg-muted/80 font-mono"
              title={`${getObjectName(qr.index)} — 0x${qr.index.toString(16).toUpperCase().padStart(4, '0')}:${qr.subindex}`}
            >
              {qr.label}
            </button>
          ))}
        </div>
      </div>

      {/* SDO History */}
      <div className="p-3 border rounded-lg bg-card space-y-2">
        <div className="flex items-center gap-2">
          <p className="text-sm font-medium">SDO History</p>
          <span className="text-xs text-muted-foreground">({sdoHistoryList.length})</span>
          <div className="ml-auto flex items-center gap-1">
            {(['all', 'read', 'write'] as const).map((f) => (
              <button
                key={f}
                onClick={() => setHistoryFilter(f)}
                className={`px-2 py-0.5 text-xs rounded transition-colors ${
                  historyFilter === f
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted hover:bg-muted/80'
                }`}
              >
                {f.charAt(0).toUpperCase() + f.slice(1)}
              </button>
            ))}
            <button
              className="px-2 py-0.5 text-xs rounded border border-red-500/30 text-red-400 hover:bg-red-500/10 ml-2"
              onClick={() => useAppStore.getState().sdo.clearHistory()}
            >
              Clear
            </button>
          </div>
        </div>

        {filteredHistory.length === 0 ? (
          <p className="text-xs text-muted-foreground">
            {sdoHistoryList.length === 0
              ? 'No SDO operations yet'
              : 'No entries match the current filter'}
          </p>
        ) : (
          <div className="space-y-0.5">
            {/* Header */}
            <div className="flex gap-1 text-[10px] text-muted-foreground font-medium px-1">
              <span className="w-10">Node</span>
              <span className="w-24">Index:Sub</span>
              <span className="w-10">Type</span>
              <span className="w-12">Format</span>
              <span className="flex-1">Value</span>
              <span className="w-12">Result</span>
            </div>
            {filteredHistory.map((entry, i) => {
              const idxNum = entry.index;
              const name = getObjectName(idxNum);
              const formattedValue = entry.value
                ? (() => {
                    // Try to parse raw bytes and format
                    const byteMatch = entry.value.match(/^([0-9a-fA-F]{2}\s*)+$/);
                    if (byteMatch) {
                      const bytes = entry.value
                        .trim()
                        .split(/\s+/)
                        .map((b) => parseInt(b, 16));
                      if (bytes.every((b) => !Number.isNaN(b))) {
                        return formatValueByType(bytes, entry.data_type || 'OCTET_STRING');
                      }
                    }
                    return entry.value;
                  })()
                : '—';
              return (
                <div key={i}>
                  <div
                    className="flex gap-1 text-xs font-mono px-1 py-0.5 rounded cursor-pointer hover:bg-muted/50"
                    onClick={() => setExpandedHistoryIdx(expandedHistoryIdx === i ? null : i)}
                  >
                    <span className="w-10">{entry.node_id}</span>
                    <span className="w-24">
                      {entry.index.toString(16).padStart(4, '0').toUpperCase()}:
                      {entry.subindex.toString(16).padStart(2, '0').toUpperCase()}
                    </span>
                    <span className={`w-10 ${entry.is_read ? 'text-blue-500' : 'text-orange-500'}`}>
                      {entry.is_read ? 'R' : 'W'}
                    </span>
                    <span className="w-12 text-muted-foreground text-[10px]">
                      {entry.data_type || '—'}
                    </span>
                    <span className="flex-1 truncate">{formattedValue}</span>
                    <span
                      className={`w-12 text-[10px] ${entry.success ? 'text-green-500' : 'text-red-500'}`}
                    >
                      {entry.success ? 'OK' : 'ERR'}
                    </span>
                  </div>
                  {expandedHistoryIdx === i && (
                    <div className="px-1 py-1 text-[10px] text-muted-foreground space-y-0.5 bg-muted/30 rounded ml-1">
                      {name && <div>Object: {name}</div>}
                      <div>Raw value: {entry.value || '—'}</div>
                      {entry.error && <div>Error: {entry.error}</div>}
                      {entry.timestamp_ms && (
                        <div>Time: {new Date(entry.timestamp_ms).toLocaleTimeString()}</div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
