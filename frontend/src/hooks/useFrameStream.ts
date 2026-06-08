// Frame stream hook with throttling

import { useEffect, useState } from 'react';
import { useAppStore } from '../lib/store';
import { onErrorFrameStream, onFrameStreamBatch } from '../lib/tauri';

export function useFrameStream() {
  const [isListening, setIsListening] = useState(false);

  useEffect(() => {
    if (!isListening) return;

    let cleanup: (() => void) | null = null;

    onFrameStreamBatch((events) => {
      if (useAppStore.getState().ui.paused) return;
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

/** Hook for subscribing to error frame stream events. */
export function useErrorFrameStream() {
  const [isListening, setIsListening] = useState(false);

  useEffect(() => {
    if (!isListening) return;

    let cleanup: (() => void) | null = null;

    onErrorFrameStream((event) => {
      if (useAppStore.getState().ui.paused) return;
      useAppStore.getState().errors.addErrorFrames([event]);
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
