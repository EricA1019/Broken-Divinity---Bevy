# Broken Divinity Kernel

A reusable terminal-based roguelike/tactics game kernel built with Rust, Bevy ECS, and Ratatui.

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
| `B` | Cycle station type (Stove→Altar→Workshop→Bed→Storage) |
| `I` | Inventory screen |
| `Z` | Combat screen |
| `T` | Travel to next location |
| `R` | Return to outpost |
| `1`-`9` | Select event/dialogue choice |
| `Q` / `Esc` | Quit / Cancel build mode |

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
│   ├── ARCHITECTURE_GUARDRAILS.md
│   ├── DEPENDENCY_MATRIX.md
│   └── decisions/          # Architecture Decision Records
```

## Configuration

Config file location: `~/.config/broken-divinity/config.toml`

Copy `config/default.toml` to this location and edit. All key bindings are configurable.

## Save Files

Save directory: `~/.local/share/broken-divinity/`

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
