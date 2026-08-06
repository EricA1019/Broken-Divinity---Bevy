# Legacy Codebase — Archived 2026-08-06

This directory contains the **Bevy 0.14 + egui + tilemap** prototype codebase
that predates the current **Bevy 0.18 + Ratatui** workspace in `crates/`.

## What is this?

These are the original Broken Divinity prototypes developed from 2025 through
mid-2026. The code uses:

- Bevy 0.14 ECS with `bevy_egui` for UI panels
- `bevy_ecs_tilemap` for map rendering
- `AppState` / `TurnPhase` state machines
- A `src/core/` / `src/game/` / `src/ui/` module layout

## Does this code compile?

**No.** The workspace `Cargo.toml` does not include this directory. The code
references crates (`bevy_egui`, `bevy_ecs_tilemap`) and internal module paths
(`broken_divinity::core::plugin`, `broken_divinity::game::colony::plugin`) that
do not resolve under the current workspace.

## Why preserve it?

This codebase contains reference implementations of:

- BSP dungeon generation (`src/game/dungeon/bsp/`)
- Enemy AI behaviors (`src/game/dungeon/ai.rs`)
- Ranged combat (`src/game/dungeon/ranged.rs`)
- Overworld graph generation (`src/game/overworld/graphgen/`)
- Shelter colony map generation (`src/game/colony/mapgen/`)
- Faction generation (`src/game/factions.rs`)
- UI panel designs (`src/ui/` — gabriel dialogue, inventory, journal, overworld, perks)
- Perk and ability catalogs (`src/core/perks.rs`, `src/core/abilities.rs`)
- Save/load schema (`src/core/save.rs`)

These algorithms and designs informed the current `crates/` implementation and
may be useful references for future procedural generation, UI layout, or system
design work.

## What should I use instead?

The active codebase lives in `crates/`:

| Concern | Active Location |
|---|---|
| Core ECS types | `crates/bd_core/src/` |
| Terminal UI | `crates/bd_tui/src/` |
| Application entry | `crates/bd_app/src/` |
| Content loading | `crates/bd_data/src/` |
| Test support | `crates/bd_test_support/src/` |
| Content data | `content/` (RON files) |
| Configuration | `config/default.toml` |

Build with: `cargo build -p bd_app`
Run with: `cargo run -p bd_app`
Test with: `cargo test --workspace`

## Can I delete this?

If you are certain none of the reference implementations above are needed,
this directory can be safely deleted — it is not referenced by any build
script, test, or configuration in the active workspace.

When in doubt, keep it. Disk space is cheap; lost reference code is not.
