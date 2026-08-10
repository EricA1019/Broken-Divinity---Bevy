# Foundation Factory & Event Pipeline Plan

> **Status**: Implemented infrastructure — green, independently unreviewed; events and raids remain product-deferred
> **Created**: 2026-08-06
> **Depends on**: Foundation content loading pipeline (`bd_data::loader`, `FoundationContent`)
> **Test-driven**: Each phase writes contract tests first (fail), then implements (pass).

The implementation described by Phases 1–4 is present as of 2026-08-09. Its
22 contract IDs are registered in `testing/foundation-contracts.ron`: factory
catalog/marker coverage is Foundation support, action-spawn coverage is
deferred infrastructure, and event/raid coverage remains deferred
infrastructure under the GDD. This document is now an implementation record,
not authorization to expose deferred events or raids as Foundation product.

## Overview

Make the entity factory production-ready by removing the hardcoded `BlueprintRegistry`, completing the RON blueprint catalog, and creating a `BlueprintCatalog` Bevy Resource. Then build a unified event pipeline on top that merges the standalone raid system into the event/dialogue flow, with events able to spawn entities from blueprints via a new `Effect::SpawnBlueprintAt`.

### Current State Problems

| Problem | Impact |
|---|---|
| Two competing blueprint sources: hardcoded `BlueprintRegistry` (dead code) + `FoundationContent.blueprints` (live RON) | Confusion about which is authoritative |
| RON file has 3 blueprints; hardcoded registry has 11 | Missing blueprints in the live data path |
| `spawn_from_blueprint` only supports 5 component types | Callers manually insert components post-spawn |
| `BlueprintRegistry` never registered as a Bevy resource | Dead code, only used by dead `spawn_dungeon_location()` |
| `save.rs` has unused `_blueprints` parameter | Dead code noise |
| `process_raids` hardcodes entity spawns | Can't reuse for different raid types or event-driven spawning |
| Event system can't spawn entities | Events and spawns are disconnected |

### Target State

- Single source of truth: `content/blueprints/foundation.ron` → `FoundationContent.blueprints` → `BlueprintCatalog` Resource
- `BlueprintCatalog::get(id)` replaces all linear scans
- `EntityBlueprint.markers: Vec<String>` enables data-driven component attachment
- `Effect::SpawnBlueprintAt` uses the factory for entity creation from any system
- Events can spawn entities via `spawn_on_enter` / `on_exit_effects`
- Raids emit events instead of spawning directly; event system handles entity creation

---

## Phase 1: Factory Cleanup

**Goal**: Remove dead code, complete RON data, create `BlueprintCatalog` Resource, add marker support.

### Steps

1. **Write Phase 1 contract tests** — Add to `crates/bd_core/src/factory.rs` tests module (FACTORY-CATALOG-001 through FACTORY-RON-001, plus FACTORY-MIGRATE). All use existing `test_app()` + `SmokeMap` pattern. *Tests fail initially.*

2. **Remove `BlueprintRegistry`** — Delete the struct, `phase10_defaults()`, `phase18_defaults()`, and `impl Default` from `factory.rs`. This is dead code — never registered as a Bevy resource, only used by the also-dead `spawn_dungeon_location()`.

3. **Complete `content/blueprints/foundation.ron`** — Add the 8 missing blueprints from the hardcoded registry:

   | Blueprint ID | Hardcoded pool values |
   |---|---|
   | `blueprint.training_dummy` | Health(5, 0, 5) |
   | `blueprint.skeleton` | Health(15, 0, 15), ActionPoints(2, 0, 2) |
   | `blueprint.ally_warden` | Health(20, 0, 20), ActionPoints(3, 0, 3) |
   | `blueprint.sword` | (item, no pools) |
   | `blueprint.shield` | (item, no pools) |
   | `blueprint.smite_scroll` | (item, no pools) |
   | `blueprint.gold_pile` | (item, no pools) |
   | `blueprint.crypt_lord` | Health(40, 0, 40), ActionPoints(3, 0, 3) |

   Add `markers: []` to all existing and new blueprints. Keep RON player values (Health 30 + 7 virtue pools) — they're more complete than the hardcoded version (Health 20, no virtues).

