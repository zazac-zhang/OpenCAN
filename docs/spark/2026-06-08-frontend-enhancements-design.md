# OpenCAN 前端增强功能设计

> 日期: 2026-06-08
> 状态: 待实施
> 范围: DS402 交互式状态机、SDO 探索器、网络拓扑图

## 概述

在现有前端基础上添加三个创意性可视化功能，提升 CAN/CANOpen 调试体验：

1. **DS402 交互式状态机流程图** — SVG 流程图 + 控制字标注 + 一键发送
2. **SDO 交互式探索器** — 树形 OD 浏览 + 双击读写 + SDO 历史
3. **网络拓扑图** — SVG 节点图 + 连线动画 + 状态颜色

## 技术栈

- React 18 + TypeScript 5
- SVG（流程图和拓扑图）
- Zustand 5（状态管理）
- Tailwind CSS 3（样式）
- Tauri IPC（后端通信）

---

## 功能 1：DS402 交互式状态机流程图

### 目标

将 CiA 402 状态机以可视化流程图形式呈现，用户可以直接点击状态转换边来发送控制字 SDO 命令，实现"看图调试"。

### 状态机定义

CiA 402 定义了以下状态和转换：

**状态节点：**

| 状态 | StatusWord 条件 | 显示名 |
|------|----------------|--------|
| Not Ready to Switch On | (SW & 0x004F) == 0x0000 | 未准备好 |
| Switch On Disabled | (SW & 0x004F) == 0x0040 | 禁止切换 |
| Ready to Switch On | (SW & 0x006F) == 0x0021 | 准备就绪 |
| Switched On | (SW & 0x006F) == 0x0023 | 已切换 |
| Operation Enabled | (SW & 0x006F) == 0x0027 | 运行使能 |
| Quick Stop Active | (SW & 0x006F) == 0x0007 | 快速停止 |
| Fault Reaction Active | (SW & 0x004F) == 0x000F | 故障反应 |
| Fault | (SW & 0x004F) == 0x0008 | 故障 |

**状态转换边：**

| 转换 | 控制字 (0x6040) | 描述 |
|------|----------------|------|
| Shutdown | 0x0006 | 关闭 |
| Switch On | 0x0007 | 切换 |
| Enable Voltage | 0x0002 | 使能电压 |
| Disable Voltage | 0x0000 | 禁用电压 |
| Quick Stop | 0x0002 | 快速停止 |
| Enable Operation | 0x000F | 使能运行 |
| Disable Operation | 0x0007 | 禁用运行 |
| Fault Reset | 0x0080 | 故障复位 |

### UI 布局

```
┌──────────────────────────────────────────────────────────────┐
│ DS402 State Machine              [PP] [PV] [PT] [HM] [CSP]   │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐   Shutdown     ┌──────────────┐           │
│  │ Switch On    │───────────────→│ Ready to     │           │
│  │ Disabled     │   CW=0x0006   │ Switch On    │           │
│  │              │←───────────────│              │           │
│  └──────────────┘   Disable      └──────┬───────┘           │
│                         Voltage         │                    │
│                                    Switch On                │
│                                    CW=0x0007                │
│                                         │                    │
│                                         ▼                    │
│  ┌──────────────┐  Disable Op    ┌──────────────┐           │
│  │ Operation    │───────────────→│ Switched     │           │
│  │ Enabled      │   CW=0x0007   │ On           │           │
│  │  ★ 当前状态  │←───────────────│              │           │
│  │  (绿色脉冲)  │  Enable Op     └──────────────┘           │
│  └──────────────┘   CW=0x000F                               │
│         │                                                    │
│         │ Quick Stop                                         │
│         │ CW=0x0002                                          │
│         ▼                                                    │
│  ┌──────────────┐                                            │
│  │ Quick Stop   │                                            │
│  │ Active       │                                            │
│  └──────────────┘                                            │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│ 当前状态: Operation Enabled                                    │
│ StatusWord: 0x0027  ControlWord: 0x000F                      │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ bit 0: Ready to Switch On    ✅                          │ │
│ │ bit 1: Switched On           ✅                          │ │
│ │ bit 2: Operation Enabled     ✅                          │ │
│ │ bit 3: Fault                 ❌                          │ │
│ │ ...                                                      │ │
│ └──────────────────────────────────────────────────────────┘ │
│ [Shutdown] [Switch On] [Enable] [Quick Stop] [Fault Reset]   │
└──────────────────────────────────────────────────────────────┘
```

