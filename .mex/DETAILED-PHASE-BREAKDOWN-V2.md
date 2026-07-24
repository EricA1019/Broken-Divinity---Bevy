# Broken Divinity — Detailed Phase Breakdown v2

**Date**: 2026-07-22
**Basis**: Senior engineer review of AP/Colony plan
**Tests**: 198 passing, 0 warnings
**Target**: MVP with turn-based combat + colony economy

---

## Phase 21: Turn Model Fix (1.5 hrs)

**Goal**: Only `.` (wait) advances time. All entities regain AP on turn start.
Enemies fight back every turn. Colony movement is free.

### P21-A: Decouple ShouldAdvanceTime from non-wait actions (TEST FIRST)

**Test to write FIRST** (`time.rs` test module):
```rust
#[test]
fn only_wait_advances_time() {
    let mut app = test_app();
    app.world_mut().insert_resource(SmokeMap::new(10, 10, Tile::Floor));
    let p = spawn_player(&mut app, 5, 5);
    let dummy = spawn_dummy(&mut app, 6, 5);

    // Move should NOT advance time
    send_action(&mut app, p, "ability.move", Some(Direction::East), None);
    app.update();
    let turn_after_move = app.world().resource::<GameTime>().turn;

    // Wait SHOULD advance time
    send_action(&mut app, p, "ability.wait", None, None);
    app.update();
    let turn_after_wait = app.world().resource::<GameTime>().turn;

    assert_eq!(turn_after_wait, turn_after_move + 1,
        "Only wait should advance time, not move");
}
```

**Implementation** (`actions.rs`, `resolve_action_effects`):
- Current: `if player_flag.is_some() { should_advance.0 = true; }` on EVERY player action
- Change: Only set `should_advance.0 = true` when the action_id is `"ability.wait"`
- Keep travel's `should_advance.0 = true` in `process_travel_day` (travel is a special turn)

### P21-B: Add TurnJustAdvanced signal + AP regen (TEST FIRST)

**Test to write FIRST** (`pools.rs` test module):
```rust
#[test]
fn all_entities_regain_ap_on_turn_start() {
    let mut app = test_app();
    app.world_mut().insert_resource(SmokeMap::new(10, 10, Tile::Floor));
    let player = spawn_player(&mut app, 5, 5);
    let rat = spawn_enemy_with_ap(&mut app, 6, 5);

    // Drain AP by attacking
    send_action_rat_attack(&mut app, rat, player);
    app.update();
    let rat_ap_after_attack = get_ap(&app, rat);
    assert!(rat_ap_after_attack < 2, "AP should be consumed by attack");

    // Wait to advance turn
    send_action(&mut app, player, "ability.wait", None, None);
    app.update();

    // Both should have max AP
    assert_eq!(get_ap(&app, player), 3, "Player AP should reset to max");
    assert_eq!(get_ap(&app, rat), 2, "Rat AP should reset to max");
}
```

**Implementation**:
- Add `TurnJustAdvanced` resource: `#[derive(Resource)] pub struct TurnJustAdvanced;`
- `advance_time` inserts `TurnJustAdvanced` after incrementing turn
- New `regenerate_action_points` system in `pools.rs`:
  - Watches for `TurnJustAdvanced`
  - Queries ALL entities with `Pools` containing `ActionPoints`
  - Sets `ap.current = ap.max` for each
  - Removes `TurnJustAdvanced` resource
  - Runs in `BdSet::Input` (before any actions this frame)

### P21-C: Remove AP restore from wait action

**File**: `actions.rs`, `ability.wait` definition

Change:
```rust
// Remove this effect entirely:
Effect::PoolDelta {
    kind: PoolKind::ActionPoints,
    amount: 1,
    tags: vec![DeltaTag::Recovery],
    reason: "wait".into(),
},
```

Wait now: sets ShouldAdvanceTime (via action resolution), logs "You wait.", NO AP effect. AP comes from the regen system.

### P21-D: Mode-gate movement AP cost (TEST FIRST)

**Test to write FIRST**:
```rust
#[test]
fn colony_movement_is_free() {
    // In Outpost mode, move should succeed with 0 AP
}

#[test]
fn tactical_movement_costs_ap() {
    // In Tactical mode, move costs 1 AP
}
```

**Implementation** (`actions.rs`, `validate_action_intents`):
- In the `ability.move` validation, check `GameMode`:
  - If `Outpost`: skip `HasPoolAtLeast(ActionPoints, 1)` check
- In `ability.move` cost_effects:
  - If `Outpost`: skip the AP delta
- ONE action definition, mode-gated. No `ability.move_colony` duplication.

### P21-E: Update HelpLine for new keys

- Add turn indicator: "Turn: N" in footer
- Update help text: "Wait:. (end turn)" instead of just "Wait:."