4. **Create `BlueprintCatalog` Resource** — In `factory.rs`, mirroring `StationCatalog` pattern (`colony/stations.rs:260-292`):

   ```rust
   #[derive(Resource, Debug, Clone)]
   pub struct BlueprintCatalog {
       entries: Vec<EntityBlueprint>,
   }

   impl BlueprintCatalog {
       pub fn new(blueprints: Vec<EntityBlueprint>) -> Self {
           // Validate: panic on duplicate IDs
           // Validate: warn on unknown markers
           Self { entries: blueprints }
       }
       pub fn get(&self, id: &str) -> Option<&EntityBlueprint> {
           self.entries.iter().find(|bp| bp.id == id)
       }
   }

   impl Default for BlueprintCatalog {
       fn default() -> Self {
           Self { entries: Vec::new() }
       }
   }
   ```

   **Marker validation at construction time**: iterate all blueprints, check marker strings against known set (`RaidEnemy`, `FactionMember:*`), log warnings once for unknown markers, panic on duplicate blueprint IDs. Unknown markers are silently ignored during spawn.

5. **Register in app startup**:
   - `bd_app/src/main.rs` (~line 326): insert `BlueprintCatalog::new(content.blueprints.clone())` alongside `StationCatalog` insertion
   - `crates/bd_core/src/lib.rs` (~line 192): add `init_resource::<BlueprintCatalog>()` in system registration

6. **Replace linear scans with catalog lookups** — In `crates/bd_core/src/spatial.rs`:
   - Line 386: `content.blueprints.iter().find(|bp| bp.id == "blueprint.player")` → `blueprint_catalog.get("blueprint.player")`
   - Line 400: `content.blueprints.iter().find(|bp| bp.id == placement.content_id)` → `blueprint_catalog.get(&placement.content_id)`
   - Line 427: Item blueprint lookup — same pattern
   - Line 582: Player re-spawn in `initialize_outpost()` — same pattern

7. **Delete dead code**:
   - Remove `spawn_dungeon_location()` from `spatial.rs` (lines 469–512)
   - Remove unused `_blueprints` parameter from `save.rs` functions: `load_world()`, `restore_world()`, `restore_snapshot_into()`

8. **Add `markers` field to `EntityBlueprint`**:
   ```rust
   pub struct EntityBlueprint {
       // ... existing fields ...
       pub markers: Vec<String>,  // e.g. ["RaidEnemy", "FactionMember:faction.demons"]
   }
   ```
   In `spawn_from_blueprint`, after existing fixed components, iterate markers:
   - `"RaidEnemy"` → insert `RaidEnemy` (imported from `colony::raids`, same crate)
   - `"FactionMember:ID"` → insert `FactionMember { faction_id: "ID".into() }`
   - Unknown markers: silently ignored (validated at catalog construction)

9. **Migrate existing factory tests** — Rewrite tests using `BlueprintRegistry::phase10_defaults()` to use `BlueprintCatalog::new(...)` with inline blueprint data. Keep assertions the same; only change the data source.

### Phase 1 Test Contracts

| ID | Test | Given | When | Then |
|---|---|---|---|---|
| FACTORY-CATALOG-001 | `catalog_get_returns_blueprint_by_id` | Catalog with player + rat | `catalog.get("blueprint.rat")` | Returns `Some`, `label == "Rat"`, `blocks_movement == true` |
| FACTORY-CATALOG-002 | `catalog_get_unknown_returns_none` | Same catalog | `catalog.get("nonexistent")` | Returns `None` |
| FACTORY-CATALOG-003 | `catalog_construction_warns_unknown_markers` | Blueprint with `markers: ["Bogus"]` | `BlueprintCatalog::new(...)` | Warning logged, catalog created, no panic |
| FACTORY-CATALOG-004 | `catalog_construction_panics_on_duplicate_ids` | Two blueprints with same id | `BlueprintCatalog::new(...)` | Panics |
| FACTORY-MARKER-001 | `spawn_inserts_marker_components` | Blueprint with `markers: ["RaidEnemy"]` | `spawn_from_blueprint(...)` | Entity has `RaidEnemy` component |
| FACTORY-MARKER-002 | `spawn_silently_ignores_unknown_marker` | Blueprint with unknown marker | `spawn_from_blueprint(...)` | Entity spawns successfully |
| FACTORY-MARKER-003 | `spawn_multiple_markers_with_data` | `markers: ["RaidEnemy", "FactionMember:faction.demons"]` | Spawn | Entity has both `RaidEnemy` and `FactionMember { faction_id: "faction.demons" }` |
| FACTORY-RON-001 | `ron_loads_required_blueprints` | Content dir | `load_foundation_content()` | Has `blueprint.player`, `blueprint.rat`, `blueprint.healing_potion`; all have non-empty id/label; no duplicate IDs |
| FACTORY-MIGRATE | Existing tests pass with catalog | N/A | `cargo test -p bd_core` | All existing factory tests pass |

