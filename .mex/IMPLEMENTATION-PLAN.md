# Broken Divinity — Implementation Plan

**Date**: 2026-07-22
**Current State**: Phases 0-8 complete (227 tests, 0 warnings). Core architecture solid. MVP loop skeleton exists. Content depth missing.
**Target**: Playable MVP with full shelter→travel→dungeon→return loop and theological consequences.

---

## Completed Phases

| Phase | What | Tests Add |
|-------|------|-----------|
| 0-4 | UX recovery — progression clarity, modal safety, feedback quality, readability | ~150 |
| 5 | Combat fix — constants, TUI guard, validation hardening | +1 |
| 6 | Enemy AI — detection, pursuit, attack | +8 (3 enemy + 5 direction) |
| 7 | Game Over — detection, screen, quit gating | +1 |
| 8 | Save/Load — serde audit, roundtrip test, key handlers | +1 |
| Bugfix | spawn_outpost_player system set + once guard, ASCII art line-split fix | — |

---

## Phase 9: Critical Bug Fixes (1 hr)

**Goal**: Fix two blocking bugs found in live preflight (2026-07-22). First WASD press after entering Outpost fires a build action instead of move. Stray enemy entity appears on shelter map.

### Preflight Findings

Live interactive test confirmed:
- Title screen clean, outpost screen renders correctly (all 5 panels: party, map, stats, log, travel)
- Move/wait actions work
- Two bugs block basic playability

**Bug 1**: `Local<i8>` defaults to `0`. WASD handlers use `if *pending_build_idx >= 0` to gate between move/build. `0 >= 0` → first WASD press builds instead of moving.

**Bug 2**: An entity with `(With<BlocksMovement>, Without<Player>)` exists on shelter map. Source unknown — spawn audit needed.

### P9-A: Fix pending_build_idx default (10 min)

**File**: `crates/bd_tui/src/lib.rs:123`
**Fix**: Reset `*pending_build_idx = -1` at start of `map_input_to_intents`.

Test: `first_keypress_in_outpost_is_move_not_build`

### P9-B: Identify and remove stray enemy (15 min)

Audit all outpost spawn systems. Remove or fix the enemy-spawning entity.
Add guard: `initialize_outpost` must never spawn `BlocksMovement` entities.

Test: `shelter_map_has_no_enemy_entities`

### P9-C: Regression tests (20 min)
- `first_keypress_in_outpost_is_move_not_build`
- `shelter_map_has_no_enemy_entities`
- `title_to_outpost_integration` — full startup flow

### P9-D: Manual controls verification (15 min)

Test all controls: w↑, s↓, a←, d→, ., f, g, b, t, i, z, ?, q

**Exit criteria**: Both bugs fixed ✅. 3 new tests pass. All 14 controls work.

### P9-E: Discovery pass — no regressions
- Run full test suite (must pass)
- Launch app, play through title→outpost→move→wait→quit
- Check for new `E` glyphs, garbled text, missing panels
- Verify footer shows correct turn/day/keybinding text
- Check stats panel values increment/decrement with actions

**Gate**: All P9 exit criteria met AND discovery pass clean.

---

## Phase 10: Outpost Content — Survivor & Station Rendering (1.5 hrs)

**Goal**: Player can move around the shelter using WASD keys. Movement is reflected in the UI immediately.

### P10-A: Move action test — `player_moves_in_outpost`
```rust
fn player_moves_in_outpost() {
    // 1. Spawn player at (20, 15) in Outpost mode
    // 2. Send move intent (Direction::East)
    // 3. Verify player Position = (21, 15)
    // 4. Verify AP decreased by 1
}
```

### P10-B: Verify action list shows correct actions
- In Outpost mode, check `ActionListViewModel` shows: move (WASD), wait (.), attack (f), guard (g), build (b), travel (t), inventory (i), assign (a)
- Actions are disabled when AP=0 with denial reason shown

### P10-C: Verify wait restores AP
- Player at 0 AP → press '.' → AP increases by 1 → log shows "You wait."

**Exit criteria**: Player can navigate shelter. AP management works. Action list shows correct available/disabled actions.

---

## Phase 11: Station Building Integration (2 hrs)

**Goal**: Player presses 'b' → cycles through station types → presses direction → station spawns in shelter at target tile.

### P11-A: Station build test — `build_station_consumes_supplies`
```rust
fn build_station_consumes_supplies() {
    // 1. Outpost mode with sufficient supplies
    // 2. Press 'b' → cycle to Stove → press direction (East)
    // 3. Verify station entity spawned at target tile
    // 4. Verify supplies decreased
    // 5. Verify station icon renders on map
}
```

