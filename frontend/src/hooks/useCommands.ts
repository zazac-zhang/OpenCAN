// Hooks for Tauri command mutations using React Query

import { useMutation } from '@tanstack/react-query';
import { useAppStore } from '../lib/store';
import * as tauri from '../lib/tauri';

// Connection hooks
export function useConnectBackend() {
  return useMutation({
    mutationFn: (params: Parameters<typeof tauri.connectBackend>[0]) =>
      tauri.connectBackend(params),
    onSuccess: (info) => {
      useAppStore.getState().can.connected = true;
      useAppStore.getState().can.backendInfo = info;
      useAppStore.getState().ui.setStatusMessage(`Connected to ${info.channel}`);
    },
    onError: (err) => {
      useAppStore.getState().ui.setStatusMessage(`Connection failed: ${err}`);
    },
  });
}

export function useDisconnect() {
  return useMutation({
    mutationFn: () => tauri.disconnect(),
    onSuccess: () => {
      const state = useAppStore.getState();
      state.can.connected = false;
      state.can.backendInfo = null;
      state.can.nodes = [];
      state.can.selectedNode = null;
      state.ui.setStatusMessage('Disconnected');
    },
  });
}

export function useGetBackends() {
  return useMutation({
    mutationFn: () => tauri.getBackends(),
  });
}

// NMT hooks
export function useScanNodes(timeoutMs: number = 3000) {
  return useMutation({
    mutationFn: () => tauri.scanNodes(timeoutMs),
    onSuccess: (nodeIds) => {
      const nodes = nodeIds.map((id) => ({
        node_id: id,
        nmt_state: 'PreOperational',
      }));
      useAppStore.getState().can.nodes = nodes;
      useAppStore.getState().ui.setStatusMessage(`Found ${nodeIds.length} nodes`);
    },
    onError: (err) => {
      useAppStore.getState().ui.setStatusMessage(`Scan failed: ${err}`);
    },
  });
}

export function useNmtCommand() {
  return useMutation({
    mutationFn: ({ nodeId, command }: { nodeId: number; command: string }) =>
      tauri.nmtCommand(nodeId, command),
    onSuccess: (_, { nodeId, command }) => {
      useAppStore.getState().ui.setStatusMessage(`NMT ${command} → Node ${nodeId}`);
    },
  });
}

// SDO hooks
export function useSdoUpload() {
  return useMutation({
    mutationFn: (params: Parameters<typeof tauri.sdoUpload>[0]) => tauri.sdoUpload(params),
    onSuccess: (result) => {
      useAppStore.getState().sdo.addHistory({
        node_id: result.node_id,
        index: result.index,
        subindex: result.subindex,
        value: result.data.map((b) => b.toString(16).padStart(2, '0')).join(' '),
        is_read: true,
        success: true,
      });
      useAppStore
        .getState()
        .ui.setStatusMessage(
          `SDO read ${result.index.toString(16).padStart(4, '0')}:${result.subindex.toString(16).padStart(2, '0')} = ${result.data.map((b) => b.toString(16).padStart(2, '0')).join(' ')}`,
        );
    },
    onError: (err, params) => {
      useAppStore.getState().sdo.addHistory({
        node_id: params.node_id,
        index: params.index,
        subindex: params.subindex,
        value: '',
        is_read: true,
        success: false,
        error: String(err),
      });
      useAppStore.getState().ui.setStatusMessage(`SDO error: ${err}`);
    },
  });
}

export function useSdoDownload() {
  return useMutation({
    mutationFn: (params: Parameters<typeof tauri.sdoDownload>[0]) => tauri.sdoDownload(params),
    onSuccess: (_, params) => {
      useAppStore.getState().sdo.addHistory({
        node_id: params.node_id,
        index: params.index,
        subindex: params.subindex,
        value: params.data.map((b) => b.toString(16).padStart(2, '0')).join(' '),
        is_read: false,
        success: true,
      });
      useAppStore
        .getState()
        .ui.setStatusMessage(
          `SDO write ${params.index.toString(16).padStart(4, '0')}:${params.subindex.toString(16).padStart(2, '0')}`,
        );
    },
    onError: (err, params) => {
      useAppStore.getState().sdo.addHistory({
        node_id: params.node_id,
        index: params.index,
        subindex: params.subindex,
        value: '',
        is_read: false,
        success: false,
        error: String(err),
      });
      useAppStore.getState().ui.setStatusMessage(`SDO error: ${err}`);
    },
  });
}

