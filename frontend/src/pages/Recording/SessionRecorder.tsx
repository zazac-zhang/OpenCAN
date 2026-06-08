/**
 * SessionRecorder — Recording control panel page.
 *
 * Provides start/stop recording with file save dialog, and playback
 * controls (load, play, pause, speed, progress) for session recordings.
 */

import { open, save } from '@tauri-apps/plugin-dialog';
import { Circle, FolderOpen, Play, Square } from 'lucide-react';
import { useEffect, useState } from 'react';
import {
  useLoadRecording,
  useStartPlayback,
  useStartRecording,
  useStopPlayback,
  useStopRecording,
} from '@/hooks/useCommands';
import { useRecording } from '@/hooks/useRecording';

export function SessionRecorder() {
  const recording = useRecording();
  const isRecording = recording.isRecording;
  const isPlaying = recording.isPlaying;
  const playbackSpeed = recording.playbackSpeed;
  const playbackProgress = recording.playbackProgress;
  const loadedMeta = recording.loadedMeta;
  const recordingPath = recording.recordingPath;

  const startRecording = useStartRecording();
  const stopRecording = useStopRecording();
  const loadRecording = useLoadRecording();
  const startPlayback = useStartPlayback();
  const stopPlayback = useStopPlayback();

  const [localSpeed, setLocalSpeed] = useState(playbackSpeed);

  useEffect(() => {
    setLocalSpeed(playbackSpeed);
  }, [playbackSpeed]);

  const handleStartRecording = async () => {
    const path = await save({
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });
    if (!path) return;
    recording.setRecordingPath(path);
    startRecording.mutate(path);
  };

  const handleStopRecording = () => {
    stopRecording.mutate();
  };

  const handleLoadRecording = async () => {
    const path = await open({
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });
    if (!path || typeof path !== 'string') return;
    loadRecording.mutate(path);
  };

  const handleStartPlayback = () => {
    if (!loadedMeta) return;
    startPlayback.mutate({ path: loadedMeta.path, speed: localSpeed });
  };

  const handleStopPlayback = () => {
    stopPlayback.mutate();
  };

  const formatDuration = (ms: number) => {
    const totalSec = Math.floor(ms / 1000);
    const min = Math.floor(totalSec / 60);
    const sec = totalSec % 60;
    return `${min}:${sec.toString().padStart(2, '0')}`;
  };

  const currentTimeMs = loadedMeta ? playbackProgress * loadedMeta.duration_ms : 0;

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      {/* Recording Section */}
      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <h2 className="text-lg font-semibold text-foreground">Recording</h2>
          {isRecording && (
            <span className="relative flex h-3 w-3">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75" />
              <span className="relative inline-flex rounded-full h-3 w-3 bg-red-500" />
            </span>
          )}
        </div>

        <div className="flex gap-2">
          <button
            onClick={handleStartRecording}
            disabled={isRecording || startRecording.isPending}
            className="flex items-center gap-2 px-3 py-2 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
          >
            <Circle className="h-4 w-4 text-red-400 fill-red-400" />
            Start Recording
          </button>
          {isRecording && (
            <button
              onClick={handleStopRecording}
              disabled={stopRecording.isPending}
              className="flex items-center gap-2 px-3 py-2 rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
            >
              <Square className="h-4 w-4" />
              Stop Recording
            </button>
          )}
        </div>

        {isRecording && recordingPath && (
          <div className="text-xs text-muted-foreground font-mono truncate bg-card border border-border rounded-md p-2">
            Recording to: {recordingPath}
          </div>
        )}
      </section>

      {/* Playback Section */}
      <section className="space-y-3">
        <h2 className="text-lg font-semibold text-foreground">Playback</h2>

        <button
          onClick={handleLoadRecording}
          disabled={loadRecording.isPending}
          className="flex items-center gap-2 px-3 py-2 rounded-md bg-card border border-border text-foreground hover:bg-card/80 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
        >
          <FolderOpen className="h-4 w-4" />
          Load Recording
        </button>

        {loadedMeta && (
          <>
            {/* Metadata */}
            <div className="bg-card border border-border rounded-md p-3 space-y-1 text-sm">
              <div className="grid grid-cols-2 gap-1">
                <span className="text-muted-foreground">Frames:</span>
                <span className="text-foreground font-mono">
                  {loadedMeta.frame_count.toLocaleString()}
                </span>
                <span className="text-muted-foreground">Duration:</span>
                <span className="text-foreground font-mono">
                  {formatDuration(loadedMeta.duration_ms)}
                </span>
                <span className="text-muted-foreground">Start:</span>
                <span className="text-foreground font-mono text-xs">{loadedMeta.start_time}</span>
              </div>
            </div>

            {/* Progress Bar */}
            <div className="space-y-1">
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>{formatDuration(currentTimeMs)}</span>
                <span>{formatDuration(loadedMeta.duration_ms)}</span>
              </div>
              <div className="h-2 bg-card border border-border rounded-full overflow-hidden">
                <div
                  className="h-full bg-primary transition-all duration-150"
                  style={{ width: `${playbackProgress * 100}%` }}
                />
              </div>
            </div>

            {/* Speed Control */}
            <div className="flex items-center gap-3">
              <label className="text-xs text-muted-foreground whitespace-nowrap">Speed</label>
              <input
                type="range"
                min={0.1}
                max={2.0}
                step={0.1}
                value={localSpeed}
                onChange={(e) => setLocalSpeed(parseFloat(e.target.value))}
                className="flex-1 accent-primary"
              />
              <span className="text-xs font-mono text-foreground w-10 text-right">
                {localSpeed.toFixed(1)}x
              </span>
            </div>

            {/* Playback Controls */}
            <div className="flex gap-2">
              {!isPlaying ? (
                <button
                  onClick={handleStartPlayback}
                  disabled={startPlayback.isPending}
                  className="flex items-center gap-2 px-3 py-2 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
                >
                  <Play className="h-4 w-4" />
                  Start Playback
                </button>
              ) : (
                <button
                  onClick={handleStopPlayback}
                  disabled={stopPlayback.isPending}
                  className="flex items-center gap-2 px-3 py-2 rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
                >
                  <Square className="h-4 w-4" />
                  Stop Playback
                </button>
              )}
            </div>
          </>
        )}
      </section>
    </div>
  );
}
