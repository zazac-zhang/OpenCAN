// Tab bar with primary and secondary tabs

import { useAppStore } from '@/lib/store';

const PRIMARY_TABS: { key: 'can' | 'canopen'; label: string }[] = [
  { key: 'can', label: 'CAN Bus' },
  { key: 'canopen', label: 'CANOpen' },
];

const SECONDARY_TABS: Record<string, { key: string; label: string }[]> = {
  can: [
    { key: 'FrameMonitor', label: 'Frame Monitor' },
    { key: 'BusStatistics', label: 'Bus Statistics' },
    { key: 'ErrorFrames', label: 'Error Frames' },
  ],
  canopen: [
    { key: 'NetworkOverview', label: 'Network' },
    { key: 'NodeDetail', label: 'SDO' },
    { key: 'PdoMonitor', label: 'PDO' },
    { key: 'Ds402Control', label: 'DS402' },
    { key: 'EmcyMonitor', label: 'EMCY' },
    { key: 'HeartbeatMonitor', label: 'Heartbeat' },
    { key: 'SyncManagement', label: 'SYNC' },
  ],
};

export function TabBar() {
  const ui = useAppStore((s) => s.ui);
  const setPrimaryTab = (tab: 'can' | 'canopen') => ui.setPrimaryTab(tab);
  const setCurrentTab = (tab: string) => ui.setCurrentTab(tab);

  return (
    <div className="px-4 py-2 border-b bg-background">
      {/* Primary tabs */}
      <div className="flex gap-1 mb-2">
        {PRIMARY_TABS.map((tab) => (
          <button
            key={tab.key}
            onClick={() => {
              setPrimaryTab(tab.key);
              const tabs = SECONDARY_TABS[tab.key];
              if (tabs && tabs.length > 0) {
                setCurrentTab(tabs[0].key);
              }
            }}
            className={`px-3 py-1 text-sm rounded ${
              ui.primaryTab === tab.key
                ? 'bg-primary text-primary-foreground'
                : 'bg-muted hover:bg-muted/80'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Secondary tabs */}
      <div className="flex gap-1 flex-wrap">
        {(SECONDARY_TABS[ui.primaryTab] || []).map((tab) => (
          <button
            key={tab.key}
            onClick={() => setCurrentTab(tab.key)}
            className={`px-2 py-0.5 text-xs rounded ${
              ui.currentTab === tab.key
                ? 'bg-primary text-primary-foreground'
                : 'bg-muted hover:bg-muted/80'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>
    </div>
  );
}
