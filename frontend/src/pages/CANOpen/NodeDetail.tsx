// Node Detail page (SDO read/write)

import { useState } from 'react';
import { useAppStore, useSelectedNode, useSdoHistory } from '@/lib/store';
import { useSdoUpload, useSdoDownload } from '@/hooks/useCommands';

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
  { label: 'Device Type', index: 0x1000, subindex: 0 },
  { label: 'Error Reg', index: 0x1001, subindex: 0 },
  { label: 'Vendor ID', index: 0x1018, subindex: 1 },
  { label: 'Status Word', index: 0x6041, subindex: 0 },
  { label: 'Actual Pos', index: 0x6064, subindex: 0 },
  { label: 'Actual Vel', index: 0x606C, subindex: 0 },
  { label: 'Actual Torque', index: 0x6077, subindex: 0 },
  { label: 'Mode', index: 0x6060, subindex: 0 },
  { label: 'Heartbeat', index: 0x1017, subindex: 0 },
];

export function NodeDetail() {
  const selectedNode = useSelectedNode();
  const sdoHistory = useSdoHistory();
  const nodeId = selectedNode ?? 1;

  const [index, setIndex] = useState('0x1000');
  const [subindex, setSubindex] = useState('0');
  const [value, setValue] = useState('');
  const [dataType, setDataType] = useState<DataType>(DATA_TYPES[2]); // UNS32 default

  const uploadMutation = useSdoUpload();
  const downloadMutation = useSdoDownload();

  const handleRead = () => {
    const idx = parseInt(index.replace('0x', ''), 16) || 0;
    const sub = parseInt(subindex) || 0;
    uploadMutation.mutate({
      node_id: nodeId,
      index: idx,
      subindex: sub,
      data_type: dataType.key,
    });
  };

  const handleWrite = () => {
    const idx = parseInt(index.replace('0x', ''), 16) || 0;
    const sub = parseInt(subindex) || 0;
    const bytes = value
      .split(' ')
      .filter(Boolean)
      .map((b) => parseInt(b, 16))
      .filter((b) => !isNaN(b));
    downloadMutation.mutate({
      node_id: nodeId,
      index: idx,
      subindex: sub,
      data: bytes,
    });
  };

  const handleQuickRead = (index: number, subindex: number) => {
    setIndex(`0x${index.toString(16).toUpperCase().padStart(4, '0')}`);
    setSubindex(subindex.toString());
    uploadMutation.mutate({
      node_id: nodeId,
      index,
      subindex,
      data_type: dataType.key,
    });
  };

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <h2 className="text-lg font-semibold">SDO Client</h2>

      {/* SDO Access */}
      <div className="space-y-2">
        <p className="text-sm font-medium">SDO Access:</p>

        <div className="flex items-center gap-2">
          <span className="text-xs">Node ID:</span>
          <input
            className="px-2 py-1 text-xs border rounded w-12 bg-background"
            value={nodeId}
            readOnly
          />
        </div>

        <div className="flex items-center gap-2">
          <span className="text-xs">Index:</span>
          <input
            className="px-2 py-1 text-xs border rounded w-20 bg-background"
            value={index}
            onChange={(e) => setIndex(e.target.value)}
          />
          <span className="text-xs">Subindex:</span>
          <input
            className="px-2 py-1 text-xs border rounded w-12 bg-background"
            value={subindex}
            onChange={(e) => setSubindex(e.target.value)}
          />
        </div>

        {/* Data type selection */}
        <div>
          <span className="text-xs">Data Type:</span>
          <div className="flex flex-wrap gap-1 mt-1">
            {DATA_TYPES.map((dt) => (
              <button
                key={dt.key}
                onClick={() => setDataType(dt)}
                className={`px-2 py-0.5 text-xs rounded ${
                  dataType.key === dt.key
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted hover:bg-muted/80'
                }`}
              >
                {dt.name}
              </button>
            ))}
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Selected: {dataType.name} ({dataType.bytes ?? 'variable'} bytes)
          </p>
        </div>

        {/* Value input */}
        <div className="flex items-center gap-2">
          <span className="text-xs">Value:</span>
          <input
            className="flex-1 px-2 py-1 text-xs border rounded bg-background"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="hex bytes (e.g. FF 00 01)"
          />
        </div>

        {/* Action buttons */}
        <div className="flex gap-2">
          <button
            className="px-4 py-1.5 text-sm bg-primary text-primary-foreground rounded"
            onClick={handleRead}
            disabled={uploadMutation.isPending}
          >
            {uploadMutation.isPending ? 'Reading...' : 'Read'}
          </button>
          <button
            className="px-4 py-1.5 text-sm bg-muted rounded"
            onClick={handleWrite}
            disabled={downloadMutation.isPending}
          >
            {downloadMutation.isPending ? 'Writing...' : 'Write'}
          </button>
        </div>
      </div>

      <hr className="border-border" />

      {/* Quick read buttons */}
      <div>
        <p className="text-sm font-medium mb-2">Quick Read:</p>
        <div className="flex flex-wrap gap-1">
          {QUICK_READS.map((qr) => (
            <button
              key={qr.index}
              onClick={() => handleQuickRead(qr.index, qr.subindex)}
              className="px-2 py-1 text-xs bg-muted rounded hover:bg-muted/80"
            >
              {qr.label} ({qr.index.toString(16).toUpperCase().padStart(4, '0')}:{qr.subindex})
            </button>
          ))}
        </div>
      </div>

      <hr className="border-border" />

      {/* SDO History */}
      <div>
        <div className="flex items-center gap-2 mb-2">
          <p className="text-sm font-medium">SDO History:</p>
          <span className="text-xs text-muted-foreground">({sdoHistory.length} entries)</span>
          <button
            className="px-2 py-0.5 text-xs bg-muted rounded"
            onClick={() => useAppStore.getState().sdo.clearHistory()}
          >
            Clear
          </button>
        </div>

        {sdoHistory.length === 0 ? (
          <p className="text-xs text-muted-foreground">No SDO operations yet</p>
        ) : (
          <div className="space-y-0.5">
            {/* Header */}
            <div className="flex gap-1 text-xs text-muted-foreground font-medium">
              <span className="w-10">Node</span>
              <span className="w-20">Index:Sub</span>
              <span className="w-10">Type</span>
              <span className="w-32">Value</span>
              <span className="flex-1">Result</span>
            </div>
            <hr className="border-border" />
            {/* Entries (reversed, latest first) */}
            {sdoHistory
              .slice()
              .reverse()
              .slice(0, 50)
              .map((entry, i) => (
                <div key={i} className="flex gap-1 text-xs font-mono">
                  <span className="w-10">{entry.node_id}</span>
                  <span className="w-20">
                    {entry.index.toString(16).padStart(4, '0').toUpperCase()}:
                    {entry.subindex.toString(16).padStart(2, '0').toUpperCase()}
                  </span>
                  <span className="w-10">{entry.is_read ? 'R' : 'W'}</span>
                  <span className="w-32 truncate">{entry.value || '-'}</span>
                  <span className={`flex-1 ${entry.success ? 'text-green-500' : 'text-red-500'}`}>
                    {entry.success ? 'OK' : entry.error || 'FAIL'}
                  </span>
                </div>
              ))}
          </div>
        )}
      </div>
    </div>
  );
}
