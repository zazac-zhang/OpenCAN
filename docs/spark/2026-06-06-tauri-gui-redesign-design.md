# OpenCAN Tauri GUI 重构设计

**日期**: 2026-06-06
**状态**: Draft — awaiting approval

---

## 目标

将 OpenCAN 的 iced GUI (`crates/gui/`) 完全替换为 Tauri v2 + React 前端架构，保留所有现有协议栈功能（CAN 帧监控、CANOpen 协议操作、DS402 运动控制、EMCY/Heartbeat/SYNC 监控、EDS 加载），同时利用前端生态增加 DBC 信号解码和会话录制/回放功能。

---

## 1. 技术选型

| 层 | 选型 | 理由 |
|---|---|---|
| 桌面框架 | Tauri v2 | 跨平台，Rust 原生后端，包体积小 |
| 前端框架 | React + Vite + TypeScript | 生态最成熟 |
| UI 组件 | shadcn/ui + Tailwind CSS | 高度可定制，社区主流 |
| 图标 | Lucide React | 风格统一，轻量 |
| 状态管理 | Zustand | 轻量，selector 精确订阅 |
| 数据获取 | @tanstack/react-query | 异步命令的 loading/error/retry 管理 |
| 表格 | @tanstack/react-table + @tanstack/react-virtual | 行业标准，支持虚拟滚动 |
| 图表 | lightweight-charts | 高频实时数据流专用（DS402 波形） |
| 表单 | react-hook-form + zod | 性能优于受控组件，类型安全 |
| DBC 解析 | candbc (Rust) | Rust 生态成熟的 DBC 解析库 |
| PCAP 导出 | pcap (Rust) | Wireshark 兼容格式 |

---

## 2. 工作区结构

```
OpenCAN/
├── crates/
│   ├── can-traits/           # 硬件抽象层（不变）
│   ├── canopen-core/         # 协议核心（不变）
│   ── canopen-ds301/        # DS301 协议栈 + DS402（不变）
├── frontend/                 # React + Vite + TypeScript
│   ├── src/
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── index.html
├── opencan-gui/              # Tauri v2 桥接层
│   ├── src-tauri/
│   │   ├── src/
│   │   │   ├── main.rs           # Tauri app 入口
│   │   │   ├── state.rs          # 全局 App 状态
│   │   │   ├── channels/         # EventChannel 定义
│   │   │   └── commands/         # Tauri Commands 按领域分类
│   │   ├── capabilities/         # 权限配置
│   │   ├── icons/                # 应用图标
│   │   ├── Cargo.toml            # 依赖底层 crates + tauri v2
│   │   ├── tauri.conf.json       # distDir: "../frontend/dist"
│   │   └── build.rs
│   └── Cargo.toml              # workspace member
├── docs/
│   └── spark/
├── Cargo.toml                  # workspace = ["crates/*", "opencan-gui"]
└── .github/workflows/ci.yml    # 新增 Tauri 构建 job
```

**关键变更**:
- 删除 `crates/gui/` 整个目录
- 新建 `frontend/`（与 crates 同级）存放 React 源码
- 新建 `opencan-gui/`（与 crates 同级）作为 Tauri 桥接层
- `Cargo.toml` workspace 成员增加 `opencan-gui`

**构建流程**:
- 开发: `cargo tauri dev`（同时启动 Vite dev server + Tauri debug 窗口）
- 生产: `cd frontend && npm run build`，再 `cargo tauri build`
- 前端独立开发: `cd frontend && npm run dev`（通过 mock adapter 模拟 Tauri API）

---

## 3. Tauri 接口设计

### Commands（离散操作）

文件分类：

```
src-tauri/src/commands/
  mod.rs              # 汇总导出，注册到 Tauri app
  connection.rs       # connect_backend, disconnect, get_backends
  nmt.rs              # scan_nodes, nmt_command
  sdo.rs              # sdo_upload, sdo_download
  ds402.rs            # ds402_enable, fault_reset, set_mode, set_target
  pdo.rs              # read_pdo_mapping
  sync.rs             # start_sync, stop_sync
  eds.rs              # load_eds_file
  recording.rs        # start_recording, stop_recording, load_recording, start_playback, stop_playback
```

Command 列表：

