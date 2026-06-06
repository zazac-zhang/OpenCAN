// PDO stream hook

import { useEffect } from 'react';
import { onPdoStreamBatch } from '../lib/tauri';
import { useAppStore } from '../lib/store';

export function usePdoStream() {
  useEffect(() => {
    let cleanup: (() => void) | null = null;

    onPdoStreamBatch((events) => {
      const entries = events.map((e) => ({
        node_id: e.node_id,
        pdo_type: e.pdo_type as 'tpdo' | 'rpdo',
        cob_id: e.cob_id,
        data: e.data,
        timestamp_ms: e.timestamp_ms,
      }));
      useAppStore.getState().pdo.addEntries(entries);
    }).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      cleanup?.();
    };
  }, []);
}
