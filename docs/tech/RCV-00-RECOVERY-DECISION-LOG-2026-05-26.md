# RCV-00 Recovery Decision Log (2026-05-26)

## Objective
Determine what is missing in this checkout and define the minimum executable recovery path.

## Crate Root Status
1. Cargo manifest status: missing (`Cargo.toml` not present at project root).
2. Rust library root status: missing (`src/lib.rs` not present).
3. Current gate status from project root:
- `cargo metadata --no-deps`: FAIL (missing `Cargo.toml`)
- `cargo check`: FAIL (missing `Cargo.toml`)

## Source Inventory
Current files under `src/`:
1. `src/main.rs`
2. `src/tests.rs`
3. `src/core/mod.rs`
4. `src/core/movement.rs`
5. `src/core/player.rs`
6. `src/core/resources.rs`
7. `src/core/save.rs`
8. `src/core/stats.rs`
9. `src/core/turn.rs`
10. `src/game/colony/stations.rs`
11. `src/game/dungeon/consumables.rs`
12. `src/game/dungeon/mod.rs`
13. `src/game/dungeon/spawn.rs`
14. `src/game/overworld/travel.rs`
15. `src/ui/colony_panel.rs`
16. `src/ui/inventory_panel.rs`
17. `src/ui/menu.rs`

## Missing Module Decision Table
Declared/referenced modules with no matching file in checkout and RCV decision:

### Core
1. `core::abilities` -> reconstruct minimal compile-safe module.
2. `core::camera` -> remove wiring in compile stabilization; revisit in feature restoration.
3. `core::components` -> reconstruct minimal compile-safe module.
4. `core::fov` -> remove wiring in compile stabilization.
5. `core::gamelog` -> reconstruct minimal compile-safe module.
6. `core::inventory` -> reconstruct minimal compile-safe module.
7. `core::items` -> reconstruct minimal compile-safe module.
8. `core::perks` -> reconstruct minimal compile-safe module.
9. `core::sanity` -> reconstruct minimal compile-safe module.
10. `core::state` -> reconstruct minimal compile-safe module.
11. `core::status` -> remove wiring in compile stabilization.
12. `core::tilemap` -> remove wiring in compile stabilization.
13. `core::escape` -> defer to post-compile restoration (AXT path).
14. `core::brp_safety` -> defer to post-compile restoration.

### Game roots
1. `game::mod.rs` root -> reconstruct minimal root.
2. `game::combat` -> remove runtime wiring in compile stabilization.
3. `game::factions` -> reconstruct minimal compile-safe module.

### Colony
1. `game::colony::mod.rs` root -> reconstruct minimal root.
2. `game::colony::raids` -> reconstruct minimal compile-safe module.
3. `game::colony::research` -> reconstruct minimal compile-safe module.
4. `game::colony::spawn` -> reconstruct minimal compile-safe module.
5. `game::colony::survivors` -> reconstruct minimal compile-safe module.

### Dungeon
1. `game::dungeon::{ai, anomalies, bsp, enemies, gabriel, hazards, loot, lore, melee, ranged, theme}` -> remove non-essential wiring in compile stabilization, then reconstruct as needed by active files.

### Overworld
1. `game::overworld::mod.rs` root -> reconstruct minimal root.
2. `game::overworld::{weather, graphgen, map}` -> reconstruct minimal compile-safe modules required by current files.

### UI
1. `ui::mod.rs` root -> reconstruct minimal root.
2. `ui::{gamelog_panel, gameover, gabriel_dialogue_panel, help_panel, hud, journal_panel, objective_prompt, overworld_panel, perk_choice_panel, readability, modal_priority}` -> remove runtime wiring in compile stabilization; restore only where required by active Alpha tickets.

## Boundary Decision
Out-of-scope runtime for RCV recovery:
1. `graphify/` subproject (Python tooling) is excluded.

## RCV Decision
1. Proceed with RCV-01 by creating crate manifest and library root.
2. Proceed with RCV-02 by stabilizing compile through minimal root/wiring reconciliation first.
3. Defer broad behavior restoration to post-compile tickets.

## Risks (Immediate)
1. Existing `src/tests.rs` references many missing modules; test execution will remain blocked until broader reconstruction.
2. Any attempt to restore all missing modules in one pass will likely trigger major scope expansion.
3. Compile stabilization may require temporarily shrinking runtime wiring before later feature restoration.