### P11-B: Station catalog completeness
- Verify all station types render: Stove (altar glyph?), Altar, Workshop, Bed, Storage
- Each station type has correct production cost and output

### P11-C: Shelter map shows placed stations
- `build_map_vm` queries station entities and includes them in enemy/entity rendering
- Stations use different glyphs than enemies (add to visual system)

**Exit criteria**: Player can build stations. Stations appear on map. Supply costs apply.

---

## Phase 12: Travel → Dungeon Loop (3 hrs)

**Goal**: Player presses 't' → selects travel node → enters Tactical mode → explores dungeon → returns to Outpost via exit.

### P12-A: Travel screen shows available destinations
- `outpost_travel` widget shows list of travel nodes
- Travel node selection works via key or cycle

### P12-B: Travel transition test — `travel_consumes_time_and_supplies`
```rust
fn travel_consumes_time_and_supplies() {
    // 1. Outpost mode, supplies >= travel cost
    // 2. Initiate travel to a dungeon node
    // 3. Verify mode = Travel
    // 4. Verify turns_remaining = 3
    // 5. Verify supplies decreased
    // 6. Verify weather affects travel duration
}
```

### P12-C: Dungeon generation — BSP rooms with enemies
- `generate_tactical_location` creates BSP rooms with spawn zones
- Enemies spawn from blueprint roster (rat, skeleton)
- Map renders correctly with walls, floors, doors

### P12-D: Dungeon extraction — exit tile
- Player walks onto exit tile → TransitionIntent → Outpost mode
- Transient entities (enemies, loot) cleaned up
- Persistent entities (player, party) survive

### P12-E: Travel weather effects
- `Weather` enum: Clear, Rain, Storm, Fog
- Rain/Storm: travel takes +1 turn
- Fog: fewer encounter rolls (Phase 2)

**Exit criteria**: Full shelter→travel→dungeon→return loop works. Weather affects travel.

---

## Phase 13: Combat Deepening (3 hrs)

**Goal**: Combat feels weighty. Cover, armor, ammo, and d100 variance matter.

### P13-A: Wire d100 damage variance
- `CombatRng::apply_damage_variance` called in pool delta resolution
- Test: same attack produces different damage amounts with different seeds

### P13-B: Cover system
- `Cover` component with damage reduction percentage
- `take_cover` action applies guarded status
- Test: player takes reduced damage when guarded

### P13-C: Armor system
- `Armor` component with durability and damage reduction
- Pool delta resolution checks armor before applying damage
- Test: armored entity takes less physical damage

### P13-D: Ammo system
- `Ammo` component for ranged weapons
- `aimed_attack` consumes ammo
- `reload` action restores ammo
- Test: aimed attack fails when out of ammo

### P13-E: Wound thresholds
- When HP drops below WOUND_THRESHOLD_PCT of max, apply wounded status
- Wounded reduces AP recovery
- Test: player at 50% HP gets wounded status

**Exit criteria**: Combat has tactical depth. Cover, armor, ammo, and wounds matter.

---

## Phase 14: Overworld Content (2 hrs)

**Goal**: Travel is interesting. Weather, encounters, and resource pressure make journeys meaningful.

### P14-A: Overworld encounter system
- Random encounter rolls during each travel turn
- Encounter types: bandit ambush, demon sighting, angel patrol, survivor camp, weather hazard
- Encounters pause travel for tactical resolution or choice

### P14-B: Travel resource pressure
- Food/water consumption during travel (from colony supplies)
- Running out of supplies causes stress/sanity drain
- Test: traveling with 0 supplies causes sanity loss

### P14-C: Overworld map visual
- `render_travel_widget` shows current position, destination, turns remaining
- Weather indicator (colored text)

**Exit criteria**: Travel feels risky. Encounters occur. Resources matter.

---

## Phase 15: Gabriel Encounter — Narrative Hook (2 hrs)

**Goal**: Player meets Gabriel in the outpost. The encounter introduces the game's theological themes and sets narrative direction.

### P15-A: Trigger condition
- After first dungeon extraction OR after building Altar → trigger Gabriel encounter
- `trigger_gabriel_encounter` fires when player enters Outpost mode after trigger condition

### P15-B: Gabriel dialogue tree
- RON content: `content/dialogue/gabriel_first.ron`
- Dialogue structure: text nodes + choices → effects (reputation, virtue, items)
- Choices affect `GabrielState` and faction reputation

### P15-C: Gabriel sprite/visual
- Gabriel appears as a character on the shelter map
- Movable, interactable (press 'g' to talk?)

### P15-D: Narrative consequences
- Player choices during Gabriel encounter affect:
  - Faction reputation (Puritans / Wanderers)
  - Virtue values (Temperance, Justice, Prudence, Fortitude)
  - Colony direction hints

