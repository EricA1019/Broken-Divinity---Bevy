# Phase Roadmap

Three development phases. Each builds on the previous. No phase requires systems from a later phase. **Read this first** — it defines what's in scope for every feature.

---

## MVP — "The Bones"

**Goal**: The complete game loop at minimum depth. Every major system present, some shallow. A player should experience: build shelter → travel overworld → enter dungeon → fight → extract → return → manage colony → repeat. The tone, tension, and core rhythm must be felt.

**Success criteria**:
- A player can start a seeded run, leave the shelter, travel the overworld, clear or extract from a dungeon, and return to the shelter with loot.
- The shelter sim keeps running while the player travels, including survivor needs, production/consumption, and raid risk.
- At least one raid path is playable end-to-end: direct defense if the player is home, auto-resolve + report if the player is away.
- The run can be saved, loaded, and continued without losing state.

### What Ships

#### Combat (the priority)
- d100 skill check system (fully functional)
- 6 damage types (3 physical, 3 supernatural)
- Environmental cover: half cover (−20% to-hit), full cover (−40% to-hit)
- Speed-based action budget (1-3 actions/round)
- Armor durability system (full AR until broken, repair at shelter)
- 2 status effects: Wounded (DoT), Stunned (skip action)
- 4 abilities: Attack, Shoot, First Aid, Sprint
- Ammo system (universal ammo, clip + reload)

#### Colony
- Walkable shelter tilemap (same renderer as dungeons, ~40×30)
- 10 station types, T1 baseline with a shallow Research Table that unlocks a limited set of T2 upgrades late in MVP
- 3-5 survivors (T1 Quarters cap)
- Survivors: name, species (human only at MVP), job assignment, basic needs (hunger, thirst, rest), health, can die
- No survivor personalities, moods, relationships, or equipment loadouts
- 5 resources: food, water, scrap, medicine, ammo
- Basic production/consumption loop (staffed stations produce, survivors consume)
- Build queue for station placement (costs scrap, takes time)

#### Shelter Defense
- Turn-based raids on the shelter map (same combat system as dungeons)
- Pre-raid transition screen: player reassigns survivor roles/presets before combat starts
- Player controls only themselves — survivors act autonomously with preset behaviors (flee, defensive position, hold station)
- 3-6 raiders per event
- Event-driven triggers (faction hostility, shelter visibility) — NOT timer-based

#### Overworld
- Node graph with Delaunay road network
- Tile walking between nodes along road segments (path-constrained, not full free-roam off-road exploration)
- 8 weather types affecting travel (Clear through Anomaly Storm)
- Basic encounters during travel
- Travel time as a resource

#### Dungeons
- BSP room generation (6-12 rooms per floor)
- 3 themes: Urban Decay, Underground, Military
- Anomalies (sanity-draining hazards, 3 types)
- Door placement, stair placement
- Themed enemy spawns and loot

#### Sanity
- Single-track sanity bar (0-100)
- Drains from supernatural exposure, anomalies, combat events
- Effects at thresholds: perception penalties, hallucinations, control loss
- Resets to 0 on returning to shelter
- Does NOT include dual-track (Long-Term Erosion deferred)

#### Stealth
- LOS-based detection only
- Noise exists (loud/quiet) with minimal mechanical impact
- No enemy facing direction, no FOV cones, no alert state machine

#### Factions
- 3 hardcoded anchor factions: Michael's Host, Fort Pershing, The Collective
- 2-3 proc-gen factions (seeded at world-gen, deterministic)
- Factions exist on the overworld map with territories
- NO reputation mechanics (factions are friend/foe by archetype, not earned)

#### Narrative
- Scripted first dungeon with Gabriel encounter
- Scattered lore fragments in dungeons (journals, inscriptions)
- Gabriel as first companion (anchors Sundering questline hook)
- No faction quest chains, no reputation-gated truths

