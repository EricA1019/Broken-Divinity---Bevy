# Broken Divinity Kernel

A reusable terminal-based roguelike/tactics game kernel built with Rust, Bevy ECS, and Ratatui.

## Current Status

The Broken Divinity Foundation MVP is in recovery. Build and unit-test gates
pass, but the project has not yet passed the canonical colony, dungeon,
persistence, progression, and manual acceptance scenario.

Project authority:

1. [Product GDD](../GDD.md)
2. [Locked decisions](../docs/DECISIONS-TO-LOCK.md)
3. [Foundation MVP scenario](../docs/MVP-SCENARIO.md)
4. [Foundation Recovery Plan](../docs/FOUNDATION-RECOVERY-PLAN.md)

## Quick Start

```bash
cargo run -p bd_app
```

Controls:
| Key | Action |
|---|---|
| `W`/`↑` `S`/`↓` `A`/`←` `D`/`→` | Move |
| `.` | Wait (restore AP) |
| `F` | Attack nearest enemy |
| `G` | Guard (defensive stance) |
| `P` | Pick up item |
| `U` | Use carried item |
| `B` | Open station build menu / cancel build mode |
| `A` | Cycle/assign the nearest survivor task |
| `E` | Assign survivor to station |
| `I` | Inventory screen |
| `Z` | Combat screen |
| `T` | Enter the fixed dungeon from the shelter |
| `R` | Extract at the dungeon exit |
| `?` | Context help |
| `F5` | Save the current game |
| `F9` | Load the current game |
| `1`-`5` | Select a station type in the build menu |
| `Q` / `Esc` | Quit / Cancel build mode |

Some controls are currently hardcoded despite the configuration schema.
Foundation Recovery Phase 7 will make actual input, help, and configured
bindings share one source of truth.

## Build & Run

```bash
# Debug build
cargo run -p bd_app

# Release build (optimized)
cargo run -p bd_app --release

# Content validation
cargo run -p bd_app -- --validate

# Run all tests
cargo test --workspace
```

## Project Structure

```
broken-divinity/
├── Cargo.toml              # Workspace root
├── config/                 # Default config files
├── content/                # Game data (RON files)
│   ├── symbols/            # ASCII symbol definitions
│   └── themes/             # Color theme definitions
├── crates/
│   ├── bd_app/             # Binary entry point
│   ├── bd_core/            # ECS components, systems, kernel
│   ├── bd_data/            # Content loading & validation
│   ├── bd_tui/             # Terminal UI (Ratatui widgets)
│   └── bd_test_support/    # Test helpers
├── docs/
│   ├── README.md           # Repository documentation index
│   ├── ARCHITECTURE_GUARDRAILS.md
│   ├── DEPENDENCY_MATRIX.md
│   ├── archive/            # Superseded local GDD/dev plan
│   └── decisions/          # Historical technical decisions
```

## Configuration

Config file location: `~/.config/broken-divinity/config.toml`

Copy `config/default.toml` to this location and edit. Theme and the currently
wired binding fields can be configured. Do not assume every displayed command
is configurable until Recovery Phase 7 passes.

## Save Files

Current save directory: `~/.local/share/broken-divinity/saves/`

The current turn-number save selection is a known recovery defect. Do not rely
on current development saves as a compatibility contract.

## Logs

Logs are written to stderr with configurable level via the `BD_LOG` environment variable:

```bash
BD_LOG=bd=debug cargo run -p bd_app
```

## Troubleshooting

### Terminal is garbled after exit

Run `reset` in your terminal to restore normal mode. If the app crashes, the terminal should restore automatically via `color-eyre` panic handler.

### "No config found" warning on startup

This is expected on first run. Copy `config/default.toml` to `~/.config/broken-divinity/config.toml` to customize.

## License

MIT
