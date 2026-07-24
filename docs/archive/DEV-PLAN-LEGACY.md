# MVP Dev Plan — Vertical Slices

**Status:** Historical technical plan, archived during Foundation Recovery Phase 0 on 2026-07-24.

**Active replacement:** [Foundation Recovery Plan](../../../docs/FOUNDATION-RECOVERY-PLAN.md)

Completion and scope statements below are historical and do not authorize implementation.

## Build Requirements

### Prerequisites
- Rust toolchain (stable, edition 2024). Verify with: `rustc --version`
- All dependencies are fetched by Cargo automatically.

### Required Environment
The Bevy dependency graph can trigger a rustc stack overflow on some Linux configurations.
Set the following environment variable before any build command:

```bash
export RUST_MIN_STACK=16777216
```

The canonical build-and-test entry point is:

```bash
bash scripts/test-gate.sh
```

This runs: clean build → all tests → clippy → release build.
The script sets `RUST_MIN_STACK` automatically.

### Fresh-Shell Reproduction
To verify the build from a clean environment:

```bash
# Open a new terminal (no inherited env)
cd /path/to/Broken\ Divinity\ --Bevy
bash scripts/test-gate.sh
```

Expected result: all 4 steps pass, exit code 0.

### Common Failures

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| `rustc` SIGSEGV during compilation | Stack too small for Bevy dependency graph | Set `RUST_MIN_STACK=16777216` (script does this automatically) |
| `cargo: command not found` | Rust toolchain not on PATH | Install via rustup.rs or verify PATH |
| Test failures after refactor | Behavior drift | Run `cargo test` and check which test failed |

## Goal
When this plan is complete, a player can start a new game from a seed, manage a shelter colony, travel the overworld, descend into themed dungeons, fight enemies with the d100 system, extract loot, defend against raids, encounter factions, experience the Gabriel narrative hook, advance skills, unlock perks, and save/load — the full MVP loop described in `phase-roadmap.md`.

## Scope

**In**: Every MVP system listed in `docs/gameplay/phase-roadmap.md` under "What Ships".

**Out**:
- Phase 2/3 features (dual sanity, faction reputation, stealth facing/FOV, expeditions, survivor personalities, crafting, caliber ammo, dynamic overworld)
- Multiplayer
- Audio beyond placeholder (Bevy built-in audio exists but no soundtrack/SFX authoring)
- Art pipeline (placeholder ASCII glyphs + colored rectangles/sprites are fine)
- Modding support
- CI/CD, packaging, distribution

**Deferred**:
- Balance tuning (numbers are initial guesses; tune after playtesting)
- Full perk trees for all 8 skills (MVP ships 3: Melee, Ranged, Toughness)
- Accessory slot content (slot exists in data, 2-3 items max)

