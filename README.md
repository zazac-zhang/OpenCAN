# OpenCAN

A CAN/CANOpen debugging tool with GUI, written in pure Rust.

[![CI](https://github.com/pony/OpenCAN/actions/workflows/ci.yml/badge.svg)](https://github.com/pony/OpenCAN/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

## Features

- **CAN Hardware Abstraction** — Unified trait for SocketCAN, Kvaser, PCAN, ZLG
- **CANOpen Protocol Stack** — DS301 (NMT/SDO/PDO/EMCY/Heartbeat/SYNC) + DS402 motion control
- **EDS Parser** — Electronic Data Sheet file parsing + Object Dictionary builder
- **GUI Application** — iced-based cross-platform desktop app (Windows + Linux)
- **Pure Rust** — Zero FFI in protocol stack, publishable crates

## Architecture

```
OpenCAN (Cargo Workspace)
│
├── crates/
│   ├── can-traits/             ← Unified CAN trait abstraction + hardware backends
│   │   └── src/
│   │       ├── socketcan.rs    ← Linux SocketCAN backend (feature-gated)
│   │       ├── kvaser.rs       ← Kvaser CANlib backend (stub)
│   │       ├── pcan.rs         ← Peak PCAN backend (stub)
│   │       └── zlg.rs          ← ZLG backend (stub)
│   ├── canopen-core/           ← Core traits, frames, Object Dictionary, EDS parser
│   │   └── src/
│   │       ├── eds/            ← EDS file parser + OD builder (feature: eds)
│   │       ├── frame.rs        ← CANOpen frame types
│   │       ├── od.rs           ← Object Dictionary trait + ConcreteOd
│   │       └── testing.rs      ← MockCanDriver for unit tests
│   ├── canopen-ds301/          ← DS301 protocol stack + DS402
│   │   └── src/
│   │       ├── ds402/          ← DS402 state machine + motion control modes
│   │       │   ├── state_machine.rs
│   │       │   ├── control.rs
│   │       │   └── modes/      ← CSP, CST, CSV, PP, PV, PT, Homing
│   │       ├── stack.rs        ← Main protocol loop
│   │       ├── sdo.rs          ← SDO client (expedited + segmented)
│   │       ├── sdo_server.rs   ← SDO server
│   │       ├── adapter.rs      ← CanDriverAdapter bridge
│   │       ├── nmt.rs          ← NMT management
│   │       ├── heartbeat.rs    ← Heartbeat producer/consumer
│   │       ├── emcy.rs         ← Emergency messages
│   │       ├── pdo.rs          ← PDO processing
│   │       └── pdo_config.rs   ← PDO configuration
│   └── gui/                    ← iced GUI application (binary: opencan)
│       └── src/
│           ├── backend/        ← Backend communication (mpsc channels)
│           ├── state/          ← Application state modules
│           └── views/          ← UI views (CAN + CANOpen pages)
└── docs/
    └── spark/                  ← Design specifications
```

## Quick Start

### Prerequisites

- Rust 1.75+ (2024 edition)
- For SocketCAN: Linux with `vcan` module (for testing) or real CAN hardware

### Build

```bash
# Clone
git clone https://github.com/pony/OpenCAN.git
cd OpenCAN

# Build workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run GUI (mock backend for testing)
cargo run -p opencan-gui
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
cargo build -p opencan-gui --features socketcan

# Kvaser
cargo build -p opencan-gui --features kvaser

# PCAN
cargo build -p opencan-gui --features pcan

# ZLG
cargo build -p opencan-gui --features zlg
```

## Development Commands

We provide multiple ways to run common development tasks:

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

# Run GUI
just gui

# Run GUI with socketcan
just gui-socketcan

# Setup vcan0 (Linux, requires sudo)
just vcan-setup

# Run vcan integration tests
just vcan-test

# Run all CI checks
just ci

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

### Using Make

Alternatively, use `make`:

```bash
# Show all targets
make help

# Type check
make check

# Build
make build

# Run tests
make test

# Run clippy
make clippy

# Format code
make fmt

# Check formatting
make fmt-check

# Run GUI
make gui

# Run GUI with socketcan
make gui-socketcan

# Setup vcan0
make vcan-setup

# Run vcan integration tests
make vcan-test

# Run all CI checks
make ci

# Clean build artifacts
make clean

# Show dependency tree
make deps

# Generate documentation
make doc

# Watch for changes
make watch

# Run tests for a specific crate
make test-crate crate=canopen-core

# Build with specific feature
make build-feature feature=socketcan
```

### Using Scripts

Helper scripts are available in the `scripts/` directory:

```bash
# Setup vcan0 interface (Linux, requires sudo)
sudo ./scripts/vcan-setup.sh

# Run all CI checks locally
./scripts/ci-check.sh
```

## Crate Overview

| Crate | Description | Tests |
|-------|-------------|-------|
| `canopen-core` | Core traits, frames, Object Dictionary, EDS parser, MockCanDriver | 15 |
| `canopen-ds301` | DS301 protocol stack + DS402 (SDO, NMT, Heartbeat, EMCY, PDO, SYNC) | 17+8 |
| `can-traits` | CAN bus trait abstraction (CanBus, CanBusFactory) + hardware backends | — |
| `opencan-gui` | iced GUI application | — |

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

## GUI Pages

### CAN Layer

| Page | Description |
|------|-------------|
| **Frame Monitor** | Real-time CAN frame capture with Classic/FD support |
| **Bus Statistics** | Bus load, error counters, throughput metrics |
| **Error Frames** | CAN error frame analysis and logging |

### CANOpen Layer

| Page | Description |
|------|-------------|
| **Network Overview** | Node list, NMT control, status cards |
| **Node Detail** | SDO read/write, OD browser, quick reads |
| **DS402 Panel** | State machine, enable sequence, motion control |
| **PDO Monitor** | Real-time PDO table with type, node, data |
| **SDO Editor** | SDO read/write with object dictionary navigation |
| **EMCY Monitor** | Emergency message capture and decoding |
| **Heartbeat** | Node heartbeat status monitoring |
| **SYNC** | SYNC producer configuration and status |
| **Trend Chart** | Real-time data visualization with Canvas |

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
- **Test**: Matrix across Linux + macOS
- **Build**: Per-feature compilation check
- **vcan E2E**: Linux-only integration test with virtual CAN

## License

MIT OR Apache-2.0
