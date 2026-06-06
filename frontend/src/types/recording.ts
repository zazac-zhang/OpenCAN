// Recording types

export interface RecordingMeta {
  path: string;
  frame_count: number;
  duration_ms: number;
  start_time: string;
}

export interface RecordingState {
  isRecording: boolean;
  recordingPath: string | null;
  isPlaying: boolean;
  playbackSpeed: number;
  playbackProgress: number;
  loadedMeta: RecordingMeta | null;
}
