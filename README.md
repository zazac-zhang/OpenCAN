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
│   ├── can-traits/             ← Unified CAN trait abstraction
│   ├── can-socketcan/          ← Linux SocketCAN backend
│   ├── can-kvaser/             ← Kvaser CANlib backend
│   ├── can-pcan/               ← Peak PCAN backend
│   ├── can-zlg/                ← ZLG backend
│   ├── canopen-core/           ← Core traits + Object Dictionary [no_std]
│   ├── canopen-ds301/          ← DS301 protocol (NMT/SDO/PDO/EMCY/Heartbeat/SYNC)
│   ├── canopen-ds402/          ← DS402 motion control
│   ├── canopen-eds/            ← EDS file parser + OD builder
│   └── gui/                    ← iced GUI application
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

## Crate Overview

| Crate | Description | Tests |
|-------|-------------|-------|
| `canopen-core` | Core traits, frames, Object Dictionary, MockCanDriver | 15 |
| `canopen-ds301` | DS301 protocol stack (SDO, NMT, Heartbeat, EMCY, PDO, SYNC) | 17+8 |
| `canopen-ds402` | DS402 state machine + motion control device | 3 |
| `canopen-eds` | EDS parser + Object Dictionary builder | 8 |
| `can-traits` | CAN bus trait abstraction (CanBus, CanBusFactory) | — |
| `can-socketcan` | Linux SocketCAN backend | — |
| `gui` | iced GUI application | — |

**Total: 53 tests, 0 clippy warnings**

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
use opencan_canopen_eds::parser::parse_eds;
use opencan_canopen_eds::builder::build_od;
use opencan_canopen_core::od::ObjectDictionary;

let eds_content = std::fs::read_to_string("device.eds").unwrap();
let eds = parse_eds(&eds_content).unwrap();
let od = build_od(&eds);

let device_type = od.read(0x1000, 0).unwrap();
```

## GUI Pages

| Page | Description |
|------|-------------|
| **Network Overview** | Node list, NMT control, status cards |
| **Node Detail** | SDO read/write, OD browser, quick reads |
| **DS402 Panel** | State machine, enable sequence, position/velocity control |
| **PDO Monitor** | Real-time PDO table with type, node, data |
| **CAN Log** | Frame log with text search, type filter, clear |

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
