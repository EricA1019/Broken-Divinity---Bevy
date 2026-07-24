# Broken Divinity — Detailed Phase Breakdown

**Date**: 2026-07-22
**Preflight**: 2026-07-22 — live interactive test performed. Title screen clean. Outpost screen renders correctly. Two bugs found.
**Current**: 227 tests, 29 bd_core modules, 9 screens, 16 widgets, 5 actions

---

## Preflight Findings (2026-07-22)

### What Actually Works

| Check | Result |
|-------|--------|
| Title screen ASCII art | ✅ Clean, centered, no garbling |
| Press any key → Outpost | ✅ Mode switches, screen updates |
| Outpost Party panel | ✅ Survivor 1, 2, 3 visible |
| Outpost Map panel | ✅ 40×30 shelter: `#` walls, `.` floor, `@` player at (20,15) |
| Outpost Stats panel | ✅ HP 20/20, AP 3/3, Supplies 10, Faith 0, Day 0 |
| Outpost Log panel | ✅ Starter messages visible |
| Outpost Travel panel | ✅ "Reachable locations:" header |
| Footer | ✅ Turn counter, keybinding help |
| Move action | ✅ AP consumed, turn advances |
| Wait action | ✅ AP restored |

### Bugs Found

**Bug 1 (critical): First WASD press triggers BUILD, not MOVE**

**Root cause**: `Local<i8>` defaults to `0` in Bevy. The WASD handlers check `if *pending_build_idx >= 0`. Since `0 >= 0` is `true`, the first directional keypress fires a build action instead of move.

**Effect**: User presses `w` to move north — instead builds a station (with no station type selected). The action log shows confusing messages like "You build a station. You build a Stove."

**File**: `crates/bd_tui/src/lib.rs`, `map_input_to_intents` — WASD key handlers (lines 196, 215, 233, 251)

**Bug 2: Stray enemy token 'E' on shelter map**

**Root cause**: An entity matching `(With<BlocksMovement>, Without<Player>)` exists on the shelter map. Enemies should only spawn in tactical dungeons.

**Effect**: An `E` glyph appears on the shelter map, confusing the player. It may be a test entity or incorrectly spawned enemy.

**File**: Source of the entity needs to be traced. Likely candidates: `initialize_outpost`, `spawn_outpost_player` blueprint loading, or a leftover test entity.

---

## PHASE 9: Critical Bug Fixes

### P9-A: Fix `pending_build_idx` default (10 min)

**File**: `crates/bd_tui/src/lib.rs:123`
**Current**: `let mut pending_build_idx: Local<i8>,`
**Fix**: Initialize to `-1` at start of system

```rust
fn map_input_to_intents(
    ...
    mut pending_build_idx: Local<i8>,
    ...
) {
    // Must be first — Local<i8> defaults to 0, which triggers build on first keypress
    if *pending_build_idx == 0 {
        *pending_build_idx = -1;
    }
    ...
}
```

**Test** (TDD):
```rust
#[test]
fn first_keypress_in_outpost_is_move_not_build() {
    let mut app = test_app();
    configure_outpost_mode(&mut app);
    
    // Simulate 'd' key without any prior 'b' press
    app.world_mut()
        .resource_mut::<Messages<KeyMessage>>()
        .write(KeyMessage(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)));
    
    app.update();
    
    // Verify a MOVE action was written, not a BUILD action
    let intents: Vec<_> = app.world()
        .resource::<Messages<ActionIntent>>()
        .reader()
        .read()
        .cloned()
        .collect();
    
    let move_intents: Vec<_> = intents.iter()
        .filter(|i| i.action_id == "ability.move")
        .collect();
    assert!(!move_intents.is_empty(), 
        "first key should produce move, not build. Got: {:?}", intents);
    
    let build_intents: Vec<_> = intents.iter()
        .filter(|i| i.action_id == "ability.build")
        .collect();
    assert!(build_intents.is_empty(), 
        "build intent should not fire without 'b' key. Got: {:?}", build_intents);
}
```

### P9-B: Identify and Fix Stray Enemy (15 min)

**Step 1**: Add a startup diagnostic log message to identify the entity

```rust
// In build_map_vm or a new diagnostic system:
for (pos, name) in enemies.iter() {
    tracing::info!("Enemy entity at ({},{}) name={:?}", pos.x, pos.y, name);
}
```

