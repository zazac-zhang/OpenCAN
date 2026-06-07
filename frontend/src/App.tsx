// Root App component — new 3-column layout with context-aware bottom panel

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { TopBar } from '@/components/layout/TopBar';
import { Sidebar } from '@/components/layout/Sidebar';
import { StatusBar } from '@/components/layout/StatusBar';
import { TabBar } from '@/components/layout/TabBar';
import { DetailPanel } from '@/components/layout/DetailPanel';
import { BottomPanel } from '@/components/layout/BottomPanel';
import { FrameMonitor } from '@/pages/CAN/FrameMonitor';
import { SendPanel } from '@/pages/CAN/SendPanel';
import { BusStatistics } from '@/pages/CAN/BusStatistics';
import { ErrorFrames } from '@/pages/CAN/ErrorFrames';
import { NetworkOverview } from '@/pages/CANOpen/NetworkOverview';
import { NodeDetail } from '@/pages/CANOpen/NodeDetail';
import { PdoMonitor } from '@/pages/CANOpen/PdoMonitor';
import { Ds402Control } from '@/pages/CANOpen/Ds402Control';
import { EmcyMonitor } from '@/pages/CANOpen/EmcyMonitor';
import { HeartbeatMonitor } from '@/pages/CANOpen/HeartbeatMonitor';
import { SyncManagement } from '@/pages/CANOpen/SyncManagement';
import { SessionRecorder } from '@/pages/Recording/SessionRecorder';
import { SessionPlayer } from '@/pages/Recording/SessionPlayer';
import { ConnectionSettings } from '@/pages/Settings/ConnectionSettings';
import { EdsManagement } from '@/pages/Settings/EdsManagement';
import { useAppStore, useConnected, useActiveGroup, useGroupTabs } from '@/lib/store';
import { useFrameStream } from '@/hooks/useFrameStream';
import { useEmcyStream, useHeartbeatStream, useDs402StateStream, useBusStatsStream } from '@/hooks/useStreams';
import { ConnectionDialog } from '@/components/common/ConnectionDialog';
import { useEffect } from 'react';

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
    Network: NetworkOverview,
    Nodes: NodeDetail,
    PDO: PdoMonitor,
    DS402: Ds402Control,
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
  }, []);

  // Resolve the component to render
  const groupComponents = TAB_COMPONENTS[activeGroup] || {};
  // If currentTab doesn't match any tab in current group, use first tab
  const validTab = groupTabs.find((t) => t.key === currentTab)?.key || groupTabs[0]?.key || 'Frames';
  const ContentComponent = groupComponents[validTab] || FrameMonitor;

  return (
    <div className="flex flex-col h-screen bg-background text-foreground">
      <TopBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <div className="flex flex-col flex-1 overflow-hidden">
          <TabBar />
          <div className="flex-1 overflow-hidden">
            <ContentComponent />
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
