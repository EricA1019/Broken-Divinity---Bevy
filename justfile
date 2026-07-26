# Broken Divinity Kernel — justfile
# Run `just` or `just list` to see all commands.

_default:
    @just --list

# Run the canonical measured development gate.
ci:
    bash scripts/test-gate.sh

# Fast compile check (no codegen)
check:
    cargo check --workspace

# Run all tests
test:
    cargo test --workspace

# Check formatting
fmt:
    cargo fmt --all -- --check

# Run clippy with warnings as errors
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Build debug binary
build:
    cargo build --workspace

# Run the game
run:
    cargo run -p bd_app

# Fix formatting automatically
fmt-fix:
    cargo fmt --all

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Run tests with output (not captured)
test-verbose:
    cargo test --workspace -- --nocapture