**Files**: `time.rs`, `pools.rs`, `actions.rs`, `lib.rs`
**New resources**: `TurnJustAdvanced`
**Constants**: None new (reuse existing `TURNS_PER_DAY`)

---

## Phase 22: Colony Economy — Resource Nodes + Gathering (2.5 hrs)

**Goal**: Shelter has resource nodes. Survivors gather from them. Stations
process raw resources into supplies. Visible production cycle.

### P22-A: Resource node spawning on shelter map (TEST FIRST)

**Test to write FIRST**:
```rust
#[test]
fn resource_nodes_spawn_on_shelter_map() {
    let map = create_shelter_map();
    let nodes = spawn_resource_nodes(&map);
    assert!(nodes.len() >= RESOURCE_NODE_COUNT_MIN);
    assert!(nodes.len() <= RESOURCE_NODE_COUNT_MAX);
    // All nodes must be on walkable tiles
    for node in &nodes {
        assert!(map.is_walkable(node.pos.x, node.pos.y));
    }
}
```

**Constants** (`colony/resources.rs` or inline):
```rust
pub const RESOURCE_NODE_COUNT_MIN: u32 = 4;
pub const RESOURCE_NODE_COUNT_MAX: u32 = 6;
```

**Components** (`components.rs`):
```rust
#[derive(Component, Debug, Clone)]
pub struct ResourceNode {
    pub kind: ResourceNodeType,
    pub depleted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceNodeType {
    Trees,       // → Materials
    WaterSource, // → Water (future)
    WildPlants,  // → Food
}
```

**Spawning** (`colony/shelter.rs`, `create_shelter_map`):
- After creating the base map, pick 4-6 random walkable positions
- Spawn `(ResourceNode { kind, depleted: false }, Position, Name)` entities
- Avoid spawning on top of survivors or the player spawn point

### P22-B: Add Materials and WildPlants to PoolKind + ColonyResources

**File**: `signals.rs` — add to `PoolKind` enum:
```rust
Materials,
WildPlants,
```

**File**: `colony/production.rs` — add to `ColonyResources::default()`:
```rust
Pool::new(PoolKind::Materials, 0, 0, 50),
Pool::new(PoolKind::WildPlants, 0, 0, 50),
```

### P22-C: Survivor gathering system (TEST FIRST)

**Test to write FIRST**:
```rust
#[test]
fn gathering_survivor_near_tree_produces_materials() {
    // Spawn survivor with Gathering task next to a Trees node
    // Advance day
    // Verify Materials increased
}

#[test]
fn idle_survivor_produces_nothing() {
    // Survivor with Idle task near node → no production
}

#[test]
fn survivor_too_far_from_node_produces_nothing() {
    // Survivor beyond GATHERING_RANGE → no production
}
```

**Constants**:
```rust
pub const GATHERING_RANGE: i32 = 3;        // Manhattan distance
pub const GATHERING_YIELD_PER_DAY: i32 = 1; // Base yield per survivor
```

**System** (`colony/production.rs` or new `colony/gathering.rs`):
```
process_survivor_gathering:
- Runs on day change (inside process_production or as separate system)
- Query: survivors with SurvivorTask::Gathering + Position
- Query: ResourceNode entities with Position
- For each gathering survivor:
  - Find nearest non-depleted resource node within GATHERING_RANGE
  - If found: add GATHERING_YIELD_PER_DAY of that node's resource to ColonyResources
  - Log: "Survivor N gathered 1 Materials from Trees"
```

### P22-D: Resource node rendering

**File**: `view_models.rs` — add to `MapViewModel`:
```rust
pub resource_glyphs: Vec<(Position, char)>,
```

**File**: `view_models.rs` — `build_map_vm`:
- Add `resource_nodes: Query<(&Position, &ResourceNode)>` parameter
- Populate: Trees→'T', Water→'W', WildPlants→'P'

**File**: `screens.rs` — `render_map_widget`:
- After station glyphs, render resource nodes with `VisualToken::Item` and green/blue/cyan colors

### P22-E: Station processing rework

**Current**: Stations produce from thin air (Stove→+3 Supplies)
**New**: Stations consume raw resources, produce processed goods

**Constants**:
```rust
pub const STOVE_WILDPLANTS_COST: i32 = 1;
pub const STOVE_SUPPLIES_OUTPUT: i32 = 2;
pub const WORKSHOP_MATERIALS_COST: i32 = 1;
pub const WORKSHOP_SUPPLIES_OUTPUT: i32 = 2;
```

**Modify** (`colony/production.rs`, `process_production`):
- Stove: if WildPlants ≥ 1, consume 1 WildPlants → produce 2 Supplies
- Workshop: if Materials ≥ 1, consume 1 Materials → produce 2 Supplies
- Altar: unchanged (faith generation is separate)
- Bed: unchanged
- Storage: increases pool caps