#### Meta
- Full permadeath only (everything lost on death)
- 1 player background (Shelter Founder)
- 1 playable species (Human)
- Deterministic RNG (ChaCha8Rng, seed-based)
- Save/load (single save slot, serde/JSON)

### Explicitly NOT in MVP

| System | Why Deferred |
|--------|-------------|
| Dual sanity (Long-Term Erosion track) | Needs narrative progression systems to matter |
| Full stealth (facing, FOV cones, alert states) | Complex AI behavior; MVP enemies are simpler |
| Faction reputation mechanics | Requires more factions and quest content to be meaningful |
| Expeditions | Player does all exploration at MVP |
| Survivor personality / moods / relationships | Colony is minimal — add when colony loop is proven |
| Survivor equipment loadouts | Phase 2 feature (random equip from pool) |
| Caravans / trade routes | Requires deeper economy |
| Faction wars / dynamic overworld events | Requires living faction system |
| Proc-gen faction history | Song of Syx feature — Phase 3 |
| Additional dungeon themes (8 remaining) | More themes once the 3 MVP themes prove the system |
| Multiple permadeath modes | MVP ships full permadeath only |
| Multiple backgrounds / species | MVP ships 1 background, human only |
| Boss AI with phase transitions | Needs more enemy variety first |
| Hell/heaven zones on the overworld | Phase 3 corrupted-zone content, not part of MVP navigation |
| Perk expansion beyond the three core combat trees | MVP ships only Melee, Ranged, and Toughness perk trees |

---

## Phase 2 — Colony Foundation

**Goal**: The colony feels alive. Survivors have identity. Economy has depth. Factions respond to player actions. The game starts to have emergent stories.

**Depends on**: All MVP systems working and tested.

### Additions

#### Colony Deepening
- Survivor equipment: survivors equip gear from available pool (random selection at Phase 2)
- Survivor traits: background trait affects work efficiency and dialogue
- Expanded T2 station coverage and a full station-upgrade queue
- Deeper production chains (intermediate resources, crafting recipes)
- Station upgrade queue functional (T1 → T2)

#### Economy & Crafting
- Crafting system at Workbench (recipes consume resources, produce equipment)
- Equipment repair at Workbench (armor durability restoration)
- Trade with friendly factions (basic barter using resources)
- Post-MVP resources added: fuel, thaumic components

#### Faction Reputation
- 5-tier reputation system activated (Hostile → Unfriendly → Neutral → Friendly → Allied)
- Actions shift reputation: completing tasks, attacking members, trading, dungeon choices
- Reputation unlocks: trade access, safe passage, recruits, information
- Reputation propagation: direct faction +100%, allies +50%, enemies −30%

#### Stealth Expansion
- Enemy facing direction (8-way)
- FOV cones with half-angle and peripheral range
- 3 alert states: Unaware → Suspicious → Alert
- Stealth attack bonus (auto-crit from Unaware)
- Noise propagation: Quiet (3 tiles) vs Loud (12 tiles) with mechanical impact

#### Dungeons
- Additional themes: Religious, Corporate, Industrial, Scientific (4 new, total 7)
- Themed decoration pools per theme
- Room features: shrines, supply caches, special encounters

#### Sanity
- Dual-track system activated (Raid Exposure + Long-Term Erosion)
- Long-Term Erosion accumulates across the campaign, never fully resets
- Unreliable narrator UI effects at high erosion

#### Overworld
- Faction territory shifts based on events
- Basic overworld encounters with faction patrols
- Event-spawning of new proc-gen factions during play

#### Narrative
- Faction-specific quest chains (1-2 per hardcoded faction)
- Reputation-gated lore reveals
- Deeper Gabriel companion arc

#### Meta
- Settlement Persists permadeath mode (player dies, colony continues with new character)
- Additional backgrounds (Exile, Scavenger)

---

## Phase 3 — Full Colony

**Goal**: The DF/RimWorld/Song of Syx vision. Deep colony management. Living overworld. Proc-gen faction history. The game that develops for years.

