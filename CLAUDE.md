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
gui → canopen-ds402 → canopen-ds301 → canopen-core
  ├── can-traits ────────────────────────┘
  ├── can-socketcan (opt) → can-traits
  ├── can-kvaser (opt)    → can-traits
  ├── can-pcan (opt)      → can-traits
  ├── can-zlg (opt)       → can-traits
  └── canopen-eds (opt)   → canopen-core
```

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
- **协议栈** — `canopen-ds301/src/stack.rs`：`CanopenStack` 主协议循环（SDO/NMT/Heartbeat/SYNC/节点扫描）
- **SDO 客户端** — `canopen-ds301/src/sdo.rs`：独立 `SdoClient`，支持 expedited + segmented transfer
- **适配器** — `canopen-ds301/src/adapter.rs`：`CanDriverAdapter` 桥接 `CanBus` → `CanDriver`
- **DS402 状态机** — `canopen-ds402/src/state_machine.rs`：`Ds402State`、StatusWord/ControlWord 位级解析
- **DS402 设备** — `canopen-ds402/src/control.rs`：`Ds402Device` 运动控制 API
- **后端通信** — `gui/src/backend.rs`：`Backend` 结构，`BackendCommand` / `BackendEvent` 通过 mpsc 双向通信

## Development Commands

```bash
cargo check --workspace                                    # 类型检查
cargo build --workspace                                    # 编译
cargo test --workspace                                     # 运行测试
cargo test --workspace --features socketcan                 # 含 SocketCAN 测试
cargo clippy --workspace --all-features -- -D warnings      # 代码检查（CI gate）
cargo fmt --check                                          # 格式检查（CI gate）
cargo fmt                                                  # 格式化
cargo run -p gui [--features socketcan]                    # 启动 GUI
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

1. 在 `crates/` 下创建 `can-<name>/`，依赖 `can-traits`
2. 实现 `CanBus` trait 和 `CanBusFactory` trait
3. 在 `gui/Cargo.toml` 添加 feature flag：`<name> = ["dep:opencan-can-<name>"]`
4. 在 `gui/src/backend.rs` 中注册新后端
5. 在 CI `build-features` job 的 matrix 中添加 feature

### 添加协议功能

- 新 SDO 功能 → `canopen-ds301/src/sdo.rs`
- 新 NMT/Heartbeat 功能 → `canopen-ds301/src/stack.rs`
- 新 PDO 功能 → `canopen-ds301/` 下新建文件
- 新功能必须通过 `MockCanDriver`（`canopen-core/src/testing.rs`）单元测试

### 添加 DS402 功能

- 新状态转换 → `canopen-ds402/src/state_machine.rs`
- 新操作模式 → `canopen-ds402/src/control.rs`

### 修改 GUI

- 新视图 → `gui/src/views.rs` 添加 renderer，`gui/src/state.rs` 添加 `View` 变体
- 新消息类型 → `gui/src/state.rs` 的 `Message` enum
- 新后端命令 → `gui/src/backend.rs` 的 `BackendCommand` / `BackendEvent`
- GUI 每 50ms 轮询 backend event，高频数据需节流

## Tech Stack

- Rust edition 2024, workspace resolver = "2"
- **iced 0.13** (tokio) — GUI
- **tokio 1** (full) — async runtime
- **thiserror 2** — 错误类型
- **socketcan 3.5** (tokio) — Linux 后端
