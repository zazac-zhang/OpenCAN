# OpenCAN development commands

# Default target: show available commands
default:
    @just --list

# === Rust workspace ===

# Type check the workspace
check:
    cargo check --workspace

# Build the workspace
build:
    cargo build --workspace

# Run all tests
test:
    cargo test --workspace

# Run clippy with all features (CI gate, Linux-only for socketcan)
clippy:
    #!/usr/bin/env bash
    if [[ "$(uname)" == "Linux" ]]; then
        cargo clippy --workspace --all-features -- -D warnings
    else
        cargo clippy --workspace -- -D warnings
        echo "(skipped --all-features: socketcan requires Linux)"
    fi

# Run clippy with all features (Linux only, for CI)
clippy-all:
    cargo clippy --workspace --all-features -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting (CI gate)
fmt-check:
    cargo fmt --check

# Run all CI checks (fmt, clippy, test)
ci: fmt-check clippy test
    @echo "All CI checks passed!"

# Clean build artifacts
clean:
    cargo clean

# Show dependency tree
deps:
    cargo tree

# Run a specific crate's tests
test-crate crate:
    cargo test -p {{ crate }}

# Build with specific feature
build-feature feature:
    cargo build --workspace --features {{ feature }}

# Watch for changes and run check (requires cargo-watch)
watch:
    cargo watch -x check

# Generate documentation
doc:
    cargo doc --workspace --open

# Run a specific binary
run binary:
    cargo run -p {{ binary }}

# === vcan (Linux only) ===

# Setup vcan0 interface (Linux only, requires sudo)
vcan-setup:
    #!/usr/bin/env bash
    sudo modprobe vcan
    sudo ip link add dev vcan0 type vcan
    sudo ip link set up vcan0

# Run vcan0 integration tests (Linux only, requires vcan0 setup)
vcan-test:
    OPENCAN_VCAN_TEST=1 cargo test --workspace --features socketcan -- --ignored

# === Frontend (React + Vite) ===

# Install frontend dependencies
frontend-install:
    cd frontend && pnpm install

# Run frontend dev server (standalone)
frontend-dev:
    cd frontend && pnpm dev

# Build frontend for production
frontend-build:
    cd frontend && pnpm build

# Preview frontend production build
frontend-preview:
    cd frontend && pnpm preview

# Run frontend type check
frontend-typecheck:
    cd frontend && pnpm typecheck

# Run frontend lint
frontend-lint:
    cd frontend && pnpm lint

# Run frontend tests
frontend-test:
    cd frontend && pnpm test

# === Tauri (GUI app) ===

# Run Tauri dev (frontend + backend, hot reload)
tauri-dev:
    cd opencan-gui/src-tauri && cargo tauri dev

# Build Tauri app (bundles frontend + backend)
tauri-build:
    cd opencan-gui/src-tauri && cargo tauri build

# Build Tauri app (debug, faster)
tauri-build-debug:
    cd opencan-gui/src-tauri && cargo tauri build --debug

# Run Tauri app without dev server (uses pre-built frontend/dist)
tauri-run: frontend-build
    cd opencan-gui/src-tauri && cargo tauri dev -- --no-watch

# Run Tauri with socketcan feature
tauri-socketcan:
    cd opencan-gui/src-tauri && cargo tauri dev -- --features socketcan

# === Full-stack convenience ===

# Run everything: frontend dev + Tauri backend (equivalent to tauri-dev)
dev: tauri-dev

# Full CI: lint + test (Rust) + frontend checks
ci-full: fmt-check clippy test frontend-build frontend-typecheck
    @echo "All full-stack CI checks passed!"