// DS402 hooks
export function useDs402Enable() {
  return useMutation({
    mutationFn: (nodeId: number) => tauri.ds402Enable(nodeId),
    onSuccess: (_, nodeId) => {
      useAppStore.getState().ui.setStatusMessage(`DS402 enabling node ${nodeId}...`);
    },
  });
}

export function useDs402FaultReset() {
  return useMutation({
    mutationFn: (nodeId: number) => tauri.ds402FaultReset(nodeId),
  });
}

export function useDs402SetMode() {
  return useMutation({
    mutationFn: ({ nodeId, mode }: { nodeId: number; mode: number }) =>
      tauri.ds402SetMode(nodeId, mode),
    onSuccess: (_, { nodeId, mode }) => {
      useAppStore.getState().ui.setStatusMessage(`DS402 mode ${mode} → Node ${nodeId}`);
    },
  });
}

export function useDs402SetTarget() {
  return useMutation({
    mutationFn: ({ nodeId, mode, target }: { nodeId: number; mode: number; target: number }) =>
      tauri.ds402SetTarget(nodeId, mode, target),
  });
}

// PDO hooks
export function useReadPdoMapping() {
  return useMutation({
    mutationFn: ({ nodeId, pdoIndex }: { nodeId: number; pdoIndex: number }) =>
      tauri.readPdoMapping(nodeId, pdoIndex),
  });
}

// Sync hooks
export function useStartSync() {
  return useMutation({
    mutationFn: (periodUs: number) => tauri.startSync(periodUs),
    onSuccess: (_, periodUs) => {
      useAppStore.getState().sync.updateStatus({ is_producer: true, producer_period_us: periodUs });
      useAppStore.getState().ui.setStatusMessage(`SYNC producer started (${periodUs} μs)`);
    },
  });
}

export function useStopSync() {
  return useMutation({
    mutationFn: () => tauri.stopSync(),
    onSuccess: () => {
      useAppStore.getState().sync.updateStatus({ is_producer: false });
      useAppStore.getState().ui.setStatusMessage('SYNC producer stopped');
    },
  });
}

// EDS hooks
export function useLoadEdsFile() {
  return useMutation({
    mutationFn: (path: string) => tauri.loadEdsFile(path),
    onSuccess: (info) => {
      useAppStore.getState().ui.setStatusMessage(`Loaded EDS: ${info.product_name}`);
    },
    onError: (err) => {
      useAppStore.getState().ui.setStatusMessage(`EDS load failed: ${err}`);
    },
  });
}

export function useGetOdEntries() {
  return useMutation({
    mutationFn: () => tauri.getOdEntries(),
  });
}

// Recording hooks
export function useStartRecording() {
  return useMutation({
    mutationFn: (path: string) => tauri.startRecording(path),
    onSuccess: () => {
      useAppStore.getState().recording.setRecording({ isRecording: true });
    },
  });
}

export function useStopRecording() {
  return useMutation({
    mutationFn: () => tauri.stopRecording(),
    onSuccess: () => {
      useAppStore.getState().recording.setRecording({ isRecording: false });
    },
  });
}

export function useLoadRecording() {
  return useMutation({
    mutationFn: (path: string) => tauri.loadRecording(path),
    onSuccess: (meta) => {
      useAppStore.getState().recording.setRecording({ loadedMeta: meta });
    },
  });
}

export function useStartPlayback() {
  return useMutation({
    mutationFn: ({ path, speed }: { path: string; speed: number }) =>
      tauri.startPlayback(path, speed),
    onSuccess: () => {
      useAppStore.getState().recording.setRecording({ isPlaying: true });
    },
  });
}

export function useStopPlayback() {
  return useMutation({
    mutationFn: () => tauri.stopPlayback(),
    onSuccess: () => {
      useAppStore.getState().recording.setRecording({ isPlaying: false });
    },
  });
}

// Recording playback with path
export function useStartRecordingWithPath() {
  return useMutation({
    mutationFn: ({ path }: { path: string }) => tauri.startRecording(path),
    onSuccess: (_, { path }) => {
      useAppStore.getState().recording.setRecording({ isRecording: true, recordingPath: path });
    },
  });
}

// CAN frame send hook
export function useSendFrame() {
  return useMutation({
    mutationFn: ({ cobId, data }: { cobId: number; data: number[] }) =>
      tauri.sendFrame(cobId, data),
    onError: (err) => {
      useAppStore.getState().ui.setStatusMessage(`Send frame failed: ${err}`);
    },
  });
}
