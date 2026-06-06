// Bus Statistics page

import { useAppStore } from '@/lib/store';
import { BusStatsCards } from '@/components/can/BusStatsCards';

export function BusStatistics() {
  const busStats = useAppStore((s) => s.frames.busStats);

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <h2 className="text-lg font-semibold">Bus Statistics</h2>
      <BusStatsCards stats={busStats} />
    </div>
  );
}