**Exit criteria**: Gabriel encounter fires after first dungeon. Player makes meaningful theological choices.

---

## Phase 16: Content Pipeline — RON Data Loading (2 hrs)

**Goal**: All game content loaded from RON files, validated at startup.

### P16-A: Dialogue loader
- Load dialogue trees from `content/dialogue/`
- Validate at startup via `--validate`
- Test: dialogue tree loads and resolves choices correctly

### P16-B: Event loader
- Load event definitions from `content/events/`
- Events have trigger conditions, text nodes, choices, effects
- Test: event triggers when conditions are met

### P16-C: Item/loot loader
- Load item definitions from `content/items/`
- Weapons, armor, consumables, lore items
- Test: items load with correct stats and effects

### P16-D: Location loader
- Load dungeon templates from `content/locations/`
- Theme, enemy roster, loot table, anomaly table
- Test: location generates correct enemies and loot

**Exit criteria**: `--validate` catches all content errors. Content separation from code complete.

---

## Phase 17: Faction Integration (1.5 hrs)

**Goal**: Faction reputation actually works. Gabriel choices affect reputation.
Encounters respond to faction standing.

### P17-A: Fix faction delta routing (critical bug)
- Gabriel encounter writes `RepPuritans` pool deltas → they go to player `Pools`
- Player doesn't have `RepPuritans` pool → delta silently dropped
- Fix: `process_faction_events` intercepts faction `PoolKind`s, routes to
  `FactionReputation` resource instead of player pools
- Test: accepting Gabriel's witness lowers RepPuritans

### P17-B: Faction status helper
- `faction_status(value: i32) -> FactionStatus` (Hostile / Neutral / Friendly /
Allied)
- `REPUTATION_ALLIED_THRESHOLD = 75` constant
- Hostile: ≤ -25 | Neutral: -24..24 | Friendly: 25..74 | Allied: ≥ 75
- Test: status transitions at threshold boundaries

### P17-C: Faction-aware overworld encounters
- In `roll_encounter`, skew encounter types based on faction standings
- Hostile factions → higher chance of BanditAmbush/DemonSighting
- Allied factions → higher chance of SurvivorCamp/AngelPatrol
- Test: low RepPuritans increases bandit encounter chance

### P17-D: Faction panel in shelter
- Small widget showing faction standings with colored labels
- Hostile=Red, Neutral=Gray, Friendly=Green, Allied=Cyan
- Test: panel shows correct status after Gabriel encounter

**Exit criteria**: Gabriel encounter changes faction standing. Faction status
affects overworld encounters. Player can see faction standings in shelter.

**Bridge to P18**: Colony survivors have faction allegiance. Station buildings
(like Altar) affect faction rep. Raids come from hostile factions.

---

## Phase 18: Colony Depth (2.5 hrs)

**Goal**: Colony feels alive. Survivors produce resources. Raids threaten continuity.

### P18-A: Survivor task UI
- `a` key shows task assignment options (not just nearest survivor)
- Tasks: gather food, build, defend, research
- Each survivor has stats that affect task efficiency

### P18-B: Production system
- Stations produce resources each day/turn cycle
- Stove → food, Altar → faith, Workshop → materials
- Test: building a Stove increases food production

### P18-C: Raid system
- Random raids triggered by colony size, reputation, or time
- Raid severity scales with colony value
- Defenders (survivors on defend task) fight raiders
- Test: raid reduces supplies/kills survivors if unprepared

### P18-D: Colony law and ideology
- Player can set colony direction (pragmatic, Puritan, mixed, thaumic, demon-bargain, military)
- Direction affects station bonuses, survivor morale, faction access
- Locked behind Gabriel encounter and faction rep milestones

**Exit criteria**: Colony has meaningful resource management. Raids create tension. Survivor tasks matter.

---

## Phase 19: Dungeon Depth — Anomalies & Contamination (2 hrs)

**Goal**: Dungeons feel theologically dangerous. Sacred contamination, anomalies, and lore fragments make exploration meaningful.

### P19-A: Anomaly system
- Dungeon rooms can contain anomalies (sacred rifts, demonic altars, angelic wards)
- Anomalies affect sanity, virtue, or spawn entities
- Test: entering anomaly room drains sanity

### P19-B: Contamination system
- `Contamination` pool kind on entities
- Sacred/demonic attacks increase contamination
- High contamination causes corruption, mutation, or faction penalties
- Test: demon attack increases contamination

### P19-C: Lore fragments
- Dungeons contain lore items (scrolls, tablets, inscriptions)
- Lore fragments reveal faction truths, world history, theology
- Collected in `DialogueLog` / lore journal

**Exit criteria**: Dungeons contain anomalies and lore. Contamination has mechanical consequences.