### 交互行为

1. **当前状态高亮**：绿色背景 + 脉冲动画（CSS keyframes）
2. **可点击边**：鼠标悬停时高亮 + 显示控制字 tooltip
3. **点击边** → 调用 `sdoDownload({ node_id, index: 0x6040, subindex: 0, data: controlWord })`
4. **操作模式切换**：顶部标签切换 PP/PV/PT/HM/CSP/CSV/CST
   - 不同模式的流程图略有差异（如 HM 模式有额外的 Homing 状态）
5. **StatusWord 实时解析**：底部面板显示 14 个位的状态
6. **快捷按钮**：直接发送常用控制字（Shutdown/Switch On/Enable/Fault Reset）

### 数据流

```
Tauri Event Stream (ds402_state)
  → Zustand store (ds402.nodeStates[nodeId])
  → DS402StateMachine component
  → StatusWord 解析 → 当前状态高亮
  → 点击边 → sdoDownload → Tauri IPC
```

### 文件结构

```
frontend/src/components/ds402/
├── StateMachineFlow.tsx    # 新：SVG 流程图组件
├── StateNode.tsx           # 新：状态节点组件
├── TransitionEdge.tsx      # 新：转换边组件
├── StatusWordPanel.tsx     # 新：StatusWord 位级解析面板
├── ControlWordButtons.tsx  # 新：快捷控制按钮
├── StateMachine.tsx        # 已有：保留或替换
├── ControlPanel.tsx        # 已有：保留
├── ModeSelector.tsx        # 已有：保留
└── WaveformDisplay.tsx     # 已有：保留
```

---

## 功能 2：SDO 交互式探索器

### 目标

提供类似 IDE 变量监视窗口的 OD 浏览和读写体验，支持树形浏览、双击读写、批量操作、SDO 历史记录。

### UI 布局

```
┌────────────────────────┬──────────────────────────────────┐
│ Object Dictionary      │ Entry Detail                      │
├────────────────────────┤──────────────────────────────────┤
│ 🔍 Search: [________]  │ Index: 0x6040                     │
│                        │ SubIndex: 0x00                    │
│ ▼ 1000 Communication   │ Name: Control Word                │
│   ├─ 0 Device Type     │ Object Type: VAR                  │
│   │   RO UNSIGNED32    │ Data Type: UNSIGNED16             │
│   │   Value: 0x00020192│ Access: RW                        │
│   ├─ 1 Error Register  │                                   │
│   │   RO UNSIGNED8     │ ┌─ Read Value ──────────────────┐ │
│   └─ 3 Predefined Error│ │                               │ │
│ ▼ 1018 Identity        │ │  Value: [0x000F        ]      │ │
│   ├─ 0 Number of Ent.  │ │  [Read] [Write] [Refresh]     │ │
│   ├─ 1 Vendor ID       │ │                               │ │
│   ├─ 2 Product Code    │ │  Hex: 0x000F                  │ │
│   └─ 3 Revision        │ │  Bin: 0000 0000 0000 1111     │ │
│ ▼ 6000 DS402 Profile   │ │  Dec: 15                      │ │
│   ├─ 6040 ControlWord  │ │  Bit 0: Switch On ✅          │ │
│   │   RW UNSIGNED16    │ │  Bit 1: Enable Voltage ✅     │ │
│   │   Value: [Read]    │ │  Bit 2: Quick Stop ✅         │ │
│   ├─ 6041 StatusWord   │ │  Bit 3: Enable Operation ✅   │ │
│   │   RO UNSIGNED16    │ └───────────────────────────────┘ │
│   └─ 6060 Mode of Op   │                                   │
│                        │ SDO History                       │
│ [Refresh All] [Export] │ ┌───────────────────────────────┐ │
│                        │ │ 10:32:01.123 R 6041:0 → 0x0027│ │
│                        │ │ 10:32:00.456 W 6040:0 ← 0x000F│ │
│                        │ │ 10:31:59.789 R 1000:0 → 0x0002│ │
│                        │ └───────────────────────────────┘ │
└────────────────────────┴──────────────────────────────────┘
```

### 交互行为

1. **树形浏览**：
   - 按 Index 范围分组（1000=Communication, 2000=Manufacturer, 6000=DS402）
   - 展开/折叠 + 显示数据类型和访问权限
   - 搜索过滤（按名称或 Index）

