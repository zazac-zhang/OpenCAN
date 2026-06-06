// Frame stream hook with throttling

import { useEffect, useState } from 'react';
import { onFrameStreamBatch } from '../lib/tauri';
import { useAppStore } from '../lib/store';

export function useFrameStream() {
  const [isListening, setIsListening] = useState(false);

  useEffect(() => {
    if (!isListening) return;

    let cleanup: (() => void) | null = null;

    onFrameStreamBatch((events) => {
      const frames = events.map((e) => ({
        cob_id: e.cob_id,
        data: e.data,
        dlc: e.dlc,
        timestamp_ms: e.timestamp_ms,
        direction: e.direction as 'tx' | 'rx',
      }));
      useAppStore.getState().frames.addFrames(frames);
    }).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      cleanup?.();
    };
  }, [isListening]);

  return {
    startListening: () => setIsListening(true),
    stopListening: () => setIsListening(false),
    isListening,
  };
}
