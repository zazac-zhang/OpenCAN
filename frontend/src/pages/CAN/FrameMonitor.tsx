/**
 * FrameMonitor — Real-time CAN frame monitor with filtering, cycle time, and frame type decoding.
 *
 * Displays live CAN frames in a virtualized table with:
 * - Frame type auto-decoding (SYNC/HB/TPDO/RPDO/SDO/NMT/EMCY) with color-coded labels
 * - COB-ID filter (exact or range)
 * - Direction filter (RX/TX/All)
 * - Data content filter (hex substring)
 * - Cycle time column (ms between consecutive frames of same COB-ID)
 * - Frame row click to select (for detail panel)
 * - Pause/resume control
 * - Auto-scroll toggle
 * - Enhanced protocol decoding (SDO commands, EMCY errors, NMT commands)
 */
import { useState, useRef, useMemo, useEffect } from 'react';
import { useFrames, useAppStore } from '@/lib/store';
import { useVirtualizer } from '@tanstack/react-virtual';
import { Pause, Play, ArrowDown, Download, Bookmark, BookmarkCheck } from 'lucide-react';
import { decodeSdoCommand, decodeFunctionCode, decodeEmcyErrorCode } from '@/lib/protocol-decoder';

function decodeFrameType(cobId: number) {
  if (cobId === 0x080) return { label: 'SYNC', color: 'text-purple-400', bgColor: 'bg-purple-500/10' };
  if (cobId >= 0x700 && cobId <= 0x77F) return { label: 'HB', color: 'text-green-400', bgColor: 'bg-green-500/10' };
  if (cobId >= 0x180 && cobId <= 0x1FF) return { label: 'TPDO1', color: 'text-blue-400', bgColor: 'bg-blue-500/10' };
  if (cobId >= 0x280 && cobId <= 0x2FF) return { label: 'TPDO2', color: 'text-blue-300', bgColor: 'bg-blue-500/10' };
  if (cobId >= 0x380 && cobId <= 0x3FF) return { label: 'TPDO3', color: 'text-blue-200', bgColor: 'bg-blue-500/10' };
  if (cobId >= 0x480 && cobId <= 0x4FF) return { label: 'TPDO4', color: 'text-blue-100', bgColor: 'bg-blue-500/10' };
  if (cobId >= 0x200 && cobId <= 0x27F) return { label: 'RPDO1', color: 'text-orange-400', bgColor: 'bg-orange-500/10' };
  if (cobId >= 0x300 && cobId <= 0x37F) return { label: 'RPDO2', color: 'text-orange-300', bgColor: 'bg-orange-500/10' };
  if (cobId >= 0x400 && cobId <= 0x47F) return { label: 'RPDO3', color: 'text-orange-200', bgColor: 'bg-orange-500/10' };
  if (cobId >= 0x500 && cobId <= 0x57F) return { label: 'RPDO4', color: 'text-orange-100', bgColor: 'bg-orange-500/10' };
  if ((cobId >= 0x580 && cobId <= 0x5FF) || (cobId >= 0x600 && cobId <= 0x67F)) {
    return { label: 'SDO', color: 'text-yellow-400', bgColor: 'bg-yellow-500/10' };
  }
  if (cobId >= 0x081 && cobId <= 0x0FF) return { label: 'NMT', color: 'text-pink-400', bgColor: 'bg-pink-500/10' };
  if (cobId >= 0x80 && cobId <= 0x7F) return { label: 'NMT', color: 'text-pink-400', bgColor: 'bg-pink-500/10' };
  if (cobId >= 0x81 && cobId <= 0xBF) return { label: 'EMCY', color: 'text-red-400', bgColor: 'bg-red-500/10' };
  return { label: '—', color: 'text-muted-foreground', bgColor: '' };
}