## Prerequisites
- Cargo.toml with all dependencies — **exists ✓**
- AppState enum — **exists ✓** (`src/core/state.rs`)
- Module scaffolding — **exists ✓** (`src/core/`, `src/game/`)
- docs/gameplay/* locked down — **exists ✓** (systemic fixes applied)
- docs/tech/ui-design.md locked down — **exists ✓**

## Current State
The codebase implements most MVP systems. 98 tests pass; 27 clippy warnings remain. Key systems in place:
- Full BSP dungeon generation (multi-floor, 3 themes, FOV, doors, stairs, anomalies)
- Complete d100 combat: melee, ranged, cover, reload, armor durability, status effects (Wounded, Stunned)
- Skill XP advancement with diminishing returns; perk unlocks (Melee, Ranged, Toughness trees)
- Sanity system with threshold effects (hallucinations, control loss, perception penalties)
- Walkable shelter colony with stations, 3 hardcoded survivors, auto-resolve raids
- Overworld node graph with Delaunay roads, weather system, tile-walking travel
- Gabriel scripted encounter + ghost companion AI
- Faction system: 3 hardcoded (Michael's Host, Fort Pershing, The Collective) + 2 proc-gen
- Save/load in JSON format (single-slot, permadeath, autosave on shelter return)
- Full egui UI: menu, colony panel, overworld panel, inventory, gamelog, journal, perk choice, game over, Gabriel dialogue
- BRP (Bevy Remote Protocol) available under `--features dev`

---

## Slice 1 — Walk the Dungeon

**Demo**: Player spawns in a BSP-generated dungeon floor, walks around with WASD/arrow keys, discovers rooms through FOV, opens doors, sees themed tiles (Urban Decay).

### Tasks
- [x] 1.1 **Tilemap foundation** — Add `bevy_ecs_tilemap` plugin to `main.rs`. Create `src/core/tilemap.rs` with helper functions for spawning a `TilemapBundle` from a 2D grid of `TileKind` (Wall, Floor, Door, StairsUp, StairsDown). Visual tile entities are strictly ephemeral and never serialized.
- [x] 1.2 **BSP dungeon generator** — Create `src/game/dungeon/gen.rs`. Implement BSP split → room placement → L-corridor connection → door placement → stair placement. Input: `(width, height, seed)` → Output: `DungeonFloor { tiles: Vec<Vec<TileKind>>, rooms: Vec<Rect>, spawn_point: IVec2 }`. Pure function, no ECS. `DungeonFloor` acts as the serialized source of truth.
- [x] 1.3 **Dungeon theme overlay** — Create `src/game/dungeon/theme.rs`. Define `DungeonTheme::UrbanDecay` enum variant. Map `TileKind` → tile atlas index per theme. Apply theme to `DungeonFloor` tiles.
- [x] 1.4 **Player entity** — Create `src/core/player.rs`. Spawn `Player` marker component + `Position(IVec2)` + sprite bundle at the dungeon's `spawn_point`. Note: All core game state components from Slice 1 onward must derive `Serialize, Deserialize`.
- [x] 1.5 **Camera setup** — Create `src/core/camera.rs`. Spawn `Camera2d`. System to follow the player entity position each frame.
- [x] 1.6 **Grid movement** — Create `src/core/movement.rs`. System reading keyboard input (WASD / arrows), moving `Position` by 1 tile if target tile is walkable. Runs only in `AppState::Dungeon`.
- [x] 1.7 **Doors** — Doors start closed (block LOS + movement). Bump-into-door opens it (changes tile to Floor, updates tilemap).
- [x] 1.8 **FOV / Visibility** — Create `src/core/fov.rs`. Implement symmetric shadowcasting (~150 LOC). `Viewshed { range: u32, visible_tiles: HashSet<IVec2> }` component on the player. System recalculates on player move. Tiles outside viewshed are hidden; previously-seen tiles are dimmed ("remembered").
- [x] 1.9 **Dungeon spawn system** — Create `src/game/dungeon/spawn.rs`. On entering `AppState::Dungeon`: generate floor, spawn tilemap, spawn player at entry, compute initial FOV.
- [x] 1.10 **State transition** — Game starts in `AppState::Menu`. Pressing Enter → transitions to `AppState::Dungeon` (temporary for this slice; proper menu comes in Slice 9).
- [x] 1.11 **Register all new systems/plugins** in `main.rs` and module `mod.rs` files.
- [x] 1.12 **Tests** — Unit test BSP generator (room count, connectivity, no overlaps). Unit test FOV (known grid → expected visible set).

### Done When
- `cargo run` shows a tile-rendered dungeon with rooms and corridors
- Player walks with keyboard, can't walk through walls
- Doors open on bump
- Unseen tiles are hidden, seen tiles persist as dimmed
- BSP generator tests pass

---

## Slice 2 — Fight

**Demo**: Enemies spawn in dungeon rooms. Player bumps to attack. Turn-based, speed-gated action budget. Enemies take turns. Things die. Player death = permadeath (back to menu).

### Tasks
- [x] 2.1 **Turn system** — Create `src/core/turn.rs`. Resources: `TurnState { phase: TurnPhase }` where `TurnPhase` = `PlayerInput | EnemyTurn | WorldTick`. `GameTime` resource (universal clock for turns, shelter ticks, and overworld travel). Speed-based action budget: each entity gets `speed` AP per round. System advances phases when all entities exhaust AP.
- [x] 2.2 **Combat stats component** — Create `src/core/stats.rs`. `CombatStats { hp, hp_max, speed, skills: HashMap<SkillId, SkillState> }`. `SkillId` enum (Melee, Ranged, Evasion, Toughness, Stealth, Awareness, Repair, Leadership). `SkillState { base: u32, xp: u32, level: u32 }`.
- [x] 2.3 **d100 skill check** — Create `src/core/combat.rs`. Pure function `roll_check(skill_level, modifiers, target_dv, rng) → CheckResult { success, critical, fumble, roll }`. Critical = `roll <= skill / 5`. Fumble = `roll == 100`.
- [x] 2.4 **Damage formula** — In `combat.rs`: pure function `calc_damage(weapon_base, skill_level, target_ar, is_crit, rng) → u32`. Includes ±20% variance, 2.0x crit mult, min 1.
- [x] 2.5 **Melee Attack ability** — `Attack` action: bump adjacent enemy → `roll_check` with Melee skill → on hit: `calc_damage` → apply to target HP. On kill: despawn enemy entity.
- [x] 2.6 **Enemy data** — Create `src/game/dungeon/enemies.rs`. `Enemy` marker component. Enemy spawn table per theme (Urban Decay: 2-3 enemy types with name, glyph, HP, speed, melee skill, damage). Spawn enemies into rooms during dungeon generation based on room type weights.
- [x] 2.7 **Enemy AI (basic)** — Create `src/game/dungeon/ai.rs`. During `TurnPhase::EnemyTurn`: each enemy with remaining AP finds shortest path to player (A* from `pathfinding` crate). If adjacent → attack. Else → move 1 tile toward player. Decrement AP.
- [x] 2.8 **Status effects** — Create `src/core/status.rs`. `StatusEffect { kind: StatusKind, remaining_time: u32 }`. `StatusKind::Wounded` (DoT: 3 + hp_max/10 per time unit, stacks to 3). `StatusKind::Stunned` (skip next action). Apply when `GameTime` advances.
- [x] 2.9 **Game log** — Create `src/core/gamelog.rs`. `GameLog` resource (ring buffer of `LogEntry { text, color, turn }`). Systems push combat messages. Minimal egui panel at bottom of screen showing last N messages. [depends: egui setup]
- [x] 2.10 **egui bootstrap** — Add `EguiPlugin` to `main.rs`. Create `src/ui/mod.rs`. Create `src/ui/gamelog_panel.rs` drawing the log in `EguiPrimaryContextPass`.
- [x] 2.11 **Player death → GameOver** — When player HP ≤ 0, transition to `AppState::GameOver`. Show "You died" egui panel with "Return to Menu" button.
- [x] 2.12 **Register all new systems** in `main.rs`. Ensure turn system ordering: PlayerInput → process player actions → EnemyTurn → process enemy actions → WorldTick → advance turn counter.
- [x] 2.13 **Tests** — Unit test d100 check edge cases (crit threshold, fumble). Unit test damage formula (min 1, crit multiplier). Integration test: spawn player + enemy, run combat to death.

### Done When
- Enemies appear in dungeon rooms
- Player bumps to attack, sees hit/miss/crit messages in game log
- Enemies chase player and attack on their turn
- Wounded/Stunned effects trigger and resolve
- Player death sends to GameOver screen
- All combat math tests pass

---

## Slice 3 — Gear Up

**Demo**: Loot spawns in dungeon rooms. Player picks up items into a 20-slot inventory. Equips weapons and armor. Shoots enemies from range. Uses cover. Heals. Sprints.

### Tasks
- [x] 3.1 **Item data model** — Create `src/core/items.rs`. `ItemKind` enum (Weapon, Armor, Consumable, Resource). `ItemDef { id, name, kind, stack_max, weight: (), properties }`. `WeaponProps { damage, damage_type, range, accuracy_mod, ammo_cost, clip_size, status_chance }`. `ArmorProps { ar, durability_max }`. Define MVP weapon/armor tiers (T0-T3) as static data.
- [x] 3.2 **Inventory system** — Create `src/core/inventory.rs`. `Inventory { slots: [Option<ItemStack>; 20] }` component on player. `ItemStack { item_id, quantity }`. Functions: `try_add(item, qty) → Result`, `remove(slot, qty)`, `is_full() → bool`.
- [x] 3.3 **Equipment slots** — In `inventory.rs`: `Equipment { weapon: Option<ItemId>, armor: Option<ItemId>, accessory: Option<ItemId> }` component. Equip/unequip moves items between inventory and equipment.
- [x] 3.4 **Loot spawning** — In `src/game/dungeon/spawn.rs`: loot rooms spawn item entities on the ground with `Position` + `ItemDrop { item_id, qty }`. Visual: glyph on tile.
- [x] 3.5 **Pickup action** — Walk onto item tile → pick up into inventory (if space). If full → game log message "Inventory full". Cost: 0 AP (free action on move).
- [x] 3.6 **Shoot ability** — Ranged attack action: select target in LOS → check Ranged skill → `calc_damage` using equipped weapon's damage → consume 1 round from the equipped weapon's clip. If the clip is empty, the action fails with an "Out of ammo" message. LOS check using Bresenham or similar. Range penalty: −2 per tile beyond optimal. Shooting generates a `Noise::Loud` event with minimal MVP alerting impact.
- [x] 3.7 **Reload ability** — 1 AP transfers ammo from inventory into the equipped weapon's clip up to `clip_size`. If no compatible ammo is available, log a failure message.
- [x] 3.8 **Cover system** — In `combat.rs`: `calc_cover(attacker_pos, target_pos, map) → CoverLevel { None, Half, Full }`. Check tiles adjacent to target on attacker's facing side. Half = −20, Full = −40 modifier. Apply in `roll_check`. UI: show cover indicator on targeted enemy.
- [x] 3.9 **Armor durability** — On player taking damage: `armor.durability -= damage_dealt` (damage after AR reduction). If durability ≤ 0: `armor.broken = true`, AR drops to 0. Game log: "Your armor breaks!"
- [ ] 3.10 **First Aid ability** — 1 AP + 1 Medicine from inventory → heal HP scaled to player's Toughness skill. Game log: "You patch yourself up for X HP."
- [ ] 3.11 **Sprint ability** — 1 AP → move 2 tiles. 3-turn cooldown tracked in `SprintCooldown(u32)` component, decremented each WorldTurn.
- [x] 3.12 **Inventory UI & Modal State** — Create `src/ui/inventory_panel.rs`. Toggle with `I` key via `AppState::Modal` (or dedicated input-block resource) to prevent WASD movement/actions while UI is open. egui grid showing 20 slots, item names, quantities. Click to equip/unequip. Show equipped weapon/armor stats.
- [x] 3.13 **HUD overlay** — Create `src/ui/hud.rs`. Top-left: HP bar, AR status (intact/broken). Bottom-center: AP pips. Show current clip and reserve ammo if a ranged weapon is equipped.
- [x] 3.14 **Damage types** — Add `DamageType` enum (Ballistic, Slash, Blunt, Celestial, Infernal, Thaumic). Weapons declare their type. AR resists Physical types; MD resists Supernatural types (MD stat on enemies).
- [x] 3.15 **Tests** — Unit test inventory add/remove/full. Unit test cover calculation. Unit test armor durability break. Integration test: shoot enemy, verify clip consumption and reload behavior.

### Done When
- Pick up loot from dungeon floor into 20-slot inventory
- Equip weapon/armor from inventory UI
- Shoot enemies at range, clip ammo consumed, Reload restores the clip
- Cover reduces to-hit (visible in UI before committing)
- Armor breaks after absorbing enough damage
- First Aid heals, Sprint moves 2 tiles with cooldown
- HUD shows HP, AP, ammo, armor status

---

## Slice 4 — Full Dungeon Crawl

**Demo**: Multi-floor dungeon with all 3 themes. Room types with varied content. Sanity drains from combat/anomalies. Skills advance through use. Extract from dungeon back to overworld stub.

### Tasks
- [x] 4.1 **Multi-floor dungeons** — Extend `dungeon/gen.rs`: generate 3-5 floors. StairsDown on each floor (except last). StairsUp on each floor (except first). Stepping on stairs → generate/load next floor, spawn player at corresponding stair tile.
- [x] 4.2 **Remaining themes** — Add `DungeonTheme::Underground` and `DungeonTheme::Military` to `theme.rs`. Each with distinct tile atlas indices and hazard types.
- [x] 4.3 **Room types** — In `spawn.rs`: assign each room a `RoomType` (Empty 30%, Loot 20%, Enemy 25%, Hazard 10%, Objective 5%, Mixed 10%). Room type determines spawn tables.
- [x] 4.4 **Enemy variety** — Add 2-3 enemy types per theme (9 total). Vary stats: some fast melee, some slow ranged, some tough. Military theme: armed enemies with Shoot ability.
- [x] 4.5 **Ranged enemies** — Enemies with `Shoot` ability: if player in LOS and range → shoot instead of closing to melee. Consumes notional ammo (enemies don't use player ammo pool).
- [x] 4.6 **Hazard tiles** — Per theme: Unstable Floor (UrbanDecay, collapse trap), Water (Underground, slows movement), Security (Military, inactive turret placeholder). Walk onto hazard → effect.
- [x] 4.7 **Anomalies** — Create `src/game/dungeon/anomalies.rs`. 3 anomaly types (visual distortion, whispers, spatial rift). Spawn 1-2 per floor. Walking near → sanity drain + game log message.
- [x] 4.8 **Sanity system** — Create `src/core/sanity.rs`. `RaidExposure { current: u32, max: 100 }` resource. Increases from: combat hits taken (+2), anomaly proximity (+5), killing (+1). Resets to 0 on shelter return. Thresholds: 50 → perception penalty (−5 skill checks), 75 → hallucination spawns (fake enemy entities that disappear on hit), 90 → intermittent control loss (random move direction 20% chance).
- [x] 4.9 **Sanity HUD** — Add sanity bar to HUD. Color shifts: green → yellow → red as exposure rises.
- [x] 4.10 **Skill advancement** — In `stats.rs`: after each skill check, grant XP (+3 success, +1 fail, +5 crit). `level_up_threshold(level) = 50 * level * level`. Each level: +2 to effective skill. Cap: level 10. Diminishing returns: if `enemy_danger_rating < player_skill / 2` → XP halved. If `< player_skill / 4` → 0 XP.
- [x] 4.11 **Dungeon extraction** — Reaching StairsUp on floor 1 with a "Leave dungeon?" confirmation dialog → transition to `AppState::Overworld` (stub: just transition to `AppState::Colony` for now, carrying inventory).
- [x] 4.12 **Loot scaling** — Deeper floors spawn higher-tier loot. Floor 1-2: T0-T1. Floor 3-4: T1-T2. Floor 5: T2-T3.
- [x] 4.13 **Tests** — Test multi-floor generation (stairs connect). Test sanity threshold effects. Test XP diminishing returns. Test loot tier distribution.

### Done When
- Descend 3-5 floors, each with themed tiles and enemies
- Room types vary: some empty, some dangerous, some rewarding
- Sanity meter fills during combat, anomaly encounters cause spikes
- Hallucinations spawn at high sanity, perception penalized
- Skills visibly level up through use (game log: "Melee increased to level 2!")
- Can extract from floor 1 stairs with loot intact
- Deeper floors have harder enemies and better loot

---

## Slice 5 — The Shelter

**Demo**: Player returns from dungeon to a walkable shelter. Assign survivors to stations. Resources produced and consumed per tick. Build new rooms and stations. Repair armor at Workbench.

### Tasks
- [x] 5.1 **Shelter generator** — Create `src/game/colony/gen.rs`. BSP variant: 40×30 walled compound with 3 starting rooms (Entrance, Quarters, Workshop). Place floor/wall tiles. Reuse `bevy_ecs_tilemap` rendering.
- [x] 5.2 **Colony state** — `AppState::Colony` activates shelter systems. Player entity reuses movement system (same grid movement as dungeon). Switch shelter to real-time loop (tick every N frames, configurable speed).
- [x] 5.3 **Survivor entities** — Create `src/game/colony/survivors.rs`. `Survivor` component + `SurvivorNeeds { hunger, thirst, rest }` (0-100, decay based on `GameTime`). `SurvivorTask` enum (Working(IVec2), Construction, Resting, Patrolling, Idle). Spawn 3 survivors at game start. Relation mapping avoids raw `Entity` IDs for serialization safety.
- [x] 5.4 **Station entities** — Create `src/game/colony/stations.rs`. `Station { kind: StationType, tier: 1, worker_slots: u8 }` (track active workers via survivor positions/tasks, not `Vec<Entity>`). 10 `StationType` variants matching colony.md. Place on valid floor tiles (2×2 footprint).
- [ ] 5.5 **Resource system** — Create `src/core/resources.rs`. `ShelterResources { food, water, scrap, medicine, ammo }` resource. Station production per tick when staffed: Cook → +1 Food (−1 Raw Meat), Purifier → +1 Water (−1 Dirty Water), AmmoPress → +1 Ammo (−1 Scrap). Survivors consume: 1 Food + 1 Water per day-equivalent ticks.
- [ ] 5.6 **Survivor AI** — In `survivors.rs`: priority system each tick: `critical_need? → address it (path to food/water/quarters)` → `has_task? → path to station, produce` → `idle → wander`. Use `pathfinding` crate A* on shelter grid.
- [ ] 5.7 **Assignment UI** — Create `src/ui/colony_panel.rs`. Side panel listing survivors with dropdown to assign task/station. Show each survivor's needs bars and current action.
- [x] 5.8 **Resource UI** — Top bar showing `Food: 12 (+2/day)`, `Water: 8 (−1/day)`, etc. Net delta calculated from production minus consumption.
- [ ] 5.9 **Build queue** — Player can designate tiles for new rooms (carve walls → floor). Queue system: each construction task costs scrap, takes N ticks, reduced by assigned builders. Station placement: select station type, place on valid empty floor tile.
- [ ] 5.10 **Workbench repair** — At Workbench station: player interaction opens repair UI. Select broken armor → costs scrap + time (3 shelter ticks base, reduced by Repair skill). Restores armor to full durability.
- [x] 5.11 **Shelter ↔ Overworld transition** — Walking to shelter Entrance tile + confirm → `AppState::Overworld`. Arriving at shelter node from overworld → `AppState::Colony`.
- [ ] 5.12 **Deposit loot** — On entering shelter: dungeon loot in player inventory can be deposited into shelter stockpile (resources added to `ShelterResources`, equipment stored).
- [x] 5.13 **Survivor death** — If hunger or thirst hits 0 → starvation/dehydration damage per tick. HP reaches 0 → survivor despawned permanently. Game log: "Survivor X has died of starvation."
- [x] 5.14 **Tests** — Test resource production with staffed/unstaffed stations. Test survivor need decay and critical need behavior. Test build queue cost and duration.

### Done When
- Walkable shelter rendered with rooms and stations
- Survivors walk around, eat, drink, rest, work
- Resources visibly tick up/down with net delta display
- Player assigns survivors to stations via UI
- Build new rooms by carving walls, place new stations
- Repair broken armor at Workbench
- Survivors die if needs hit 0

---

## Slice 6 — The Overworld

**Demo**: Leave shelter, see node graph on the map. Travel between nodes. Weather affects travel. Random encounters. The first dungeon run triggers Gabriel's introduction. The run can be quick-saved and quick-loaded once the full loop exists.

### Tasks
- [x] 6.1 **Overworld generator** — Create `src/game/overworld/gen.rs`. Poisson disk sampling to place nodes in difficulty bands. Delaunay triangulation via `spade` crate. MST + 30% random edges for road network. Assign node types: 1 Shelter, 3-5 Dungeon, 2-3 Ruins, 2-3 Crossroads, 1-2 Landmark.
- [x] 6.2 **Overworld map rendering** — Create `src/game/overworld/map.rs`. Render node graph: nodes as labeled icons, roads as lines. Camera centered on player position. Fog of war: unvisited nodes hidden, discovered nodes visible.
- [x] 6.3 **Node travel** — Player clicks/selects connected node → calculate travel time from distance. During travel: simulate tile-walking segments. Time ticks pass. Food/water consumed (1 each per day of travel).
- [x] 6.4 **Weather system** — Create `src/game/overworld/weather.rs`. Roll weather per travel day using `derive_weather_rng(world_seed, day)`. 8 types with weights from overworld.md. Apply effects: visibility reduction, ranged penalty in encounters, travel speed reduction.
- [ ] 6.5 **Encounters** — During travel: roll per road segment (5%/15%/25% by distance band). Hostile → mini-combat on small generated tile map. Scavenge → small loot popup. Nothing → continue.
- [x] 6.6 **Overworld HUD** — Create `src/ui/overworld_panel.rs`. Top: weather display. Left: travel log (days, resources, events). Right: node inspector on hover (type, difficulty, theme).
- [ ] 6.7 **Shelter ticks during travel** — While traveling, shelter simulation ticks advance (N ticks per travel day). Production continues. Survivor needs decay. Auto-eat/drink.
- [x] 6.8 **Node interactions** — Arrive at Dungeon node → "Enter dungeon?" → `AppState::Dungeon` with that node's theme. Ruins node → scavenge event (flat loot). Crossroads → choose next node. Landmark → placeholder narrative text.
- [x] 6.9 **Gabriel intro dungeon** — Mark the closest dungeon node as `GabrielIntro`. On floor 2 of that dungeon, spawn the scripted Gabriel room, dialogue, and the initial companion join sequence. Gabriel uses a "Ghost AI" (shares tiles, no physics collision) to prevent corridor blocking, and attacks alongside the player.
- [x] 6.10 **SaveGame schema + Save & Quit** — Create `src/core/save.rs`, aggregating the `Serialize/Deserialize` types built since Slice 1. Implements Autosave on shelter return and "Save & Quit" to prevent save-scumming. Quick-save/load is strictly a dev-only debug toggle.
- [x] 6.11 **Full loop wiring** — Verify complete loop: Menu → Colony (shelter) → Overworld (travel) → Dungeon (explore/fight) → extract → Overworld (return) → Colony (deposit loot, manage) → repeat.
- [x] 6.12 **Tests** — Test overworld generator (all nodes connected). Test travel resource consumption. Test weather determinism (same seed → same weather). Test save/load round-trip for an in-progress run.

### Done When
- Overworld shows node graph with roads and fog of war
- Travel consumes time, food, water
- Weather rolled per day, affects encounters
- Encounters happen on roads (combat or scavenge)
- Shelter sim ticks forward while traveling
- The first dungeon run triggers Gabriel's intro and companion join
- Quick save/load works on an in-progress campaign
- Can reach a dungeon node, enter it, extract, return to shelter
- Full gameplay loop works end-to-end

---

## Slice 7 — Raids

**Demo**: Raids trigger at the shelter. Pre-raid planning screen. Turn-based combat on shelter map with survivor presets. Auto-resolve if player is away. Post-action report.

### Tasks
- [x] 7.1 **Raid trigger** — Create `src/game/colony/raids.rs`. `RaidChance` resource tracking accumulated probability. Each shelter tick: roll against `base_chance + visibility_bonus + hostility_bonus`. On trigger: queue raid event.
- [x] 7.2 **Raid forecasting** — 3-5 ticks before raid: game log warnings ("Scout tracks spotted near perimeter", "Hostile movement detected").
- [ ] 7.3 **Pre-raid transition** — When raid fires and player is at shelter: show egui modal with raider count, estimated strength. UI to reassign survivor combat presets (Flee, Defend, Support, Hold Gate). "Ready" button → shelter switches to turn-based (`AppState::Combat` reusing dungeon combat systems).
- [ ] 7.4 **Raider spawning** — Spawn 3-6 raider entities at perimeter gate tiles. Raider stats scale with game progression (ticks elapsed). Raider AI: pathfind to food/water storage → loot → fight defenders in path → retreat at 50% casualties.
- [ ] 7.5 **Survivor combat presets** — Survivors act autonomously during raid combat. Flee: pathfind to interior. Defend: hold position, fight if engaged. Support: follow player at 3-tile range. Hold Gate: move to entrance, block.
- [ ] 7.6 **Raid resolution** — Raiders retreat when losses > 50% OR when loot target reached (steal X resources). Remaining raiders flee toward perimeter exit. Combat ends when all raiders dead or fled.
- [ ] 7.7 **Post-raid assessment** — Show summary modal: survivors lost, resources stolen, walls/stations damaged. Damaged stations need repair (scrap + time).
- [x] 7.8 **Schrodinger's Raid (auto-resolve)** — If raid triggers while player is in dungeon/overworld: auto-resolve using `shelter_defense_rating` vs `raider_strength`. Calculate casualties and losses proportionally. Store result. Show post-action report on returning to shelter/overworld.
- [x] 7.9 **Tests** — Test raid probability scaling. Test auto-resolve outcomes. Test survivor preset pathfinding (flee → interior, hold gate → entrance).

### Done When
- Raids trigger based on shelter visibility/hostility
- Forecast warnings appear in game log
- Player plan defense via preset UI, then fights alongside survivors
- Raiders target resources, flee when losing
- Post-raid summary shows losses
- Auto-resolved raids when player is away with report on return

---

## Slice 8 — Factions & Narrative

**Demo**: Factions exist on the overworld with territories. Themed enemies spawn based on faction. Landmark nodes surface narrative beats. Lore fragments are found in dungeons.

### Tasks
- [x] 8.1 **Faction data model** — Create `src/game/factions.rs`. `Faction { id, name, archetype, disposition, home_node, traits, named_npcs }`. `FactionArchetype` enum (Puritan, Military, Commune, Cult, Traders). `FactionDisposition` toward player (Hostile, Neutral, Friendly — static at MVP, no reputation shifts).
- [x] 8.2 **Faction generation** — On new game: instantiate 3 hardcoded factions (Survivors, Michael's Host, Fort Pershing) + generate 2-3 proc-gen factions using algorithm from procgen.md. Assign home nodes. Generate names from `naming-conventions.md` patterns.
- [x] 8.3 **Territory overlay** — On overworld map: tint nodes/roads within faction influence radius. Show faction name on hover.
- [x] 8.4 **Faction-themed encounters** — Overworld encounters in faction territory spawn faction-appropriate enemies (Puritans → zealot enemies, Military → armed soldiers, etc.). Dungeon enemy spawns influenced by nearest faction.
- [x] 8.5 **Landmark narrative events** — Add lightweight narrative text events to landmark nodes that surface lore, faction activity, or world-state warnings without creating full quest chains.
- [x] 8.6 **Lore fragments** — Create `src/game/dungeon/lore.rs`. Lore items spawn in loot rooms (10-15% chance per room). Pickup adds entry to a `LoreJournal` resource. Journal viewable via `J` key → egui panel showing collected fragments.
- [x] 8.7 **Tests** — Test faction generation determinism (same seed → same factions). Test territory assignment (no overlaps on home nodes). Test landmark narrative events trigger on the correct node types.

### Done When
- 5-6 factions exist on overworld with visible territories
- Encounters spawn faction-themed enemies
- Landmark nodes surface narrative beats or warnings
- Lore fragments collected and viewable in journal

---

## Slice 9 — Progression & Polish

**Demo**: Full MVP loop polished. Perks unlock at skill thresholds. Equipment tiers in loot tables. Research Table progression. Main menu and permadeath flow are complete.

### Tasks
- [x] 9.1 **Perk system** — Create `src/core/perks.rs`. `PerkTree` per skill with T1/T2/T3 gates. MVP ships only the Melee, Ranged, and Toughness trees. On level-up to threshold: egui popup with perk choice. Perks stored in `PlayerPerks` component, applied as passive modifiers.
- [x] 9.2 **Loot tables** — Create `src/game/dungeon/loot.rs`. Tiered loot tables: floor depth → weighted random from appropriate tier. Include resource drops (Raw Meat, Dirty Water, Scrap, Medicine, Ammo) + equipment drops.
- [ ] 9.3 **Research Table** — At shelter: interact with Research Table station → egui panel showing the shallow MVP unlock tree. Spend scrap + ticks to unlock the limited T2 station upgrades used in MVP (Workbench, Cook, Purifier, Generator).
- [x] 9.4 **Main menu** — `AppState::Menu` system. egui panel: "New Game" (optional seed input), "Load Game" (if save exists), "Quit". New Game → generate world → `AppState::Colony`.
- [x] 9.5 **Permadeath flow** — On player death: delete save file. `AppState::GameOver` screen shows stats (turns survived, enemies killed, floors explored). "New Game" button.
- [x] 9.6 **HUD polish** — Integrate all HUD elements from `ui-design.md`: sanity meter, AP pips, health bar, minimap (show revealed rooms as dots), ammo counter, clip status, armor status.
- [x] 9.7 **Deterministic RNG verification** — End-to-end test: two runs with same seed produce identical dungeon layouts, overworld graphs, faction names, weather sequences, and loot tables.
- [x] 9.8 **Full loop smoke test** — Manual playthrough: New Game → shelter → overworld → dungeon → fight → extract → return → manage colony → survive raid → save → load → continue.
- [x] 9.9 **Tests** — Perk application test. Loot tier distribution test. RNG determinism test.

### Done When
- Perks unlock at skill level thresholds with choice UI
- Research Table unlocks the limited MVP T2 station upgrades
- Loot tables scale with dungeon depth
- Main menu with New Game (seed), Load, Quit
- Death deletes save, shows stats, returns to menu
- Same seed produces identical worlds
- Full MVP loop playable start to finish

---

## Dependency Graph

```
Slice 1 (Walk)
  └─→ Slice 2 (Fight)        [needs: tilemap, player, movement, FOV]
       └─→ Slice 3 (Gear)    [needs: combat, enemies, turn system]
            └─→ Slice 4 (Crawl) [needs: inventory, equipment, abilities]
                 ├─→ Slice 5 (Shelter) [needs: tilemap, items, resources]
                 │    └─→ Slice 6 (Overworld) [needs: shelter, travel resources]
                 │         └─→ Slice 7 (Raids) [needs: shelter map, combat, survivors]
                 │              └─→ Slice 8 (Factions) [needs: overworld, enemies, dungeons]
                 │                   └─→ Slice 9 (Polish) [needs: all systems integrated]
                 └─────────────────────────┘ (Slice 5 also depends on Slice 4's item model)
```

Slices are strictly sequential. Each builds on the previous. No parallel paths.

---

## Estimated Scope Per Slice

| Slice | New Systems | Approx. New LOC | New Tests |
|-------|------------|-----------------|-----------|
| 1 — Walk the Dungeon | 6 (tilemap, BSP, theme, player, camera, FOV) | ~1200 | ~8 |
| 2 — Fight | 6 (turns, stats, d100, AI, status, gamelog) | ~1000 | ~10 |
| 3 — Gear Up | 6 (items, inventory, equipment, cover, abilities, HUD) | ~1100 | ~10 |
| 4 — Full Crawl | 5 (multi-floor, sanity, anomalies, skills, loot scaling) | ~900 | ~8 |
| 5 — The Shelter | 7 (shelter gen, survivors, stations, resources, build, repair, AI) | ~1400 | ~10 |
| 6 — The Overworld | 7 (node gen, travel, weather, encounters, Gabriel intro, save/load, loop wiring) | ~1400 | ~10 |
| 7 — Raids | 4 (triggers, raider AI, presets, auto-resolve) | ~800 | ~6 |
| 8 — Factions & Narrative | 4 (faction model, gen, territories, lore) | ~700 | ~5 |
| 9 — Polish | 5 (perks, loot tables, research, menu, RNG verify) | ~800 | ~6 |
| **Total** | | **~9300** | **~73** |
