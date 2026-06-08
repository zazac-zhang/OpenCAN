// Status bar

import { useAppStore, useConnected, useErrorFrames, useFrames, useNodes } from '@/lib/store';

export function StatusBar() {
  const connected = useConnected();
  const nodes = useNodes();
  const frames = useFrames();
  const busStats = useAppStore((s) => s.frames.busStats);
  const errorFrames = useErrorFrames();
  const ui = useAppStore((s) => s.ui);
  const isRecording = useAppStore((s) => s.recording.recording.isRecording);
  const isPlaying = useAppStore((s) => s.recording.recording.isPlaying);

  // Count RX/TX
  const rxCount = frames.filter((f) => f.direction === 'rx').length;
  const txCount = frames.filter((f) => f.direction === 'tx').length;

  return (
    <div className="flex items-center gap-3 px-4 py-1 border-t bg-muted/50 h-7 text-xs shrink-0">
      <span className={connected ? 'text-green-400' : 'text-muted-foreground'}>
        {connected ? '● Connected' : '○ Disconnected'}
      </span>

      {connected && (
        <>
          <span>Load: {busStats.bus_load.toFixed(1)}%</span>
          <span>Rate: {busStats.frame_rate} fps</span>
          <span>RX: {rxCount.toLocaleString()}</span>
          <span>TX: {txCount.toLocaleString()}</span>
          <span>Frames: {frames.length.toLocaleString()}</span>
          {errorFrames.length > 0 && (
            <span className="text-red-400">Errors: {errorFrames.length}</span>
          )}
        </>
      )}

      <span>Nodes: {nodes.length}</span>

      {isRecording && <span className="text-red-400 animate-pulse">● REC</span>}
      {isPlaying && <span className="text-blue-400">▶ PLAY</span>}

      <div className="flex-1" />

      <span className="truncate max-w-md text-muted-foreground">{ui.statusMessage}</span>
    </div>
  );
}