| Command | 参数 | 返回 | 用途 |
|---|---|---|---|
| `connect_backend` | `backend_type, channel, bitrate, node_id` | `Result<BackendInfo, String>` | 连接 CAN 硬件 |
| `disconnect` | — | `Result<(), String>` | 断开连接 |
| `get_backends` | — | `Vec<BackendDescriptor>` | 获取可用后端列表 |
| `scan_nodes` | `timeout_ms` | `Result<Vec<u8>, String>` | 节点扫描 |
| `nmt_command` | `node_id, command` | `Result<(), String>` | NMT 控制 |
| `sdo_upload` | `node_id, index, subindex, data_type` | `Result<SdoResult, String>` | SDO 读 |
| `sdo_download` | `node_id, index, subindex, data` | `Result<(), String>` | SDO 写 |
| `ds402_enable` | `node_id` | `Result<(), String>` | DS402 使能 |
| `ds402_fault_reset` | `node_id` | `Result<(), String>` | DS402 故障复位 |
| `ds402_set_mode` | `node_id, mode` | `Result<(), String>` | DS402 模式切换 |
| `ds402_set_target` | `node_id, mode, target` | `Result<(), String>` | 目标位置/速度/扭矩 |
| `read_pdo_mapping` | `node_id, pdo_index` | `Result<PdoMapping, String>` | 读取 PDO 映射 |
| `start_sync` | `period_us` | `Result<(), String>` | 启动 SYNC 生产者 |
| `stop_sync` | — | `Result<(), String>` | 停止 SYNC 生产者 |
| `load_eds_file` | `path` | `Result<EdsInfo, String>` | 加载 EDS |
| `start_recording` | `path` | `Result<(), String>` | 开始录制 |
| `stop_recording` | — | `Result<(), String>` | 停止录制 |
| `load_recording` | `path` | `Result<RecordingMeta, String>` | 加载录制文件 |
| `start_playback` | `speed` | `Result<(), String>` | 开始回放 |
| `stop_playback` | — | `Result<(), String>` | 停止回放 |

### Event Channels（连续数据流）

```
src-tauri/src/channels/
  mod.rs              # Channels 结构体定义
  frame.rs            # CanFrameEvent
  pdo.rs              # PdoEvent
  log.rs              # LogEvent
  emcy.rs             # EmcyEvent
  heartbeat.rs        # HeartbeatEvent
  ds402.rs            # Ds402StateEvent
  bus_stats.rs        # BusStatsEvent
```

Channel 列表：

| Channel | 事件类型 | 频率 | 用途 |
|---|---|---|---|
| `frame_stream` | `CanFrameEvent` | 高（批量 50ms） | 帧监控 |
| `pdo_stream` | `PdoEvent` | 中（批量 100ms） | PDO 监控 |
| `log_stream` | `LogEvent` | 低（实时） | 操作日志 |
| `emcy_stream` | `EmcyEvent` | 低（实时） | EMCY 监控 |
| `heartbeat_stream` | `HeartbeatEvent` | 低（实时） | 心跳监控 |
| `bus_stats_stream` | `BusStatsEvent` | 中（~1/s） | 总线统计 |
| `ds402_state_stream` | `Ds402StateEvent` | 中（实时） | DS402 实时状态 |

### 高频数据批处理

帧流每 50ms 批量推送（最多 100 帧/批），PDO 每 100ms 批量推送，其他实时推送。避免事件风暴导致前端卡顿。

### Rust ↔ TS 类型对应

Rust 侧通过 `serde::Serialize` 序列化，TypeScript 侧通过 Zod schema 校验。所有 Tauri 返回数据必须经过 Zod 校验。

---

## 4. Rust 后端设计

### 全局状态

```rust
// opencan-gui/src-tauri/src/state.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use opencan_canopen_ds301::CanopenStack;

pub struct AppState {
    pub stack: Option<CanopenStack>,
    pub backend_info: Option<BackendInfo>,
    pub node_registry: NodeRegistry,
    pub recording: Option<SessionRecorder>,
}

pub type SharedState = Arc<RwLock<AppState>>;
```

通过 `tauri::State<SharedState>` 注入到所有 commands。

### 与协议栈集成

Tauri 后端创建 `CanDriverAdapter` 桥接 `CanBus` → `CanDriver`，然后创建 `CanopenStack`。主循环处理协议事件并推送到对应的 EventChannel。