---

## Phase 20: Polish & Playtest (2 hrs)

**Goal**: Smooth edges, fix bugs, run structured playtest.

### P20-A: Terminal size handling
- Minimum terminal size warning (80×24 recommended)
- All screens degrade gracefully on small terminals
- Test: screen renders at 60×15, 80×24, 120×40

### P20-B: Keybinding consistency
- All keybindings listed in HelpLine
- F1 debug toggle works in all modes
- Esc/q quits consistently

### P20-C: Structured playtest
- Playtest checklist: title→outpost→build→travel→dungeon→combat→return→encounter→game over
- Document bugs, UX issues, balance problems

### P20-D: Performance
- Verify 60fps target on average hardware
- Check for frame drops during dungeon generation
- Profile pool delta pipeline for entity count scaling

**Exit criteria**: Playtest passes. All screens work at minimum terminal size. 60fps stable.

---

## Implementation Order

```
Phase 9:   Title→Outpost Flow Polish    ← IMMEDIATE (user can see/play)
Phase 10:  Basic Movement Verification  ← IMMEDIATE (controls must work)
Phase 11:  Station Building             ← HIGH (core shelter loop)
Phase 12:  Travel→Dungeon Loop          ← HIGH (MVP requires full loop)
Phase 13:  Combat Deepening             ← MEDIUM (loop works, now deepen)
Phase 14:  Overworld Content            ← MEDIUM (travel needs encounters)
Phase 15:  Gabriel Encounter            ← MEDIUM (narrative hook for MVP)
Phase 16:  Content Pipeline             ← LOW (technical debt, not user-facing)
Phase 17:  Faction Integration          ← MEDIUM (foundation for colony depth)
Phase 18:  Colony Depth                 ← MEDIUM (last meaty phase before polish)
Phase 19:  Dungeon Depth                ← LOW (depth, not blocking MVP)
Phase 20:  Polish & Playtest            ← FINAL (lock it down)
```

## Test Inventory

| Test | Phase | What it verifies |
|------|-------|-----------------|
| `title_to_outpost_shows_correct_state` | P9 | Full startup → outpost integration |
| `player_moves_in_outpost` | P10 | WASD movement + AP management |
| `build_station_consumes_supplies` | P11 | Station building + cost + rendering |
| `travel_consumes_time_and_supplies` | P12 | Travel transition + resource cost |
| `combat_damage_variance_modifies_damage` | P13 | d100 variance in action pipeline |
| `travel_encounter_fires_during_journey` | P14 | Overworld encounter system |
| `gabriel_encounter_triggers_after_dungeon` | P15 | Narrative hook trigger |
| `dialogue_tree_loaded_from_ron` | P16 | Content loading + validation |
| `reputation_change_affects_encounters` | P17 | Faction system consequences |
| `raid_reduces_colony_supplies` | P18 | Colony defense mechanics |
| `anomaly_room_drains_sanity` | P19 | Dungeon depth systems |
| `screen_renders_at_minimum_terminal_size` | P20 | Polish |

## Global Delivery Rules (unchanged)

- Every phase starts with failing tests (TDD)
- After every phase: **discovery pass** — launch the app, play through the affected flow, check for regressions
- No magic numbers — name every constant
- Private by default — expose only what must be public
- Clippy warnings are errors
- Extend, do not modify (Open/Closed)
- Prefer composition over modification
- One module, one responsibility
- DRY — no duplicate logic
- Systems always use `BdSet::X` — never register without a set
- Message-based communication (MessageWriter/MessageReader)
- Graceful query failure — `player.single().ok()`, never `unwrap()`

---

---

## Phases 21–24: AP Redesign + Colony Economy

**Full detailed breakdown**: See 

### Summary

| Phase | Focus | Time | Key Deliverable |
|-------|-------|------|-----------------|
| P21 | Turn Model Fix | 1.5 hrs | Only  advances time. AP regens for ALL entities at turn start. Colony movement free. |
| P22 | Colony Economy | 2.5 hrs | Resource nodes on shelter map. Survivors gather. Stations process raw→supplies. |
| P23 | Build + Task UI | 1.5 hrs |  build menu (1-5 keys).  task menu (1-4 keys). Survivor glyphs by task. |
| P24 | Polish | 1 hr | Combat logs, game over screen, HelpLine update, full playtest. |

### Design decisions:
- **ONE move action**, mode-gated AP cost (no  duplication)
- **AP regens to MAX** at turn start via  signal
- **ShouldAdvanceTime only set by wait** (and travel), not by every player action
- **TDD**: every subtask starts with a failing test
- **All numeric values use named constants** — zero magic numbers

### New systems, components, resources, and constants detailed in the V2 breakdown.
