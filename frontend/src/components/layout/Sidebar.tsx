// Sidebar with collapsible navigation groups and node list

import { useAppStore, useNodes, useSelectedNode, useActiveGroup, useConnected, useSidebarCollapsed, GROUP_TABS } from '@/lib/store';
import { useNmtCommand, useConnectBackend } from '@/hooks/useCommands';
import {
  ChevronDown,
  ChevronRight,
  Send,
  Activity,
  AlertTriangle,
  Network,
  Users,
  Radio,
  Gauge,
  AlertCircle,
  Heart,
  Wifi,
  CircleDot,
  Play,
  RotateCcw,
  FileCode2,
  BookOpen,
} from 'lucide-react';
import type { NavGroup } from '@/lib/store';

const GROUP_ICONS: Record<string, React.ReactNode> = {
  can: <Radio className="w-3.5 h-3.5" />,
  canopen: <Network className="w-3.5 h-3.5" />,
  recording: <CircleDot className="w-3.5 h-3.5" />,
  eds: <FileCode2 className="w-3.5 h-3.5" />,
};

const TAB_ICONS: Record<string, React.ReactNode> = {
  Frames: <Send className="w-3 h-3" />,
  Send: <Send className="w-3 h-3" />,
  Statistics: <Gauge className="w-3 h-3" />,
  Errors: <AlertTriangle className="w-3 h-3" />,
  Network: <Network className="w-3 h-3" />,
  Nodes: <Users className="w-3 h-3" />,
  PDO: <Radio className="w-3 h-3" />,
  DS402: <Activity className="w-3 h-3" />,
  EMCY: <AlertCircle className="w-3 h-3" />,
  Heartbeat: <Heart className="w-3 h-3" />,
  SYNC: <Wifi className="w-3 h-3" />,
  Record: <Play className="w-3 h-3" />,
  Playback: <RotateCcw className="w-3 h-3" />,
  'EDS Files': <FileCode2 className="w-3 h-3" />,
  'OD Browser': <BookOpen className="w-3 h-3" />,
};

const GROUP_LABELS: Record<NavGroup, string> = {
  can: 'CAN Bus',
  canopen: 'CANOpen',
  recording: 'Recording',
  eds: 'EDS',
};

// Track which tabs trigger the right detail panel
const DETAIL_PANEL_TABS = new Set(['Nodes', 'DS402']);

function NavGroupItem({ group, icon, label, tabs }: {
  group: NavGroup;
  icon: React.ReactNode;
  label: string;
  tabs: { key: string; label: string }[];
}) {
  const activeGroup = useActiveGroup();
  const isCollapsed = useSidebarCollapsed(group);
  const { setActiveGroup, toggleGroup } = useAppStore((s) => s.sidebar);

  const isActive = activeGroup === group;

  return (
    <div className="mb-1">
      {/* Group header */}
      <button
        className={`w-full flex items-center gap-2 px-2 py-1.5 text-xs font-medium rounded transition-colors ${
          isActive
            ? 'bg-primary/10 text-primary'
            : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'
        }`}
        onClick={() => {
          if (isCollapsed) {
            toggleGroup(group);
          }
          setActiveGroup(group);
        }}
      >
        {isCollapsed ? (
          <ChevronRight className="w-3 h-3 shrink-0" />
        ) : (
          <ChevronDown className="w-3 h-3 shrink-0" />
        )}
        {icon}
        <span className="flex-1 text-left truncate">{label}</span>
      </button>

      {/* Tab items */}
      {!isCollapsed && (
        <div className="ml-3 pl-3 border-l border-border/50 mt-0.5 space-y-0.5">
          {tabs.map((tab) => (
            <TabItem key={tab.key} group={group} tab={tab} />
          ))}
        </div>
      )}
    </div>
  );
}

function TabItem({ group, tab }: { group: NavGroup; tab: { key: string; label: string } }) {
  const activeGroup = useActiveGroup();
  const activeTab = useAppStore((s) => {
    const g = s.sidebar.activeGroup;
    const t = s.ui.currentTab || GROUP_TABS[g]?.[0]?.key;
    return t;
  });
  const { setActiveGroup } = useAppStore((s) => s.sidebar);
  const { setCurrentTab, toggleDetailPanel, detailPanelVisible } = useAppStore((s) => s.ui);
  const selectedNode = useSelectedNode();

  const isActive = activeGroup === group && (activeTab === tab.key || (!activeTab && tab.key === GROUP_TABS[group]?.[0]?.key));

  const handleClick = () => {
    setActiveGroup(group);
    setCurrentTab(tab.key);
    // Auto-show detail panel for certain tabs if a node is selected
    if (DETAIL_PANEL_TABS.has(tab.key) && selectedNode !== null && !detailPanelVisible) {
      toggleDetailPanel();
    }
  };

  const Icon = TAB_ICONS[tab.key];

  return (
    <button
      className={`w-full flex items-center gap-2 px-2 py-1 text-xs rounded transition-colors ${
        isActive
          ? 'bg-primary/10 text-primary font-medium'
          : 'text-muted-foreground hover:text-foreground hover:bg-muted/30'
      }`}
      onClick={handleClick}
      title={tab.label}
    >
      {Icon && <span className="shrink-0 opacity-60">{Icon}</span>}
      <span className="truncate">{tab.label}</span>
    </button>
  );
}

