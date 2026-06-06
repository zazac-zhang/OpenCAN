// Root App component

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { TopBar } from '@/components/layout/TopBar';
import { Sidebar } from '@/components/layout/Sidebar';
import { StatusBar } from '@/components/layout/StatusBar';
import { TabBar } from '@/components/layout/TabBar';
import { DetailPanel } from '@/components/layout/DetailPanel';
import { FrameMonitor } from '@/pages/CAN/FrameMonitor';
import { BusStatistics } from '@/pages/CAN/BusStatistics';
import { ErrorFrames } from '@/pages/CAN/ErrorFrames';
import { NetworkOverview } from '@/pages/CANOpen/NetworkOverview';
import { NodeDetail } from '@/pages/CANOpen/NodeDetail';
import { PdoMonitor } from '@/pages/CANOpen/PdoMonitor';
import { Ds402Control } from '@/pages/CANOpen/Ds402Control';
import { EmcyMonitor } from '@/pages/CANOpen/EmcyMonitor';
import { HeartbeatMonitor } from '@/pages/CANOpen/HeartbeatMonitor';
import { SyncManagement } from '@/pages/CANOpen/SyncManagement';
import { useAppStore, useConnected } from '@/lib/store';
import { useFrameStream } from '@/hooks/useFrameStream';

const queryClient = new QueryClient({
  defaultOptions: {
    mutations: {
      retry: false,
    },
  },
});

const TAB_COMPONENTS: Record<string, React.ComponentType> = {
  FrameMonitor,
  BusStatistics,
  ErrorFrames,
  NetworkOverview,
  NodeDetail,
  PdoMonitor,
  Ds402Control,
  EmcyMonitor,
  HeartbeatMonitor,
  SyncManagement,
};

function AppContent() {
  const currentTab = useAppStore((s) => s.ui.currentTab);
  const detailVisible = useAppStore((s) => s.ui.detailPanelVisible);
  const connected = useConnected();
  const { startListening } = useFrameStream();

  // Start listening to frame stream when connected
  if (connected) {
    startListening();
  }

  const ContentComponent = TAB_COMPONENTS[currentTab] || FrameMonitor;

  return (
    <div className="flex flex-col h-screen bg-background text-foreground">
      <TopBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <div className="flex flex-col flex-1 overflow-hidden">
          <TabBar />
          <div className="flex flex-1 overflow-hidden">
            <div className="flex-1 overflow-auto">
              <ContentComponent />
            </div>
            {detailVisible && <DetailPanel />}
          </div>
        </div>
      </div>
      <StatusBar />
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
