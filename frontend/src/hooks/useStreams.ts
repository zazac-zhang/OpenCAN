/** Hooks for subscribing to various Tauri event channels and writing to the Zustand store. */

import { useEffect } from 'react';
import {
  onEmcyStream,
  onHeartbeatStream,
  onDs402StateStream,
  onBusStatsStream,
} from '../lib/tauri';
import { useAppStore } from '../lib/store';

/** Subscribe to EMCY stream events. */
export function useEmcyStream() {
  useEffect(() => {
    let cleanup: (() => void) | null = null;

    onEmcyStream((event) => {
      useAppStore.getState().emcy.addEntries([
        {
          node_id: event.node_id,
          error_code: event.error_code,
          error_register: event.error_register,
          data: event.data,
          timestamp_ms: event.timestamp_ms,
        },
      ]);
    }).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      cleanup?.();
    };
  }, []);
}

/** Subscribe to heartbeat stream events. */
export function useHeartbeatStream() {
  useEffect(() => {
    let cleanup: (() => void) | null = null;

    onHeartbeatStream((event) => {
      useAppStore.getState().heartbeat.updateEntry({
        node_id: event.node_id,
        alive: event.state !== 'Stopped',
        last_seen_ms: event.timestamp_ms,
      });
    }).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      cleanup?.();
    };
  }, []);
}

/** Subscribe to DS402 state stream events and push telemetry to history. */
export function useDs402StateStream() {
  useEffect(() => {
    let cleanup: (() => void) | null = null;

    onDs402StateStream((event) => {
      const store = useAppStore.getState();
      const nodeId = event.node_id;

      // Ensure node state exists
      if (!store.ds402.nodeStates[nodeId]) {
        store.ds402.updateNodeState(nodeId, {
          node_id: nodeId,
          state: event.state,
          status_word: event.status_word,
          actual_position: event.actual_position,
          actual_velocity: event.actual_velocity,
          actual_torque: event.actual_torque,
          selected_mode: '',
          target_position: '',
          target_velocity: '',
          target_torque: '',
          auto_refresh: false,
          raw_values: false,
          position_history: [],
          velocity_history: [],
          torque_history: [],
        });
      } else {
        // Update current values
        store.ds402.updateNodeState(nodeId, {
          state: event.state,
          status_word: event.status_word,
          actual_position: event.actual_position,
          actual_velocity: event.actual_velocity,
          actual_torque: event.actual_torque,
        });

        // Push to history for waveforms
        store.ds402.pushPosition(nodeId, event.actual_position);
        store.ds402.pushVelocity(nodeId, event.actual_velocity);
        store.ds402.pushTorque(nodeId, event.actual_torque);
      }
    }).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      cleanup?.();
    };
  }, []);
}

/** Subscribe to bus stats stream events. */
export function useBusStatsStream() {
  useEffect(() => {
    let cleanup: (() => void) | null = null;

    onBusStatsStream((event) => {
      useAppStore.getState().frames.updateBusStats({
        bus_load: event.bus_load,
        frame_rate: event.frame_rate,
        tx_errors: event.tx_errors,
        rx_errors: event.rx_errors,
      });
    }).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      cleanup?.();
    };
  }, []);
}
