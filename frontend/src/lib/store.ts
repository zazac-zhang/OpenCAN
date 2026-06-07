// Zustand store for global application state

import { create } from 'zustand';
import type { NodeInfo, SdoEntry, PdoEntry, EmcyEntry, HeartbeatEntry, SyncStatus } from '../types/canopen';
import type { CanFrame, BusStats, ErrorFrame } from '../types/can';
import type { Ds402NodeState } from '../types/ds402';
import type { RecordingState } from '../types/recording';

// Navigation group keys
export type NavGroup = 'can' | 'canopen' | 'recording' | 'eds';

// Tab keys per group
export const GROUP_TABS: Record<NavGroup, { key: string; label: string }[]> = {
  can: [
    { key: 'Frames', label: 'Frames' },
    { key: 'Send', label: 'Send' },
    { key: 'Statistics', label: 'Statistics' },
    { key: 'Errors', label: 'Errors' },
  ],
  canopen: [
    { key: 'Network', label: 'Network' },
    { key: 'Nodes', label: 'Nodes' },
    { key: 'PDO', label: 'PDO' },
    { key: 'DS402', label: 'DS402' },
    { key: 'EMCY', label: 'EMCY' },
    { key: 'Heartbeat', label: 'Heartbeat' },
    { key: 'SYNC', label: 'SYNC' },
  ],
  recording: [
    { key: 'Record', label: 'Record' },
    { key: 'Playback', label: 'Playback' },
  ],
  eds: [
    { key: 'EDS Files', label: 'EDS Files' },
    { key: 'OD Browser', label: 'OD Browser' },
  ],
};

// Legacy tab key mapping for backwards compatibility
const LEGACY_TAB_MAP: Record<string, { group: NavGroup; tab: string }> = {
  FrameMonitor: { group: 'can', tab: 'Frames' },
  BusStatistics: { group: 'can', tab: 'Statistics' },
  ErrorFrames: { group: 'can', tab: 'Errors' },
  NetworkOverview: { group: 'canopen', tab: 'Network' },
  NodeDetail: { group: 'canopen', tab: 'Nodes' },
  PdoMonitor: { group: 'canopen', tab: 'PDO' },
  Ds402Control: { group: 'canopen', tab: 'DS402' },
  EmcyMonitor: { group: 'canopen', tab: 'EMCY' },
  HeartbeatMonitor: { group: 'canopen', tab: 'Heartbeat' },
  SyncManagement: { group: 'canopen', tab: 'SYNC' },
  SessionRecorder: { group: 'recording', tab: 'Record' },
  SessionPlayer: { group: 'recording', tab: 'Playback' },
  ConnectionSettings: { group: 'eds', tab: 'EDS Files' },
  EdsManagement: { group: 'eds', tab: 'OD Browser' },
};

// CAN connection state
interface CanState {
  connected: boolean;
  backendInfo: { backend_type: string; channel: string; bitrate: number; node_id: number } | null;
  nodes: NodeInfo[];
  selectedNode: number | null;
}

// CAN frame state
interface FrameState {
  frames: CanFrame[];
  busStats: BusStats;
  addFrames: (frames: CanFrame[]) => void;
  clearFrames: () => void;
  updateBusStats: (stats: Partial<BusStats>) => void;
}

// SDO state
interface SdoState {
  history: SdoEntry[];
  addHistory: (entry: SdoEntry) => void;
  clearHistory: () => void;
}

// PDO state
interface PdoState {
  entries: PdoEntry[];
  addEntries: (entries: PdoEntry[]) => void;
  clearEntries: () => void;
}

// EMCY state
interface EmcyState {
  entries: EmcyEntry[];
  addEntries: (entries: EmcyEntry[]) => void;
  clearEntries: () => void;
}

// Heartbeat state
interface HeartbeatState {
  entries: HeartbeatEntry[];
  updateEntry: (entry: HeartbeatEntry) => void;
}

// DS402 state
interface Ds402State {
  nodeStates: Record<number, Ds402NodeState>;
  updateNodeState: (nodeId: number, state: Partial<Ds402NodeState>) => void;
  pushPosition: (nodeId: number, value: number) => void;
  pushVelocity: (nodeId: number, value: number) => void;
  pushTorque: (nodeId: number, value: number) => void;
}

// Sync state
interface SyncState {
  status: SyncStatus;
  updateStatus: (status: Partial<SyncStatus>) => void;
}

// Connection dialog state
interface ConnectionDialogState {
  visible: boolean;
  selectedBackend: string;
  channel: string;
  bitrate: number;
  nodeId: string;
  show: () => void;
  hide: () => void;
}

// Sidebar state
interface SidebarState {
  activeGroup: NavGroup;
  groupsCollapsed: Record<string, boolean>;
  setActiveGroup: (group: NavGroup) => void;
  toggleGroup: (group: string) => void;
}

// Bottom panel state
interface BottomPanelState {
  visible: boolean;
  activeTab: string;
  height: number;
  setVisible: (visible: boolean) => void;
  setActiveTab: (tab: string) => void;
  setHeight: (height: number) => void;
}

