# Broken Divinity Kernel

A reusable terminal-based roguelike/tactics game kernel built with Rust, Bevy ECS, and Ratatui.

## Current Status

Foundation remediation is automated-green, but Foundation acceptance remains
partially reopened. Registered contracts are currently `GreenUnreviewed`, and
the visual matrix retains explicit open evidence. The
[Foundation Test and Colony UX Hardening Plan](docs/active/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md)
owns the remaining behavior work; the
[Authoritative Testing Standard](docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md)
owns evidence sufficiency and suite migration. Automated developer-console
hardening is implemented; its
[active stabilization plan](docs/active/FOUNDATION-STABILIZATION-AND-CONSOLE-HARDENING-PLAN.md)
is `ReviewedGreen` after repaired 80x24/60x20 real-PTY review and an 841-test
canonical gate. Product P2 remains unauthorized and requires a separate
owner-approved plan.

Project authority:

1. [Product GDD](GDD.md)
2. [Locked decisions](docs/authority/DECISIONS-TO-LOCK.md)
3. [Foundation MVP scenario](docs/authority/MVP-SCENARIO.md)
4. [Foundation Test and Colony UX Hardening Plan](docs/active/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md) — active behavior work
5. [Authoritative Testing Standard](docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md) — active evidence and migration work
6. [Foundation Basic Colony Loop Plan](docs/active/FOUNDATION-BASIC-COLONY-LOOP-PLAN.md) — active colony vertical-slice work
7. [Foundation UI Improvement Plan](docs/active/FOUNDATION-UI-IMPROVEMENT-PLAN.md) — active presentation work
8. [Documentation hub](docs/README.md) — full doc inventory and navigation
9. [Completed Foundation plans](docs/archive/) — historical evidence records

## Quick Start

```bash
cargo run -p bd_app
```

Controls:
| Key | Action |
|---|---|
| `W`/`↑` `S`/`↓` `A`/`←` `D`/`→` | Move |
| `.` | Wait (restore AP) |
| `N` | Rest until next day (shelter only) |
| `F` | Attack nearest enemy |
| `G` | Guard (defensive stance) |
| `P` | Pick up item |
| `U` | Use carried item |
| `B` | Open station build menu / cancel build mode |
| `C` | Open colony management and select a survivor/task |
| `E` | Open colony management at station staffing |
| `I` | Inventory screen |
| `T` | Enter the fixed dungeon from the shelter |
| `R` | Extract at the dungeon exit |
| `?` | Context help |
| `F5` | Save the current game |
| `F9` | Load the current game |
| `1`-`5` | Select a station type in the build menu |
| `Q` / `Esc` | Quit / Cancel the active interaction |

Semantic command bindings, runtime input, contextual help, action panels, and
the footer share one binding source. Numbered build-menu selection remains a
fixed menu interaction rather than a configurable gameplay command.

Construction is modal. `B` opens station selection; `1`–`5` or Up/Down moves
the highlight; Enter changes to adjacent-tile placement; movement keys choose
the tile; and Enter builds. `B` or Escape cancels either phase. Gameplay input
is paused and previously queued gameplay is discarded while either
construction phase is active.

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

# Required final gate for every development task
bash scripts/test-gate.sh
```

Development follows the repository contract in [`AGENTS.md`](AGENTS.md) and
the owner-approved
[Authoritative Testing Standard](docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md).
Behavior changes must proceed red → green through focused contract tests before
the complete measured gate is run. A green automated gate does not replace
required GDD drift review, visual evidence, or real-terminal playtesting.

## Project Structure

```
broken-divinity/
├── Cargo.toml              # Workspace root
├── config/                 # Default config files
├── content/                # Game data (RON files)
│   ├── symbols/            # ASCII symbol definitions
│   ├── themes/             # Color theme definitions
│   ├── dungeons/           # Fixed Foundation dungeon
│   └── stations/           # Validated station catalog
├── crates/
│   ├── bd_app/             # Binary entry point
│   ├── bd_core/            # ECS components, systems, kernel
│   ├── bd_data/            # Content loading & validation
│   ├── bd_tui/             # Terminal UI (Ratatui widgets)
│   └── bd_test_support/    # Test helpers
├── docs/
│   ├── README.md           # Documentation inventory
│   ├── ARCHITECTURE_GUARDRAILS.md
│   ├── DEPENDENCY_MATRIX.md
│   ├── archive/            # Completed plans + legacy GDD/dev-plan
│   └── decisions/          # Historical technical decisions
├── legacy/                 # Archived Bevy 0.14 + egui prototype
│   └── README.md           # Explains what this is and why it's preserved
```

## Configuration

Config file location: `~/.config/broken-divinity/config.toml`

Copy `config/default.toml` to this location and edit. Theme and semantic command
bindings are validated at startup; invalid or conflicting bindings fail with a
readable configuration error. `save_dir_override` may set an explicit save
directory; otherwise the platform data directory is used.

## Save Files

Current save directory: `~/.local/share/broken-divinity/saves/`

Foundation uses one atomic `manual-slot.ron`. It supports colony, active
dungeon, extracted, and defeated states. Save/content version checks are
enforced. The current format is save version 7; development saves are not a
permanent cross-version compatibility contract.

## Current Foundation Limitations

- The playable dungeon is fixed and hand-authored; procgen is preserved but
  inactive on the Foundation path.
- Travel is the direct shelter-to-dungeon interaction, not the full overworld.
- Raids, colony events, sanity, theology-driven mechanics, faction reputation,
  final faction canon, and deeper narrative are deferred.
- Colony management is intentionally thin but explicit: three named survivors,
  targeted gathering/rest assignments, staffed station production, forecast
  and daily summary, five represented station types, and extracted-item
  storage. Storage construction is disabled because it has no Foundation
  effect.
- The supported terminal profiles are 80x24 and 60x20.

## Logs

Logs are written to stderr with configurable level via the standard
`RUST_LOG` environment variable:

```bash
RUST_LOG=bd=debug cargo run -p bd_app
```

## Troubleshooting

### Terminal is garbled after exit

Run `reset` in your terminal to restore normal mode. If the app crashes, the terminal should restore automatically via `color-eyre` panic handler.

### "No config found" warning on startup

This is expected on first run. Copy `config/default.toml` to `~/.config/broken-divinity/config.toml` to customize.

## License

MIT
