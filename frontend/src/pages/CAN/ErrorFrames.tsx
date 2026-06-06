/**
 * ErrorFrames page — CAN error frame details and error counter information.
 *
 * Displays error frames via ErrorFrameList with filtering by error type,
 * summary statistics, TEC/REC trend visualization, and a "Simulate Error" button.
 */
import { useState, useMemo, useEffect } from 'react';
import { useErrorFrames, useAppStore } from '@/lib/store';
import { ErrorFrameList } from '@/components/can/ErrorFrameList';
import { useErrorFrameStream } from '@/hooks/useFrameStream';
import type { ErrorFrame } from '@/types/can';
import { Activity } from 'lucide-react';

const ERROR_TYPES = ['Bus Off', 'Error Passive', 'Warning', 'Normal'];

let errorCounter = 0;

function generateMockErrorFrame(): ErrorFrame {
  const errorType = ERROR_TYPES[Math.floor(Math.random() * ERROR_TYPES.length)];
  errorCounter++;
  return {
    timestamp_ms: Date.now() - 100000 + errorCounter * 500,
    error_type: errorType,
    tec: errorType === 'Bus Off' ? 255 : errorType === 'Error Passive' ? 128 + Math.floor(Math.random() * 127) : errorType === 'Warning' ? 96 + Math.floor(Math.random() * 32) : Math.floor(Math.random() * 50),
    rec: Math.floor(Math.random() * 200),
  };
}

/** Mini sparkline for TEC/REC trends */
function MiniSparkline({ data, color }: { data: number[]; color: string }) {
  if (data.length < 2) return <span className="text-xs text-muted-foreground">—</span>;
  const max = Math.max(...data, 1);
  const height = 24;
  const width = data.length * 3;
  const points = data.map((v, i) => {
    const x = (i / (data.length - 1)) * width;
    const y = height - (v / max) * (height - 4) - 2;
    return `${x},${y}`;
  }).join(' ');
  return (
    <svg width={width} height={height} className="w-full">
      <polyline points={points} fill="none" stroke={color} strokeWidth="1.5" />
    </svg>
  );
}

