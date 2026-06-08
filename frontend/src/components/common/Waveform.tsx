/**
 * Waveform — A wrapper for `lightweight-charts` (v4) that renders
 * real-time waveform charts with auto-scroll, responsive resize,
 * and dark theme support.
 */

import {
  ColorType,
  createChart,
  type IChartApi,
  type ISeriesApi,
  type UTCTimestamp,
} from 'lightweight-charts';
import { useEffect, useRef } from 'react';
import { cn } from '@/lib/utils';

export interface WaveformPoint {
  time: number;
  value: number;
}

export interface WaveformProps {
  data: WaveformPoint[];
  label: string;
  color?: string;
  height?: number;
  className?: string;
}

export function Waveform({
  data,
  label,
  color = '#3b82f6',
  height = 200,
  className,
}: WaveformProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<'Line'> | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const backgroundColor = isDark ? '#0d1117' : '#ffffff';
    const textColor = isDark ? '#8b949e' : '#6e7681';

    const chart = createChart(containerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: backgroundColor },
        textColor,
      },
      width: containerRef.current.clientWidth,
      height,
      grid: {
        vertLines: { color: isDark ? '#21262d' : '#e8e8e8' },
        horzLines: { color: isDark ? '#21262d' : '#e8e8e8' },
      },
      timeScale: {
        timeVisible: true,
        secondsVisible: true,
        borderColor: isDark ? '#30363d' : '#e0e0e0',
      },
      rightPriceScale: {
        borderColor: isDark ? '#30363d' : '#e0e0e0',
        autoScale: true,
      },
      crosshair: {
        mode: 1,
      },
    });

    const series = chart.addLineSeries({
      color,
      lineWidth: 1,
      priceLineVisible: false,
      lastValueVisible: true,
      crosshairMarkerVisible: true,
    });

    chartRef.current = chart;
    seriesRef.current = series;

    const handleResize = () => {
      if (containerRef.current) {
        chart.applyOptions({ width: containerRef.current.clientWidth });
      }
    };
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      chart.remove();
      chartRef.current = null;
      seriesRef.current = null;
    };
  }, [color, height]);

  // Update data whenever it changes
  useEffect(() => {
    if (!seriesRef.current || data.length === 0) return;

    const chartData = data.map((point) => ({
      time: point.time as UTCTimestamp,
      value: point.value,
    }));

    seriesRef.current.setData(chartData);

    // Auto-scroll to the latest data point
    const latest = chartData[chartData.length - 1];
    if (latest && chartRef.current) {
      chartRef.current.timeScale().scrollToRealTime();
    }
  }, [data]);

  if (data.length === 0) {
    return (
      <div
        className={cn('flex items-center justify-center border rounded bg-card', className)}
        style={{ height }}
      >
        <span className="text-sm text-muted-foreground">No data</span>
      </div>
    );
  }

  return (
    <div className={cn('flex flex-col border rounded bg-card', className)}>
      {label && (
        <div className="flex items-center px-3 py-1 border-b shrink-0">
          <span className="w-2 h-2 rounded-full mr-2" style={{ backgroundColor: color }} />
          <span className="text-xs font-medium">{label}</span>
        </div>
      )}
      <div ref={containerRef} style={{ height: label ? height - 28 : height }} />
    </div>
  );
}
