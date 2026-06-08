# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Purpose

OpenCAN 是一个 CAN/CANOpen 调试工具，提供跨平台桌面 GUI（Tauri 2 + React）、可独立使用的 CANOpen 协议栈 crate（DS301 + DS402），以及统一的 CAN 硬件抽象层。

## Architecture

### Crate Structure

```
crates/
├── can-traits/          # CAN 硬件抽象层
├── canopen-core/        # DS301 标准协议实现
├── canopen-ds402/       # DS402 运动控制配置文件
└── canopen-master/      # 主站增强功能
```

### Two-Layer Trait System

```
┌─ canopen-core: CanDriver ──────────────────────┐
│  协议栈内部使用。操作 CanOpenFrame (COB-ID + 8B) │
├─ canopen-master: CanDriverAdapter<B: CanBus> ──┤
│  桥接层。CanOpenFrame ↔ CanFrame 编解码          │
├─ can-traits: CanBus + CanBusFactory ────────────┤
│  硬件后端实现。操作 CanFrame (Classic/FD)         │
└─────────────────────────────────────────────────┘
```

- **`CanBus`** — 运行时接口，`send()` + `recv() -> impl Future`，trait object 安全
- **`CanBusFactory`** — 构造接口，`open()` 返回 `Box<dyn CanBusDyn>`，GUI 用此动态选择后端
- **`CanBusDyn`** — blanket impl over `T: CanBus`，通过 `Box::pin()` 提供动态分发

### Crate Dependency Chain

```
opencan (Tauri binary) → canopen-ds402 → canopen-core
  ├── can-traits ──────────┘
  │   ├── socketcan (opt, feature)
  │   ├── kvaser   (opt, feature)
  │   ├── pcan     (opt, feature)
  │   └── zlg      (opt, feature)
  └── eds (opt, feature in canopen-core)

frontend (React + Vite + TypeScript) → @tauri-apps/api + plugins
  ↓
opencan (Tauri IPC via commands.rs)
```

### Crate Responsibilities

| Crate | 职责 | 可独立发布 |
|-------|------|----------|
| `can-traits` | CAN 硬件抽象层 (CanBus trait) | ✅ |
| `canopen-core` | DS301 标准协议 (NMT/SDO/PDO/Heartbeat/EMCY) | ✅ |
| `canopen-master` | 主站增强功能 (CanDriverAdapter, 节点管理) | ❌ (依赖 core) |
| `canopen-ds402` | DS402 运动控制配置文件 | ❌ (依赖 core) |

#### canopen-core 设计约束

`canopen-core` 必须满足以下约束：

1. **独立可发布** — 可单独发布到 crates.io，不依赖项目内其他 crate
2. **主从站通用** — 同时满足主站和从站的功能需求
3. **嵌入式友好** — 满足嵌入式环境的使用需求（no_std 兼容、低资源占用）
4. **标准协议** — 仅包含 DS301 标准协议功能，设备配置文件（DS402）和主站增强功能放在其他 crate

因此，以下内容应放在 `canopen-core`：
- DS301 标准通信对象（SDO/PDO/NMT/Heartbeat/EMCY）
- 主从站通用的增强功能（PDO 事件处理、SYNC 触发、SDO 错误恢复）
- SDO Server 增强（从站使用）

以下内容不应放在 `canopen-core`：
- 主站专用功能（多客户端管理）→ 放在 `canopen-master`
- 设备配置文件特定功能（DS402 模板）→ 放在 `canopen-ds402`

DS402 代码位于 `canopen-ds402/src/ds402/` 下。
EDS 解析器位于 `canopen-core/src/eds/` 下，通过 `canopen-core` 的 `eds` feature 启用。

### GUI Architecture

```
Frontend (React + Vite, localhost:5173 in dev)
  ↕ Tauri IPC commands (invoke/handle)
  ↕ Zustand stores + React Query
Tauri Backend (tokio runtime)
  ↕ CanopenStack (canopen-core)
  ↕ CanDriverAdapter (canopen-master)
  ↕ CanBus (can-traits 硬件后端)
```

Tauri 配置在 `opencan-gui/src-tauri/tauri.conf.json` 中。`beforeDevCommand` 启动 Vite dev server，`frontendDist` 指向 `../../frontend/dist`。

### Key Modules

#### canopen-core (DS301 标准协议)

`canopen-core` 按照协议层次组织：基础类型扁平放置，DS301 通信协议按通信对象类型分组在 `protocol/` 下。