export function FrameMonitor() {
  const frames = useFrames();
  const parentRef = useRef<HTMLDivElement>(null);

  const [cobFilter, setCobFilter] = useState('');
  const [dirFilter, setDirFilter] = useState<'all' | 'rx' | 'tx'>('all');
  const [dataFilter, setDataFilter] = useState('');
  const [paused, setPaused] = useState(false);
  const [autoScroll, setAutoScroll] = useState(true);
  const [selectedFrameIdx, setSelectedFrameIdx] = useState<number | null>(null);
  const [presetName, setPresetName] = useState('');
  const [showPresets, setShowPresets] = useState(false);

  // Filter presets (localStorage)
  interface FilterPreset { name: string; cob: string; dir: string; data: string }
  const PRESETS_KEY = 'frame-filter-presets';
  const loadPresets = (): FilterPreset[] => {
    try { return JSON.parse(localStorage.getItem(PRESETS_KEY) || '[]'); } catch { return []; }
  };
  const savePresets = (presets: FilterPreset[]) => localStorage.setItem(PRESETS_KEY, JSON.stringify(presets));
  const handleSavePreset = () => {
    if (!presetName.trim()) return;
    const presets = loadPresets().filter((p) => p.name !== presetName);
    presets.push({ name: presetName, cob: cobFilter, dir: dirFilter, data: dataFilter });
    savePresets(presets);
    setPresetName('');
  };
  const handleLoadPreset = (preset: FilterPreset) => {
    setCobFilter(preset.cob);
    setDirFilter(preset.dir as 'all' | 'rx' | 'tx');
    setDataFilter(preset.data);
  };
  const handleDeletePreset = (name: string) => {
    savePresets(loadPresets().filter((p) => p.name !== name));
  };
  const presets = loadPresets();

  // Cycle time: ms between consecutive frames of the same COB-ID
  const framesWithCycle = useMemo(() => {
    const lastSeen = new Map<number, number>();
    return frames.map((frame) => {
      const prev = lastSeen.get(frame.cob_id);
      const cycle = prev !== undefined ? frame.timestamp_ms - prev : null;
      lastSeen.set(frame.cob_id, frame.timestamp_ms);
      return { ...frame, cycleTime: cycle };
    });
  }, [frames]);

  // Apply filters
  const filteredFrames = useMemo(() => {
    let result = framesWithCycle;

    if (dirFilter !== 'all') {
      result = result.filter((f) => f.direction === dirFilter);
    }

    if (cobFilter !== '') {
      const cobNum = parseInt(cobFilter, 16);
      if (!isNaN(cobNum)) {
        result = result.filter((f) => f.cob_id === cobNum);
      } else {
        // Treat as partial hex match
        const hex = cobFilter.toLowerCase().replace(/^0x/i, '');
        result = result.filter((f) => f.cob_id.toString(16).toLowerCase().includes(hex));
      }
    }

    if (dataFilter !== '') {
      const search = dataFilter.toLowerCase().replace(/\s/g, '');
      result = result.filter((f) => {
        const dataHex = f.data.map((b) => b.toString(16).padStart(2, '0')).join('');
        return dataHex.includes(search);
      });
    }

    return result;
  }, [framesWithCycle, cobFilter, dirFilter, dataFilter]);

  // Export filtered frames as CSV
  const handleExportCsv = () => {
    const header = 'Timestamp_ms,COB-ID,Direction,DLC,Data,Type,FrameType,BRS,ESI\n';
    const rows = filteredFrames.map((f) => {
      const type = decodeFrameType(f.cob_id).label;
      const dataHex = f.data.slice(0, f.dlc).map((b) => b.toString(16).padStart(2, '0')).join(' ');
      const frameType = f.frame_type === 'fd' ? 'FD' : 'Classic';
      const brs = f.brs ? 'BRS' : '';
      const esi = f.esi ? 'ESI' : '';
      return `${f.timestamp_ms.toFixed(3)},0x${f.cob_id.toString(16).toUpperCase().padStart(3, '0')},${f.direction},${f.dlc},"${dataHex}",${type},${frameType},${brs},${esi}`;
    }).join('\n');
    downloadFile(header + rows, 'can_frames.csv', 'text/csv');
  };

  // Export filtered frames as ASC (Vector format)
  const handleExportAsc = () => {
    const baseTime = filteredFrames.length > 0 ? filteredFrames[0].timestamp_ms / 1000 : 0;
    const lines = filteredFrames.map((f) => {
      const time = (f.timestamp_ms / 1000 - baseTime).toFixed(6);
      const id = f.cob_id.toString(16).toUpperCase().padStart(3, '0');
      const dir = f.direction === 'rx' ? 'Rx' : 'Tx';
      const dlc = f.dlc;
      const data = f.data.slice(0, dlc).map((b) => b.toString(16).padStart(2, '0')).join(' ');
      return `${time} ${id} ${dir} d ${dlc} ${data}`;
    });
    const header = `date ${new Date().toISOString().split('T')[0]}\ntimebase absolute\n`;
    downloadFile(header + lines.join('\n') + '\n', 'can_frames.asc', 'text/plain');
  };

  function downloadFile(content: string, filename: string, mimeType: string) {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  const rowVirtualizer = useVirtualizer({
    count: filteredFrames.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 28,
    overscan: 10,
  });

  // Auto-scroll to bottom when new frames arrive
  const totalFramesRef = useRef(frames.length);
  useEffect(() => {
    if (autoScroll && !paused && frames.length !== totalFramesRef.current) {
      totalFramesRef.current = frames.length;
      parentRef.current?.scrollTo({ top: parentRef.current.scrollHeight });
    }
  }, [frames.length, autoScroll, paused]);

  const handleClear = () => {
    useAppStore.getState().frames.clearFrames();
    setSelectedFrameIdx(null);
  };

  // Format helpers
  const formatData = (bytes: number[]) =>
    bytes.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');

  const formatTimestamp = (ms: number) => {
    const sec = Math.floor(ms / 1000);
    const millis = ms % 1000;
    return `${sec}.${millis.toString().padStart(3, '0')}`;
  };

  // Count by type for the status bar
  const typeCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const f of frames) {
      const t = decodeFrameType(f.cob_id).label;
      counts[t] = (counts[t] || 0) + 1;
    }
    return counts;
  }, [frames]);

  return (
    <div className="flex flex-col h-full">
      {/* Filter bar */}
      <div className="flex items-center gap-2 px-3 py-1.5 border-b bg-card shrink-0">
        <input
          type="text"
          placeholder="COB-ID (hex)"
          value={cobFilter}
          onChange={(e) => setCobFilter(e.target.value)}
          className="w-24 px-2 py-0.5 text-xs font-mono rounded border border-border bg-background"
        />
        <select
          value={dirFilter}
          onChange={(e) => setDirFilter(e.target.value as 'all' | 'rx' | 'tx')}
          className="px-2 py-0.5 text-xs rounded border border-border bg-background"
        >
          <option value="all">All Dir</option>
          <option value="rx">RX</option>
          <option value="tx">TX</option>
        </select>
        <input
          type="text"
          placeholder="Data (hex)"
          value={dataFilter}
          onChange={(e) => setDataFilter(e.target.value)}
          className="w-32 px-2 py-0.5 text-xs font-mono rounded border border-border bg-background"
        />
        <div className="flex items-center gap-1">
          <button
            onClick={() => setShowPresets(!showPresets)}
            className="px-2 py-0.5 text-xs rounded border border-border hover:bg-muted"
            title="Filter presets"
          >
            <Bookmark className="h-3 w-3" />
          </button>
        </div>
        <span className="text-xs text-muted-foreground ml-auto">
          {filteredFrames.length} / {frames.length} frames
        </span>
        <button
          onClick={handleClear}
          className="px-2 py-0.5 text-xs rounded border border-red-500/30 text-red-400 hover:bg-red-500/10"
        >
          Clear
        </button>
        <div className="flex items-center gap-1 ml-1">
          <button
            onClick={handleExportCsv}
            disabled={filteredFrames.length === 0}
            className="flex items-center gap-1 px-2 py-0.5 text-xs rounded border border-border hover:bg-muted disabled:opacity-50"
            title="Export filtered frames as CSV"
          >
            <Download className="h-3 w-3" />
            CSV
          </button>
          <button
            onClick={handleExportAsc}
            disabled={filteredFrames.length === 0}
            className="flex items-center gap-1 px-2 py-0.5 text-xs rounded border border-border hover:bg-muted disabled:opacity-50"
            title="Export filtered frames as ASC (Vector format)"
          >
            <Download className="h-3 w-3" />
            ASC
          </button>
        </div>
      </div>

      {/* Controls bar */}
      <div className="flex items-center gap-2 px-3 py-1 border-b bg-card shrink-0">
        <button
          onClick={() => setPaused(!paused)}
          className="flex items-center gap-1 px-2 py-0.5 text-xs rounded border border-border hover:bg-muted"
        >
          {paused ? <Play className="h-3 w-3" /> : <Pause className="h-3 w-3" />}
          {paused ? 'Resume' : 'Pause'}
        </button>
        <button
          onClick={() => setAutoScroll(!autoScroll)}
          className={`flex items-center gap-1 px-2 py-0.5 text-xs rounded border transition-colors ${
            autoScroll
              ? 'border-primary bg-primary/10 text-primary'
              : 'border-border hover:bg-muted'
          }`}
        >
          <ArrowDown className="h-3 w-3" />
          Auto-scroll
        </button>
        <div className="flex-1" />
        {/* Type legend */}
        <div className="flex items-center gap-2 text-[10px]">
          {['SYNC', 'HB', 'TPDO1', 'RPDO1', 'SDO', 'NMT'].map((type) => {
            const sample = frames.find((f) => decodeFrameType(f.cob_id).label === type);
            if (!sample) return null;
            const info = decodeFrameType(sample.cob_id);
            return (
              <span key={type} className={`font-mono ${info.color}`}>
                {type}
              </span>
            );
          })}
        </div>
      </div>

      {/* Filter Presets Panel */}
      {showPresets && (
        <div className="flex items-center gap-2 px-3 py-1.5 border-b bg-card/50 shrink-0">
          <span className="text-xs text-muted-foreground">Presets:</span>
          {presets.length > 0 && presets.map((p) => (
            <div key={p.name} className="flex items-center">
              <button
                onClick={() => handleLoadPreset(p)}
                className="flex items-center gap-1 px-2 py-0.5 text-xs rounded-l border border-border hover:bg-muted"
                title={`COB: ${p.cob || 'any'}, Dir: ${p.dir}, Data: ${p.data || 'any'}`}
              >
                <BookmarkCheck className="h-3 w-3" />
                {p.name}
              </button>
              <button
                onClick={() => handleDeletePreset(p.name)}
                className="px-1 py-0.5 text-xs rounded-r border-y border-r border-border hover:bg-red-500/10 text-red-400"
                title="Delete preset"
              >
                ×
              </button>
            </div>
          ))}
          <input
            type="text"
            placeholder="Preset name"
            value={presetName}
            onChange={(e) => setPresetName(e.target.value)}
            className="w-24 px-2 py-0.5 text-xs rounded border border-border bg-background"
          />
          <button
            onClick={handleSavePreset}
            disabled={!presetName.trim()}
            className="px-2 py-0.5 text-xs rounded bg-primary text-primary-foreground disabled:opacity-50"
          >
            Save
          </button>
        </div>
      )}

      {/* Table header */}
      <div className="flex items-center gap-2 px-3 py-1 bg-muted text-xs font-medium border-b shrink-0">
        <span className="w-16">Time</span>
        <span className="w-16">COB-ID</span>
        <span className="w-14">Type</span>
        <span className="w-8">Dir</span>
        <span className="w-8">DLC</span>
        <span className="w-16 text-muted-foreground">Cycle</span>
        <span className="flex-1">Data</span>
      </div>

      {/* Virtualized rows */}
      <div ref={parentRef} className="flex-1 overflow-auto">
        {paused && (
          <div className="sticky top-0 z-10 px-3 py-0.5 text-xs bg-yellow-500/10 text-yellow-500 border-b">
            Paused — {filteredFrames.length} frames frozen
          </div>
        )}
        {filteredFrames.length === 0 && !paused && (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <p className="text-sm text-muted-foreground">No frames captured</p>
            <p className="text-xs text-muted-foreground">Connect to a CAN bus to start monitoring</p>
          </div>
        )}
        <div
          style={{
            height: `${rowVirtualizer.getTotalSize()}px`,
            position: 'relative',
          }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualRow) => {
            const frame = filteredFrames[virtualRow.index];
            const isSelected = selectedFrameIdx === virtualRow.index;
            const typeInfo = decodeFrameType(frame.cob_id);
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
                className={`flex items-center gap-2 px-3 text-xs font-mono border-b cursor-pointer transition-colors ${
                  isSelected
                    ? 'bg-primary/20 border-primary/30'
                    : 'hover:bg-muted/50'
                }`}
                onClick={() => setSelectedFrameIdx(virtualRow.index)}
              >
                <span className="w-16 text-muted-foreground">
                  {formatTimestamp(frame.timestamp_ms)}
                </span>
                <span className="w-16 font-medium">
                  0x{frame.cob_id.toString(16).padStart(3, '0').toUpperCase()}
                </span>
                <span className={`w-14 ${typeInfo.color}`}>
                  {typeInfo.label}
                </span>
                <span className={`w-8 ${frame.direction === 'tx' ? 'text-blue-400' : 'text-green-400'}`}>
                  {frame.direction.toUpperCase()}
                </span>
                <span className="w-8">
                  {frame.dlc}
                  {frame.frame_type === 'fd' && (
                    <span className="ml-0.5 text-[10px] text-purple-400 font-bold">FD</span>
                  )}
                </span>
                {frame.frame_type === 'fd' && (
                  <span className="w-12 text-[10px]">
                    {frame.brs && <span className="text-yellow-400">BRS</span>}
                    {frame.esi && <span className="text-red-400">ESI</span>}
                  </span>
                )}
                <span className="w-16 text-muted-foreground">
                  {frame.cycleTime !== null ? `${frame.cycleTime}ms` : '—'}
                </span>
                <span className="flex-1 truncate">
                  {formatData(frame.data)}
                </span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Status bar */}
      <div className="px-3 py-1 text-xs text-muted-foreground border-t shrink-0 flex items-center justify-between">
        <span>
          {filteredFrames.length !== frames.length
            ? `Showing ${filteredFrames.length} of ${frames.length} frames`
            : `${frames.length} frames`}
        </span>
        {/* Type distribution */}
        <div className="flex items-center gap-2">
          {Object.entries(typeCounts)
            .filter(([, count]) => count > 0)
            .slice(0, 5)
            .map(([type, count]) => {
              const typeInfo = frames.find((f) => decodeFrameType(f.cob_id).label === type);
              const color = typeInfo ? decodeFrameType(typeInfo.cob_id).color : 'text-muted-foreground';
              return (
                <span key={type} className={`font-mono ${color}`}>
                  {type}: {count}
                </span>
              );
            })}
        </div>
        {selectedFrameIdx !== null && filteredFrames[selectedFrameIdx] && (
          <span className="font-mono">
            Selected: COB-ID 0x{filteredFrames[selectedFrameIdx].cob_id.toString(16).toUpperCase()}
            {' — '}{decodeFunctionCode(filteredFrames[selectedFrameIdx].cob_id).description}
          </span>
        )}
      </div>
      {/* Protocol decode panel */}
      {selectedFrameIdx !== null && filteredFrames[selectedFrameIdx] && (() => {
        const frame = filteredFrames[selectedFrameIdx];
        const typeInfo = decodeFrameType(frame.cob_id);
        
        // Enhanced protocol decode
        let protocolInfo: string | null = null;
        if (frame.data.length >= 4) {
          // Check if this is an SDO frame
          if (typeInfo.label === 'SDO_TX' || typeInfo.label === 'SDO_RX') {
            const sdo = decodeSdoCommand(frame.data);
            protocolInfo = `${sdo.command}`;
            if (sdo.index !== undefined) {
              protocolInfo += ` (Index: 0x${sdo.index.toString(16).padStart(4, '0')}, Sub: 0x${sdo.subindex?.toString(16).padStart(2, '0')})`;
            }
          }
          // Check if this is an EMCY frame
          if (typeInfo.label === 'EMCY') {
            const errorCode = frame.data[0] | (frame.data[1] << 8);
            const errorInfo = decodeEmcyErrorCode(errorCode);
            protocolInfo = `Emergency: ${errorInfo.name} - ${errorInfo.description}`;
          }
        }
        
        return protocolInfo ? (
          <div className="px-3 py-1 text-xs text-muted-foreground border-t bg-muted/30 shrink-0">
            <span className="text-yellow-400">Protocol:</span> {protocolInfo}
          </div>
        ) : null;
      })()}
    </div>
  );
}