### P22-F: Daily production log summary

At day change, log one summary line per event:
- "Survivor 1 gathered 1 Materials from Trees"
- "Stove consumed 1 WildPlants, produced 2 Supplies"
- "Survivors consumed 3 food"

**Files**: `colony/production.rs`, `colony/gathering.rs`
**New modules**: `colony/gathering.rs`
**New components**: `ResourceNode`, `ResourceNodeType`
**New PoolKinds**: `Materials`, `WildPlants`

---

## Phase 23: Build Menu + Task Assignment UI (1.5 hrs)

**Goal**: Player can choose what to build and assign survivors with clear UI.

### P23-A: Build mode state machine (TEST FIRST)

**Test to write FIRST**:
```rust
#[test]
fn build_menu_shows_available_stations() {
    // Enter build mode, verify 5 options displayed
}

#[test]
fn build_menu_cancel_returns_to_normal_mode() {
    // ESC in build mode returns to normal
}
```

**Resource** (reuse `PendingStationBuild`):
```rust
#[derive(Resource, Debug, Clone)]
pub struct BuildModeState {
    pub active: bool,
    pub selected_station: Option<StationType>,
}
```

**Key flow** (`bd_tui/src/lib.rs`):
1. `b` → set `BuildModeState { active: true, selected_station: None }`
2. In build mode:
   - `1`→Stove, `2`→Altar, `3`→Workshop, `4`→Bed, `5`→Storage (set selected_station)
   - `ESC` or `b` again → cancel (set active=false)
   - Direction key (if station selected) → write `ActionIntent { action_id: "ability.build", direction: Some(dir), target: None }` + exit build mode
3. Show "BUILD: 1=Stove 2=Altar 3=Workshop 4=Bed 5=Storage ESC=cancel" in HelpLine

### P23-B: Build action carries station type

**File**: `actions.rs`, `Effect::SpawnEntity` handler:
- Already reads `PendingStationBuild` resource for station type
- Set `PendingStationBuild` before writing build ActionIntent
- After build, station glyph appears on map

### P23-C: Task assignment menu (TEST FIRST)

**Test to write FIRST**:
```rust
#[test]
fn task_menu_assigns_gathering() {
    // Open task menu on nearest survivor
    // Select Gathering (2)
    // Verify survivor task is Gathering
}
```

**Resource**:
```rust
#[derive(Resource, Debug, Clone, Default)]
pub struct TaskMenuState {
    pub active: bool,
    pub target_survivor: Option<Entity>,
}
```

**Key flow** (`bd_tui/src/lib.rs`):
1. `a` → find nearest survivor, set `TaskMenuState { active: true, target: Some(entity) }`
2. In task menu:
   - `1`→Idle, `2`→Gathering, `3`→Defending, `4`→Resting
   - Write `ActionIntent { action_id: "ability.assign_task", target: Some(survivor) }` with appropriate task
   - Exit menu
3. Show "TASK: 1=Idle 2=Gather 3=Defend 4=Rest ESC=cancel" in HelpLine

### P23-D: Survivor glyph by task

**File**: `view_models.rs` — `build_map_vm` survivor glyph mapping:
```rust
let glyph = match task {
    SurvivorTask::Idle => 'A',
    SurvivorTask::Gathering => 'G',
    SurvivorTask::Defending => 'D',
    SurvivorTask::Resting => 'R',
    SurvivorTask::AssignedTo(_) => 'A',
};
```

Need to add SurvivorTask to the survivor query in `build_map_vm`.

**Files**: `bd_tui/src/lib.rs`, `bd_tui/src/screens.rs`, `bd_tui/src/view_models.rs`
**New resources**: `BuildModeState`, `TaskMenuState`

---

## Phase 24: Polish — Combat Logs, Game Over, Playtest (1 hr)

**Goal**: Combat has visible feedback. Death has consequences. MVP loop verified.

### P24-A: Combat damage log messages (TEST FIRST)

**Current**: `log_combat_damage` exists but may not show damage amounts clearly.
**Change**: In `resolve_pool_deltas`, when Health delta is negative, push:
```
"Rat bites you for 7 damage! (HP: 13/20)"
```

Include entity name from Name component. Show HP before and after.

### P24-B: Resource change log messages

At day change, push ONE summary block:
```
--- Day 1 ---
Survivor 1 gathered 1 Materials
Stove produced 2 Supplies
Survivors consumed 3 food
Net: -1 Supplies, +1 Materials
```

### P24-C: Game over screen (TEST FIRST)

**Test**:
```rust
#[test]
fn player_death_shows_game_over_screen() {
    // Reduce player HP to 0 via damage
    // Verify GameMode transitions to GameOver
    // Verify game over screen renders
}
```

