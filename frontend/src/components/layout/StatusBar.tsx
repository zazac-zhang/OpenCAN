// Status bar

import { useAppStore, useConnected, useNodes } from '@/lib/store';

export function StatusBar() {
  const connected = useConnected();
  const nodes = useNodes();
  const busStats = useAppStore((s) => s.frames.busStats);
  const ui = useAppStore((s) => s.ui);

  return (
    <div className="flex items-center gap-3 px-4 py-1 border-t bg-muted/50 h-7 text-xs shrink-0">
      <span>{connected ? '● Connected' : '○ Disconnected'}</span>

      {connected && (
        <>
          <span>Load: {busStats.bus_load.toFixed(1)}%</span>
          <span>Rate: {busStats.frame_rate} fps</span>
          <span>Frames: {busStats.frame_rate > 0 ? '—' : '0'}</span>
        </>
      )}

      <span>Nodes: {nodes.length}</span>

      <div className="flex-1" />

      <span className="truncate max-w-md text-muted-foreground">{ui.statusMessage}</span>
    </div>
  );
}
