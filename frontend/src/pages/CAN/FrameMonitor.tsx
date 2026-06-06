// Frame Monitor page

import { useRef } from 'react';
import { useFrames } from '@/lib/store';
import { useVirtualizer } from '@tanstack/react-virtual';

export function FrameMonitor() {
  const frames = useFrames();
  const parentRef = useRef<HTMLDivElement>(null);

  const rowVirtualizer = useVirtualizer({
    count: frames.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 28,
    overscan: 10,
  });

  return (
    <div className="flex flex-col h-full">
      {/* Table header */}
      <div className="flex items-center gap-2 px-3 py-1 bg-muted text-xs font-medium border-b shrink-0">
        <span className="w-16">Time</span>
        <span className="w-20">COB-ID</span>
        <span className="w-8">Dir</span>
        <span className="w-8">DLC</span>
        <span className="flex-1">Data</span>
      </div>

      {/* Virtualized rows */}
      <div ref={parentRef} className="flex-1 overflow-auto">
        <div
          style={{
            height: `${rowVirtualizer.getTotalSize()}px`,
            position: 'relative',
          }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualRow) => {
            const frame = frames[virtualRow.index];
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
                <span className="w-16 text-muted-foreground">
                  {formatTimestamp(frame.timestamp_ms)}
                </span>
                <span className="w-20">0x{frame.cob_id.toString(16).padStart(3, '0').toUpperCase()}</span>
                <span className={`w-8 ${frame.direction === 'tx' ? 'text-blue-500' : 'text-green-500'}`}>
                  {frame.direction.toUpperCase()}
                </span>
                <span className="w-8">{frame.dlc}</span>
                <span className="flex-1 truncate">
                  {frame.data.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ')}
                </span>
              </div>
            );
          })}
        </div>
      </div>

      <div className="px-3 py-1 text-xs text-muted-foreground border-t shrink-0">
        {frames.length} frames
      </div>
    </div>
  );
}

function formatTimestamp(ms: number): string {
  const sec = Math.floor(ms / 1000);
  const millis = ms % 1000;
  return `${sec}.${millis.toString().padStart(3, '0')}`;
}
