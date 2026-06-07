# OpenCAN GUI Layout Redesign

**Date**: 2026-06-07
**Status**: Draft — pending implementation plan

## Context

OpenCAN is a CAN/CANOpen debugging tool with a Tauri 2 + React frontend. The current layout suffers from:

- Large empty space below the frame table in Frame Monitor
- Sparse left sidebar with only node list + NMT buttons
- No send frame panel
- No bus load visualization
- No node detail panel
- Flat visual hierarchy — TopBar is overloaded with 15+ buttons
- TopBar Tab system (CAN Bus / CANOpen / Recording / Settings) exists but content doesn't fill the space

**Tech stack**: Tauri 2 + React 18 + TypeScript + Tailwind CSS 3 + Zustand

**Reference tools**: CANalyzer, PCAN-View, Vector CANoe

## Architecture

### Overall Layout

```
┌──────────────────────────────────────────────────────────────────┐
│ TopBar (minimal: logo, connection status, theme toggle)           │
────────┬────────────────────────────────────────────────────────┤
│ Left   │     Main Content Area      │   Right Detail Panel       │
│ Sidebar│  (context-dependent Tabs)  │   (node-driven, collapsible)│
│ ~200px │                            │   ~320px                    │
│        │                            │                             │
│ Groups │  Tab content fills space   │   Accordion sections:       │
│ + Nav  │                            │   Overview / SDO / DS402    │
│        │                            │                             │
│ Nodes  │                            │                             │
│ list   │                            │                             │
├────────┴────────────────────────────┴────────────────────────────┤
│ Bottom Panel (context-aware, collapsible, resizable ~150px)       │
──────────────────────────────────────────────────────────────────┤
│ StatusBar (frame count, RX/TX, rate, status message)             │
└──────────────────────────────────────────────────────────────────┘
```

### Left Sidebar (~200px)

Three-layer structure:

1. **Header**: Logo + connection status (one compact line showing backend type + bitrate)
2. **Navigation groups** (collapsible):
   - **CAN Bus**: Frames, Send, Statistics, Errors
   - **CANOpen**: Network, Nodes, PDO, DS402, EMCY, Heartbeat, SYNC
   - **Recording**: Record, Playback
   - **EDS**: EDS Files, OD Browser
3. **Node list** at the bottom — each node shows state indicator + NMT state. Clicking a node auto-switches main content to "Nodes" Tab and opens the right detail panel.
4. **EDS file list** embedded in the EDS navigation group — shows loaded files with quick view/remove actions.

**Key decisions**:
- Remove the top-level TabBar — the sidebar handles all navigation
- Each left-nav group has its own independent set of secondary Tabs in the main content area
- Switching left-nav groups does NOT preserve Tab state of other groups (user rarely jumps back)
- Bottom panel Tabs switch with the left-nav group (context-aware)

### Main Content Area

Each left-nav group activates a specific set of secondary Tabs:

#### CAN Bus group

| Tab | Description |
|-----|-------------|
| **Frames** (default) | Real-time frame table + send panel in a split layout (65/35 ratio, draggable divider). Frame type auto-decoding (SYNC/HB/TPDO/SDO/NMT/EMCY) with color-coded labels. Long frames (DLC>8) wrap to next line. Filter bar below the table (compact). |
| **Send** | Raw frame send (COB-ID, DLC, data with Hex/ASCII/DEC toggle), send history (quick-replay), cyclic send list, SDO Quick Access (node selector, index/sub, read/write). |
| **Statistics** | Bus load percentage, error count, frame rate, uptime cards. Load-over-time chart (60s window). Frame distribution by type bar chart. Per-COB-ID stats table (count, type, avg/min interval). |
| **Errors** | Error frame list with time, COB-ID, error type, description. Filter by error type. TECount/RECount display. |

#### CANOpen group

| Tab | Description |
|-----|-------------|
| **Network** (default) | NMT quick actions (Start/Stop/Reset All + Scan). Node status cards grid (state indicator, device type, HB rate, detail link). Node state timeline chart (last 60s per node). |
| **Nodes** | Node table (ID, name, NMT state, device type, vendor, last HB). Clicking a row opens the right detail panel. |
| **PDO** | TPDO/RPDO sub-tabs. Per-node filter. Table with node, PDO name, cycle time, mapped objects (with EDS decoding), data payload. Active count + rate. |
| **DS402** | Node selector. Left: DS402 state machine diagram. Right: control panel (mode selector, target position/velocity/torque inputs, execute buttons, actual values, CW/SW hex display with decode buttons). Quick stop + fault reset. |
| **EMCY** | Emergency message list with timestamp, node, error code, error register, manufacturer-specific data. Error code decoder. |
| **Heartbeat** | Node heartbeat status table (node, state, last HB time, timeout indicator). |
| **SYNC** | SYNC producer config (period, COB-ID, counter usage). SYNC consumer table (node, expected period, jitter). |

#### Recording group

| Tab | Description |
|-----|-------------|
| **Record** (default) | Session name, start/stop controls, elapsed time, output file path. Filter selector, max size limit, auto-split. Stats: recorded frames, size, rate. |
| **Playback** | File loader, transport controls (play/pause/step/skip), speed selector, loop toggle, timeline scrubber with current position marker. Frame counter + time display. |

#### EDS group

| Tab | Description |
|-----|-------------|
| **EDS Files** (default) | Load EDS button. List of loaded EDS files with device name, vendor, object count, view/remove actions. |
| **OD Browser** | Source EDS selector. Filter input. Object dictionary tree table (index, name, type, value). Sub-index expansion for RECORD types. Read-on-demand for live values from connected nodes. |

### Bottom Panel