**Step 2**: Trace the spawn origin. Check:
- `initialize_outpost` — does it spawn entities with `BlocksMovement`?
- `spawn_outpost_player` — does the player blueprint include enemies?
- Blueprint registry — is there a blueprint that spawns with `BlocksMovement` accidentally?

**Step 3**: Fix the root cause (likely a stray spawn or incorrect blueprint flag)

**Test** (TDD):
```rust
#[test]
fn shelter_map_has_no_enemy_entities() {
    let mut app = test_app();
    app.world_mut().insert_resource(GameMode::Outpost);
    app.update();
    
    let enemies: Vec<_> = app.world()
        .query::<(), (With<BlocksMovement>, Without<Player>)>()
        .iter(app.world())
        .collect();
    assert!(enemies.is_empty(), 
        "shelter should have 0 enemies, found {}", enemies.len());
}
```

### P9-C: Regression Test — Title→Outpost Full Flow

**File**: `crates/bd_app/tests/mvp.rs`

```rust
#[test]
fn title_to_outpost_integration() {
    let mut app = test_app();
    assert_eq!(*app.world().resource::<GameMode>(), GameMode::Title);
    
    app.world_mut().insert_resource(GameMode::Outpost);
    app.update();
    
    // Player exists at correct position
    let player_pos = app.world()
        .query::<&Position, With<Player>>()
        .get_single(app.world())
        .expect("player should be spawned");
    assert_eq!(player_pos.x, 20); // SHELTER_WIDTH / 2
    assert_eq!(player_pos.y, 15); // SHELTER_HEIGHT / 2
    
    // No enemies on shelter map
    let enemy_count = app.world()
        .query::<(), (With<BlocksMovement>, Without<Player>)>()
        .iter(app.world())
        .count();
    assert_eq!(enemy_count, 0, "shelter should have no enemies");
    
    // Game log has starter messages
    let log = app.world().resource::<GameLog>();
    let msgs: Vec<_> = log.iter().map(|e| &e.message).collect();
    assert!(msgs.iter().any(|m| m.contains("survey")));
}
```

### P9-D: Manual Controls Verification

Run the app and verify:
```
w → player moves north
s → player moves south
a → player moves west (or assigns survivor if survivors present)
d → player moves east
. → wait (restores AP)
f → attack nearest enemy (or "No targets in range" feedback)
g → guard
b → cycle station types → build station when direction pressed
t → travel
i → inventory screen
z → combat screen
? → help screen
q → quit cleanly
```

### P9-E: Discovery Pass — No Regressions

After all P9 fixes:
1. Run full test suite → must pass
2. Launch app: title screen → press key → outpost loads
3. Verify: no garbled text, all panels rendered, no stray `E` glyphs
4. Test all controls: WASD, ., f, g, b, t, i, z, ?, q
5. Verify: stats update (AP decreases on move, turn counter advances)
6. Verify: footer shows correct turn/day/keybinding text
7. Verify: game log shows expected messages, no unexpected build spam
8. Verify: quit restores terminal cleanly

### P9 Exit Criteria
- [ ] `first_keypress_in_outpost_is_move_not_build` passes (Bug 1 fixed)
- [ ] `shelter_map_has_no_enemy_entities` passes (Bug 2 fixed)
- [ ] `title_to_outpost_integration` passes
- [ ] Manual test: all 14 controls work correctly
- [ ] **Discovery pass clean**: no regressions, no new bugs found

---

## PHASE 11: Station Building Integration

### Current State

- `stations.rs:101` — `register_station_actions()` defines the build action
- `PendingStationBuild` resource in `colony/stations.rs:19`
- `b` key in TUI cycles through station types
- Build action requires `ResourcePoolAbove(Supplies, cost)` and `TileWalkable`

### P11-A: Station Build Test

```rust
#[test]
fn build_station_consumes_supplies_and_spawns_entity() {
    let mut app = test_app();
    // Setup: outpost mode, player at (10,10), tile (11,10) is Floor
    // Set PendingStationBuild to Stove
    app.world_mut().insert_resource(PendingStationBuild(Some(StationType::Stove)));
    
    // Send build intent
    app.world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .write(ActionIntent {
            actor: player,
            action_id: "ability.build".into(),
            direction: Some(Direction::East),
            target: None,
        });
    
    app.update();
    
    // Verify station spawned at (11, 10)
    // Verify supplies decreased by stove cost
    // Verify station has StationType::Stove component
}
```