function NodeListItem() {
  const nodes = useNodes();
  const selectedNode = useSelectedNode();
  const { setActiveGroup } = useAppStore((s) => s.sidebar);
  const { setCurrentTab, toggleDetailPanel, detailPanelVisible } = useAppStore((s) => s.ui);

  if (nodes.length === 0) return null;

  return (
    <div className="mt-2 pt-2 border-t border-border/50">
      <div className="px-2 py-1 text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
        Discovered Nodes
      </div>
      <div className="space-y-0.5 px-1">
        {nodes.map((node) => {
          const isSelected = selectedNode === node.node_id;
          return (
            <button
              key={node.node_id}
              className={`w-full flex items-center gap-2 px-2 py-1 text-xs rounded transition-colors ${
                isSelected
                  ? 'bg-primary/10 text-primary'
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted/30'
              }`}
              onClick={() => {
                useAppStore.setState((s) => ({
                  can: { ...s.can, selectedNode: node.node_id },
                }));
                setActiveGroup('canopen');
                setCurrentTab('Nodes');
                if (!detailPanelVisible) toggleDetailPanel();
              }}
            >
              <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                node.nmt_state === 'Operational'
                  ? 'bg-green-500'
                  : node.nmt_state === 'PreOperational'
                    ? 'bg-yellow-500'
                    : 'bg-red-500'
              }`} />
              <span className="flex-1 text-left truncate">Node {node.node_id}</span>
              <span className="text-[10px] text-muted-foreground truncate">
                {node.nmt_state === 'Operational' ? 'OP' : node.nmt_state === 'PreOperational' ? 'Pre' : 'ERR'}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

export function Sidebar() {
  const connected = useConnected();
  const backendInfo = useAppStore((s) => s.can.backendInfo);
  const dialogBitrate = useAppStore((s) => s.connectionDialog.bitrate);
  const { show: showDialog } = useAppStore((s) => s.connectionDialog);
  const connectMutation = useConnectBackend();

  const formatBitrate = (bps: number) => {
    if (bps >= 1000000) return `${bps / 1000000}M`;
    return `${bps / 1000}k`;
  };

  const handleQuickConnect = () => {
    connectMutation.mutate({ backend_type: 'mock', channel: 'mock0', bitrate: dialogBitrate, node_id: 0 });
  };

  return (
    <div className="w-52 border-r bg-background flex flex-col shrink-0 overflow-hidden">
      {/* Connection status bar */}
      <div className="px-3 py-2 border-b border-border/50 bg-muted/30">
        {!connected ? (
          <div className="space-y-1.5">
            <button
              className="w-full flex items-center justify-center gap-1.5 px-2 py-1.5 text-xs font-medium bg-primary text-primary-foreground rounded hover:bg-primary/90 transition-colors"
              onClick={handleQuickConnect}
            >
              <span className="w-2 h-2 rounded-full bg-white/80" />
              Quick Connect
            </button>
            <button
              className="w-full flex items-center justify-center gap-1.5 px-2 py-1 text-xs border border-border rounded hover:bg-muted transition-colors text-muted-foreground"
              onClick={showDialog}
            >
              Configure...
            </button>
          </div>
        ) : (
          <div className="space-y-1">
            <div className="flex items-center gap-1.5 text-xs">
              <span className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
              <span className="text-foreground font-medium">Connected</span>
            </div>
            {backendInfo && (
              <div className="flex items-center gap-1 text-[10px] font-mono text-muted-foreground">
                <span>{backendInfo.backend_type}</span>
                <span>·</span>
                <span>{formatBitrate(backendInfo.bitrate)}</span>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Navigation groups */}
      <div className="flex-1 overflow-auto px-2 py-2">
        <NavGroupItem
          group="can"
          icon={GROUP_ICONS.can}
          label={GROUP_LABELS.can}
          tabs={GROUP_TABS.can}
        />
        <NavGroupItem
          group="canopen"
          icon={GROUP_ICONS.canopen}
          label={GROUP_LABELS.canopen}
          tabs={GROUP_TABS.canopen}
        />
        <NavGroupItem
          group="recording"
          icon={GROUP_ICONS.recording}
          label={GROUP_LABELS.recording}
          tabs={GROUP_TABS.recording}
        />
        <NavGroupItem
          group="eds"
          icon={GROUP_ICONS.eds}
          label={GROUP_LABELS.eds}
          tabs={GROUP_TABS.eds}
        />

        {/* Node list at bottom */}
        <NodeListItem />
      </div>

      {/* Quick NMT actions for selected node */}
      <QuickNodeActions />
    </div>
  );
}

function QuickNodeActions() {
  const selectedNode = useSelectedNode();
  const nmtMutation = useNmtCommand();

  if (selectedNode === null) return null;

  return (
    <div className="px-2 py-2 border-t border-border/50 bg-muted/20 space-y-1">
      <div className="px-2 py-0.5 text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
        Node {selectedNode} Actions
      </div>
      <div className="grid grid-cols-3 gap-1">
        <button
          className="px-2 py-1 text-[10px] rounded bg-green-500/10 text-green-500 border border-green-500/20 hover:bg-green-500/20 transition-colors"
          onClick={() => nmtMutation.mutate({ nodeId: selectedNode, command: 'start' })}
        >
          Start
        </button>
        <button
          className="px-2 py-1 text-[10px] rounded bg-red-500/10 text-red-500 border border-red-500/20 hover:bg-red-500/20 transition-colors"
          onClick={() => nmtMutation.mutate({ nodeId: selectedNode, command: 'stop' })}
        >
          Stop
        </button>
        <button
          className="px-2 py-1 text-[10px] rounded bg-muted border border-border hover:bg-muted/80 transition-colors"
          onClick={() => nmtMutation.mutate({ nodeId: selectedNode, command: 'reset' })}
        >
          Reset
        </button>
      </div>
    </div>
  );
}