### Phase 1 Relevant Files

| File | Change |
|---|---|
| `crates/bd_core/src/factory.rs` | BlueprintRegistry removal, BlueprintCatalog, markers, validation, tests |
| `content/blueprints/foundation.ron` | Add 8 missing blueprints + markers field |
| `bd_app/src/main.rs` | Insert BlueprintCatalog resource (~line 326) |
| `crates/bd_core/src/lib.rs` | `init_resource::<BlueprintCatalog>()` |
| `crates/bd_core/src/spatial.rs` | Replace linear scans, delete `spawn_dungeon_location()` |
| `crates/bd_core/src/save.rs` | Remove unused `_blueprints` params |
| `crates/bd_core/src/content.rs` | No changes (data source unchanged) |

### Phase 1 Decisions

- Marker validation at catalog construction, not spawn time — avoids `GameLog` dependency in `spawn_from_blueprint`
- Duplicate blueprint IDs → panic (data integrity error, fail fast)
- Linear scan lookup (not HashMap) — matches `StationCatalog`, blueprint count ~11
- Save round-trip for `blueprint_id` **deferred** — `_blueprints` params removed, re-added when save system gets own pass
- `RaidEnemy` import from `colony::raids` into `factory.rs` — same crate, unit struct
- RON player values (Health 30 + virtues) kept over hardcoded (Health 20)

---

## Phase 2: Effect::SpawnBlueprintAt

**Goal**: Add a new `Effect` variant that spawns entities from blueprints via the factory, usable from actions and (later) events.

### Steps

1. **Write Phase 2 contract tests** — Add to `crates/bd_core/src/actions.rs` tests module (ACTION-SPAWN-001 through 004). Uses `test_app()` with `BdCorePlugin` + `BlueprintCatalog` + `SmokeMap`. *Tests fail initially.*

2. **Add `Effect::SpawnBlueprintAt` variant** — In `actions.rs` `Effect` enum (near line 103, alongside existing `SpawnEntity`):
   ```rust
   SpawnBlueprintAt {
       blueprint_id: String,
       x: i32,
       y: i32,
       mutators: Vec<Mutator>,
   }
   ```
   Distinct from existing `SpawnEntity(String)` which is hardcoded for station construction.

3. **Wire resolver in `resolve_action_effects`** — In the effect match block (~line 1240):
   - Look up `blueprint_catalog.get(blueprint_id)`, warn-and-skip if missing
   - Call `spawn_from_blueprint(bp, Some(Position { x, y }), mutators, &mut commands)`
   - Attach `EntityScope::Tactical` (dungeon) or `EntityScope::ColonyPersistent` (outpost) based on current `GameMode`
   - Log spawn to `GameLog` if player present

4. **Add `BlueprintCatalog` to `ActionResolutionLocation`** — Add `blueprint_catalog: Res<'w, BlueprintCatalog>` field.

### Phase 2 Test Contracts

| ID | Test | Given | When | Then |
|---|---|---|---|---|
| ACTION-SPAWN-001 | `spawn_blueprint_at_creates_entity` | Action with `SpawnBlueprintAt { "blueprint.rat", 5, 3, [] }` | `resolve_action_effects` | Entity at Position(5,3), Name("Rat"), BlocksMovement, no Player |
| ACTION-SPAWN-002 | `spawn_blueprint_at_applies_mutators` | Same with `mutators: [Elite]`, rat Health(11,0,11) | Resolve | Health.max == 16 (11 × 1.5 truncated) |
| ACTION-SPAWN-003 | `spawn_blueprint_at_missing_warns` | `blueprint_id: "blueprint.missing"` | Resolve | No entity spawned, warning in GameLog, no panic |
| ACTION-SPAWN-004 | `spawn_blueprint_at_correct_scope` | Tactical mode | Resolve | `EntityScope::Tactical` |
| | | Outpost mode | Resolve | `EntityScope::ColonyPersistent` |

### Phase 2 Relevant Files

| File | Change |
|---|---|
| `crates/bd_core/src/actions.rs` | Effect variant, resolver match arm, ActionResolutionLocation, tests |

---

## Phase 3: Event System Extension

**Goal**: Enable events to spawn entities and fire effects on node exit. Wire `Commands` + `BlueprintCatalog` into event processing.

### Design Decision: `on_exit_effects` Semantics