**Implementation**:
- Already have `GameMode::GameOver` variant
- Already have `game_over_splash` screen defined
- Already have `observe_player_defeat` observer that sets GameOver mode
- Add: "Press R to restart, Q to quit" on game over screen
- Add: stats summary (turns survived, supplies gathered, enemies killed, faction standings)

### P24-D: HelpLine update

Add all new keybindings to the footer:
- Outpost: "b:build a:assign t:travel i:inventory .:end day q:quit"
- Combat: "WASD:move f:attack g:guard .:end turn r:return q:quit"
- Build mode: "1-5:select dir:place ESC:cancel"
- Task mode: "1-4:select ESC:cancel"

### P24-E: Full MVP playtest checklist

Run through and verify every step:
- [ ] Title screen renders at 80×24
- [ ] Enter outpost — all 5 panels visible
- [ ] Resource nodes visible on shelter map (T/W/P)
- [ ] Build menu opens with `b`, all 5 stations selectable
- [ ] Station appears on map after building
- [ ] Task menu opens with `a`, all 4 tasks selectable
- [ ] Survivor glyph changes to G/D/R based on task
- [ ] Wait (.) advances to next day
- [ ] Daily production log shows gathering + processing
- [ ] Travel to dungeon via `t`
- [ ] Gabriel encounter fires on first dungeon entry
- [ ] Combat: enemy attacks and damages player
- [ ] Combat: player can attack with `f`
- [ ] Combat: guard (`g`) reduces damage
- [ ] AP regenerates on wait (both player and enemies)
- [ ] Colony movement is free (no AP drain)
- [ ] Tactical movement costs AP
- [ ] Exit dungeon via `+` tile or `r` key
- [ ] Return to shelter — Gabriel G visible
- [ ] Faction panel shows standings
- [ ] Game over when player HP ≤ 0
- [ ] Restart from game over screen
- [ ] Quit with `q` — clean exit, no crash

**Files**: `combat.rs`, `pools.rs`, `colony/production.rs`, `screens.rs`, `lib.rs`
**No new modules** — polish pass only

---

## Summary: Files Changed Per Phase

| Phase | New Files | Modified Files |
|-------|-----------|---------------|
| P21 | `TurnJustAdvanced` resource | `time.rs`, `pools.rs`, `actions.rs`, `lib.rs` |
| P22 | `colony/gathering.rs` | `signals.rs`, `colony/production.rs`, `colony/shelter.rs`, `components.rs`, `view_models.rs`, `screens.rs`, `lib.rs` |
| P23 | — | `bd_tui/src/lib.rs`, `view_models.rs`, `actions.rs`, `colony/stations.rs` |
| P24 | — | `pools.rs`, `screens.rs`, `lib.rs` (HelpLine update) |

## New Systems

| System | Set | Phase | Purpose |
|--------|-----|-------|---------|
| `regenerate_action_points` | `BdSet::Input` | P21 | Restore AP→max for all entities at turn start |
| `process_survivor_gathering` | `BdSet::Mutation` | P22 | Survivors near nodes produce resources at day change |

## New Components

| Component | Phase | Purpose |
|-----------|-------|---------|
| `ResourceNode` | P22 | Marks a resource node entity on the shelter map |
| `ResourceNodeType` | P22 | Enum: Trees, WaterSource, WildPlants |

## New Resources

| Resource | Phase | Purpose |
|----------|-------|---------|
| `TurnJustAdvanced` | P21 | Signal that a turn has passed (consumed by AP regen) |
| `BuildModeState` | P23 | Track build mode UI state (active + selected station) |
| `TaskMenuState` | P23 | Track task assignment UI state |

## New PoolKinds

| PoolKind | Phase | Purpose |
|----------|-------|---------|
| `Materials` | P22 | Raw building materials (from Trees) |
| `WildPlants` | P22 | Raw food (from WildPlants, processed by Stove) |

## New Named Constants

| Constant | Value | Module | Phase |
|----------|-------|--------|-------|
| `RESOURCE_NODE_COUNT_MIN` | 4 | `colony/shelter.rs` | P22 |
| `RESOURCE_NODE_COUNT_MAX` | 6 | `colony/shelter.rs` | P22 |
| `GATHERING_RANGE` | 3 | `colony/gathering.rs` | P22 |
| `GATHERING_YIELD_PER_DAY` | 1 | `colony/gathering.rs` | P22 |
| `STOVE_WILDPLANTS_COST` | 1 | `colony/production.rs` | P22 |
| `STOVE_SUPPLIES_OUTPUT` | 2 | `colony/production.rs` | P22 |
| `WORKSHOP_MATERIALS_COST` | 1 | `colony/production.rs` | P22 |
| `WORKSHOP_SUPPLIES_OUTPUT` | 2 | `colony/production.rs` | P22 |