### Commands 执行模型

所有 commands 通过 `AppHandle` 获取 `SharedState`，对 `stack` 执行操作后返回结果。错误统一转换为 `String` 返回前端。

---

## 5. 前端组件架构

```
frontend/src/
├── main.tsx                          # 入口，挂载 Tauri Provider
├── App.tsx                           # 根布局：TopBar + Sidebar + Content + StatusBar
├── lib/
│   ├── tauri.ts                      # invoke/listen 封装 + 类型定义
│   ├── utils.ts                      # clsx + twMerge
│   ├── store.ts                      # Zustand store 定义
│   ── schemas.ts                    # Zod 校验 schemas
── components/
│   ├── layout/
│   │   ├── TopBar.tsx                # 连接、比特率、NMT 快捷操作、录制控制
│   │   ├── Sidebar.tsx               # 节点列表面板
│   │   ├── StatusBar.tsx             # 底部状态栏
│   │   ├── TabBar.tsx                # 主/次 Tab 导航
│   │   └── DetailPanel.tsx           # 右侧可折叠详情面板
│   ├── common/
│   │   ├── DataTable.tsx             # 通用虚拟滚动表格
│   │   ├── Waveform.tsx              # lightweight-charts 封装
│   │   ├── NodeCard.tsx              # 节点状态卡片
│   │   ├── ConnectionDialog.tsx      # 连接配置弹窗
│   │   └── EdsLoader.tsx             # EDS 文件加载
│   ├── sdo/
│   │   ├── SdoEditor.tsx             # SDO 读写表单
│   │   ├── DataTypeSelector.tsx      # 数据类型选择器
│   │   ── QuickRead.tsx             # 常用地址快捷读取
│   ├── ds402/
│   │   ├── StateMachine.tsx          # DS402 状态机可视化
│   │   ├── ModeSelector.tsx          # 操作模式选择
│   │   ├── ControlPanel.tsx          # 运动控制面板
│   │   └── WaveformDisplay.tsx       # 实时波形叠加显示
│   └── can/
│       ├── FrameTable.tsx            # 帧监控表格
│       ├── BusStatsCards.tsx         # 总线统计卡片
│       └── ErrorFrameList.tsx        # 错误帧列表
├── pages/
│   ├── CAN/
│   │   ├── FrameMonitor.tsx          # 帧监控页
│   │   ├── BusStatistics.tsx         # 总线统计页
│   │   └── ErrorFrames.tsx           # 错误帧页
│   ├── CANOpen/
│   │   ├── NetworkOverview.tsx       # 网络概览
│   │   ├── NodeDetail.tsx            # 节点详情
│   │   ├── PdoMonitor.tsx            # PDO 监控
│   │   ├── Ds402Control.tsx          # DS402 控制
│   │   ├── EmcyMonitor.tsx           # EMCY 监控
│   │   ├── HeartbeatMonitor.tsx      # 心跳监控
│   │   └── SyncManagement.tsx        # 同步管理
│   ├── Settings/
│   │   ├── ConnectionSettings.tsx    # 连接设置
│   │   └── EdsManagement.tsx         # EDS 管理
│   └── Recording/                    # 第一阶段新增
│       ├── SessionRecorder.tsx       # 录制控制面板
│       └── SessionPlayer.tsx         # 回放控制面板
├── hooks/
│   ├── useFrameStream.ts             # 帧流订阅 + 节流
│   ├── usePdoStream.ts               # PDO 流订阅
│   ├── useCommands.ts                # Commands 封装（react-query）
│   └── useRecording.ts               # 录制/回放状态
├── types/
│   ├── can.ts                        # CAN 帧相关类型
│   ├── canopen.ts                    # CANOpen 协议类型
│   ├── ds402.ts                      # DS402 状态类型
│   └── recording.ts                  # 录制相关类型
└── assets/
    └── styles/
        └── globals.css               # Tailwind + shadcn 基础样式
```

### 组件设计原则

- 与 Tauri 交互的组件通过 `hooks/` 层封装，不直接调用 `invoke`/`listen`
- DataTable 基于 `@tanstack/react-table` + `@tanstack/react-virtual`
- Waveform 封装 `lightweight-charts`，支持多线叠加
- 三栏布局：左侧节点面板（可拖拽 200-400px）| 中间内容区 | 右侧详情面板（可折叠）

