# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- **Phase 5: Feature Expansion**
  - CAN FD support (64-byte frames, BRS/ESI flags)
  - Automation script engine
  - Protocol analyzer enhancements
  - Documentation improvements

## [0.4.0] - 2026-06-08

### Added
- **Phase A: DS402 Interactive State Machine Flowchart**
  - SVG-based flowchart with 8 states and transitions
  - Edge labels showing ControlWord hex values (0x6040 writes)
  - Current state highlighting with color-coded pulse animation
  - Clickable edges to send SDO commands
  - StatusWord bit-level parsing panel (14 bits with icons)
  - Quick control word buttons for available transitions

- **Phase B: SDO Interactive Explorer**
  - Split-pane layout: OD tree browser + detail panel
  - Tree-view OD browser grouped by index range (5 areas)
  - Double-click to read OD entry via SDO upload
  - Write values via SDO download with hex input
  - Multi-format display (Hex/Bin/Dec)
  - SDO history with replay and CSV export

- **Phase C: Network Topology Visualization**
  - SVG canvas with master node at center
  - Slave nodes in semicircle layout
  - Node colors: Green=Operational, Yellow=PreOp, Red=Stopped, Gray=Offline
  - Click node to select + NMT command panel
  - Drag nodes to rearrange layout
  - Scan button for auto-discovery

- **Phase 3: Advanced Features**
  - EDS browser enhancements (search, detail panel, CSV export)
  - CAN frame log export (CSV + ASC/Vector formats)
  - Frame filter presets (save/load/delete)
  - Global keyboard shortcuts (Ctrl+K, Ctrl+L, Space, Ctrl+1-4, Escape)

- **Phase 2: GUI Polish**
  - Code splitting with React.lazy (631KB → 264KB main bundle)
  - Vitest infrastructure with 20 unit tests
  - ConnectionDialog UX improvements

- **Phase 1: Hardware Backends**
  - ZLG, Kvaser, PCAN backends fully implemented
  - SocketCAN support for Linux

- **Phase 0: Quality Hardening**
  - Zero Clippy warnings across all features
  - DS402 PDO templates + SDO multi-client restructured

### Changed
- Refactored crate structure: canopen-ds301 → canopen-core, canopen-master, canopen-ds402
- Improved error handling and user feedback

## [0.3.0] - 2026-06-01

### Added
- DS402 motion control modes (CSP, CSV, CST, PP, PV, PT, Homing)
- Emergency (EMCY) message monitoring
- SYNC producer/consumer support
- PDO configuration management

### Changed
- Enhanced SDO client with block transfer support
- Improved heartbeat monitoring

## [0.2.0] - 2026-05-15

### Added
- Tauri desktop application framework
- React frontend with Tailwind CSS
- Zustand state management
- React Query for data fetching
- Basic CAN frame monitoring

### Changed
- Migrated from web-only to Tauri desktop app

## [0.1.0] - 2026-05-01

### Added
- Initial release
- CAN hardware abstraction layer (can-traits)
- DS301 protocol stack (canopen-core)
- Basic SDO client/server
- NMT master commands
- Heartbeat producer/consumer
- Object Dictionary trait and implementation

---

## Release Notes

### v0.4.0 — Frontend Enhancement Release

This release brings three major visualization features to the OpenCAN GUI:

1. **DS402 State Machine Flowchart** — Interactive SVG visualization of the CiA 402 state machine with clickable transitions and real-time StatusWord decoding.

2. **SDO Explorer** — Professional-grade Object Dictionary browser with tree view, search, read/write operations, and SDO operation history.

3. **Network Topology** — Visual representation of your CANopen network with drag-to-rearrange nodes and integrated NMT commands.

Plus: EDS enhancements, frame log export, filter presets, keyboard shortcuts, and performance improvements (code splitting reduced main bundle by 58%).

### v0.3.0 — DS402 Motion Control

Full DS402 motion control profile implementation with support for all major operation modes: Cyclic Synchronous Position/Velocity/Torque, Profile Position/Velocity/Torque, and Homing.

### v0.2.0 — Desktop GUI

Migration from web-only prototype to full Tauri desktop application with modern React frontend, state management, and data fetching.

### v0.1.0 — Foundation

Initial release with core CANopen protocol stack (DS301) and hardware abstraction layer supporting SocketCAN, ZLG, Kvaser, and PCAN interfaces.
