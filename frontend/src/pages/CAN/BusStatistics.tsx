/**
 * BusStatistics — CAN bus performance statistics with historical trends.
 *
 * Displays:
 * - Real-time stat cards (Bus Load, Frame Rate, TX Errors, RX Errors, Error Frames)
 * - Historical sparkline charts for bus load and frame rate
 * - Peak/average statistics
 * - Error trend indicators (rising/falling/stable)
 */

import { Minus, TrendingDown, TrendingUp } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { BusStatsCards } from '@/components/can/BusStatsCards';
import { useAppStore, useBusStats } from '@/lib/store';

const MAX_HISTORY = 60; // Keep last 60 samples (1 minute at 1s interval)

interface StatSample {
  timestamp: number;
  busLoad: number;
  frameRate: number;
  txErrors: number;
  rxErrors: number;
  errorFrames: number;
}

function Sparkline({
  data,
  color,
  height = 32,
}: {
  data: number[];
  color: string;
  height?: number;
}) {
  if (data.length < 2) return null;
  const max = Math.max(...data, 1);
  const min = Math.min(...data, 0);
  const range = max - min || 1;
  const width = data.length * 3;

  const points = data
    .map((v, i) => {
      const x = (i / (data.length - 1)) * width;
      const y = height - ((v - min) / range) * (height - 4) - 2;
      return `${x},${y}`;
    })
    .join(' ');

  // Area fill
  const areaPoints = `0,${height} ${points} ${width},${height}`;

  return (
    <svg width={width} height={height} className="w-full">
      <polygon points={areaPoints} fill={color} opacity="0.1" />
      <polyline points={points} fill="none" stroke={color} strokeWidth="1.5" />
    </svg>
  );
}

function TrendIndicator({ current, previous }: { current: number; previous: number }) {
  const diff = current - previous;
  const threshold = Math.max(Math.abs(current) * 0.05, 0.1);
  if (diff > threshold) {
    return <TrendingUp className="h-3 w-3 text-red-400" />;
  }
  if (diff < -threshold) {
    return <TrendingDown className="h-3 w-3 text-green-400" />;
  }
  return <Minus className="h-3 w-3 text-muted-foreground" />;
}

export function BusStatistics() {
  const busStats = useBusStats();
  const [history, setHistory] = useState<StatSample[]>([]);
  const [peakLoad, setPeakLoad] = useState(0);
  const [peakRate, setPeakRate] = useState(0);

  // Record stats every second
  useEffect(() => {
    const sample: StatSample = {
      timestamp: Date.now(),
      busLoad: busStats.bus_load,
      frameRate: busStats.frame_rate,
      txErrors: busStats.tx_errors,
      rxErrors: busStats.rx_errors,
      errorFrames: busStats.error_frame_count,
    };
    setHistory((prev) => {
      const next = [...prev, sample].slice(-MAX_HISTORY);
      return next;
    });
    setPeakLoad((p) => Math.max(p, busStats.bus_load));
    setPeakRate((p) => Math.max(p, busStats.frame_rate));
  }, [
    busStats.bus_load,
    busStats.frame_rate,
    busStats.tx_errors,
    busStats.rx_errors,
    busStats.error_frame_count,
  ]);

  // Compute averages from history
  const averages = useMemo(() => {
    if (history.length < 2) return null;
    const last10 = history.slice(-10);
    return {
      avgLoad: Math.round((last10.reduce((s, h) => s + h.busLoad, 0) / last10.length) * 10) / 10,
      avgRate: Math.round((last10.reduce((s, h) => s + h.frameRate, 0) / last10.length) * 10) / 10,
    };
  }, [history]);

  // Error delta (errors added since last sample)
  const lastSample = history.length > 1 ? history[history.length - 2] : null;
  const newTxErrors = lastSample ? busStats.tx_errors - lastSample.txErrors : 0;
  const newRxErrors = lastSample ? busStats.rx_errors - lastSample.rxErrors : 0;

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Bus Statistics</h2>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span>Peak Load: {peakLoad.toFixed(1)}%</span>
          <span>Peak Rate: {peakRate} fps</span>
        </div>
      </div>

      {/* Stat cards */}
      <BusStatsCards stats={busStats} />

      {/* Historical trends */}
      {history.length > 3 && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Bus Load trend */}
          <div className="p-3 border rounded-lg bg-card space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium">Bus Load Trend</span>
              <div className="flex items-center gap-1">
                <TrendIndicator
                  current={busStats.bus_load}
                  previous={lastSample?.busLoad ?? busStats.bus_load}
                />
                {averages && (
                  <span className="text-[10px] text-muted-foreground">
                    avg: {averages.avgLoad}%
                  </span>
                )}
              </div>
            </div>
            <Sparkline
              data={history.map((h) => h.busLoad)}
              color="hsl(var(--primary))"
              height={40}
            />
          </div>

          {/* Frame Rate trend */}
          <div className="p-3 border rounded-lg bg-card space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium">Frame Rate Trend</span>
              <div className="flex items-center gap-1">
                <TrendIndicator
                  current={busStats.frame_rate}
                  previous={lastSample?.frameRate ?? busStats.frame_rate}
                />
                {averages && (
                  <span className="text-[10px] text-muted-foreground">
                    avg: {averages.avgRate} fps
                  </span>
                )}
              </div>
            </div>
            <Sparkline
              data={history.map((h) => h.frameRate)}
              color="hsl(var(--blue-500, #3b82f6))"
              height={40}
            />
          </div>

          {/* Error counters */}
          <div className="p-3 border rounded-lg bg-card space-y-2">
            <span className="text-xs font-medium">Error Counters</span>
            <div className="flex justify-between text-xs font-mono">
              <div>
                <div className="text-muted-foreground">TX Errors</div>
                <div className="text-lg font-bold">{busStats.tx_errors}</div>
                {newTxErrors > 0 && (
                  <div className="text-[10px] text-red-400">+{newTxErrors} new</div>
                )}
              </div>
              <div>
                <div className="text-muted-foreground">RX Errors</div>
                <div className="text-lg font-bold">{busStats.rx_errors}</div>
                {newRxErrors > 0 && (
                  <div className="text-[10px] text-red-400">+{newRxErrors} new</div>
                )}
              </div>
              <div>
                <div className="text-muted-foreground">Error Frames</div>
                <div className="text-lg font-bold">{busStats.error_frame_count}</div>
              </div>
            </div>
          </div>

          {/* Connection info */}
          <div className="p-3 border rounded-lg bg-card space-y-2">
            <span className="text-xs font-medium">Connection</span>
            {useAppStore.getState().can.backendInfo ? (
              <div className="text-xs font-mono space-y-1">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Backend</span>
                  <span>{useAppStore.getState().can.backendInfo?.backend_type}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Channel</span>
                  <span>{useAppStore.getState().can.backendInfo?.channel}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Bitrate</span>
                  <span>{(useAppStore.getState().can.backendInfo?.bitrate || 0) / 1000} kbps</span>
                </div>
              </div>
            ) : (
              <div className="text-xs text-muted-foreground italic">Not connected</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