### P11-B: Station Map Rendering

**File**: `crates/bd_tui/src/view_models.rs` — `build_map_vm`

Add station positions to `MapViewModel`:
```rust
pub struct MapViewModel {
    // ... existing fields ...
    pub station_glyphs: Vec<(Position, char)>,
}
```

Query stations:
```rust
stations: Query<(&Position, &StationType)>,
for (pos, stype) in stations.iter() {
    let glyph = match stype {
        StationType::Stove => 'F',    // Fire/Furnace
        StationType::Altar => 'A',    // Altar
        StationType::Workshop => 'W', // Workshop
        StationType::Bed => 'B',      // Bed
        StationType::Storage => 'S',  // Storage
    };
    vm.station_glyphs.push((*pos, glyph));
}
```

### P11-C: Station Visual System

**File**: `crates/bd_tui/src/visual.rs` — Add station glyph definitions
**File**: `content/symbols/default.ron` — Add station visual entries

### P11 Exit Criteria
- [ ] `build_station_consumes_supplies_and_spawns_entity` test passes
- [ ] Station glyphs render on shelter map
- [ ] 5 station types are buildable
- [ ] Supply costs are deducted
- [ ] `b` key cycles through station types with visual feedback

---

## PHASE 12: Travel → Dungeon Loop

### Current State

- `overworld.rs` — `OverworldState`, `TravelMap`, `TravelNode`
- `spatial.rs` — `process_transitions` handles GameMode switches
- `t` key in TUI writes `TransitionIntent { target: GameMode::Travel }`
- Travel takes 3 turns (hardcoded in `process_transitions`)

### P12-A: Travel Node Selection

**File**: `crates/bd_tui/src/lib.rs:341` — `t` key handler

Currently `KeyCode::Char('t')` writes a `TransitionIntent`. But it doesn't let the player SELECT which node to travel to. Add node cycling:

```rust
KeyCode::Char('t') => {
    let travel_map = app.world().resource::<TravelMap>();
    if travel_map.nodes.is_empty() {
        game_log.push("No destinations available.", LogLevel::Warn);
    } else {
        // Cycle through travel nodes
        // Write TransitionIntent with selected node
    }
}
```

### P12-B: Travel Transition Test

```rust
#[test]
fn travel_to_dungeon_transitions_to_tactical() {
    // Setup: Outpost mode, TravelMap with one node
    // Simulate 't' key to start travel
    // Advance 3 turns
    // Verify mode = Tactical
    // Verify dungeon map generated
    // Verify enemies spawned
}
```

### P12-C: Dungeon Extraction Test

```rust
#[test]
fn exit_tile_returns_to_outpost() {
    // Setup: Tactical mode, player at exit tile
    // Move onto exit
    // Verify mode = Outpost
    // Verify transient entities (enemies) cleaned up
    // Verify persistent entities (player) survive
    // Verify supplies deducted for travel
}
```

### P12-D: Weather Effects

**File**: `crates/bd_core/src/overworld.rs:27` — `Weather` enum exists (Clear, Rain, Storm, Fog)

Add weather roll during travel:
```rust
fn roll_weather() -> Weather {
    // Simple: random, weighted toward Clear
    // Rain: +1 travel turn
    // Storm: +2 travel turns, sanity drain
    // Fog: fewer encounter rolls
}
```

### P12 Exit Criteria
- [ ] Player can select travel destination
- [ ] Travel consumes time (3+ turns based on weather)
- [ ] Dungeon generates with BSP rooms + enemies
- [ ] Player can explore dungeon and return via exit
- [ ] Weather affects travel duration
- [ ] Full round-trip: shelter → travel → dungeon → return

---

## PHASE 13: Combat Deepening

### Current State

- `combat.rs` — `CombatRng`, aimed/quick attack actions, `apply_damage_variance`
- `actions.rs` — basic attack (`ability.attack`), guard (`ability.guard`)
- Enemy AI — `ability.enemy_melee` (bypasses TargetHostile)

### P13-A: Wire d100 Variance

