---
name: setup
description: Dev environment setup and commands. Load when setting up the project for the first time or when environment issues arise.
triggers:
  - "setup"
  - "install"
  - "environment"
  - "getting started"
  - "how do I run"
  - "local development"
edges:
  - target: context/stack.md
    condition: when specific technology versions or library details are needed
  - target: context/architecture.md
    condition: when understanding how components connect during setup
last_updated: 2026-04-21
---

# Setup

## Prerequisites

- Rust 1.85+ (edition 2024) — install via [rustup](https://rustup.rs/)
- Bevy system dependencies — see [Bevy setup guide](https://bevyengine.org/learn/quick-start/getting-started/setup/) for platform-specific libs (Linux: `libudev-dev`, `libasound2-dev`, etc.)
- Node.js 18+ (for MEX scaffold CLI: `npx promexeus`)
- Python 3.10+ (for graphify codebase knowledge graph: `pip install graphifyy`)

## First-time Setup

1. Clone the repository
2. `cargo build -p broken_divinity` — compile and fetch all dependencies
3. `cargo test -p broken_divinity` — verify everything compiles and tests pass
4. `cargo run -p broken_divinity` — launch the game

## Environment Variables

No environment variables required. All configuration is compile-time or in RON data files under `native/assets/data/` (for example `native/assets/data/rosters.ron`).

## Project Tooling

Two meta-tools sit alongside Cargo in the standard workflow:

| Tool | Purpose | Install |
|------|---------|---------|
| **MEX** (`npx promexeus`) | Scaffold drift detection, context routing, pattern management | Node.js 18+ (already a prerequisite) |
| **graphify** (`graphifyy` on PyPI) | Codebase knowledge graph — builds an interactive dependency/concept graph from source, docs, and media | `pip install graphifyy && graphify install --platform copilot` |

graphify output lives in `graphify-out/` (`.gitignore`-d). Re-run with `/graphify .` inside any supported AI assistant or `graphify build .` from the CLI. The interactive graph is at `graphify-out/graph.html`; the queryable JSON is `graphify-out/graph.json`.

## Common Commands

- `cargo build -p broken_divinity` — compile (debug mode)
- `cargo run -p broken_divinity` — build and run
- `cargo test -p broken_divinity` — run all tests
- `cargo clippy -p broken_divinity -- -W clippy::all` — lint
- `cargo build -p broken_divinity --release` — optimized build
- `scripts/prune-build-artifacts.sh` — prune rebuildable build artifacts; add `--dry-run` to preview reclaimable Cargo output first
- `mex check --quiet` — scaffold drift score
- `mex sync` — fix scaffold drift
- `graphify build .` — rebuild the codebase knowledge graph
- `/graphify .` — run graphify from inside an AI coding assistant

## Common Issues

- **Bevy linker errors on Linux:** Install system dependencies — `sudo apt install libudev-dev libasound2-dev` (and other platform deps from the Bevy setup guide)
- **Slow first compile:** Bevy compiles many crates. Use `cargo build` once, then incremental rebuilds are fast. Consider enabling dynamic linking during development via `.cargo/config.toml`
- **`target/` got huge:** Run `scripts/prune-build-artifacts.sh` with the `--dry-run` flag first, then rerun it without flags to reclaim rebuildable Cargo outputs without removing the current top-level binaries
- **RON parse errors at startup:** Check `native/assets/data/rosters.ron` and other RON data files for syntax — RON is strict about trailing commas and field names matching the Rust struct exactly
