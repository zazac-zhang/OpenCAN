// Bottom panel — context-aware tabs that switch with the active navigation group

import { useAppStore, useActiveGroup, useBottomPanel, BOTTOM_PANEL_TABS } from '@/lib/store';
import { ChevronUp, ChevronDown, Gauge, Radio, AlertCircle, Activity, Heart, FileText, Clock, BarChart3 } from 'lucide-react';
import { useRef, useCallback } from 'react';

const TAB_ICONS: Record<string, React.ReactNode> = {
  'Signals': <BarChart3 className="w-3 h-3" />,
  'Bus Load': <Gauge className="w-3 h-3" />,
  'Error Log': <AlertCircle className="w-3 h-3" />,
  'Timing': <Clock className="w-3 h-3" />,
  'PDO Stream': <Radio className="w-3 h-3" />,
  'EMCY': <AlertCircle className="w-3 h-3" />,
  'DS402 State': <Activity className="w-3 h-3" />,
  'Heartbeat': <Heart className="w-3 h-3" />,
  'Session Info': <FileText className="w-3 h-3" />,
  'Timeline': <Clock className="w-3 h-3" />,
  'OD Entries': <FileText className="w-3 h-3" />,
  'Parse Log': <FileText className="w-3 h-3" />,
};

function BottomPanelContent({ tab }: { tab: string }) {
  const busStats = useAppStore((s) => s.frames.busStats);
  const pdoEntries = useAppStore((s) => s.pdo.entries);
  const emcyEntries = useAppStore((s) => s.emcy.entries);
  const hbEntries = useAppStore((s) => s.heartbeat.entries);
  const errorFrames = useAppStore((s) => s.errors.errorFrames);
  const frames = useAppStore((s) => s.frames.frames);
  const ds402States = useAppStore((s) => s.ds402.nodeStates);

  switch (tab) {
    case 'Bus Load':
      return (
        <div className="flex items-center gap-6 px-4 h-full text-xs">
          <div className="flex items-center gap-2">
            <Gauge className="w-3.5 h-3.5 text-muted-foreground" />
            <span className="text-muted-foreground">Load</span>
            <span className={`font-mono font-bold text-lg ${
              busStats.bus_load >= 70 ? 'text-red-400' : busStats.bus_load >= 30 ? 'text-yellow-400' : 'text-green-400'
            }`}>
              {busStats.bus_load.toFixed(1)}%
            </span>
          </div>
          <div className="w-px h-6 bg-border" />
          <div>
            <span className="text-muted-foreground">Rate</span>
            <span className="font-mono ml-1.5">{busStats.frame_rate} fps</span>
          </div>
          <div className="w-px h-6 bg-border" />
          <div>
            <span className="text-muted-foreground">Frames</span>
            <span className="font-mono ml-1.5">{frames.length.toLocaleString()}</span>
          </div>
          {busStats.tx_errors > 0 && (
            <>
              <div className="w-px h-6 bg-border" />
              <span className="text-red-400 font-mono">TX Err: {busStats.tx_errors}</span>
            </>
          )}
          {busStats.rx_errors > 0 && (
            <>
              <div className="w-px h-6 bg-border" />
              <span className="text-red-400 font-mono">RX Err: {busStats.rx_errors}</span>
            </>
          )}
        </div>
      );

    case 'PDO Stream':
      return (
        <div className="flex items-center gap-4 px-4 h-full text-xs">
          <span className="font-mono">{pdoEntries.length} PDO entries</span>
          {pdoEntries.length > 0 && (
            <>
              <span className="text-muted-foreground">Last:</span>
              <span className="font-mono">
                Node {pdoEntries[pdoEntries.length - 1]?.node_id}
                {' '}
                0x{pdoEntries[pdoEntries.length - 1]?.cob_id.toString(16).padStart(3, '0').toUpperCase()}
              </span>
            </>
          )}
        </div>
      );

    case 'EMCY':
      return (
        <div className="flex items-center gap-4 px-4 h-full text-xs">
          <span className="font-mono">{emcyEntries.length} EMCY messages</span>
          {emcyEntries.length > 0 && (
            <>
              <span className="text-muted-foreground">Last:</span>
              <span className="font-mono">
                Node {emcyEntries[emcyEntries.length - 1]?.node_id}
                {' '}
                0x{emcyEntries[emcyEntries.length - 1]?.error_code.toString(16).padStart(4, '0').toUpperCase()}
              </span>
            </>
          )}
        </div>
      );

    case 'DS402 State':
      return (
        <div className="flex items-center gap-4 px-4 h-full text-xs">
          {Object.keys(ds402States).length === 0 ? (
            <span className="text-muted-foreground">No DS402 nodes active</span>
          ) : (
            Object.entries(ds402States).map(([nodeId, state]) => (
              <div key={nodeId} className="flex items-center gap-2">
                <span className="font-mono">Node {nodeId}</span>
                <span className={`px-1.5 py-0.5 rounded text-[10px] ${
                  state.state === 'Operation Enabled'
                    ? 'bg-green-500/20 text-green-400'
                    : state.state === 'Fault'
                      ? 'bg-red-500/20 text-red-400'
                      : 'bg-yellow-500/20 text-yellow-400'
                }`}>
                  {state.state}
                </span>
              </div>
            ))
          )}
        </div>
      );

    case 'Heartbeat':
      return (
        <div className="flex items-center gap-4 px-4 h-full text-xs">
          {hbEntries.length === 0 ? (
            <span className="text-muted-foreground">No heartbeat data</span>
          ) : (
            hbEntries.map((hb) => {
              const elapsed = Date.now() - hb.last_seen_ms;
              const timedOut = elapsed > 10000;
              return (
                <div key={hb.node_id} className="flex items-center gap-2">
                  <span className={`w-1.5 h-1.5 rounded-full ${
                    timedOut ? 'bg-red-500' : hb.alive ? 'bg-green-500' : 'bg-muted'
                  }`} />
                  <span className="font-mono">Node {hb.node_id}</span>
                  <span className="text-muted-foreground">
                    {elapsed < 1000 ? `${elapsed}ms` : `${(elapsed / 1000).toFixed(1)}s`} ago
                  </span>
                  {timedOut && <span className="text-red-400 text-[10px]">TIMEOUT</span>}
                </div>
              );
            })
          )}
        </div>
      );

    case 'Error Log':
      return (
        <div className="flex items-center gap-4 px-4 h-full text-xs">
          <span className="font-mono">{errorFrames.length} error frames</span>
          {errorFrames.length > 0 && (
            <>
              <span className="text-muted-foreground">Last:</span>
              <span className="font-mono">{errorFrames[errorFrames.length - 1]?.error_type}</span>
            </>
          )}
        </div>
      );

    case 'Signals':
    case 'Timing':
    case 'Session Info':
    case 'Timeline':
    case 'OD Entries':
    case 'Parse Log':
      return (
        <div className="flex items-center px-4 h-full text-xs text-muted-foreground">
          {tab} data will appear here when available
        </div>
      );

    default:
      return (
        <div className="flex items-center px-4 h-full text-xs text-muted-foreground">
          Select a tab to view data
        </div>
      );
  }
}