2. **双击读写**：
   - 双击树节点 → 右侧显示详情面板
   - 点击 [Read] → 调用 `sdoUpload({ node_id, index, subindex, data_type })`
   - 输入值后点击 [Write] → 调用 `sdoDownload({ node_id, index, subindex, data })`
   - 自动解析数据类型（UNSIGNED16 → 2 字节小端）

3. **值解析**：
   - 同时显示 Hex/Bin/Dec
   - 对于 StatusWord/ControlWord 等已知对象，显示位级解析
   - 对于 VISIBLE_STRING，直接显示文本

4. **SDO 历史**：
   - 记录所有 SDO 读写操作
   - 显示时间戳、方向(R/W)、Index:Subindex、值
   - 支持重放（双击历史条目重新发送）
   - 支持导出为 CSV

5. **批量操作**：
   - 选中多个条目 → [Read All] 一键读取
   - 常用对象快速访问面板

### 数据流

```
用户点击 [Read]
  → sdoUpload({ node_id, index, subindex, data_type })
  → Tauri IPC → Rust SdoClient
  → 响应 → 更新详情面板
  → 添加到 SDO 历史

用户输入值 + 点击 [Write]
  → 解析输入 → 转换为 byte[]
  → sdoDownload({ node_id, index, subindex, data })
  → Tauri IPC → Rust SdoClient
  → 确认 → 更新历史
```

### 文件结构

```
frontend/src/pages/CANOpen/
├── SdoExplorer.tsx         # 新：SDO 探索器主页面
├── OdTreeView.tsx          # 新：树形 OD 浏览
├── EntryDetail.tsx         # 新：条目详情/编辑面板
├── SdoHistory.tsx          # 新：SDO 历史记录
└── ValueEditor.tsx         # 新：值编辑器（多格式）

frontend/src/lib/
└── od-utils.ts             # 新：OD 工具函数（数据类型转换等）
```

---

## 功能 3：网络拓扑图

### 目标

以可视化拓扑图形式展示 CANopen 网络中的节点及其状态，支持交互式操作。

### UI 布局

```
┌──────────────────────────────────────────────────────────┐
│ Network Topology                   [Scan] [Auto Layout]   │
├──────────────────────────────────────────────────────────┤
│                                                          │
│                     ┌─────────┐                          │
│                     │ Master  │                          │
│                     │ Node 1  │                          │
│                     │ 🟢 Op   │                          │
│                     └────┬────┘                          │
│                          │                               │
│         ┌────────────────┼────────────────┐              │
│         │                │                │              │
│    ┌────┴────┐     ┌─────┴─────┐    ┌─────┴─────┐       │
│    │ Slave   │     │ Slave     │    │ Slave     │       │
│    │ Node 2  │     │ Node 3    │    │ Node 5    │       │
│    │ 🟢 Op   │     │ 🟡 PreOp  │    │ 🔴 Stop   │       │
│    │ DS402   │     │ DS402     │    │ DS402     │       │
│    └────┬────┘     └─────┬─────┘    └───────────┘       │
│         │                │                               │
│    ┌────┴────┐     ┌─────┴─────┐                         │
│    │ Slave   │     │ Slave     │                         │
│    │ Node 4  │     │ Node 6    │                         │
│    │ 🟢 Op   │     │ ⚫ Offline│                         │
│    └─────────┘     └───────────┘                         │
│                                                          │
├──────────────────────────────────────────────────────────┤
│ 节点详情: Node 3 (PreOperational)                         │
│ Vendor: 0x00000123  Product: 0x00020192                  │
│ [NMT Start] [NMT Stop] [NMT Reset] [View Detail]         │
└──────────────────────────────────────────────────────────┘
```

### 节点表示

每个节点用圆角矩形表示：

```
┌─────────────────┐
│  Node {id}      │  ← 节点 ID
│  {device_name}  │  ← 设备名称（来自 EDS）
│  {nmt_state}    │  ← NMT 状态
│  ● {color}      │  ← 状态指示灯
└─────────────────┘
```

**状态颜色：**
- 🟢 绿色：Operational
- 🟡 黄色：PreOperational
- 🔴 红色：Stopped
- ⚫ 灰色：Offline / Unknown
- 🔵 蓝色：当前选中节点

### 交互行为

1. **节点点击**：
   - 点击节点 → 高亮 + 右侧面板显示节点详情
   - 双击节点 → 导航到 NodeDetail 页面