```
canopen-core/src/
├── frame.rs              # 帧编解码（CanOpenFrame, CobId, FunctionCode, 各类帧类型）
├── od.rs                 # ObjectDictionary trait + OdValue（25 种 CANOpen 数据类型）
├── concrete_od.rs        # BTreeMap 实现的 ConcreteOd + OdBuilder
├── error.rs              # 全局错误类型
├── node_id.rs            # NodeId 类型（1-127 验证）
├── stack.rs              # CanopenStack 主协议循环
├── testing.rs            # MockCanDriver 测试工具
├── eds/                  # EDS 解析器（feature-gated）
│   ├── parser.rs / model.rs / builder.rs
├── protocol/             # DS301 通信协议（按通信对象分组）
│   ├── sdo/              # SDO：client.rs + server.rs + enhanced_server.rs + recovery.rs + abort.rs
│   ├── pdo/              # PDO：types.rs + config.rs + dynamic.rs + event.rs + sync.rs
│   ├── nmt/              # NMT 主站命令
│   ├── heartbeat/        # Heartbeat 生产者/消费者 + SYNC 生产者/消费者
│   └── emcy/             # Emergency 消息处理
```

- **CANOpen 帧编解码** — `frame.rs`：`CanOpenFrame`、`CobId`、`FunctionCode`、`SdoRequest`/`SdoResponse`、`TimestampFrame`
- **对象字典** — `od.rs` + `concrete_od.rs`：`ObjectDictionary` trait、`ConcreteOd`、`OdBuilder`、`OdValue`
- **SDO** — `protocol/sdo/`：`SdoClient`（client.rs，支持 expedited + segmented + block transfer）、`SdoServer`（server.rs）、`EnhancedSdoServer`（增强型，含访问控制）、`SdoErrorRecovery`（错误恢复）、abort codes（abort.rs）
- **PDO** — `protocol/pdo/`：PDO 类型与 pack/unpack（types.rs）、`PdoConfigManager` 配置管理（config.rs）、`DynamicPdoMapper` 动态映射（dynamic.rs）、`PdoEventHandler` 事件处理（event.rs）、`SyncPdoProcessor` SYNC 触发（sync.rs）
- **NMT** — `protocol/nmt/`：NMT 主站命令（启动/停止/复位节点）
- **Heartbeat + SYNC** — `protocol/heartbeat/`：Heartbeat 生产者/消费者、SYNC 生产者/消费者
- **EMCY** — `protocol/emcy/`：Emergency 消息处理
- **协议栈** — `stack.rs`：`CanopenStack` 主协议循环
- **节点 ID** — `node_id.rs`：`NodeId` 类型（1-127 有效值验证）
- **EDS 解析器** — `eds/`：`parser.rs`、`model.rs`、`builder.rs`

#### canopen-master (主站增强功能)

- **适配器** — `canopen-master/src/adapter.rs`：`CanDriverAdapter` 桥接 `CanBus` → `CanDriver`
- **节点管理** — `canopen-master/src/node_manager.rs`：`NodeManager` 自动节点发现、状态跟踪
- **心跳监控** — `canopen-master/src/heartbeat_monitor.rs`：`HeartbeatMonitor` 增强心跳监控
- **NMT 状态机** — `canopen-master/src/nmt_state_machine.rs`：`NmtStateMachine` 状态跟踪
- **EMCY 处理** — `canopen-master/src/emergency_handler.rs`：`EmergencyHandler` 错误处理
- **SDO 多客户端** — `canopen-master/src/sdo_multi_client.rs`：`SdoMultiClient` 多会话管理

#### canopen-ds402 (DS402 运动控制)

- **DS402 状态机** — `canopen-ds402/src/ds402/state_machine.rs`：`Ds402State`、StatusWord/ControlWord 位级解析
- **DS402 设备** — `canopen-ds402/src/ds402/control.rs`：`Ds402Device` 运动控制 API
- **DS402 模式** — `canopen-ds402/src/ds402/modes/`：CSP、CST、CSV、PP、PV、PT、Homing、IP
- **错误处理** — `canopen-ds402/src/ds402/error.rs`：`Ds402Error`、限制检查
- **模式验证** — `canopen-ds402/src/ds402/mode_validator.rs`：模式切换状态验证
- **PDO 模板** — `canopen-ds402/src/ds402/pdo_templates.rs`：DS402 PDO 配置模板

#### Tauri GUI

- **Tauri 后端** — `opencan-gui/src-tauri/src/`：`main.rs`、`state.rs`、`commands/`、`channels/`
- **前端** — `frontend/src/`：React 组件、hooks、Zustand store、页面视图、类型定义

## Development Commands

```bash
cargo check --workspace                                    # 类型检查
cargo build --workspace                                    # 编译
cargo test --workspace                                     # 运行测试
cargo test --workspace --features socketcan                 # 含 SocketCAN 测试
cargo clippy --workspace --all-features -- -D warnings      # 代码检查（CI gate）
cargo fmt --check                                          # 格式检查（CI gate）
cargo fmt                                                  # 格式化
```

### Tauri GUI

```bash
just tauri-dev                                             # 开发模式（前端 hot-reload + Rust）
just tauri-build                                           # 生产构建
just tauri-build-debug                                     # debug 构建
just tauri-socketcan                                       # 带 socketcan feature 运行
just frontend-build                                        # 仅构建前端
just frontend-typecheck                                    # 前端 TypeScript 检查
just frontend-test                                         # 前端 Vitest 测试
```