---

## 6. 状态管理 & 数据流

### Zustand Store

- 核心 store: `AppState`（can, frames, sdo, ds402, recording）
- 细粒度 selectors 避免全量重渲染
- 帧数据只保留最近 10000 条，旧数据自动导出到录制文件

### 数据流

```
Tauri EventChannel → listen() → hooks (节流/批量) → Zustand store → React components (selector 订阅)
```

### Commands 调用

通过 `@tanstack/react-query` 的 `useMutation` 封装，自动管理 loading/error 状态，成功后更新 store。

### 高频数据优化

| 策略 | 实现 |
|---|---|
| 节流 | useFrameStream 每 100ms 合并数据到 store |
| Selector 精确订阅 | 组件只订阅需要的字段 |
| 虚拟滚动 | react-virtual 只渲染可视区域 |
| 数据分片 | store 保留最近 10000 帧 |

---

## 7. Capabilities 权限

采用实用主义策略，开放调试工具可能用到的所有权限：

| 权限域 | 权限 | 用途 |
|---|---|---|
| `fs` | read-all, write-all | 读写配置文件、导出日志、加载 EDS/DBC、保存录制 |
| `dialog` | open, save, message | 文件选择、确认对话框 |
| `shell` | open | 用默认程序打开日志文件 |
| `clipboard` | read, write | 复制帧数据、SDO 值 |
| `os` | platform, arch, version | 系统信息 |
| `window` | all | 窗口管理 |
| `app` | all | 应用控制 |

---

## 8. 测试 & CI/CD

| 层级 | 工具 |
|---|---|
| 前端单元测试 | Vitest + React Testing Library |
| 前端集成测试 | Playwright（Tauri 官方推荐） |
| Rust 单元测试 | cargo test（现有协议栈测试保留） |
| Rust 集成测试 | MockCanDriver + Tauri test harness |

CI 新增:
- `frontend-lint`: npm lint + typecheck
- `frontend-test`: npm test
- `tauri-build`: 三平台矩阵构建（ubuntu/windows/macos）

---

## 9. 实施阶段

### 第一阶段（核心）

1. 删除 `crates/gui/`，新建 `frontend/` 和 `opencan-gui/`
2. 实现 Tauri 后端：State + Channels + Commands（按文件分类）
3. 实现前端：布局 + 所有现有 GUI 页面的迁移（CAN 3 页 + CANOpen 7 页）
4. 集成 DBC 信号解码（Rust candbc → Tauri command → 前端信号级表格）
5. 实现会话录制 & 回放（PCAP/JSON 格式，变速回放）

### 第二阶段（增强）

1. 帧时序分析（间隔分析、抖动分析、频率直方图、时间轴视图）
2. 网络拓扑可视化（reactflow 展示节点关系、OD 对比）
3. 可配置 Dashboard（react-grid-layout 拖拽布局）
4. 自动化测试（操作序列脚本、断言验证、测试报告）
5. EDS/DCF 编辑器（树形展示 OD、参数编辑、EDS 对比）

---

## 10. 前端 npm 依赖清单

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-fs": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-shell": "^2",
    "@tauri-apps/plugin-clipboard-manager": "^2",
    "@tauri-apps/plugin-os": "^2",
    "zustand": "^5",
    "@tanstack/react-query": "^5",
    "@tanstack/react-table": "^8",
    "@tanstack/react-virtual": "^3",
    "lightweight-charts": "^4",
    "recharts": "^2",
    "react-hook-form": "^7",
    "zod": "^3",
    "@hookform/resolvers": "^3",
    "lucide-react": "latest",
    "clsx": "^2",
    "tailwind-merge": "^3",
    "dayjs": "^1",
    "use-debounce": "^10"
  },
  "devDependencies": {
    "vite": "^6",
    "@vitejs/plugin-react": "^4",
    "typescript": "^5",
    "@types/react": "^18",
    "@types/react-dom": "^18",
    "tailwindcss": "^3",
    "postcss": "^8",
    "autoprefixer": "^10",
    "vitest": "^2",
    "@testing-library/react": "^16",
    "@testing-library/jest-dom": "^6",
    "@playwright/test": "^1"
  }
}
```
