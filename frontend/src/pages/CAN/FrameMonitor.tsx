/**
 * FrameMonitor — Real-time CAN frame monitor with filtering and cycle time.
 *
 * Displays live CAN frames in a virtualized table with:
 * - COB-ID filter (exact or range)
 * - Direction filter (RX/TX/All)
 * - Data content filter (hex substring)
 * - Cycle time column (ms between consecutive frames of same COB-ID)
 * - Frame row click to select (for detail panel)
 * - Pause/resume control
 * - Auto-scroll toggle
 */
import { useState, useRef, useMemo, useEffect } from 'react';
import { useFrames, useAppStore } from '@/lib/store';
import { useVirtualizer } from '@tanstack/react-virtual';
import { Pause, Play, ArrowDown } from 'lucide-react';

export function FrameMonitor() {
  const frames = useFrames();
  const parentRef = useRef<HTMLDivElement>(null);

  const [cobFilter, setCobFilter] = useState('');
  const [dirFilter, setDirFilter] = useState<'all' | 'rx' | 'tx'>('all');
  const [dataFilter, setDataFilter] = useState('');
  const [paused, setPaused] = useState(false);
  const [autoScroll, setAutoScroll] = useState(true);
  const [selectedFrameIdx, setSelectedFrameIdx] = useState<number | null>(null);

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

  // Track original indexes for selection (used by detail panel)
  // Selection is tracked via selectedFrameIdx state

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
        <span className="text-xs text-muted-foreground ml-auto">
          {filteredFrames.length} / {frames.length} frames
        </span>
        <button
          onClick={handleClear}
          className="px-2 py-0.5 text-xs rounded border border-red-500/30 text-red-400 hover:bg-red-500/10"
        >
          Clear
        </button>
      </div>

      {/* Table header */}
      <div className="flex items-center gap-2 px-3 py-1 bg-muted text-xs font-medium border-b shrink-0">
        <span className="w-16">Time</span>
        <span className="w-20">COB-ID</span>
        <span className="w-8">Dir</span>
        <span className="w-8">DLC</span>
        <span className="w-16 text-muted-foreground">Cycle</span>
        <span className="flex-1">Data</span>
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
      </div>

      {/* Virtualized rows */}
      <div ref={parentRef} className="flex-1 overflow-auto">
        {paused && (
          <div className="sticky top-0 z-10 px-3 py-0.5 text-xs bg-yellow-500/10 text-yellow-500 border-b">
            Paused — {filteredFrames.length} frames frozen
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
                <span className="w-20">0x{frame.cob_id.toString(16).padStart(3, '0').toUpperCase()}</span>
                <span className={`w-8 ${frame.direction === 'tx' ? 'text-blue-500' : 'text-green-500'}`}>
                  {frame.direction.toUpperCase()}
                </span>
                <span className="w-8">{frame.dlc}</span>
                <span className="w-16 text-muted-foreground">
                  {frame.cycleTime !== null ? `${frame.cycleTime}ms` : '—'}
                </span>
                <span className="flex-1 truncate">
                  {frame.data.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ')}
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
        {selectedFrameIdx !== null && filteredFrames[selectedFrameIdx] && (
          <span className="font-mono">
            Selected: COB-ID 0x{filteredFrames[selectedFrameIdx].cob_id.toString(16).toUpperCase()}
          </span>
        )}
      </div>
    </div>
  );
}

function formatTimestamp(ms: number): string {
  const sec = Math.floor(ms / 1000);
  const millis = ms % 1000;
  return `${sec}.${millis.toString().padStart(3, '0')}`;
}