`on_exit_effects` fire **whenever a node is exited** — both:
1. Transition to another node (choice with `next_node`)
2. Event termination (choice with no `next_node`)

This is the more general contract: "whenever the player leaves this node, these effects fire." For raid events, this allows on_exit to trigger mode transitions regardless of which dialogue branch was chosen.

### Steps

1. **Write Phase 3 contract tests** — Add to `crates/bd_core/src/events.rs` tests module (EVENT-SPAWN-001 through 005). Uses `BdCorePlugin` (not foundation) with manually inserted `EventRegistry` + `BlueprintCatalog` + `CurrentEvent`. *Tests fail initially.*

2. **Add `on_exit_effects` to `EventNode`**:
   ```rust
   pub struct EventNode {
       // ... existing fields ...
       pub on_exit_effects: Vec<Effect>,  // fires on node exit (transition OR event end)
   }
   ```

3. **Add `spawn_on_enter` to `EventDefinition`**:
   ```rust
   pub struct EventDefinition {
       // ... existing fields ...
       pub spawn_on_enter: Vec<Effect>,  // entity creation, fires after on_enter_effects
   }
   ```
   Kept separate from `on_enter_effects` for clarity: `on_enter_effects` = gameplay mutations (PoolDelta, Flag), `spawn_on_enter` = entity creation.

4. **Add `Commands` + `BlueprintCatalog` to event systems** — Both `process_event_triggers` and `process_event_choices` need:
   ```rust
   commands: Commands,
   blueprint_catalog: Res<BlueprintCatalog>,
   ```
   Currently they skip `SpawnEntity` with `_ => {}` (events.rs line ~356). After this change, they resolve `SpawnBlueprintAt` inline.

5. **Wire `on_exit_effects` in `process_event_choices`** — After choice effects resolve and before advancing/ending the node, apply `on_exit_effects` from the node being exited. Both transition and termination paths.

6. **Wire `spawn_on_enter` in `process_event_triggers`** — After applying `on_enter_effects`, apply `spawn_on_enter`.

7. **Define RON event data** — Create `content/events/foundation.ron` with a sample raid alert event.

### Phase 3 Test Contracts

| ID | Test | Given | When | Then |
|---|---|---|---|---|
| EVENT-SPAWN-001 | `spawn_on_enter_creates_entities` | Event with `spawn_on_enter: [SpawnBlueprintAt { "blueprint.rat", 2, 2, [] }]` | `process_event_triggers` | Entity at (2,2), Name("Rat"), event active |
| EVENT-SPAWN-002 | `on_exit_fires_on_event_end` | Node with `on_exit_effects: [SpawnBlueprintAt { "blueprint.rat", 3, 3, [] }]`, choice with no `next_node` | Choice selected | Entity at (3,3), event not active |
| EVENT-SPAWN-003 | `on_exit_fires_on_node_transition` | Node with `on_exit_effects`, choice WITH `next_node` | Choice selected | on_exit effects fire, event advances to next_node |
| EVENT-SPAWN-004 | `mixed_effects_in_spawn_on_enter` | `spawn_on_enter: [PoolDelta { ... }, SpawnBlueprintAt { ... }]` | Event triggers | Both effects resolve (pool changed + entity spawned) |
| EVENT-SPAWN-005 | `invalid_blueprint_does_not_block_event` | `spawn_on_enter` with missing blueprint_id | Event triggers | Event active, dialogue displays, warning logged, no crash |

### Phase 3 Relevant Files

| File | Change |
|---|---|
| `crates/bd_core/src/events.rs` | EventNode.on_exit_effects, EventDefinition.spawn_on_enter, Commands+Catalog params, wiring, tests |
| `content/events/foundation.ron` | Sample event data (new file) |
| `crates/bd_data/src/loader.rs` | Load events from content dir (check if needed) |
| `crates/bd_core/src/content.rs` | FoundationContent.events field (check if needed) |

---

## Phase 4: Raid Unification

**Goal**: Replace hardcoded entity spawning in `process_raids` with event-driven spawning. Raids push `CurrentEvent`; the event system handles entity creation via `spawn_on_enter`.

### Prerequisites

Before Phase 4 implementation:
- Add `FoundationDriver::current_event_id() -> Option<String>` and `FoundationDriver::event_is_active() -> bool` to `bd_test_support`
- Create `RaidTestPlugin` that registers `EventRegistry` + `RaidState` + `BlueprintCatalog` for use with `FoundationDriver::new_with_plugin(seed, RaidTestPlugin)`

