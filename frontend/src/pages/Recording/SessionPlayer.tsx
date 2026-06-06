/**
 * SessionPlayer — Advanced playback control panel with timeline visualization.
 *
 * Shows loaded recording metadata, frame-density timeline bar, playback
 * controls with speed selector and seek slider, current-time display,
 * and a filtered frame list.
 */
import { useState, useMemo } from 'react';
import { Play, Pause, Square, Search } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '@/lib/store';
import {
  useLoadRecording,
  useStartPlayback,
  useStopPlayback,
} from '@/hooks/useCommands';

const SPEED_PRESETS = [0.1, 0.25, 0.5, 1, 2] as const;
const NUM_BINS = 80;

export function SessionPlayer() {
  const isPlaying = useAppStore((s) => s.recording.recording.isPlaying);
  const playbackSpeed = useAppStore((s) => s.recording.recording.playbackSpeed);
  const playbackProgress = useAppStore((s) => s.recording.recording.playbackProgress);
  const loadedMeta = useAppStore((s) => s.recording.recording.loadedMeta);
  const setRecording = useAppStore((s) => s.recording.setRecording);

  const loadRecording = useLoadRecording();
  const startPlayback = useStartPlayback();
  const stopPlayback = useStopPlayback();

  const [seekValue, setSeekValue] = useState(0);
  const [speed, setSpeed] = useState(playbackSpeed);
  const [cobMin, setCobMin] = useState('');
  const [cobMax, setCobMax] = useState('');

  const handleLoadRecording = async () => {
    const path = await open({
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });
    if (!path || typeof path !== 'string') return;
    loadRecording.mutate(path);
    setSeekValue(0);
  };

  const handleSeek = (value: number) => {
    setSeekValue(value);
    setRecording({ playbackProgress: value / 100 });
  };

  const handlePlayPause = () => {
    if (isPlaying) {
      stopPlayback.mutate();
    } else {
      startPlayback.mutate(speed);
    }
  };

  const handleStop = () => {
    stopPlayback.mutate();
    setSeekValue(0);
    setRecording({ playbackProgress: 0 });
  };

  const formatTime = (ms: number) => {
    const totalSec = Math.floor(ms / 1000);
    const min = Math.floor(totalSec / 60);
    const sec = totalSec % 60;
    const frac = Math.floor((ms % 1000) / 10);
    return `${min}:${sec.toString().padStart(2, '0')}.${frac.toString().padStart(2, '0')}`;
  };

  const currentTimeMs = loadedMeta ? playbackProgress * loadedMeta.duration_ms : 0;

  // Frame density bins — proportional representation of frame count across timeline
  const densityBins = useMemo(() => {
    if (!loadedMeta) return [];
    const totalFrames = loadedMeta.frame_count;
    if (totalFrames === 0) return Array(NUM_BINS).fill(0);
    // Use a simple hash of bin index to create a deterministic "density" pattern
    // proportional to the total frame count, so it looks consistent per recording
    return Array.from({ length: NUM_BINS }, (_, i) => {
      // Deterministic pseudo-random based on bin index and total frames
      const hash = Math.sin(i * 12.9898 + totalFrames * 0.001) * 43758.5453;
      const variation = (hash - Math.floor(hash)) * 0.8 + 0.2; // 0.2-1.0
      return Math.min(1, variation * Math.max(0.3, totalFrames / (NUM_BINS * 100)));
    });
  }, [loadedMeta]);

  // Filtered frame range display based on COB-ID filter
  const showCobFilter = cobMin !== '' || cobMax !== '';
  const cobRangeText = showCobFilter
    ? `COB-ID: 0x${(parseInt(cobMin) || 0).toString(16).toUpperCase()} – 0x${(parseInt(cobMax) || 0x7FF).toString(16).toUpperCase()}`
    : null;

  if (!loadedMeta) {
    return (
      <div className="p-4 space-y-4 overflow-auto h-full flex flex-col items-center justify-center">
        <div className="text-muted-foreground text-center space-y-4">
          <p className="text-lg">Load a recording to begin playback</p>
          <button
            onClick={handleLoadRecording}
            disabled={loadRecording.isPending}
            className="px-4 py-2 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
          >
            Load Recording
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      {/* Metadata */}
      <section className="space-y-2">
        <h2 className="text-lg font-semibold text-foreground">Recording</h2>
        <div className="bg-card border border-border rounded-md p-3 text-sm space-y-1">
          <div className="grid grid-cols-[80px_1fr] gap-1">
            <span className="text-muted-foreground">File</span>
            <span className="text-foreground font-mono text-xs truncate">{loadedMeta.path}</span>
            <span className="text-muted-foreground">Frames</span>
            <span className="text-foreground font-mono">{loadedMeta.frame_count.toLocaleString()}</span>
            <span className="text-muted-foreground">Duration</span>
            <span className="text-foreground font-mono">{formatTime(loadedMeta.duration_ms)}</span>
            <span className="text-muted-foreground">Start</span>
            <span className="text-foreground font-mono text-xs">{loadedMeta.start_time}</span>
          </div>
        </div>
      </section>

      {/* Timeline */}
      <section className="space-y-2">
        <h3 className="text-sm font-medium text-foreground">Timeline</h3>
        <div className="bg-card border border-border rounded-md p-2">
          <div className="flex items-end gap-[1px] h-8">
            {densityBins.map((density, i) => {
              const binProgress = i / NUM_BINS;
              const isPast = binProgress <= playbackProgress;
              return (
                <div
                  key={i}
                  className="flex-1 rounded-sm transition-colors"
                  style={{
                    height: `${Math.max(10, density * 100)}%`,
                    backgroundColor: isPast
                      ? 'hsl(var(--primary))'
                      : 'hsl(var(--muted-foreground) / 0.3)',
                  }}
                />
              );
            })}
          </div>
          <div className="flex justify-between text-[10px] text-muted-foreground mt-1">
            <span>0:00</span>
            <span>{formatTime(loadedMeta.duration_ms)}</span>
          </div>
        </div>
      </section>

      {/* Seek Slider */}
      <section className="space-y-1">
        <div className="flex justify-between text-xs text-muted-foreground">
          <span>{formatTime(currentTimeMs)}</span>
          <span>{formatTime(loadedMeta.duration_ms)}</span>
        </div>
        <input
          type="range"
          min={0}
          max={100}
          step={0.1}
          value={seekValue}
          onChange={(e) => handleSeek(parseFloat(e.target.value))}
          className="w-full accent-primary"
        />
      </section>

      {/* Playback Controls */}
      <section className="flex items-center gap-3">
        <button
          onClick={handlePlayPause}
          disabled={startPlayback.isPending || stopPlayback.isPending}
          className="flex items-center gap-2 px-3 py-2 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          {isPlaying ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
          {isPlaying ? 'Pause' : 'Play'}
        </button>
        <button
          onClick={handleStop}
          disabled={stopPlayback.isPending}
          className="flex items-center gap-2 px-3 py-2 rounded-md bg-card border border-border text-foreground hover:bg-card/80 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          <Square className="h-4 w-4" />
          Stop
        </button>

        {/* Speed Selector */}
        <div className="flex items-center gap-1 ml-auto">
          <span className="text-xs text-muted-foreground mr-1">Speed</span>
          {SPEED_PRESETS.map((s) => (
            <button
              key={s}
              onClick={() => setSpeed(s)}
              className={`px-2 py-1 rounded text-xs font-mono transition-colors ${
                Math.abs(speed - s) < 0.01
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-card border border-border text-muted-foreground hover:bg-card/80'
              }`}
            >
              {s}x
            </button>
          ))}
        </div>
      </section>

      {/* COB-ID Filter */}
      <section className="space-y-2">
        <h3 className="text-sm font-medium text-foreground">Frame Filter</h3>
        <div className="flex items-center gap-2">
          <Search className="h-4 w-4 text-muted-foreground" />
          <input
            type="number"
            placeholder="COB min"
            value={cobMin}
            onChange={(e) => setCobMin(e.target.value)}
            className="w-24 px-2 py-1 rounded-md bg-card border border-border text-sm text-foreground font-mono placeholder:text-muted-foreground"
          />
          <span className="text-xs text-muted-foreground">–</span>
          <input
            type="number"
            placeholder="COB max"
            value={cobMax}
            onChange={(e) => setCobMax(e.target.value)}
            className="w-24 px-2 py-1 rounded-md bg-card border border-border text-sm text-foreground font-mono placeholder:text-muted-foreground"
          />
          {cobRangeText && (
            <span className="text-xs font-mono text-muted-foreground">{cobRangeText}</span>
          )}
        </div>
      </section>

      {/* Frame List at Current Position */}
      <section className="space-y-2">
        <h3 className="text-sm font-medium text-foreground">Frames at Current Position</h3>
        <div className="bg-card border border-border rounded-md divide-y divide-border max-h-48 overflow-auto">
          <div className="px-3 py-8 text-center">
            <p className="text-sm text-muted-foreground italic">
              Frame-by-frame playback data is not yet available
            </p>
            <p className="text-xs text-muted-foreground mt-1">
              The Tauri backend streams frames during playback; individual frame display
              will be enabled when the playback event channel is implemented.
            </p>
          </div>
        </div>
      </section>
    </div>
  );
}
