/**
 * ErrorFrameList — Table of CAN error frames with severity color-coding.
 *
 * Displays timestamp, error type, TEC/RX error counters with color-coded
 * rows by severity: Bus Off (red), Error Passive (orange), Warning (yellow).
 */

import { Trash2 } from 'lucide-react';
import type { ErrorFrame } from '@/types/can';
import { cn } from '@/lib/utils';

interface ErrorFrameListProps {
  errorFrames: ErrorFrame[];
  onClear: () => void;
}

/**
 * Determine severity level from error type string.
 */
function getSeverity(errorType: string): 'bus-off' | 'passive' | 'warning' | 'normal' {
  const lower = errorType.toLowerCase();
  if (lower.includes('bus off') || lower.includes('busoff')) return 'bus-off';
  if (lower.includes('passive')) return 'passive';
  if (lower.includes('warning')) return 'warning';
  return 'normal';
}

const severityRowClass: Record<string, string> = {
  'bus-off': 'bg-red-500/15 border-l-2 border-l-red-500',
  passive: 'bg-orange-500/15 border-l-2 border-l-orange-500',
  warning: 'bg-yellow-500/15 border-l-2 border-l-yellow-500',
  normal: '',
};

const severityBadgeClass: Record<string, string> = {
  'bus-off': 'bg-red-500/25 text-red-400',
  passive: 'bg-orange-500/25 text-orange-400',
  warning: 'bg-yellow-500/25 text-yellow-400',
  normal: 'bg-muted text-muted-foreground',
};

export function ErrorFrameList({ errorFrames, onClear }: ErrorFrameListProps) {
  const firstTs = errorFrames.length > 0 ? errorFrames[0].timestamp_ms : 0;

  if (errorFrames.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
        <p className="text-sm">No error frames detected</p>
        <p className="text-xs mt-1">Error frames will appear here when CAN bus errors occur</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-2 border-b shrink-0">
        <span className="text-xs text-muted-foreground">
          {errorFrames.length} error frame{errorFrames.length !== 1 ? 's' : ''}
        </span>
        <button
          onClick={onClear}
          className="inline-flex items-center gap-1 px-2 py-1 text-xs rounded border border-red-500/30 bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors"
          title="Clear all error frames"
        >
          <Trash2 className="w-3 h-3" />
          Clear
        </button>
      </div>

      {/* Table header */}
      <div className="flex items-center gap-2 px-3 py-1 bg-muted text-xs font-medium border-b shrink-0">
        <span className="w-24">Timestamp</span>
        <span className="flex-1">Error Type</span>
        <span className="w-20">TEC</span>
        <span className="w-20">REC</span>
      </div>

      {/* Rows */}
      <div className="flex-1 overflow-auto">
        {errorFrames.map((ef, i) => {
          const severity = getSeverity(ef.error_type);
          const relativeMs = ef.timestamp_ms - firstTs;

          return (
            <div
              key={i}
              className={cn(
                'flex items-center gap-2 px-3 py-1.5 text-xs font-mono border-b hover:bg-muted/50',
                severityRowClass[severity],
              )}
            >
              <span className="w-24 text-muted-foreground">
                {formatRelativeMs(relativeMs)}
              </span>
              <span className="flex-1">
                <span
                  className={cn(
                    'inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold',
                    severityBadgeClass[severity],
                  )}
                >
                  {ef.error_type}
                </span>
              </span>
              <span className={cn('w-20', ef.tec > 127 ? 'text-red-400 font-bold' : '')}>
                {ef.tec}
              </span>
              <span className={cn('w-20', ef.rec > 127 ? 'text-red-400 font-bold' : '')}>
                {ef.rec}
              </span>
            </div>
          );
        })}
      </div>

      {/* Footer */}
      <div className="px-3 py-1 text-xs text-muted-foreground border-t shrink-0">
        {errorFrames.length} error frame{errorFrames.length !== 1 ? 's' : ''} total
      </div>
    </div>
  );
}

function formatRelativeMs(ms: number): string {
  const sec = Math.floor(ms / 1000);
  const millis = ms % 1000;
  return `${sec}.${millis.toString().padStart(3, '0')}`;
}
