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

# Monthly workspace hygiene audit
hygiene:
    @echo "=== Workspace Hygiene Audit ==="
    @echo ""
    @echo "1. Loose files at workspace root (should be 0 beyond authority docs):"
    @/usr/bin/ls -1 *.md *.json *.log *.toml *.lock justfile 2>/dev/null
    @echo ""
    @echo "2. docs/root files (should be only README.md):"
    @/usr/bin/ls -1 docs/*.md 2>/dev/null || echo "  (clean)"
    @echo ""
    @echo "3. Unused dependencies:"
    @echo "  bevy_time, color-eyre, insta, schemars are declared but unused"
    @echo "  Run: cargo tree --workspace -e no-dev --depth 1 | grep -E 'bevy_time|color-eyre|insta|schemars'"
    @echo ""
    @echo "4. Check for stale references to legacy paths:"
    @/usr/bin/grep -r 'src/core/\|src/game/\|bevy_egui\|bevy_ecs_tilemap\|AppState\|TurnPhase' docs/authority/ docs/active/ .mex/ 2>/dev/null || echo "  (none found)"
    @echo ""
    @echo "5. .gitignore coverage:"
    @echo "  Verify .artifacts/, __pycache__/, *.log are in .gitignore"
    @/usr/bin/grep -c 'artifacts\|pycache\|\.log' .gitignore
    @echo ""
    @echo "=== Audit complete ==="
