# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Purpose

OpenCAN 是一个 CAN/CANOpen 调试工具，提供跨平台桌面 GUI（Tauri 2 + React）、可独立使用的 CANOpen 协议栈 crate（DS301 + DS402），以及统一的 CAN 硬件抽象层。

## Architecture

### Two-Layer Trait System

```
┌─ canopen-core: CanDriver ──────────────────────┐
│  协议栈内部使用。操作 CanOpenFrame (COB-ID + 8B) │
├─ canopen-ds301: CanDriverAdapter<B: CanBus> ───┤
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
opencan (Tauri binary) → canopen-ds301 → canopen-core
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

DS402 代码位于 `canopen-ds301/src/ds402/` 下，通过 `canopen-ds301` 的 `ds402` feature 启用。
EDS 解析器位于 `canopen-core/src/eds/` 下，通过 `canopen-core` 的 `eds` feature 启用。

### GUI Architecture

```
Frontend (React + Vite, localhost:5173 in dev)
  ↕ Tauri IPC commands (invoke/handle)
  ↕ Zustand stores + React Query
Tauri Backend (tokio runtime)
  ↕ CanopenStack (canopen-ds301)
  ↕ CanDriverAdapter (bridge)
  ↕ CanBus (硬件后端)
```

Tauri 配置在 `opencan-gui/src-tauri/tauri.conf.json` 中。`beforeDevCommand` 启动 Vite dev server，`frontendDist` 指向 `../../frontend/dist`。

### Key Modules

- **CANOpen 帧编解码** — `canopen-core/src/frame.rs`：`CanOpenFrame`、`CobId`、`FunctionCode`（`SyncOrEmergency` 合并变体，通过 node_id 区分）、`SdoRequest`/`SdoResponse`、`TimestampFrame`
- **对象字典** — `canopen-core/src/od.rs`：`ObjectDictionary` trait、`ConcreteOd`（`concrete_od.rs`，BTreeMap 实现）、`OdValue`（25 种 CANOpen 数据类型的序列化，含 40/48/56 位类型 roundtrip 测试）
- **SDO 错误码** — `canopen-core/src/sdo_abort.rs`：SDO abort codes 定义
- **节点 ID** — `canopen-core/src/node_id.rs`：`NodeId` 类型（1-127 有效值验证）
- **PDO 类型** — `canopen-core/src/pdo.rs`：PDO 相关类型定义
- **EDS 解析器** — `canopen-core/src/eds/`：`parser.rs`（INI 格式解析）、`model.rs`（EDS 数据结构）、`builder.rs`（从 EDS 构建 OD）
- **协议栈** — `canopen-ds301/src/stack.rs`：`CanopenStack` 主协议循环（SDO/NMT/Heartbeat/SYNC/TIME_STAMP/节点扫描/原始帧发送）
- **SDO 客户端** — `canopen-ds301/src/sdo.rs`：独立 `SdoClient`，支持 expedited + segmented + block transfer
- **SDO 服务端** — `canopen-ds301/src/sdo_server.rs`：SDO server 实现
- **适配器** — `canopen-ds301/src/adapter.rs`：`CanDriverAdapter` 桥接 `CanBus` → `CanDriver`
- **DS402 状态机** — `canopen-ds301/src/ds402/state_machine.rs`：`Ds402State`、StatusWord/ControlWord 位级解析
- **DS402 设备** — `canopen-ds301/src/ds402/control.rs`：`Ds402Device` 运动控制 API
- **DS402 模式** — `canopen-ds301/src/ds402/modes/`：CSP、CST、CSV、PP、PV、PT、Homing
- **NMT** — `canopen-ds301/src/nmt.rs`：NMT 状态管理
- **Heartbeat** — `canopen-ds301/src/heartbeat.rs`：Heartbeat 生产者/消费者
- **EMCY** — `canopen-ds301/src/emcy.rs`：Emergency 消息处理
- **PDO** — `canopen-ds301/src/pdo.rs` + `pdo_config.rs`：PDO 处理与配置（write_comm_params、disable_pdo、enable_pdo、动态配置）
- **TIME_STAMP** — `canopen-ds301/src/stack.rs`：TIME_STAMP 帧收发（`send_timestamp()`、TimestampFrame 处理）
- **Tauri 后端** — `opencan-gui/src-tauri/src/`：`main.rs`（app entry）、`state.rs`（application state）、`commands/`（IPC commands 模块化：connection/sdo/nmt/pdo/ds402/sync/eds/recording）、`channels/`（backend event channels）
- **前端** — `frontend/src/`：React 组件（`components/` 下按功能分子目录）、自定义 hooks（`hooks/`）、Zustand store（`lib/store.ts`）、Tauri IPC 封装（`lib/tauri.ts`）、页面视图（`pages/` 下按 CAN/CANOpen/Recording/Settings 分组）、类型定义（`types/`）

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

### 添加协议功能

- 新 SDO 功能 → `canopen-ds301/src/sdo.rs`
- 新 NMT/Heartbeat/TIME_STAMP 功能 → `canopen-ds301/src/stack.rs` 或对应模块（`nmt.rs`、`heartbeat.rs`）
- 新 PDO 功能 → `canopen-ds301/src/pdo.rs` 或 `pdo_config.rs`
- 新功能必须通过 `MockCanDriver`（`canopen-core/src/testing.rs`）单元测试

### 添加 DS402 功能

- 新状态转换 → `canopen-ds301/src/ds402/state_machine.rs`
- 新操作模式 → `canopen-ds301/src/ds402/modes/` 下新建文件
- 新设备 API → `canopen-ds301/src/ds402/control.rs`

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