// UI state
interface UiState {
  currentTab: string;
  detailPanelVisible: boolean;
  paused: boolean;
  statusMessage: string;
  setCurrentTab: (tab: string) => void;
  toggleDetailPanel: () => void;
  togglePause: () => void;
  setStatusMessage: (msg: string) => void;
}

// Recording state
interface RecordingStateStore {
  recording: RecordingState;
  setRecording: (state: Partial<RecordingState>) => void;
}

// Error frame state
interface ErrorFrameState {
  errorFrames: ErrorFrame[];
  addErrorFrames: (frames: ErrorFrame[]) => void;
  clearErrorFrames: () => void;
}

// Root store
interface AppState {
  can: CanState;
  frames: FrameState;
  sdo: SdoState;
  pdo: PdoState;
  emcy: EmcyState;
  heartbeat: HeartbeatState;
  ds402: Ds402State;
  sync: SyncState;
  connectionDialog: ConnectionDialogState;
  sidebar: SidebarState;
  bottomPanel: BottomPanelState;
  ui: UiState;
  recording: RecordingStateStore;
  errors: ErrorFrameState;
}

// Default bottom panel tabs per group
const DEFAULT_BOTTOM_TAB: Record<NavGroup, string> = {
  can: 'Bus Load',
  canopen: 'PDO Stream',
  recording: 'Session Info',
  eds: 'OD Entries',
};

export const useAppStore = create<AppState>()((set) => ({
  can: {
    connected: false,
    backendInfo: null,
    nodes: [],
    selectedNode: null,
  },
  frames: {
    frames: [],
    busStats: { bus_load: 0, frame_rate: 0, tx_errors: 0, rx_errors: 0, error_frame_count: 0 },
    addFrames: (frames) =>
      set((state) => {
        const all = [...state.frames.frames, ...frames];
        return { frames: { ...state.frames, frames: all.slice(-10000) } };
      }),
    clearFrames: () => set((state) => ({ frames: { ...state.frames, frames: [] } })),
    updateBusStats: (stats) =>
      set((state) => ({
        frames: { ...state.frames, busStats: { ...state.frames.busStats, ...stats } },
      })),
  },
  sdo: {
    history: [],
    addHistory: (entry) =>
      set((state) => ({ sdo: { ...state.sdo, history: [...state.sdo.history, entry] } })),
    clearHistory: () => set((state) => ({ sdo: { ...state.sdo, history: [] } })),
  },
  pdo: {
    entries: [],
    addEntries: (entries) =>
      set((state) => {
        const all = [...state.pdo.entries, ...entries];
        return { pdo: { ...state.pdo, entries: all.slice(-1000) } };
      }),
    clearEntries: () => set((state) => ({ pdo: { ...state.pdo, entries: [] } })),
  },
  emcy: {
    entries: [],
    addEntries: (entries) =>
      set((state) => ({ emcy: { ...state.emcy, entries: [...state.emcy.entries, ...entries] } })),
    clearEntries: () => set((state) => ({ emcy: { ...state.emcy, entries: [] } })),
  },
  heartbeat: {
    entries: [],
    updateEntry: (entry) =>
      set((state) => {
        const idx = state.heartbeat.entries.findIndex((e) => e.node_id === entry.node_id);
        const entries = [...state.heartbeat.entries];
        if (idx >= 0) {
          entries[idx] = entry;
        } else {
          entries.push(entry);
        }
        return { heartbeat: { ...state.heartbeat, entries } };
      }),
  },
  ds402: {
    nodeStates: {},
    updateNodeState: (nodeId, partial) =>
      set((state) => ({
        ds402: {
          ...state.ds402,
          nodeStates: {
            ...state.ds402.nodeStates,
            [nodeId]: { ...state.ds402.nodeStates[nodeId], ...partial },
          },
        },
      })),
    pushPosition: (nodeId, value) =>
      set((state) => {
        const node = state.ds402.nodeStates[nodeId];
        if (!node) return state;
        const now = Date.now();
        return {
          ds402: {
            ...state.ds402,
            nodeStates: {
              ...state.ds402.nodeStates,
              [nodeId]: {
                ...node,
                actual_position: value,
                position_history: [...node.position_history.slice(-1000), { time: now, value }],
              },
            },
          },
        };
      }),
    pushVelocity: (nodeId, value) =>
      set((state) => {
        const node = state.ds402.nodeStates[nodeId];
        if (!node) return state;
        const now = Date.now();
        return {
          ds402: {
            ...state.ds402,
            nodeStates: {
              ...state.ds402.nodeStates,
              [nodeId]: {
                ...node,
                actual_velocity: value,
                velocity_history: [...node.velocity_history.slice(-1000), { time: now, value }],
              },
            },
          },
        };
      }),
    pushTorque: (nodeId, value) =>
      set((state) => {
        const node = state.ds402.nodeStates[nodeId];
        if (!node) return state;
        const now = Date.now();
        return {
          ds402: {
            ...state.ds402,
            nodeStates: {
              ...state.ds402.nodeStates,
              [nodeId]: {
                ...node,
                actual_torque: value,
                torque_history: [...node.torque_history.slice(-1000), { time: now, value }],
              },
            },
          },
        };
      }),
  },
  sync: {
    status: { is_producer: false, producer_period_us: 1000 },
    updateStatus: (partial) =>
      set((state) => ({ sync: { ...state.sync, status: { ...state.sync.status, ...partial } } })),
  },
  connectionDialog: {
    visible: false,
    selectedBackend: 'mock',
    channel: 'can0',
    bitrate: 500000,
    nodeId: '0',
    show: () => set((state) => ({ connectionDialog: { ...state.connectionDialog, visible: true } })),
    hide: () => set((state) => ({ connectionDialog: { ...state.connectionDialog, visible: false } })),
  },
  sidebar: {
    activeGroup: 'can',
    groupsCollapsed: { can: false, canopen: false, recording: false, eds: false },
    setActiveGroup: (group) =>
      set((state) => ({
        sidebar: {
          ...state.sidebar,
          activeGroup: group,
        },
        bottomPanel: {
          ...state.bottomPanel,
          activeTab: DEFAULT_BOTTOM_TAB[group],
        },
      })),
    toggleGroup: (group) =>
      set((state) => ({
        sidebar: {
          ...state.sidebar,
          groupsCollapsed: {
            ...state.sidebar.groupsCollapsed,
            [group]: !state.sidebar.groupsCollapsed[group],
          },
        },
      })),
  },
  bottomPanel: {
    visible: false,
    activeTab: 'Bus Load',
    height: 150,
    setVisible: (visible) => set((state) => ({ bottomPanel: { ...state.bottomPanel, visible } })),
    setActiveTab: (tab) => set((state) => ({ bottomPanel: { ...state.bottomPanel, activeTab: tab } })),
    setHeight: (height) => set((state) => ({ bottomPanel: { ...state.bottomPanel, height } })),
  },
  ui: {
    currentTab: 'Frames',
    detailPanelVisible: false,
    paused: false,
    statusMessage: '',
    setCurrentTab: (tab) => set((state) => ({ ui: { ...state.ui, currentTab: tab } })),
    toggleDetailPanel: () =>
      set((state) => ({ ui: { ...state.ui, detailPanelVisible: !state.ui.detailPanelVisible } })),
    togglePause: () => set((state) => ({ ui: { ...state.ui, paused: !state.ui.paused } })),
    setStatusMessage: (msg) => set((state) => ({ ui: { ...state.ui, statusMessage: msg } })),
  },
  recording: {
    recording: {
      isRecording: false,
      recordingPath: null,
      isPlaying: false,
      playbackSpeed: 1,
      playbackProgress: 0,
      loadedMeta: null,
    },
    setRecording: (partial) =>
      set((state) => ({
        recording: { ...state.recording, recording: { ...state.recording.recording, ...partial } },
      })),
  },
  errors: {
    errorFrames: [],
    addErrorFrames: (frames) =>
      set((state) => ({
        errors: { ...state.errors, errorFrames: [...state.errors.errorFrames, ...frames] },
      })),
    clearErrorFrames: () => set((state) => ({ errors: { ...state.errors, errorFrames: [] } })),
  },
}));

