import { beforeEach, describe, expect, it } from 'vitest';
import { useAppStore } from '@/lib/store';

describe('useAppStore', () => {
  beforeEach(() => {
    // Reset store state before each test (preserve functions)
    const state = useAppStore.getState();
    state.frames.clearFrames();
    state.sdo.clearHistory();
    state.pdo.clearEntries();
    state.emcy.clearEntries();
    useAppStore.setState((s) => ({
      can: {
        ...s.can,
        connected: false,
        backendInfo: null,
        nodes: [],
        selectedNode: null,
      },
    }));
  });

  describe('CAN connection state', () => {
    it('starts disconnected', () => {
      const state = useAppStore.getState();
      expect(state.can.connected).toBe(false);
      expect(state.can.backendInfo).toBeNull();
    });

    it('can set connected state', () => {
      useAppStore.setState({
        can: {
          ...useAppStore.getState().can,
          connected: true,
          backendInfo: {
            backend_type: 'SocketCAN',
            channel: 'vcan0',
            bitrate: 500000,
            node_id: 1,
          },
        },
      });

      const state = useAppStore.getState();
      expect(state.can.connected).toBe(true);
      expect(state.can.backendInfo?.backend_type).toBe('SocketCAN');
    });
  });

  describe('frame management', () => {
    it('starts with empty frames', () => {
      const state = useAppStore.getState();
      expect(state.frames.frames).toEqual([]);
    });

    it('can add frames', () => {
      const state = useAppStore.getState();
      state.frames.addFrames([
        {
          cob_id: 0x185,
          data: [0x01, 0x02, 0x03, 0x04, 0, 0, 0, 0],
          dlc: 4,
          direction: 'rx',
          timestamp_ms: Date.now(),
        },
      ]);

      const newState = useAppStore.getState();
      expect(newState.frames.frames).toHaveLength(1);
      expect(newState.frames.frames[0].cob_id).toBe(0x185);
    });

    it('can clear frames', () => {
      const state = useAppStore.getState();
      state.frames.addFrames([
        {
          cob_id: 0x185,
          data: [0, 0, 0, 0, 0, 0, 0, 0],
          dlc: 0,
          direction: 'rx',
          timestamp_ms: Date.now(),
        },
      ]);
      state.frames.clearFrames();

      expect(useAppStore.getState().frames.frames).toEqual([]);
    });
  });

  describe('UI state', () => {
    it('starts with Frames tab', () => {
      expect(useAppStore.getState().ui.currentTab).toBe('Frames');
    });

    it('can change active group', () => {
      useAppStore.setState((s) => ({
        sidebar: { ...s.sidebar, activeGroup: 'canopen' },
        ui: { ...s.ui, currentTab: 'Network' },
      }));

      const state = useAppStore.getState();
      expect(state.sidebar.activeGroup).toBe('canopen');
      expect(state.ui.currentTab).toBe('Network');
    });
  });

  describe('node management', () => {
    it('starts with no nodes', () => {
      expect(useAppStore.getState().can.nodes).toEqual([]);
    });

    it('can select a node', () => {
      useAppStore.setState((s) => ({
        can: { ...s.can, selectedNode: 5 },
      }));

      expect(useAppStore.getState().can.selectedNode).toBe(5);
    });
  });
});
