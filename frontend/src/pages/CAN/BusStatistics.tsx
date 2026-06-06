// Bus Statistics page

import { useAppStore } from '@/lib/store';

export function BusStatistics() {
  const busStats = useAppStore((s) => s.frames.busStats);

  return (
    <div className="p-4 space-y-4">
      <h2 className="text-lg font-semibold">Bus Statistics</h2>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard label="Bus Load" value={`${busStats.bus_load.toFixed(1)}%`} />
        <StatCard label="Frame Rate" value={`${busStats.frame_rate} fps`} />
        <StatCard label="TX Errors" value={busStats.tx_errors.toString()} />
        <StatCard label="RX Errors" value={busStats.rx_errors.toString()} />
      </div>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="p-4 border rounded-lg bg-card">
      <div className="text-sm text-muted-foreground">{label}</div>
      <div className="text-2xl font-bold mt-1">{value}</div>
    </div>
  );
}
