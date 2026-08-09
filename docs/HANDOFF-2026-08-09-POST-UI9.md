# Broken Divinity — Post-UI9 State Handoff (2026-08-09)

## Timeline

| Period | Focus | Commits |
|---|---|---|
| 2026-07-25/30 | Foundation Recovery + Colony Production Loop | — |
| 2026-07-31 | Foundation Colony UX Hardening + UI9-C Context | 3 (342b82d, fd94c98, ccc9e83) |
| 2026-08-01/02 | Cinder Rite theme, bug log entries | — |
| 2026-08-07 | Factory overhaul + event pipeline RON migration (Session 1) | ~10 |
| 2026-08-07/08 | Raid tier selection + RON migration (Session 2) | ~8 |
| 2026-08-08/09 | **Developer CLI console (`bd_console` crate)** (Session 3) | 14 |

## Architecture State

### Crates (5 → 6)
- `bd_core` — ECS types, signals, systems, colony, spatial, events, factory
- `bd_tui` — Ratatui rendering, input routing, screens, view models
- `bd_app` — Binary entry point, plugin wiring, content loading
- `bd_data` — RON content loader
- `bd_test_support` — FoundationDriver, test helpers, contract registry
- **`bd_console`** ← NEW: developer CLI console (Aug 8-9)

### Plugin graph
```
bd_app
  ├── BdCorePlugin (full: events, raids, Gabriel, sanity, overworld, factions)
  ├── BdConsolePlugin ← NEW
  ├── BdTuiPlugin (input guard checks ConsoleState.open)
  └── content loading (blueprints, events, stations, symbols, themes)
```

### System pipeline (unchanged)
```
Input → IntentCollection → Validation → CostResolution → Mutation → ResultEmission → ViewModelBuild → Render
                                                          └── PoolDeltas (entity pools only)
```

### Key architectural decisions (post-UI9)

1. **D-20 Colony loop**: Physical gather/refine, construction turns, deterministic fixtures. Data-driven via `content/colony_*` RON files.

2. **BlueprintCatalog**: Replaced hardcoded `BlueprintRegistry`. Blueprints loaded from `content/blueprints/foundation.ron`, inserted as Resource. `spawn_from_blueprint()` is the canonical factory. Marker system for `RaidEnemy`, `FactionMember:id`.

3. **Event pipeline**: All events in `content/events/foundation.ron` (data-driven). `process_raids` emits `EventTrigger` instead of direct spawn. Tier selection: enemy_count 1-2 → `event.raid.small`, 3 → `event.raid.medium`.

4. **Colony resources vs entity pools**: Two separate mutation models by design:
   - Entity pools (Health, AP, virtues) → `PoolDeltaRequested` → `resolve_pool_deltas()`
   - Colony resources (Supplies, Materials, WildPlants, Faith) → direct `ColonyResources.pools.get_mut()` in 8+ systems
   - `pools.rs` doc comment clarified 2026-08-09

5. **Multiple spawn paths are intentional**: Factory for statted entities (player, enemies, items), direct spawn for special entities (Gabriel, exit tiles, resource nodes, survivors). Survivors lack a blueprint — adding one would be a feature.

6. **Colony movement ≠ dungeon movement**: Dungeon uses `ActionIntent` turn-granular pipeline. Colony uses 3 independent work/movement systems (direct gather, station assignment, logistics recipes) on a day-granular temporal model. By design — not duplication.

7. **`observe_player_defeat` bypasses TransitionIntent**: Must switch to GameOver before entity despawn. Documented 2026-08-09.

### Effect::SpawnEntity deprecation
Logged as STRUCT-002 in `docs/bug-reports/BUGLOG.md`. The variant persists in `stations.rs` for station construction but name is misleading — should be renamed to `BuildStation`.

## bd_console Crate (Developer CLI Console)

### Files
```
crates/bd_console/
  ├── Cargo.toml
  └── src/
      ├── lib.rs          — BdConsolePlugin, ConsoleCommand message, integration tests
      ├── state.rs        — ConsoleState resource (open, buffer, history, pending, output)
      ├── commands.rs     — DebugCommand enum (20 variants) + parser (51 tests)
      ├── dispatch.rs     — execute_console_command() exclusive system (22 tests)
      ├── input.rs        — capture_console_input() system (now runs via bd_tui guard)
      └── render.rs       — render_console() ratatui overlay (6 tests)
```