2. **连线动画**：
   - PDO 通信时：TPDO/RPDO 连线闪烁蓝色
   - SDO 通信时：SDO 连线闪烁绿色
   - EMCY 事件时：节点红色闪烁

3. **布局算法**：
   - 默认：Master 在顶部，Slaves 在下方圆形排列
   - 支持拖拽重新排列
   - [Auto Layout] 按钮重新排列

4. **扫描功能**：
   - [Scan] 按钮 → 调用 `scanNodes()`
   - 扫描过程中节点逐个出现（动画）
   - 扫描完成更新拓扑

5. **NMT 控制**：
   - 选中节点后底部显示 NMT 命令按钮
   - Start / Stop / Pre-Op / Reset

### SVG 渲染

- 使用 SVG 渲染拓扑图（不依赖第三方库）
- 节点：`<rect>` + `<text>` + 状态指示灯 `<circle>`
- 连线：`<line>` 或 `<path>` + 动画 `<animate>`
- 视口：支持缩放和平移（鼠标滚轮 + 拖拽）

### 数据流

```
Tauri Event Stream (heartbeat / emcy)
  → Zustand store (heartbeat.entries / emcy.entries)
  → NetworkTopology component
  → 节点状态更新 → 颜色变化
  → 连线动画触发

scanNodes()
  → Tauri IPC → Rust NodeManager
  → 返回 node_id[]
  → 更新 store (can.nodes)
  → 拓扑图重新布局
```

### 文件结构

```
frontend/src/pages/CANOpen/
├── NetworkTopology.tsx     # 新：网络拓扑图主页面
├── TopologyCanvas.tsx      # 新：SVG 画布组件
├── TopologyNode.tsx        # 新：节点组件
├── TopologyEdge.tsx        # 新：连线组件
└── NodeInfoPanel.tsx       # 新：节点信息面板

frontend/src/lib/
└── topology-layout.ts      # 新：自动布局算法
```

---

## 集成方式

### 路由集成

三个新功能分别集成到现有页面组：

| 功能 | 导航组 | Tab 名 | 说明 |
|------|--------|--------|------|
| DS402 状态机 | canopen | DS402 | 替换现有 Ds402Control 页面 |
| SDO 探索器 | canopen | SDO | 新增 Tab |
| 网络拓扑图 | canopen | Network | 替换现有 NetworkOverview 页面 |

### Store 扩展

```typescript
// 新增 store slice
interface SdoExplorerState {
  selectedEntry: { index: number; subindex: number } | null;
  readCache: Map<string, { value: number[]; timestamp: number }>;
  history: SdoHistoryEntry[];
  setSelectedEntry: (entry: { index: number; subindex: number } | null) => void;
  cacheRead: (key: string, value: number[]) => void;
  addHistory: (entry: SdoHistoryEntry) => void;
  clearHistory: () => void;
}
```

### Tauri 命令依赖

| 功能 | 需要的 Tauri 命令 | 状态 |
|------|-------------------|------|
| DS402 状态机 | `ds402_enable`, `ds402_fault_reset`, `sdo_download` | ✅ 已有 |
| SDO 探索器 | `sdo_upload`, `sdo_download`, `get_od_entries` | ✅ 已有 |
| 网络拓扑图 | `scan_nodes`, `nmt_command` | ✅ 已有 |

所有需要的后端命令已就绪，无需新增 Tauri 命令。

---

## 实施顺序

1. **Phase A: DS402 状态机流程图**（最高优先级）
   - 替换现有 Ds402Control 页面
   - SVG 流程图 + 控制字边 + 点击发送
   - StatusWord 位级解析面板

2. **Phase B: SDO 交互式探索器**
   - 新增 SdoExplorer 页面
   - 树形 OD 浏览 + 读写操作
   - SDO 历史记录

3. **Phase C: 网络拓扑图**
   - 替换现有 NetworkOverview 页面
   - SVG 节点图 + 连线动画
   - 节点交互 + NMT 控制

---

## 风险和注意事项

1. **SVG 性能**：节点数量超过 50 时可能需要虚拟化渲染
2. **状态同步**：确保 StatusWord 实时更新频率不超过 50ms
3. **SDO 超时**：读写操作需要显示 loading 状态和超时处理
4. **布局算法**：自动布局需要处理节点重叠和连线交叉
5. **移动端适配**：SVG 交互在触摸设备上需要特殊处理（暂不考虑）