// Fine-grained selectors
export const useConnected = () => useAppStore((s) => s.can.connected);
export const useSelectedNode = () => useAppStore((s) => s.can.selectedNode);
export const useNodes = () => useAppStore((s) => s.can.nodes);
export const useFrames = () => useAppStore((s) => s.frames.frames);
export const useBusStats = () => useAppStore((s) => s.frames.busStats);
export const useSdoHistory = () => useAppStore((s) => s.sdo.history);
export const useUi = () => useAppStore((s) => s.ui);
export const useErrorFrames = () => useAppStore((s) => s.errors.errorFrames);

// Navigation selectors
export const useActiveGroup = () => useAppStore((s) => s.sidebar.activeGroup);
export const useActiveTab = () => {
  const group = useAppStore((s) => s.sidebar.activeGroup);
  const tabs = GROUP_TABS[group];
  return tabs ? tabs[0].key : 'Frames';
};
export const useGroupTabs = () => {
  const group = useAppStore((s) => s.sidebar.activeGroup);
  return GROUP_TABS[group];
};
export const useSidebarCollapsed = (group: string) =>
  useAppStore((s) => s.sidebar.groupsCollapsed[group] ?? false);

// Bottom panel selectors
export const useBottomPanel = () => useAppStore((s) => s.bottomPanel);

// Bottom panel tab definitions per group
export const BOTTOM_PANEL_TABS: Record<NavGroup, string[]> = {
  can: ['Signals', 'Bus Load', 'Error Log', 'Timing'],
  canopen: ['PDO Stream', 'EMCY', 'DS402 State', 'Heartbeat'],
  recording: ['Session Info', 'Timeline'],
  eds: ['OD Entries', 'Parse Log'],
};

/** Convert a legacy tab key (e.g. 'FrameMonitor') to the new navigation state */
export function migrateLegacyTab(legacyKey: string): { group: NavGroup; tab: string } | null {
  return LEGACY_TAB_MAP[legacyKey] ?? null;
}