### vcan0 集成测试（Linux）

```bash
sudo modprobe vcan && sudo ip link add dev vcan0 type vcan && sudo ip link set up vcan0
OPENCAN_VCAN_TEST=1 cargo test --workspace --features socketcan -- --ignored
```

### Feature Flags (opencan crate)

`socketcan` / `kvaser` / `pcan` / `zlg` — 启用对应硬件后端。`eds` — 启用 EDS 解析器。

Feature 是非排他的，可同时启用多个后端。GUI 通过 `CanBusFactory` 注册表在运行时选择。

### CI Pipeline (`.github/workflows/ci.yml`)

1. **lint** — `cargo fmt --check` + `cargo clippy --workspace --all-features -- -D warnings`
2. **test** — ubuntu + macos 双平台，`cargo test --workspace` + socketcan feature（仅 Linux）
3. **build-features** — 逐个 feature 编译验证
4. **vcan-test** — Linux vcan0 端到端，依赖 lint + test 通过

## Extending the Codebase

### 添加新硬件后端

1. 在 `can-traits/src/` 下创建 `<name>.rs`，实现 `CanBus` trait 和 `CanBusFactory` trait
2. 在 `can-traits/Cargo.toml` 添加 feature flag 和可选依赖
3. 在 `opencan-gui/src-tauri/Cargo.toml` 添加 feature flag：`<name> = ["opencan-can-traits/<name>"]`
4. 在 Tauri 后端代码中注册新后端到 `CanBusFactory` 注册表
5. 在 CI `build-features` job 的 matrix 中添加 feature

### 添加协议功能 (canopen-core)

- 新 SDO 功能 → `canopen-core/src/protocol/sdo/` 下的 `client.rs`、`server.rs` 或 `abort.rs`
- 新 NMT/Heartbeat/SYNC 功能 → `canopen-core/src/protocol/nmt/` 或 `protocol/heartbeat/`
- 新 PDO 功能 → `canopen-core/src/protocol/pdo/types.rs` 或 `config.rs`
- 新 EMCY 功能 → `canopen-core/src/protocol/emcy/`
- 新通信对象 → 在 `canopen-core/src/protocol/` 下新建子目录，在 `protocol/mod.rs` 中注册，在 `lib.rs` 中添加 `pub use`
- 新功能必须通过 `MockCanDriver`（`canopen-core/src/testing.rs`）单元测试

### 添加主站增强功能 (canopen-master)

- 节点管理 → `canopen-master/src/node_manager.rs`（新建）
- 网络诊断 → `canopen-master/src/diagnostics.rs`（新建）
- 硬件桥接 → `canopen-master/src/adapter.rs`

### 添加 DS402 功能 (canopen-ds402)

- 新状态转换 → `canopen-ds402/src/ds402/state_machine.rs`
- 新操作模式 → `canopen-ds402/src/ds402/modes/` 下新建文件
- 新设备 API → `canopen-ds402/src/ds402/control.rs`

### 修改 GUI

- 新 Tauri 命令 → `opencan-gui/src-tauri/src/commands/` 下新建模块文件，在 `mod.rs` 中导出，用 `#[tauri::command]` 标记
- 新状态管理 → `opencan-gui/src-tauri/src/state.rs` 或前端 `frontend/src/lib/store.ts` 更新 Zustand store
- 新前端页面 → `frontend/src/pages/` 下对应分组目录（CAN/CANOpen/Recording/Settings）创建模块，在 `App.tsx` 的 `TAB_COMPONENTS` 和 `LEGACY_MAP` 中注册
- 新前端组件 → `frontend/src/components/` 下对应功能子目录创建
- 新后端命令/事件 → 在 `commands/` 对应模块和 `channels/mod.rs` 添加变体，在 Tauri IPC handler 中处理
- 前端每 50ms 轮询 backend event，高频数据需节流
- GUI 采用 3 列布局：Sidebar（可折叠导航组）+ Main Content（组内 Tab）+ DetailPanel（右侧节点驱动面板）+ BottomPanel（上下文感知底部面板）

## Tech Stack

### Rust
- Rust edition 2024, workspace resolver = "2"
- **tauri 2** — Desktop app framework
- **tokio 1** (full) — async runtime
- **thiserror 2** — 错误类型
- **socketcan 3.5** (tokio) — Linux 后端
- **tracing / tracing-subscriber** — 日志
- **serde / serde_json** — 序列化
- **ini 1.3** — EDS 文件解析
- **mockall 0.13** — 测试 mock

### Frontend
- **React 18** + **TypeScript 5**
- **Vite 6** — 构建工具
- **Tailwind CSS 3** — 样式
- **Zustand 5** — 状态管理
- **React Query 5** — 数据获取
- **React Hook Form 7** + **Zod 3** — 表单验证
- **lightweight-charts 4** — 图表
- **Vitest 2** — 测试
- **Playwright 1** — E2E 测试
