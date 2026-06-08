/**
 * BusStatsCards — Responsive bus statistics dashboard cards.
 *
 * Displays 5 stat cards: Bus Load (with progress bar), Frame Rate,
 * TX Errors, RX Errors, and Error Frames. Color-coded by severity.
 */

import { Activity, AlertTriangle, Gauge } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { BusStats } from '@/types/can';

interface BusStatsCardsProps {
  stats: BusStats;
}

export function BusStatsCards({ stats }: BusStatsCardsProps) {
  const busLoadColor =
    stats.bus_load >= 70 ? 'bg-red-500' : stats.bus_load >= 30 ? 'bg-yellow-500' : 'bg-green-500';

  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3">
      {/* Bus Load */}
      <StatCard
        icon={<Gauge className="w-4 h-4" />}
        label="Bus Load"
        value={`${stats.bus_load.toFixed(1)}`}
        unit="%"
      >
        <div className="mt-2 h-1.5 w-full bg-muted rounded-full overflow-hidden">
          <div
            className={cn('h-full rounded-full transition-all duration-300', busLoadColor)}
            style={{ width: `${Math.min(stats.bus_load, 100)}%` }}
          />
        </div>
      </StatCard>

      {/* Frame Rate */}
      <StatCard
        icon={<Activity className="w-4 h-4" />}
        label="Frame Rate"
        value={stats.frame_rate.toFixed(0)}
        unit="fps"
      />

      {/* TX Errors */}
      <StatCard
        icon={<AlertTriangle className="w-4 h-4" />}
        label="TX Errors"
        value={stats.tx_errors.toString()}
        valueClassName={stats.tx_errors > 0 ? 'text-red-400' : ''}
      />

      {/* RX Errors */}
      <StatCard
        icon={<AlertTriangle className="w-4 h-4" />}
        label="RX Errors"
        value={stats.rx_errors.toString()}
        valueClassName={stats.rx_errors > 0 ? 'text-red-400' : ''}
      />

      {/* Error Frames */}
      <StatCard
        icon={<AlertTriangle className="w-4 h-4" />}
        label="Error Frames"
        value={stats.error_frame_count.toString()}
        valueClassName={stats.error_frame_count > 0 ? 'text-red-400' : ''}
      />
    </div>
  );
}

function StatCard({
  icon,
  label,
  value,
  unit,
  valueClassName,
  children,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  unit?: string;
  valueClassName?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className="p-4 border rounded-lg bg-card">
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        {icon}
        {label}
      </div>
      <div className={cn('text-2xl font-bold mt-2 font-mono', valueClassName)}>
        {value}
        {unit && <span className="text-sm font-normal text-muted-foreground ml-1">{unit}</span>}
      </div>
      {children}
    </div>
  );
}