### Steps

1. **Write Phase 4 contract tests** — Add to `crates/bd_core/src/colony/raids.rs` tests module (RAID-EVENT-001 through 003). Uses `FoundationDriver::new_with_plugin(seed, RaidTestPlugin)`. *Tests fail initially.*

2. **Define raid events in RON** — Add to `content/events/foundation.ron`:
   - `event.raid.small`: 2–3 rats, dialogue about noises at the gate
   - `event.raid.medium`: 4–5 skeletons, dialogue about organized assault
   - Each uses `spawn_on_enter` with `SpawnBlueprintAt` at randomized positions near outpost perimeter

3. **Refactor `process_raids` to emit events**:
   - When a raid triggers, push `CurrentEvent` with the appropriate event definition ID
   - Event system handles spawning via `spawn_on_enter`
   - Remove hardcoded spawn + `RaidEnemy` marker logic from `raids.rs`

4. **Add markers to raid blueprints** — In `foundation.ron`, add `markers: ["RaidEnemy"]` to `blueprint.rat` and `blueprint.skeleton`.

5. **Remove `RaidState` standalone spawning** — Delete hardcoded spawn section. Keep `RaidState` for timing/probability; decouple from entity creation.

### Phase 4 Test Contracts

| ID | Test | Given | When | Then |
|---|---|---|---|---|
| RAID-EVENT-001 | `raid_pushes_event_not_direct_spawn` | Outpost, day 5, raid roll succeeds | `process_raids` | `CurrentEvent` pushed with raid event ID, event active |
| RAID-EVENT-001b | `no_raid_enemy_before_event_resolution` | After `process_raids`, before event resolution | Query `RaidEnemy` | No entities have `RaidEnemy` component |
| RAID-EVENT-002 | `event_spawn_creates_raiders` | Raid event with `spawn_on_enter` | Event triggers | Entities have `RaidEnemy` marker (from blueprint), `EntityScope::ColonyPersistent` |
| RAID-EVENT-003 | `raid_spawn_uses_blueprint_pools` | Raid using `blueprint.rat` with `mutators: [Elite]` | Spawn | Health.max == 16, ActionPoints.max == 2 |

### Phase 4 Relevant Files

| File | Change |
|---|---|
| `crates/bd_core/src/colony/raids.rs` | Refactor to emit events, add tests |
| `crates/bd_test_support/src/lib.rs` | Add `current_event_id()`, `event_is_active()`, `RaidTestPlugin` |
| `content/events/foundation.ron` | Raid event definitions |
| `content/blueprints/foundation.ron` | Add markers to raid enemy blueprints |

---

## Phase 5: Polish & Integration Testing

### Steps

1. **End-to-end test** — Integration test loading foundation content, triggering raid, verifying spawn + dialogue + combat.

2. **Edge case tests** — No blueprints available, duplicate spawns, event chaining (on_exit triggers another event).

3. **Evaluate `Effect::SpawnEntity` deprecation** — The old `SpawnEntity(String)` variant stays for station construction. Add doc comment:
   ```rust
   /// DEPRECATED: prefer SpawnBlueprintAt for entity spawning. This variant is specialized for station construction.
   SpawnEntity(String),
   ```

4. **Update contract registry** — Add all 21 test contracts to `testing/foundation-contracts.ron`.

### Phase 5 Verification

- `just test` — full suite passes
- `cargo test -p bd_core` — all unit tests pass
- Manual playtest: enter outpost, wait for raid, verify dialogue + spawn + combat

---

## Test Contract Master List

