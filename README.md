# OpenCAN

A CAN/CANOpen debugging tool with a cross-platform desktop GUI (Tauri 2 + React), powered by publishable Rust crates for the CANOpen protocol stack.

[![CI](https://github.com/pony/OpenCAN/actions/workflows/ci.yml/badge.svg)](https://github.com/pony/OpenCAN/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

## Features

- **CAN Hardware Abstraction** — Unified trait for SocketCAN, Kvaser, PCAN, ZLG
- **CANOpen Protocol Stack** — DS301 (NMT/SDO/PDO/EMCY/Heartbeat/SYNC) + DS402 motion control
- **EDS Parser** — Electronic Data Sheet file parsing + Object Dictionary builder
- **Desktop GUI** — Tauri 2 + React application (Windows + macOS + Linux)
- **Pure Rust** — Zero FFI in protocol stack, publishable crates

## Architecture

```
OpenCAN (Cargo Workspace)
│
├── crates/
│   ├── can-traits/             ← Unified CAN trait abstraction + hardware backends
│   │   └── src/
│   │       ├── lib.rs          ← CanBus, CanBusFactory, CanBusDyn traits
│   │       ├── error.rs        ← CAN error types
│   │       ├── socketcan.rs    ← Linux SocketCAN backend (implemented, feature-gated)
│   │       ├── kvaser.rs       ← Kvaser CANlib backend (stub)
│   │       ├── pcan.rs         ← Peak PCAN backend (stub)
│   │       └── zlg.rs          ← ZLG backend (stub)
│   ├── canopen-core/           ← Core traits, frames, Object Dictionary, EDS parser
│   │   └── src/
│   │       ├── eds/            ← EDS file parser + OD builder (feature: eds)
│   │       ├── frame.rs        ← CANOpen frame types (CanOpenFrame, CobId, TimestampFrame)
│   │       ├── od.rs           ← Object Dictionary trait + ConcreteOd
│   │       ├── sdo_abort.rs    ← SDO abort code definitions
│   │       ├── node_id.rs      ← NodeId type (1-127 validation)
│   │       ├── pdo.rs          ← PDO type definitions
│   │       ├── concrete_od.rs  ← BTreeMap-based OD implementation
│   │       ├── testing.rs      ← MockCanDriver for unit tests
│   │       └── error.rs        ← Core error types
│   └── canopen-ds301/          ← DS301 protocol stack + DS402
│       └── src/
│           ├── ds402/          ← DS402 state machine + motion control modes
│           │   ├── state_machine.rs
│           │   ├── control.rs
│           │   └── modes/      ← CSP, CST, CSV, PP, PV, PT, Homing
│           ├── stack.rs        ← Main protocol loop (SDO/NMT/HB/SYNC/TIME_STAMP/scan)
│           ├── sdo.rs          ← SDO client (expedited + segmented + block transfer)
│           ├── sdo_server.rs   ← SDO server
│           ├── adapter.rs      ← CanDriverAdapter bridge
│           ├── nmt.rs          ← NMT management
│           ├── heartbeat.rs    ← Heartbeat producer/consumer
│           ├── emcy.rs         ← Emergency messages
│           ├── pdo.rs          ← PDO processing
│           └── pdo_config.rs   ← PDO configuration (write_comm_params, enable/disable)
│
├── frontend/                   ← React + Vite + TypeScript frontend
│   └── src/
│       ├── components/         ← Reusable UI components
│       │   ├── can/            ← CAN-specific components
│       │   ├── common/         ← Shared UI primitives
│       │   ├── ds402/          ← DS402 motion control components
│       │   ├── layout/         ← App shell (Sidebar, TopBar, BottomPanel, DetailPanel)
│       │   └── sdo/            ← SDO editor components
│       ├── hooks/              ← Custom React hooks (useFrameStream, useCommands, etc.)
│       ├── lib/                ← Core utilities
│       │   ├── store.ts        ← Zustand application state (navigation, UI, connection)
│       │   ├── tauri.ts        ← Tauri IPC command wrappers
│       │   └── utils.ts        ← Shared helpers
│       ├── pages/              ← Page-level view components
│       │   ├── CAN/            ← FrameMonitor, SendPanel, BusStatistics, ErrorFrames
│       │   ├── CANOpen/        ← NetworkOverview, NodeDetail, PdoMonitor, Ds402Control, etc.
│       │   ├── Recording/      ← SessionRecorder, SessionPlayer
│       │   └── Settings/       ← ConnectionSettings, EdsManagement
│       └── types/              ← TypeScript type definitions
│
└── opencan-gui/
    └── src-tauri/              ← Tauri 2 desktop app (binary: opencan)
        └── src/
            ├── main.rs         ← App entry + Tauri setup
            ├── state.rs        ← Shared application state
            ├── commands/       ← Tauri IPC commands (mod.rs-based modules)
            │   ├── connection.rs  ← Connect/disconnect, status
            │   ├── sdo.rs         ← SDO upload/download
            │   ├── nmt.rs         ← NMT start/stop/reset
            │   ├── pdo.rs         ← PDO configuration and monitoring
            │   ├── ds402.rs       ← DS402 motion control
            │   ├── sync.rs        ← SYNC producer control
            │   ├── eds.rs         ← EDS file loading
            │   └── recording.rs   ← CAN session recording
            └── channels/       ← Backend event channels (mod.rs)
```

## Quick Start

### Prerequisites

- Rust 1.75+ (2024 edition)
- Node.js 18+ / pnpm
- For SocketCAN: Linux with `vcan` module (for testing) or real CAN hardware

### Build

```bash
# Clone
git clone https://github.com/pony/OpenCAN.git
cd OpenCAN

# Build workspace (Rust crates)
cargo build --workspace

# Run tests
cargo test --workspace

# Run GUI (Tauri dev mode — starts frontend dev server + Rust backend)
just tauri-dev
```

### Linux vcan Setup (for testing)

```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set up vcan0
```

### Build with Hardware Backend

```bash
# SocketCAN (Linux only)
cargo build -p opencan --features socketcan

# Kvaser
cargo build -p opencan --features kvaser

# PCAN
cargo build -p opencan --features pcan

# ZLG
cargo build -p opencan --features zlg
```

## Development Commands

### Using just (Recommended)

Install [just](https://github.com/casey/just) and run:

```bash
# Show all available commands
just

# Type check
just check

# Build
just build

# Run tests
just test

# Run clippy (CI gate)
just clippy

# Format code
just fmt

# Check formatting (CI gate)
just fmt-check

# Run all CI checks
just ci

# === Tauri GUI ===

# Run Tauri dev (frontend + backend, hot reload)
just tauri-dev

# Build Tauri app (production bundle)
just tauri-build

# Build Tauri app (debug, faster)
just tauri-build-debug

# Run Tauri with socketcan feature
just tauri-socketcan

# === Frontend ===

# Install frontend dependencies
just frontend-install

# Build frontend for production
just frontend-build

# Run frontend type check
just frontend-typecheck

# Run frontend lint
just frontend-lint

# Run frontend tests
just frontend-test

# === vcan (Linux only) ===

# Setup vcan0 interface
just vcan-setup

# Run vcan integration tests
just vcan-test

# === Full-stack ===

# Full CI: Rust + frontend checks
just ci-full

# === Misc ===

# Clean build artifacts
just clean

# Show dependency tree
just deps

# Generate and open documentation
just doc

# Watch for changes and run check
just watch

# Run tests for a specific crate
just test-crate canopen-core

# Build with specific feature
just build-feature socketcan
```

## Crate Overview

| Crate | Description | Tests |
|-------|-------------|-------|
| `can-traits` | CAN bus trait abstraction (CanBus, CanBusFactory) + hardware backends | — |
| `canopen-core` | Core traits, frames, Object Dictionary, EDS parser, MockCanDriver, SDO abort codes, NodeId | 84 |
| `canopen-ds301` | DS301 protocol stack + DS402 (SDO, NMT, Heartbeat, EMCY, PDO, SYNC, TIME_STAMP) | 181 total |
| `opencan` (opencan-gui/src-tauri) | Tauri 2 desktop application | — |

## Protocol Stack Usage

```rust
use opencan_canopen_ds301::CanopenStack;
use opencan_canopen_core::testing::MockCanDriver;
use opencan_canopen_core::od::DataType;

#[tokio::main]
async fn main() {
    let driver = MockCanDriver::new();
    let mut stack = CanopenStack::new(driver, 0);

    // SDO read
    let value = stack.sdo_upload(3, 0x1000, 0, DataType::Unsigned32).await.unwrap();

    // NMT start node
    stack.nmt_start(3).unwrap();

    // Enable SYNC production (10ms period)
    stack.enable_sync_production(std::time::Duration::from_millis(10));

    // Enable heartbeat production (1000ms period)
    stack.enable_heartbeat_production(std::time::Duration::from_secs(1));

    // Send TIME_STAMP frame
    stack.send_timestamp();

    // Send raw CAN frame
    stack.send_frame(can_id, data).await?;
}
```

## EDS → Object Dictionary

```rust
use opencan_canopen_core::eds::parser::parse_eds;
use opencan_canopen_core::eds::builder::build_od;
use opencan_canopen_core::od::ObjectDictionary;

let eds_content = std::fs::read_to_string("device.eds").unwrap();
let eds = parse_eds(&eds_content).unwrap();
let od = build_od(&eds);

let device_type = od.read(0x1000, 0).unwrap();
```

## GUI

The GUI uses a 3-column layout with a collapsible sidebar, context-aware bottom panel, and node-driven detail panel.

### Navigation Groups

Navigation is organized into four sidebar groups, each with its own set of tabs:

#### CAN Bus

| Tab | Description |
|-----|-------------|
| **Frames** | Real-time CAN frame capture with Classic/FD support and frame type auto-decoding (SYNC/HB/TPDO/SDO/NMT/EMCY) |
| **Send** | Raw CAN frame send, SDO quick access, send history, and cyclic send |
| **Statistics** | Bus load, error counters, throughput metrics |
| **Errors** | CAN error frame analysis and logging |

#### CANOpen

| Tab | Description |
|-----|-------------|
| **Network** | Node list, NMT control, status cards |
| **Nodes** | Node-driven detail with accordion sections (Overview/SDO/DS402) |
| **PDO** | Real-time PDO table with type, node, data |
| **DS402** | State machine visualization, enable sequence, motion control |
| **EMCY** | Emergency message capture and decoding |
| **Heartbeat** | Node heartbeat status monitoring |
| **SYNC** | SYNC producer configuration and status |

#### Recording

| Tab | Description |
|-----|-------------|
| **Record** | CAN session recording with frame capture and export |
| **Playback** | Session replay with speed control and frame inspection |

#### EDS

| Tab | Description |
|-----|-------------|
| **EDS Files** | Electronic Data Sheet file loading and management |
| **OD Browser** | Object Dictionary browser from loaded EDS files |

### Layout Components

- **TopBar** — Minimal top bar showing connection status
- **Sidebar** — Collapsible nav groups with node list
- **DetailPanel** — Node-driven accordion (Overview/SDO/DS402), right-side panel
- **BottomPanel** — Context-aware tabs that switch with the active nav group
- **StatusBar** — Application status bar at the bottom

## Hardware Backends

| Backend | Status | Platform | External Dependency |
|---------|--------|----------|---------------------|
| SocketCAN | ✅ Implemented | Linux | Kernel built-in |
| Kvaser | 🔲 Stub | Win/Linux | [Kvaser CANlib SDK](https://www.kvaser.com/developer-downloads/) |
| PCAN | 🔲 Stub | Win/Linux | [PCAN-Basic API](https://www.peak-system.com/PCAN-Basic.239.0.html) |
| ZLG | 🔲 Stub | Win/Linux | ZLG CAN driver (随硬件附带) |

All backends require the vendor's driver/SDK installed on the system. The Rust crate provides FFI bindings only.

## CI/CD

GitHub Actions workflow (`.github/workflows/ci.yml`):
- **Lint**: `cargo clippy -- -D warnings` + `cargo fmt --check`
- **Test**: Matrix across Linux + macOS (181 tests total)
- **Build**: Per-feature compilation check
- **vcan E2E**: Linux-only integration test with virtual CAN

## License

MIT OR Apache-2.0