### Commands
| Category | Commands |
|---|---|
| Resources | `s`/`supplies <n>`, `m`/`materials <n>`, `f`/`faith <n>`, `p`/`plants <n>` |
| Time | `day <n>`, `turn <n>`, `skip_day` |
| Events | `event <id>`, `end_event` |
| Combat | `kill_all`, `heal`, `god on`/`god off` |
| Survivors | `survivor <name>`, `task <idx> idle\|defending\|resting` |
| Spawn/Teleport | `spawn <bp_id> <x> <y>`, `goto <x> <y>`, `shelter` |
| Info | `blueprints`, `events`, `stats`, `help`, `clear` |

### Architecture notes
- **Input routing**: Lives in `bd_tui::console_input_guard` (NOT in bd_console). Runs `.before(map_input_to_intents)` in `BdSet::Input`. Uses `Option<ResMut<ConsoleState>>` for graceful degradation when console not available.
- **Dispatch**: Exclusive system (`&mut World`) in `BdSet::Mutation`. Reads `ConsoleState.pending`. Writes standard signals for gameplay ops, direct mutation for admin ops.
- **Render**: Ratatui overlay in `BdSet::Render` with `.after(draw_ui)`. Bottom 40% of terminal.
- **Guard**: `map_input_to_intents` checks `input.console_state` (added to `InputQueries` SystemParam) and returns early when open.

### Bug fix (2026-08-09)
Resource commands (`supplies`/`materials`/`faith`/`plants`) were silent no-ops. They wrote `PoolDeltaRequested` targeting the player entity, which lacks colony resource pools. `resolve_pool_deltas` silently dropped them. Now mutate `ColonyResources.pools` directly, showing before/after values.

### Completion features
- Tab completion (24 known commands, common prefix, multi-match suggestions)
- History search (Up/Down filtered by buffer prefix)
- Welcome message on open
- Color-coded output (ERROR=red, OK=green)
- Entity completeness (EntityScope, PersistentEntity, Statuses on spawned entities)
- GodMode marker component (invincibility toggle)

### Tests
- 99 tests in `bd_console`: 51 parser, 14 input state machine, 22 dispatch, 6 render, 6 integration
- Full workspace green (only pre-existing `contract_registry` seeded_registry failure)

## Remaining Known Issues

### ACTIVE (from BUGLOG.md)
- **STRUCT-002**: `Effect::SpawnEntity` misnamed — should be `BuildStation`

### Pre-existing test failure
- `seeded_registry_maps_current_foundation_contract_batches` — contract_registry test, pre-dates all post-UI9 work

### Outstanding cleanup
- `crates/bd_console/src/lib.rs` has 3 stale system-dependent tests (backtick_toggles, typing_populates, escape_closes) that pass coincidentally because `capture_console_input` is no longer in bd_console's schedule. Should be removed or migrated to bd_app integration tests.

## Key Files by Concern

| Concern | Files |
|---|---|
| Console | `crates/bd_console/` (7 files), `crates/bd_tui/src/lib.rs` (console_input_guard + InputQueries + guard check) |
| Factory | `crates/bd_core/src/factory.rs` (BlueprintCatalog, spawn_from_blueprint) |
| Events | `crates/bd_core/src/events.rs`, `content/events/foundation.ron` |
| Raids | `crates/bd_core/src/colony/raids.rs` (tier selection, event-driven) |
| Colony | `crates/bd_core/src/colony/production.rs`, `survivors.rs`, `logistics.rs`, `resources.rs` |
| Spatial | `crates/bd_core/src/spatial.rs` (GameMode, transitions, initialize_outpost) |
| Pools | `crates/bd_core/src/pools.rs` (entity pool mutation, observe_player_defeat) |
| Signals | `crates/bd_core/src/signals.rs` (PoolDeltaRequested, EventTrigger, etc.) |
| Content | `crates/bd_data/src/loader.rs`, `content/` (RON data) |

## Dependencies

```
bd_app → bd_tui, bd_console, bd_core, bd_data
bd_tui → bd_console, bd_core
bd_console → bd_core, bevy_ratatui, ratatui, crossterm
bd_data → bd_core
bd_test_support → bd_core, bd_data
```

## Commands

```bash
# Run all tests
cargo test --workspace

# Run console tests only
cargo test -p bd_console --lib

# Run specific test suite
cargo test -p bd_console --lib dispatch::tests

# Run the game
cargo run --bin bd

# Clean build cache (if incremental cache corrupts)
cargo clean -p bd_tui
```
