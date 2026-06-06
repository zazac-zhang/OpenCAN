# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Purpose

OpenCAN 是一个 CAN/CANOpen 调试工具，提供跨平台 GUI（iced）、可独立使用的 CANOpen 协议栈 crate（DS301 + DS402），以及统一的 CAN 硬件抽象层。

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
gui → canopen-ds301 → canopen-core
  ├── can-traits ──────────┘
  │   ├── socketcan (opt, feature)
  │   ├── kvaser   (opt, feature)
  │   ├── pcan     (opt, feature)
  │   └── zlg      (opt, feature)
  └── eds (opt, feature in canopen-core)
```

DS402 代码位于 `canopen-ds301/src/ds402/` 下，通过 `canopen-ds301` 的 `ds402` feature 启用。
EDS 解析器位于 `canopen-core/src/eds/` 下，通过 `canopen-core` 的 `eds` feature 启用。

### GUI Architecture

Elm 架构通过 iced。核心通信模型：

```
GUI (iced main loop)
  ↕ 50ms polling via time::every()
  ↕ mpsc::channel(BackendCommand → BackendEvent)
Backend tokio task
  ↕ CanopenStack (canopen-ds301)
  ↕ CanDriverAdapter (bridge)
  ↕ CanBus (硬件后端)
```

### Key Modules

- **CANOpen 帧编解码** — `canopen-core/src/frame.rs`：`CanOpenFrame`、`CobId`、`FunctionCode`（`SyncOrEmergency` 合并变体，通过 node_id 区分）、`SdoRequest`/`SdoResponse`
- **对象字典** — `canopen-core/src/od.rs`：`ObjectDictionary` trait、`ConcreteOd`（BTreeMap 实现）、`OdValue`（25 种 CANOpen 数据类型的序列化）
- **EDS 解析器** — `canopen-core/src/eds/`：`parser.rs`（INI 格式解析）、`model.rs`（EDS 数据结构）、`builder.rs`（从 EDS 构建 OD）
- **协议栈** — `canopen-ds301/src/stack.rs`：`CanopenStack` 主协议循环（SDO/NMT/Heartbeat/SYNC/节点扫描）
- **SDO 客户端** — `canopen-ds301/src/sdo.rs`：独立 `SdoClient`，支持 expedited + segmented transfer
- **SDO 服务端** — `canopen-ds301/src/sdo_server.rs`：SDO server 实现
- **适配器** — `canopen-ds301/src/adapter.rs`：`CanDriverAdapter` 桥接 `CanBus` → `CanDriver`
- **DS402 状态机** — `canopen-ds301/src/ds402/state_machine.rs`：`Ds402State`、StatusWord/ControlWord 位级解析
- **DS402 设备** — `canopen-ds301/src/ds402/control.rs`：`Ds402Device` 运动控制 API
- **DS402 模式** — `canopen-ds301/src/ds402/modes/`：CSP、CST、CSV、PP、PV、PT、Homing
- **NMT** — `canopen-ds301/src/nmt.rs`：NMT 状态管理
- **Heartbeat** — `canopen-ds301/src/heartbeat.rs`：Heartbeat 生产者/消费者
- **EMCY** — `canopen-ds301/src/emcy.rs`：Emergency 消息处理
- **PDO** — `canopen-ds301/src/pdo.rs` + `pdo_config.rs`：PDO 处理与配置
- **后端通信** — `gui/src/backend/`：`real.rs`（真实后端）、`mock.rs`（模拟后端）、`command.rs`/`event.rs`（消息类型）

## Development Commands

```bash
cargo check --workspace                                    # 类型检查
cargo build --workspace                                    # 编译
cargo test --workspace                                     # 运行测试
cargo test --workspace --features socketcan                 # 含 SocketCAN 测试
cargo clippy --workspace --all-features -- -D warnings      # 代码检查（CI gate）
cargo fmt --check                                          # 格式检查（CI gate）
cargo fmt                                                  # 格式化
cargo run -p opencan-gui [--features socketcan]            # 启动 GUI（二进制名: opencan）
```

### vcan0 集成测试（Linux）

```bash
sudo modprobe vcan && sudo ip link add dev vcan0 type vcan && sudo ip link set up vcan0
OPENCAN_VCAN_TEST=1 cargo test --workspace --features socketcan -- --ignored
```

### Feature Flags (gui crate)

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
3. 在 `gui/Cargo.toml` 添加 feature flag：`<name> = ["opencan-can-traits/<name>"]`
4. 在 `gui/src/backend/real.rs` 中注册新后端到 `CanBusFactory` 注册表
5. 在 CI `build-features` job 的 matrix 中添加 feature

### 添加协议功能

- 新 SDO 功能 → `canopen-ds301/src/sdo.rs`
- 新 NMT/Heartbeat 功能 → `canopen-ds301/src/stack.rs` 或对应模块（`nmt.rs`、`heartbeat.rs`）
- 新 PDO 功能 → `canopen-ds301/src/pdo.rs` 或 `pdo_config.rs`
- 新功能必须通过 `MockCanDriver`（`canopen-core/src/testing.rs`）单元测试

### 添加 DS402 功能

- 新状态转换 → `canopen-ds301/src/ds402/state_machine.rs`
- 新操作模式 → `canopen-ds301/src/ds402/modes/` 下新建文件
- 新设备 API → `canopen-ds301/src/ds402/control.rs`

### 修改 GUI

- 新视图 → `gui/src/views/` 下创建模块，在 `views/mod.rs` 导出，在 `detail_panel.rs` 或对应面板中集成
- 新状态 → `gui/src/state/` 下创建模块，在 `state/mod.rs` 导出
- 新后端命令 → `gui/src/backend/command.rs` 添加变体，`gui/src/backend/real.rs` 处理
- 新后端事件 → `gui/src/backend/event.rs` 添加变体，`gui/src/app.rs` 的 `update()` 处理
- GUI 每 50ms 轮询 backend event，高频数据需节流

## Tech Stack

- Rust edition 2024, workspace resolver = "2"
- **iced 0.13** (tokio) — GUI
- **tokio 1** (full) — async runtime
- **thiserror 2** — 错误类型
- **socketcan 3.5** (tokio) — Linux 后端
- **rfd 0.14** — 文件对话框
- **tracing / tracing-subscriber** — 日志
- **serde** — 序列化
- **ini 1.3** — EDS 文件解析
- **mockall 0.13** — 测试 mock