**Depends on**: Phase 2 reputation, expanded economy, dual sanity all working.

### Additions

#### Deep Colony Management
- Survivor personalities: traits affecting work efficiency, social compatibility, stress responses
- Relationships between survivors (friendship, rivalry, romance)
- Mental break system (survivors crack under pressure — flee, fight, catatonia)
- Detailed scheduling and priority assignment
- Individual survivor combat stats for shelter defense
- T3 station unlocks (requires Research Table at T2+)
- Large survivor cap scaling (20-30 with Quarters upgrades)

#### Song of Syx — Proc-Gen History
- Pre-game history simulation: centuries of faction history generated before play starts
- Faction founding, wars, alliances, collapses, migrations — all simulated
- Generated history shapes the starting world state (faction territories, ruins, artifacts, grudges)
- Generated NPCs with history connections
- History is discoverable through lore fragments — the world existed before you

#### Living Overworld
- Faction wars: factions fight each other for territory independent of the player
- Caravans: trade caravans move between settlements, can be intercepted or protected
- Dynamic events: plagues, Veil surges, faction collapses, new settlements founded
- Weather affects colony directly (storms damage buildings, ashfall contaminates water)
- Overworld random encounters with meaningful choices and consequences

#### Expeditions
- Dispatch survivors to distant locations
- Semi-interactive: text event chain during expedition with Oregon Trail-style choices
- Duration based on distance and weather
- Success/failure → resource gains or survivor casualties

#### Full Stealth System
- Complete noise propagation model
- Cover and concealment as distinct systems
- Enemy communication (alert one → alert nearby)
- Terrain interaction with pathfinding (water, crumbled walls, anomalies)

#### Remaining Dungeon Themes
- Infernal, Celestial, Medical, Reality-warped (4 final themes, total 11)
- Theme mixing within dungeons (zone transitions)
- Boss encounters with phase transitions and special abilities

#### Meta
- Roguelite permadeath mode (meta-progression carries over)
- Angelic and Demonic playable species
- Full background system (Founder, Exile, Scavenger, Deserter, more)
- Multiple save slots

---

## Dependency Chain

```
MVP (The Bones)
 │
 ├─ Core combat loop (d100, cover, action budget, damage)
 ├─ Walkable shelter + basic colony management
 ├─ 3 dungeon themes + BSP generation
 ├─ Overworld node graph + tile walking + weather
 ├─ Single-track sanity
 ├─ Simple LOS detection
 ├─ 3 hardcoded + 2-3 proc-gen factions (no rep)
 ├─ Gabriel scripted encounter
 └─ Full permadeath
       │
Phase 2 (Colony Foundation) — requires MVP
 │
 ├─ Faction reputation (requires factions from MVP)
 ├─ Expanded stealth (requires LOS from MVP)
 ├─ Dual sanity (requires single-track from MVP)
 ├─ Crafting + economy (requires stations + resources from MVP)
 ├─ Survivor equipment (requires crafting)
 ├─ +4 dungeon themes (requires BSP gen from MVP)
 ├─ Faction quest chains (requires reputation)
 └─ Settlement Persists mode (requires save/load from MVP)
       │
Phase 3 (Full Colony) — requires Phase 2
 │
 ├─ Deep colony management (requires survivor traits from Phase 2)
 ├─ Song of Syx history (requires faction event-spawning from Phase 2)
 ├─ Living overworld (requires faction territory from Phase 2)
 ├─ Expeditions (requires expanded economy from Phase 2)
 ├─ Full stealth (requires expanded stealth from Phase 2)
 ├─ Remaining themes + bosses (requires +4 themes from Phase 2)
 └─ Roguelite mode + species + backgrounds
```

See also: [combat.md](combat.md), [colony.md](colony.md), [overworld.md](overworld.md), [procgen.md](procgen.md), [progression.md](progression.md) for system-level detail.