Context-aware: Tab set changes with the active left-nav group. Default height ~150px, collapsible, height-resizable via drag.

| Active Group | Bottom Tabs | Default Tab | Content |
|-------------|-------------|-------------|---------|
| CAN Bus | Signals, Bus Load, Error Log, Timing | Bus Load | Real-time load curve (60s), or selected signals, or compact error log, or timing jitter analysis |
| CANOpen | PDO Stream, EMCY, DS402 State, Heartbeat | PDO Stream | Decoded PDO data table, or EMCY list, or per-node DS402 state summary, or HB timeline |
| Recording | Session Info, Timeline | Session Info | Session metadata, or timeline visualization |
| EDS | OD Entries, Parse Log | OD Entries | Object dictionary entries summary, or EDS parse warnings/errors |

**Behavior**:
- Default collapsed state
- Expand/collapse toggle on the bar
- Tab state resets when switching left-nav groups (always starts from default Tab)
- Collapse state persists across group switches

### Right Detail Panel (~320px)

Opens automatically when a node is selected from the sidebar node list or the Nodes table. Closes with × button or by clicking elsewhere.

Three accordion sections:

1. **Overview** — NMT state with start/stop/reset buttons, device type, vendor ID, error register, manufacturer device name (from EDS)
2. **SDO Quick Read** — Pre-populated common objects (0x1000 Device Type, 0x1008 Manufacturer, 0x1018 Vendor ID) with one-click read. Below that, a free-form SDO Read/Write (index, sub-index, value input).
3. **DS402 Control** — Mode selector (CSP/CST/CSV/PP/PV/PT/Homing), target value inputs with execute button, status word display, quick stop + fault reset. Only shown for DS402 device types.

### TopBar (minimal)

Reduced to essential global controls only:
- Logo / app name
- Connection status indicator (connected/disconnected + backend + bitrate)
- Theme toggle
- Settings shortcut

All action buttons (Pause, Clear, Export, Import, EDS load, NMT actions) are moved to their respective pages.

### StatusBar

Single-line bar at the very bottom:
- Total frame count
- RX/TX split
- Current frame rate
- Status message (right-aligned)

## Data Flow

- Frame data flows from Tauri backend → Zustand store → all consumers (Frames table, bottom panel, statistics)
- Node selection (sidebar click) → Zustand state → right panel opens with node context
- Left-nav group change → Tab set changes → bottom panel Tab set changes
- Bottom panel Tab change → renders corresponding compact view
- Send actions → Tauri command → frame injected into bus → appears in Frames table

## Implementation Notes

### Components to create/restructure

| Component | Action | Location |
|-----------|--------|----------|
| `Sidebar` | Restructure with collapsible groups + nav items + node list | `components/layout/Sidebar.tsx` |
| `TopBar` | Strip down to minimal elements | `components/layout/TopBar.tsx` |
| `TabBar` | Remove — navigation moves to sidebar | `components/layout/TabBar.tsx` → delete |
| `DetailPanel` | Add accordion sections for SDO/DS402 | `components/layout/DetailPanel.tsx` |
| `BottomPanel` | New component — context-aware tabbed panel | `components/layout/BottomPanel.tsx` |
| `FrameMonitor` | Restructure: table 65% + send panel 35% split | `pages/CAN/FrameMonitor.tsx` |
| `SendPanel` | New page or merge into FrameMonitor | `pages/CAN/SendPanel.tsx` |
| `BusStatistics` | Enhance with charts and per-COB-ID table | `pages/CAN/BusStatistics.tsx` |
| `ErrorFrames` | Enhance with error type filter and TECount/RECount | `pages/CAN/ErrorFrames.tsx` |
| `NetworkOverview` | Add node cards grid + timeline chart | `pages/CANOpen/NetworkOverview.tsx` |
| `NodeDetail` | Table view + right panel trigger | `pages/CANOpen/NodeDetail.tsx` |
| `PdoMonitor` | TPDO/RPDO sub-tabs + EDS decoding | `pages/CANOpen/PdoMonitor.tsx` |
| `Ds402Control` | State machine diagram + control panel | `pages/CANOpen/Ds402Control.tsx` |
| `EmcyMonitor` | Error code decoder | `pages/CANOpen/EmcyMonitor.tsx` |
| `HeartbeatMonitor` | Timeout indicators | `pages/CANOpen/HeartbeatMonitor.tsx` |
| `SyncManagement` | Producer config + consumer table | `pages/CANOpen/SyncManagement.tsx` |
| `EdsManagement` | File list with view/remove | `pages/Settings/EdsManagement.tsx` |
| `SessionRecorder` | Session controls + stats | `pages/Recording/SessionRecorder.tsx` |
| `SessionPlayer` | Transport controls + timeline | `pages/Recording/SessionPlayer.tsx` |

### Store changes

- Add `sidebar.activeGroup` to track current left-nav group
- Add `bottomPanel.visible` and `bottomPanel.activeTab` 
- Add `sidebar.groupsCollapsed` map for group collapse state
- Tab routing: instead of flat `ui.currentTab`, use `{ group: string, tab: string }` to support group-specific Tab sets

### Dependencies

- `lucide-react` — already used for icons
- `recharts` — already in package.json for charts
- No new external dependencies needed

## Naming Conventions

Tab names follow industrial tool conventions (CANalyzer, PCAN-View):

- Short nouns, no ampersands, no concatenated names
- English throughout (UI is English)
- CAN Bus: Frames, Send, Statistics, Errors
- CANOpen: Network, Nodes, PDO, DS402, EMCY, Heartbeat, SYNC
- Recording: Record, Playback
- EDS: EDS Files, OD Browser
