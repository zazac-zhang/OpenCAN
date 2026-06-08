// Root App component — new 3-column layout with context-aware bottom panel

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { lazy, Suspense, useEffect } from 'react';
import { ConnectionDialog } from '@/components/common/ConnectionDialog';
import { BottomPanel } from '@/components/layout/BottomPanel';
import { DetailPanel } from '@/components/layout/DetailPanel';
import { Sidebar } from '@/components/layout/Sidebar';
import { StatusBar } from '@/components/layout/StatusBar';
import { TabBar } from '@/components/layout/TabBar';
import { TopBar } from '@/components/layout/TopBar';
import { useFrameStream } from '@/hooks/useFrameStream';
import { useKeyboardShortcuts } from '@/hooks/useKeyboardShortcuts';
import {
  useBusStatsStream,
  useDs402StateStream,
  useEmcyStream,
  useHeartbeatStream,
} from '@/hooks/useStreams';
import { useActiveGroup, useAppStore, useConnected, useGroupTabs } from '@/lib/store';

// Lazy-loaded page components — split by navigation group
const FrameMonitor = lazy(() =>
  import('@/pages/CAN/FrameMonitor').then((m) => ({ default: m.FrameMonitor })),
);
const SendPanel = lazy(() =>
  import('@/pages/CAN/SendPanel').then((m) => ({ default: m.SendPanel })),
);
const BusStatistics = lazy(() =>
  import('@/pages/CAN/BusStatistics').then((m) => ({ default: m.BusStatistics })),
);
const ErrorFrames = lazy(() =>
  import('@/pages/CAN/ErrorFrames').then((m) => ({ default: m.ErrorFrames })),
);
const NetworkTopology = lazy(() =>
  import('@/pages/CANOpen/NetworkTopology').then((m) => ({ default: m.NetworkTopology })),
);
const NodeDetail = lazy(() =>
  import('@/pages/CANOpen/NodeDetail').then((m) => ({ default: m.NodeDetail })),
);
const PdoMonitor = lazy(() =>
  import('@/pages/CANOpen/PdoMonitor').then((m) => ({ default: m.PdoMonitor })),
);
const Ds402Control = lazy(() =>
  import('@/pages/CANOpen/Ds402Control').then((m) => ({ default: m.Ds402Control })),
);
const EmcyMonitor = lazy(() =>
  import('@/pages/CANOpen/EmcyMonitor').then((m) => ({ default: m.EmcyMonitor })),
);
const HeartbeatMonitor = lazy(() =>
  import('@/pages/CANOpen/HeartbeatMonitor').then((m) => ({ default: m.HeartbeatMonitor })),
);
const SyncManagement = lazy(() =>
  import('@/pages/CANOpen/SyncManagement').then((m) => ({ default: m.SyncManagement })),
);
const SdoExplorer = lazy(() =>
  import('@/pages/CANOpen/SdoExplorer').then((m) => ({ default: m.SdoExplorer })),
);
const ScriptEditor = lazy(() =>
  import('@/pages/CANOpen/ScriptEditor').then((m) => ({ default: m.ScriptEditor })),
);
const SessionRecorder = lazy(() =>
  import('@/pages/Recording/SessionRecorder').then((m) => ({ default: m.SessionRecorder })),
);
const SessionPlayer = lazy(() =>
  import('@/pages/Recording/SessionPlayer').then((m) => ({ default: m.SessionPlayer })),
);
const ConnectionSettings = lazy(() =>
  import('@/pages/Settings/ConnectionSettings').then((m) => ({ default: m.ConnectionSettings })),
);
const EdsManagement = lazy(() =>
  import('@/pages/Settings/EdsManagement').then((m) => ({ default: m.EdsManagement })),
);

const queryClient = new QueryClient({
  defaultOptions: {
    mutations: {
      retry: false,
    },
  },
});

// Map group+tab to component
const TAB_COMPONENTS: Record<string, Record<string, React.ComponentType>> = {
  can: {
    Frames: FrameMonitor,
    Send: SendPanel,
    Statistics: BusStatistics,
    Errors: ErrorFrames,
  },
  canopen: {
    Network: NetworkTopology,
    Nodes: NodeDetail,
    PDO: PdoMonitor,
    DS402: Ds402Control,
    SDO: SdoExplorer,
    Script: ScriptEditor,
    EMCY: EmcyMonitor,
    Heartbeat: HeartbeatMonitor,
    SYNC: SyncManagement,
  },
  recording: {
    Record: SessionRecorder,
    Playback: SessionPlayer,
  },
  eds: {
    'EDS Files': ConnectionSettings,
    'OD Browser': EdsManagement,
  },
};

// Legacy tab key → new group+tab mapping
const LEGACY_MAP: Record<string, { group: string; tab: string }> = {
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

function AppContent() {
  const activeGroup = useActiveGroup();
  const groupTabs = useGroupTabs();
  const currentTab = useAppStore((s) => s.ui.currentTab);
  const detailVisible = useAppStore((s) => s.ui.detailPanelVisible);
  const connected = useConnected();
  const dialogVisible = useAppStore((s) => s.connectionDialog.visible);
  const { startListening } = useFrameStream();

  // Subscribe to all Tauri event streams
  useEmcyStream();
  useHeartbeatStream();
  useDs402StateStream();
  useBusStatsStream();

  // Global keyboard shortcuts
  useKeyboardShortcuts();

  // Start listening to frame stream when connected
  useEffect(() => {
    if (connected) {
      startListening();
    }
  }, [connected, startListening]);

  // Migrate legacy tab key on first render
  useEffect(() => {
    const migrated = LEGACY_MAP[currentTab];
    if (migrated) {
      useAppStore.setState((s) => ({
        sidebar: { ...s.sidebar, activeGroup: migrated.group as any },
        ui: { ...s.ui, currentTab: migrated.tab },
      }));
    }
  }, [currentTab]);

  // Resolve the component to render
  const groupComponents = TAB_COMPONENTS[activeGroup] || {};
  // If currentTab doesn't match any tab in current group, use first tab
  const validTab =
    groupTabs.find((t) => t.key === currentTab)?.key || groupTabs[0]?.key || 'Frames';
  const ContentComponent = groupComponents[validTab] || FrameMonitor;

  return (
    <div className="flex flex-col h-screen bg-background text-foreground">
      <TopBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <div className="flex flex-col flex-1 overflow-hidden">
          <TabBar />
          <div className="flex-1 overflow-hidden">
            <Suspense
              fallback={
                <div className="flex items-center justify-center h-full text-muted-foreground">
                  Loading...
                </div>
              }
            >
              <ContentComponent />
            </Suspense>
          </div>
        </div>
        {detailVisible && <DetailPanel />}
      </div>
      <BottomPanel />
      <StatusBar />
      {dialogVisible && <ConnectionDialog />}
    </div>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppContent />
    </QueryClientProvider>
  );
}
