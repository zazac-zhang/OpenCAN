/**
 * FrameTable — Virtualized CAN frame table for monitoring.
 *
 * Displays CAN frames with Time, COB-ID, Direction, DLC, Data, and Cycle columns.
 * Supports auto-scroll with pause/resume, frame count footer, and a clear button.
 */

import { useEffect, useRef, useState } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { Pause, Play, Trash2 } from 'lucide-react';
import type { CanFrame } from '@/types/can';
import { cn } from '@/lib/utils';

interface FrameTableProps {
  frames: CanFrame[];
  onClear: () => void;
  autoScroll?: boolean;
}

const ROW_HEIGHT = 28;

export function FrameTable({ frames, onClear, autoScroll = true }: FrameTableProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const [paused, setPaused] = useState(!autoScroll);

  const rowVirtualizer = useVirtualizer({
    count: frames.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 10,
  });

  // Auto-scroll to bottom when new frames arrive
  useEffect(() => {
    if (!paused && frames.length > 0) {
      rowVirtualizer.scrollToIndex(frames.length - 1, { align: 'end' });
    }
  }, [frames.length, paused, rowVirtualizer]);

  // Detect if user manually scrolled away from bottom
  const handleScroll = () => {
    const el = parentRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < ROW_HEIGHT * 2;
    if (!atBottom && !paused) {
      setPaused(true);
    }
    if (atBottom && paused) {
      setPaused(false);
    }
  };

  const firstTs = frames.length > 0 ? frames[0].timestamp_ms : 0;

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-2 border-b shrink-0">
        <span className="text-xs text-muted-foreground">
          {frames.length} frame{frames.length !== 1 ? 's' : ''}
        </span>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setPaused(!paused)}
            className={cn(
              'inline-flex items-center gap-1 px-2 py-1 text-xs rounded border transition-colors',
              paused
                ? 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30 hover:bg-yellow-500/30'
                : 'bg-green-500/20 text-green-400 border-green-500/30 hover:bg-green-500/30',
            )}
            title={paused ? 'Resume auto-scroll' : 'Pause auto-scroll'}
          >
            {paused ? <Play className="w-3 h-3" /> : <Pause className="w-3 h-3" />}
            {paused ? 'Resume' : 'Pause'}
          </button>
          <button
            onClick={onClear}
            className="inline-flex items-center gap-1 px-2 py-1 text-xs rounded border border-red-500/30 bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors"
            title="Clear all frames"
          >
            <Trash2 className="w-3 h-3" />
            Clear
          </button>
        </div>
      </div>

      {/* Table header */}
      <div className="flex items-center gap-2 px-3 py-1 bg-muted text-xs font-medium border-b shrink-0">
        <span className="w-20">Time</span>
        <span className="w-20">COB-ID</span>
        <span className="w-12">Dir</span>
        <span className="w-10">DLC</span>
        <span className="flex-1">Data</span>
        <span className="w-16">Cycle</span>
      </div>

      {/* Virtualized rows */}
      <div ref={parentRef} onScroll={handleScroll} className="flex-1 overflow-auto">
        <div
          style={{
            height: `${rowVirtualizer.getTotalSize()}px`,
            position: 'relative',
          }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualRow) => {
            const frame = frames[virtualRow.index];
            const prevFrame = virtualRow.index > 0 ? frames[virtualRow.index - 1] : null;
            const cycleMs = prevFrame ? frame.timestamp_ms - prevFrame.timestamp_ms : 0;

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
                className="flex items-center gap-2 px-3 text-xs font-mono border-b hover:bg-muted/50"
              >
                <span className="w-20 text-muted-foreground">
                  {formatRelativeMs(frame.timestamp_ms - firstTs)}
                </span>
                <span>0x{frame.cob_id.toString(16).padStart(3, '0').toUpperCase()}</span>
                <span className="w-12">
                  <span
                    className={cn(
                      'inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold',
                      frame.direction === 'tx'
                        ? 'bg-blue-500/20 text-blue-400'
                        : 'bg-green-500/20 text-green-400',
                    )}
                  >
                    {frame.direction.toUpperCase()}
                  </span>
                </span>
                <span className="w-10">{frame.dlc}</span>
                <span className="flex-1 truncate">
                  {frame.data.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ')}
                </span>
                <span className="w-16 text-muted-foreground">
                  {cycleMs > 0 ? `${cycleMs}ms` : '—'}
                </span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Footer */}
      <div className="px-3 py-1 text-xs text-muted-foreground border-t shrink-0">
        {frames.length} frame{frames.length !== 1 ? 's' : ''} captured
      </div>
    </div>
  );
}

function formatRelativeMs(ms: number): string {
  const sec = Math.floor(ms / 1000);
  const millis = ms % 1000;
  return `${sec}.${millis.toString().padStart(3, '0')}`;
}