export function BottomPanel() {
  const activeGroup = useActiveGroup();
  const { visible, activeTab, height, setVisible, setActiveTab, setHeight } = useBottomPanel();
  const tabs = BOTTOM_PANEL_TABS[activeGroup] || [];

  const panelRef = useRef<HTMLDivElement>(null);
  const resizeRef = useRef<{ startY: number; startHeight: number } | null>(null);

  const handleResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    resizeRef.current = { startY: e.clientY, startHeight: height };

    const handleMove = (e: MouseEvent) => {
      if (!resizeRef.current) return;
      const delta = resizeRef.current.startY - e.clientY;
      setHeight(Math.max(60, Math.min(400, resizeRef.current.startHeight + delta)));
    };

    const handleUp = () => {
      resizeRef.current = null;
      document.removeEventListener('mousemove', handleMove);
      document.removeEventListener('mouseup', handleUp);
    };

    document.addEventListener('mousemove', handleMove);
    document.addEventListener('mouseup', handleUp);
  }, [height, setHeight]);

  if (!visible) {
    return (
      <div className="flex items-center justify-center h-7 bg-muted/30 border-t border-border shrink-0">
        <button
          className="flex items-center gap-1.5 px-3 py-0.5 text-[10px] text-muted-foreground hover:text-foreground transition-colors"
          onClick={() => setVisible(true)}
        >
          <ChevronUp className="w-3 h-3" />
          Show Panel
        </button>
      </div>
    );
  }

  return (
    <div className="shrink-0 bg-card border-t border-border flex flex-col" style={{ height }}>
      {/* Resize handle */}
      <div
        className="h-1 bg-border/50 hover:bg-primary/30 cursor-row-resize shrink-0 transition-colors"
        onMouseDown={handleResizeStart}
      />

      {/* Tab bar */}
      <div className="flex items-center gap-0.5 px-2 border-b border-border/50 shrink-0 h-7">
        {tabs.map((tab) => {
          const Icon = TAB_ICONS[tab];
          return (
            <button
              key={tab}
              className={`flex items-center gap-1.5 px-2.5 py-0.5 text-[10px] rounded transition-colors ${
                activeTab === tab
                  ? 'bg-primary/10 text-primary font-medium'
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'
              }`}
              onClick={() => setActiveTab(tab)}
            >
              {Icon && <span className="opacity-60">{Icon}</span>}
              {tab}
            </button>
          );
        })}
        <div className="flex-1" />
        <button
          className="p-0.5 text-muted-foreground hover:text-foreground transition-colors"
          onClick={() => setVisible(false)}
          title="Collapse panel"
        >
          <ChevronDown className="w-3 h-3" />
        </button>
      </div>

      {/* Content */}
      <div ref={panelRef} className="flex-1 overflow-auto">
        <BottomPanelContent tab={activeTab} />
      </div>
    </div>
  );
}
