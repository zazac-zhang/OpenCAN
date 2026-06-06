// SYNC Management page

import { useState } from 'react';
import { useAppStore } from '@/lib/store';
import { useStartSync, useStopSync } from '@/hooks/useCommands';

export function SyncManagement() {
  const syncStatus = useAppStore((s) => s.sync.status);
  const [period, setPeriod] = useState('1000');
  const startMutation = useStartSync();
  const stopMutation = useStopSync();

  return (
    <div className="p-4 space-y-4">
      <h2 className="text-lg font-semibold">SYNC Management</h2>

      {/* Producer configuration */}
      <div className="p-4 border rounded bg-card space-y-3">
        <p className="text-sm font-medium">SYNC Producer</p>

        <div className="flex items-center gap-2">
          <span className="text-xs">Period (μs):</span>
          <input
            className="px-2 py-1 text-xs border rounded w-24 bg-background"
            value={period}
            onChange={(e) => setPeriod(e.target.value)}
          />
        </div>

        <div className="flex gap-2">
          {!syncStatus.is_producer ? (
            <button
              className="px-3 py-1.5 text-sm bg-green-600 text-white rounded hover:bg-green-700"
              onClick={() => startMutation.mutate(parseInt(period) || 1000)}
            >
              Start Producer
            </button>
          ) : (
            <button
              className="px-3 py-1.5 text-sm bg-red-600 text-white rounded hover:bg-red-700"
              onClick={() => stopMutation.mutate()}
            >
              Stop Producer
            </button>
          )}
        </div>

        {syncStatus.is_producer && (
          <div className="text-xs text-muted-foreground">
            Running at {syncStatus.producer_period_us} μs period
          </div>
        )}
      </div>

      {/* Consumer status */}
      <div className="p-4 border rounded bg-card">
        <p className="text-sm font-medium mb-2">SYNC Consumer</p>
        <p className="text-xs text-muted-foreground">
          SYNC consumption is handled automatically by the protocol stack.
        </p>
      </div>
    </div>
  );
}