| ID | Phase | Location | Category |
|---|---|---|---|
| FACTORY-CATALOG-001 | 1 | factory.rs | Catalog lookup |
| FACTORY-CATALOG-002 | 1 | factory.rs | Missing blueprint |
| FACTORY-CATALOG-003 | 1 | factory.rs | Unknown marker warning |
| FACTORY-CATALOG-004 | 1 | factory.rs | Duplicate ID detection |
| FACTORY-MARKER-001 | 1 | factory.rs | Marker component insertion |
| FACTORY-MARKER-002 | 1 | factory.rs | Unknown marker silent skip |
| FACTORY-MARKER-003 | 1 | factory.rs | Multiple markers + data convention |
| FACTORY-RON-001 | 1 | factory.rs | RON data integrity |
| FACTORY-MIGRATE | 1 | factory.rs | Existing test regression |
| ACTION-SPAWN-001 | 2 | actions.rs | Spawn at position |
| ACTION-SPAWN-002 | 2 | actions.rs | Mutator application |
| ACTION-SPAWN-003 | 2 | actions.rs | Missing blueprint safety |
| ACTION-SPAWN-004 | 2 | actions.rs | EntityScope by GameMode |
| EVENT-SPAWN-001 | 3 | events.rs | spawn_on_enter creates entities |
| EVENT-SPAWN-002 | 3 | events.rs | on_exit on event end |
| EVENT-SPAWN-003 | 3 | events.rs | on_exit on node transition |
| EVENT-SPAWN-004 | 3 | events.rs | Mixed effects |
| EVENT-SPAWN-005 | 3 | events.rs | Invalid blueprint safety |
| RAID-EVENT-001 | 4 | raids.rs | Raid pushes event |
| RAID-EVENT-001b | 4 | raids.rs | No spawn before event resolution |
| RAID-EVENT-002 | 4 | raids.rs | RaidEnemy marker via event |
| RAID-EVENT-003 | 4 | raids.rs | Blueprint pools + mutators |

## Brittleness Safeguards

What these tests deliberately avoid:

- **No raw ECS query-order assertions** — use stable accessors (`catalog.get()`, `FoundationDriver` methods)
- **No exact HP values except mutator math** — use presence/absence and proportional checks
- **No RON string matching** — parse and check structured fields
- **No frame-counting for timing** — use state assertions (`is_active()`, `RaidState::Active`)
- **No bare negative assertions** — split RAID-EVENT-001 into positive (event pushed) + negative (no RaidEnemy yet)
- **Marker tests use string match, not `TypeId`** — renaming a component only changes the RON string
- **No internal `World` mutation in tests** — all through public APIs (`spawn_from_blueprint`, `resolve_action_effects`, `process_event_triggers`)

## Key Architecture Decisions

| Decision | Rationale |
|---|---|
| Marker validation at catalog construction | Avoids `GameLog` dependency in `spawn_from_blueprint`; validates once not per-spawn |
| Duplicate IDs → panic | Data integrity error, fail fast at startup |
| `on_exit_effects` fire on ALL node exits | More general contract; supports mode transitions from any dialogue branch |
| Linear scan catalog lookup | Matches `StationCatalog`; ~11 blueprints; HashMap overhead not justified |
| `spawn_on_enter` separate from `on_enter_effects` | Semantic clarity: entity creation vs gameplay mutation |
| Raid timing stays in `raids.rs` | Only entity creation moves to events; probability/timing is raid-specific logic |
| `RaidEnemy` imported into `factory.rs` | Same crate, unit struct, zero-cost; no architectural violation |
| Event + action resolvers stay separate | DRY refactor is deferred debt; not in scope for this plan |

## Scope Exclusions

- Save/restore blueprint wiring (deferred to save system pass)
- Component bundle system (only string-matched markers)
- Blueprint hot-reload
- Station blueprint unification (`SpawnEntity` stays for stations)
- Event/action effect resolution DRY refactor
- Dialogue tree authoring tools

---

## Relevant Files Master List

| File | Phase | Change |
|---|---|---|
| `crates/bd_core/src/factory.rs` | 1 | BlueprintRegistry removal, BlueprintCatalog, markers, catalog validation, tests |
| `content/blueprints/foundation.ron` | 1, 4 | Complete blueprints, add markers |
| `bd_app/src/main.rs` | 1 | Insert BlueprintCatalog resource |
| `crates/bd_core/src/lib.rs` | 1 | `init_resource::<BlueprintCatalog>()` |
| `crates/bd_core/src/spatial.rs` | 1 | Replace linear scans, delete dead code |
| `crates/bd_core/src/save.rs` | 1 | Remove unused `_blueprints` params |
| `crates/bd_core/src/actions.rs` | 2 | Effect variant, resolver, tests |
| `crates/bd_core/src/events.rs` | 3 | EventNode/EventDefinition fields, Commands+Catalog, wiring, tests |
| `content/events/foundation.ron` | 3, 4 | Event definitions (new file) |
| `crates/bd_core/src/colony/raids.rs` | 4 | Event-driven refactor, tests |
| `crates/bd_test_support/src/lib.rs` | 4 | Driver event inspection, RaidTestPlugin |
| `crates/bd_data/src/loader.rs` | 3 | Events loading (check) |
| `crates/bd_core/src/content.rs` | 3 | Events field (check) |
| `testing/foundation-contracts.ron` | 5 | Contract registry updates |