export function ErrorFrames() {
  const errorFrames = useErrorFrames();
  const { startListening: startErrorStream } = useErrorFrameStream();
  const [filterType, setFilterType] = useState<string>('All');
  const [showMockControls, setShowMockControls] = useState(!errorFrames.length);

  // TEC/REC trend history
  const [tecHistory, setTecHistory] = useState<number[]>([]);
  const [recHistory, setRecHistory] = useState<number[]>([]);

  // Start listening to error frame stream
  if (errorFrames.length === 0) {
    startErrorStream();
  }

  // Update TEC/REC history from latest error frame
  useEffect(() => {
    if (errorFrames.length > 0) {
      const latest = errorFrames[errorFrames.length - 1];
      setTecHistory((prev) => [...prev.slice(-59), latest.tec]);
      setRecHistory((prev) => [...prev.slice(-59), latest.rec]);
    }
  }, [errorFrames.length]);

  const filteredFrames = useMemo(
    () =>
      filterType === 'All'
        ? errorFrames
        : errorFrames.filter((ef) => ef.error_type === filterType),
    [errorFrames, filterType],
  );

  // Summary statistics
  const totalErrors = errorFrames.length;
  const typeCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const ef of errorFrames) {
      counts[ef.error_type] = (counts[ef.error_type] || 0) + 1;
    }
    return counts;
  }, [errorFrames]);

  const mostCommonType = useMemo(() => {
    let maxCount = 0;
    let maxType = '—';
    for (const [type, count] of Object.entries(typeCounts)) {
      if (count > maxCount) {
        maxCount = count;
        maxType = type;
      }
    }
    return { type: maxType, count: maxCount };
  }, [typeCounts]);

  // Latest TEC/REC values
  const latestTec = errorFrames.length > 0 ? errorFrames[errorFrames.length - 1].tec : 0;
  const latestRec = errorFrames.length > 0 ? errorFrames[errorFrames.length - 1].rec : 0;

  const handleClear = () => {
    useAppStore.getState().errors.clearErrorFrames();
    setTecHistory([]);
    setRecHistory([]);
    errorCounter = 0; // Reset mock counter on clear
  };

  const handleSimulateError = () => {
    useAppStore.getState().errors.addErrorFrames([generateMockErrorFrame()]);
  };

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Error Frames</h2>
        <div className="flex items-center gap-2">
          {errorFrames.length === 0 && (
            <button
              onClick={() => setShowMockControls(!showMockControls)}
              className="px-2 py-1 text-xs rounded border border-border bg-card hover:bg-muted transition-colors"
            >
              Test Mode
            </button>
          )}
          {showMockControls && (
            <button
              onClick={handleSimulateError}
              className="px-2 py-1 text-xs rounded border border-border bg-card hover:bg-muted transition-colors"
            >
              Simulate Error
            </button>
          )}
          <button
            onClick={handleClear}
            disabled={totalErrors === 0}
            className="px-2 py-1 text-xs rounded border border-red-500/30 bg-red-500/10 text-red-400 hover:bg-red-500/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Clear All
          </button>
        </div>
      </div>

      {/* Summary statistics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground">Total Errors</div>
          <div className="text-xl font-bold font-mono mt-1">{totalErrors}</div>
        </div>
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground">Most Common</div>
          <div className="text-lg font-bold mt-1 truncate">{mostCommonType.type}</div>
          <div className="text-xs text-muted-foreground">{mostCommonType.count} occurrences</div>
        </div>
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground">Bus Off Events</div>
          <div className="text-xl font-bold font-mono mt-1 text-red-400">
            {typeCounts['Bus Off'] || 0}
          </div>
        </div>
        <div className="p-3 border rounded-lg bg-card">
          <div className="text-xs text-muted-foreground">Filtered Count</div>
          <div className="text-xl font-bold font-mono mt-1">{filteredFrames.length}</div>
        </div>
      </div>

      {/* TEC/REC trend visualization */}
      {(tecHistory.length > 0 || recHistory.length > 0) && (
        <div className="grid grid-cols-2 gap-3">
          <div className="p-3 border rounded-lg bg-card space-y-1">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium flex items-center gap-1">
                <Activity className="h-3 w-3 text-red-400" /> TEC (Transmit)
              </span>
              <span className="text-lg font-bold font-mono text-red-400">{latestTec}</span>
            </div>
            <MiniSparkline data={tecHistory} color="hsl(0, 84%, 60%)" />
            <div className="text-[10px] text-muted-foreground">
              {latestTec >= 255 ? 'Bus Off' : latestTec >= 128 ? 'Error Passive' : latestTec >= 96 ? 'Warning' : 'Active'}
            </div>
          </div>
          <div className="p-3 border rounded-lg bg-card space-y-1">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium flex items-center gap-1">
                <Activity className="h-3 w-3 text-blue-400" /> REC (Receive)
              </span>
              <span className="text-lg font-bold font-mono text-blue-400">{latestRec}</span>
            </div>
            <MiniSparkline data={recHistory} color="hsl(217, 91%, 60%)" />
            <div className="text-[10px] text-muted-foreground">
              {latestRec >= 128 ? 'Error Passive' : latestRec >= 96 ? 'Warning' : 'Active'}
            </div>
          </div>
        </div>
      )}

      {/* Severity distribution */}
      {totalErrors > 0 && (
        <div className="p-3 border rounded-lg bg-card space-y-2">
          <span className="text-xs font-medium">Error Type Distribution</span>
          <div className="space-y-1">
            {ERROR_TYPES.map((type) => {
              const count = typeCounts[type] || 0;
              const pct = totalErrors > 0 ? (count / totalErrors) * 100 : 0;
              const color = type === 'Bus Off' ? 'bg-red-500' : type === 'Error Passive' ? 'bg-orange-500' : type === 'Warning' ? 'bg-yellow-500' : 'bg-green-500';
              return (
                <div key={type} className="flex items-center gap-2 text-xs">
                  <span className="w-28 truncate">{type}</span>
                  <div className="flex-1 h-3 bg-muted rounded overflow-hidden">
                    <div className={`h-full ${color} transition-all`} style={{ width: `${pct}%` }} />
                  </div>
                  <span className="w-12 text-right font-mono">{count}</span>
                  <span className="w-10 text-right text-muted-foreground">{pct.toFixed(0)}%</span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Filter dropdown */}
      <div className="flex items-center gap-2">
        <label htmlFor="error-filter" className="text-xs text-muted-foreground">
          Filter by type:
        </label>
        <select
          id="error-filter"
          value={filterType}
          onChange={(e) => setFilterType(e.target.value)}
          className="px-2 py-1 text-xs rounded border border-border bg-card text-foreground"
        >
          <option value="All">All</option>
          {ERROR_TYPES.map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </select>
      </div>

      {/* Error frame list */}
      <div className="border rounded-lg bg-card overflow-hidden" style={{ minHeight: '300px' }}>
        {errorFrames.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <p className="text-sm text-muted-foreground mb-2">No error frames</p>
            <p className="text-xs text-muted-foreground">
              Error frames will appear here when the bus reports errors
            </p>
          </div>
        ) : (
          <ErrorFrameList errorFrames={filteredFrames} onClear={handleClear} />
        )}
      </div>
    </div>
  );
}
