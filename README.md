# Broken Divinity Kernel

A reusable terminal-based roguelike/tactics game kernel built with Rust, Bevy ECS, and Ratatui.

## Current Status

The Broken Divinity Foundation MVP passed its final recovery gate on
2026-07-24. The canonical scenario passes 14/14, automated workspace/content
gates pass without warnings, and terminal extraction, resume, defeat, and
save/load paths have been manually audited.

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

Semantic command bindings, runtime input, contextual help, action panels, and
the footer share one binding source. Numbered build-menu selection remains a
fixed menu interaction rather than a configurable gameplay command.

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

Copy `config/default.toml` to this location and edit. Theme and semantic command
bindings are validated at startup; invalid or conflicting bindings fail with a
readable configuration error.

## Save Files

Current save directory: `~/.local/share/broken-divinity/saves/`

Foundation uses one atomic `manual-slot.ron`. It supports colony, active
dungeon, extracted, and defeated states. Save/content version checks are
enforced; development saves are not a permanent cross-version compatibility
contract.

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
