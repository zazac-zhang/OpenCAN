# OpenCAN development commands

# Default target: show available commands
default:
    @just --list

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
    #!/bin/bash
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

# Run the GUI (with optional feature)
gui feature='':
    #!/bin/bash
    if [ -n "{{ feature }}" ]; then
        cargo run -p opencan-gui --features {{ feature }}
    else
        cargo run -p opencan-gui
    fi

# Setup vcan0 interface (Linux only, requires sudo)
vcan-setup:
    #!/bin/bash
    sudo modprobe vcan
    sudo ip link add dev vcan0 type vcan
    sudo ip link set up vcan0

# Run vcan0 integration tests (Linux only, requires vcan0 setup)
vcan-test:
    OPENCAN_VCAN_TEST=1 cargo test --workspace --features socketcan -- --ignored

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

# Run GUI with socketcan feature
gui-socketcan:
    cargo run -p opencan-gui --features socketcan

# Watch for changes and run check (requires cargo-watch)
watch:
    cargo watch -x check

# Generate documentation
doc:
    cargo doc --workspace --open

# Run a specific binary
run binary:
    cargo run -p {{ binary }}