**File**: `crates/bd_core/src/pools.rs:102` — `resolve_pool_deltas`

Currently applies raw `modified_amount` from status modifiers. Need to also apply d100 variance:

```rust
// After status modifier application, before pool delta:
let variance_amount = CombatRng::apply_damage_variance(modified_amount, rng);
pool.current = (pool.current + variance_amount).max(pool.min).min(pool.max);
```

Need to add `CombatRng` as a system parameter in `resolve_pool_deltas`.

### P13-B: Cover/Guard System

**File**: `crates/bd_core/src/combat.rs:63` — `Cover` component exists
**File**: `crates/bd_core/src/statuses.rs` — `status.guarded` status

Verify guarded status reduces incoming damage. Test:
```rust
#[test]
fn guarded_player_takes_half_damage() {
    // Apply guarded status to player
    // Deal 10 damage
    // Verify player takes 5 damage (50% reduction)
}
```

### P13-C: Armor

`Armor` component doesn't exist yet. Add it:
```rust
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Armor {
    pub damage_reduction: i32,  // flat damage reduction
    pub durability: i32,        // remaining uses
    pub max_durability: i32,
}
```

Modify `resolve_pool_deltas` to check armor.

### P13-D: Ammo

`Ammo` component already exists in `combat.rs:69`. Wire it:
- `aimed_attack` requires ammo → consumes 1
- `reload` restores ammo to max
- `quick_attack` uses no ammo but less damage

### P13-E: Wound Threshold

When HP < 50% of max, apply `status.wounded`:
- Reduces AP recovery by 1
- Visible in stats panel as red HP text (already has color threshold)

### P13 Exit Criteria
- [ ] d100 variance affects damage amounts
- [ ] Guarded status halves physical damage
- [ ] Armor reduces damage (flat reduction)
- [ ] Ammo constrains ranged attacks
- [ ] Wounded status triggers at <50% HP

---

## PHASE 14-20: Content & Depth (Summarized)

These phases add content rather than fixing architectural issues. They are lower priority because the core loop works without them.

### Phase 14: Overworld Encounters
- Add encounter rolls during travel
- Encounter types: bandits, demons, angels, survivors, weather hazards
- Each encounter → tactical resolution or choice

### Phase 15: Gabriel Encounter
- Trigger: first dungeon return OR Altar built
- Dialogue tree in RON
- Choices affect faction rep and virtues
- Gabriel sprite on shelter map

### Phase 16: RON Content Pipeline
- `content/dialogue/` — dialogue trees
- `content/events/` — event definitions
- `content/items/` — item stats
- `content/locations/` — dungeon templates
- All validated at startup via `--validate`

### Phase 17: Faction Integration
- Reputation changes from actions
- Hostile/Allied thresholds
- Faction-specific encounters and NPCs

### Phase 18: Colony Depth
- Survivor task assignment UI
- Production cycles (stations → resources)
- Raids (random, scaling)
- Colony ideology/law

### Phase 19: Dungeon Depth
- Anomalies (sacred rifts, demonic altars)
- Contamination pool kind
- Lore fragments (scrolls, tablets)

### Phase 20: Polish
- Terminal size handling
- Keybinding consistency
- Structured playtest
- Performance profiling

---

## Immediate Next Actions (P9)

1. Write `title_to_outpost_shows_correct_state` test → red
2. Fix survivor rendering on shelter map → green
3. Write `outpost_log_shows_starter_messages` test → red
4. Verify map renders shelter correctly → green
5. Run full suite → all pass

---

## Discovery Pass Checklist (Reusable — Run After Every Phase)

```
□ cargo test --workspace → all pass
□ cargo clippy --workspace → 0 new warnings
□ Launch app: title screen renders clean
□ Press key: outpost loads without errors
□ Verify: all panels rendered (party, map, stats, log, travel)
□ Verify: no stray enemy 'E' glyphs on shelter map
□ Verify: no "Unknown widget" red blocks
□ WASD moves player, AP decreases, turn advances
□ '.' waits, AP restores
□ Stats panel: HP/AP/Supplies/Faith/Day values correct
□ Log panel: messages make sense (no spam, no stale messages)
□ Footer: turn counter, day counter, keybinding help visible
□ q → quits cleanly, terminal restored
□ No panics, no tracing errors in stderr
```

