// Recording state hook

import { useAppStore } from '../lib/store';

export function useRecording() {
  const recording = useAppStore((s) => s.recording.recording);
  const setRecording = useAppStore((s) => s.recording.setRecording);

  return {
    ...recording,
    setIsRecording: (v: boolean) => setRecording({ isRecording: v }),
    setRecordingPath: (v: string | null) => setRecording({ recordingPath: v }),
    setIsPlaying: (v: boolean) => setRecording({ isPlaying: v }),
    setPlaybackSpeed: (v: number) => setRecording({ playbackSpeed: v }),
    setPlaybackProgress: (v: number) => setRecording({ playbackProgress: v }),
    setLoadedMeta: (v: typeof recording.loadedMeta) => setRecording({ loadedMeta: v }),
  };
}
