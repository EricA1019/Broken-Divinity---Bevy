# Decision: Packaging and Release Candidate (Phase 22)

**Date**: 2026-07-09  
**Status**: Accepted

## Context

Phase 22 prepares the kernel for use outside the development environment — release build configuration, documentation, directory scaffolding, and a smoke test script.

## Decision

### Release profile

Added to `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
```

`lto = "thin"` is preferred over `lto = "fat"` for faster incremental release builds while still getting most of the optimization benefit.

### Default config file

`config/default.toml` ships with the repository. Users copy it to `~/.config/broken-divinity/config.toml` to customize. The file matches the `AppConfig::default()` struct layout.

### README

Created `README.md` with:
- Quick start (controls table)
- Build & run instructions
- Project structure overview
- Configuration guide
- Save/log directory locations
- Troubleshooting section

### Release smoke test

`scripts/release-smoke.sh` automates:
1. `cargo build --release`
2. Quick smoke run
3. `--validate` content check
4. `cargo test --workspace`

### Startup directory creation

`main.rs` now creates `~/.local/share/broken-divinity/` and `~/.local/share/broken-divinity/logs/` at startup. This ensures save and log directories exist even on first run.

### Content directory scaffolding

Created empty directories for future content packs: `actions/`, `blueprints/`, `items/`, `statuses/`, `locations/`, `screens/`. Each contains a `.gitkeep` to track the directory in git.

## Consequences

- **Positive**: `cargo build --release` produces an optimized binary.
- **Positive**: Smoke test script enables automated release validation.
- **Positive**: Save/log dirs created automatically on first run.
- **Positive**: Content directory structure ready for Phase 23 (Roguelike Prototype) content packs.
- **Negative**: Release build is unoptimized for the AppImage/flatpak format — this is a binary release, not a packaged distribution.
