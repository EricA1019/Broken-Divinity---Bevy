# Procedural Generation

Broken Divinity uses deterministic procedural generation seeded from a single world seed. All randomness derives from this seed using ChaCha8Rng, meaning identical seeds produce identical worlds.

---

## Seed Architecture

### World Seed

A single `u64` seed entered at new-game (or randomized). From it, all subsystem seeds are derived:

```
world_seed (u64)
  ├── overworld_seed     = hash(world_seed, "overworld")
  ├── dungeon_seed(id)   = hash(world_seed, "dungeon", dungeon_id)
  ├── weather_seed       = hash(world_seed, "weather")
  ├── faction_seed       = hash(world_seed, "factions")
  ├── encounter_seed     = hash(world_seed, "encounter")
  └── loot_seed(id)      = hash(world_seed, "loot", context_id)
```

### RNG Implementation

| Component | Crate | Type |
|-----------|-------|------|
| RNG engine | `rand_chacha` | `ChaCha8Rng` (fast, deterministic, portable) |
| Seed hashing | `seahash` or similar | Combine seed + domain string → u64 |
| API | `rand` traits | `Rng`, `SeedableRng` |

### Why Determinism Matters
- **Reproducible bugs** — share a seed, reproduce the world
- **Fair challenge** — same seed = same difficulty, enables seed racing
- **Save compatibility** — only need to store seed + player decisions, not world state

---

## Dungeon Generation

### MVP: BSP Room-Corridor

Dungeon floors use Binary Space Partitioning (BSP) for room placement, then connect rooms with L-shaped corridors.

### Algorithm

```
1. Start with full floor rectangle (e.g., 80×50)
2. Recursively split into partitions:
   - Choose axis: alternate H/V, bias toward splitting the longer dimension
   - Split point: random within 40%-60% of current dimension
   - Stop when partition < min_room_size (8×8)
3. Place a room in each leaf partition:
   - Room size: random within partition bounds, min 5×5
   - Room position: random within partition, 1-tile wall buffer
4. Connect rooms:
   - Walk the BSP tree bottom-up
   - Connect sibling rooms with L-shaped corridor (random bend point)
5. Place doors at corridor-room junctions
6. Place stairs (up: near start, down: far from start by pathfinding distance)
```

### Theme Overlay

After generating the base layout, the dungeon theme applies:

| Theme | Floor Tile | Wall Tile | Feature Tiles | Hazards |
|-------|-----------|-----------|---------------|---------|
| **Urban Decay** | Cracked concrete | Broken walls, rebar | Rubble piles, collapsed sections | Unstable floors (collapse chance) |
| **Underground** | Stone, mud | Rough stone, timber supports | Flooded sections, cave-ins | Darkness (reduced visibility), water tiles |
| **Military** | Metal grating, linoleum | Reinforced concrete | Locked doors, terminals | Security turrets (inactive at MVP) |

### Room Decoration

Each room gets a **room type** that determines its contents:

| Room Type | Spawn Weight | Contents |
|-----------|-------------|----------|
| Empty | 30% | Nothing — breathing room |
| Loot | 20% | 1-3 resource containers |
| Enemy | 25% | 1-3 enemies per difficulty band |
| Hazard | 10% | Environmental danger (theme-specific) |
| Objective | 5% | Floor objective (keycard, valve, quest item) |
| Mixed | 10% | Enemy + loot (risk/reward room) |

### Dungeon Scaling

| Property | Scaling |
|----------|---------|
| Floor count | 3-5 floors (deeper = harder) |
| Room count | 6-10 per floor |
| Enemy density | +15% per floor depth |
| Loot quality | +1 tier per 2 floors |
| Exit placement | Always on deepest floor, maximum distance from stairs |

### Phase 2 Dungeon Additions
- 4 additional themes (Ecclesiastical, Angelic Ruin, Demonic Breach, Sewers)
- Multi-zone dungeons (theme transitions mid-dungeon)
- Special rooms (boss rooms, puzzle rooms, shrine rooms)( )
- Vault rooms (locked behind key items, high-value loot)

### Phase 3 Dungeon Additions
- Remaining themes (Thaumic Lab, Hospital, Residential, Commercial)
- Theme mixing (two themes blended, e.g., flooded military bunker)
- Persistent dungeons (cleared rooms stay cleared, enemies don't respawn)
- Dungeon events (mid-dungeon faction incursions, anomaly surges)

---

## Overworld Generation

### MVP: Node Graph with Difficulty Bands

The overworld is generated as a node graph, NOT a free-roam tile map.

### Algorithm

```
1. Place shelter node at map center
2. Define difficulty bands as concentric rings:
   - Band 0 (near): radius 0-30%  → easy nodes
   - Band 1 (mid):  radius 30-65% → medium nodes
   - Band 2 (far):  radius 65-100% → hard nodes
3. Place nodes using Poisson disk sampling (min distance between nodes):
   - 3-5 dungeon nodes spread across bands
   - 2-3 ruins nodes (mostly Band 0-1)
   - 2-3 crossroads nodes (junctions)
   - 1-2 landmark nodes (Band 1-2)
4. Generate road network:
   - Delaunay triangulate all nodes
   - Compute minimum spanning tree (ensures connectivity)
   - Add ~30% of remaining Delaunay edges (creates alternate routes)
   - Remove edges that would create unnaturally long roads
5. Assign terrain to road segments (visual only at MVP)
6. Assign dungeon themes based on node band and nearby landmarks
```

### Node Properties

```
OverworldNode {
    id: NodeId,
    name: String,          // procedurally generated
    node_type: NodeType,   // Shelter, Dungeon, Ruins, Crossroads, Landmark
    position: Vec2,        // world position
    difficulty_band: u8,   // 0-2
    discovered: bool,      // fog of war
    visited: bool,
    dungeon_theme: Option<DungeonTheme>,  // if Dungeon type
}
```

### Phase 2 Overworld Additions
- Faction territory overlay (nodes claimed by factions, territory borders)
- Moving patrol entities on roads
- Dynamic node states (node raided, node abandoned, node fortified)

### Phase 3 Overworld Additions
- Song of Syx-style pre-game history generation (factions found, war, collapse)
- Living world simulation (factions expand/contract territory between player turns)
- Caravan routes (AI trader entities traveling between faction nodes)

---

## Faction Generation

### MVP: Seed-Only (2-3 Proc-Gen Factions)

At MVP, factions are **generated at world creation** from the faction seed. No faction spawning or evolution during gameplay.

### Algorithm

```
1. Roll number of proc-gen factions: 2-3 (plus 3 hardcoded: Survivors, Michael's Host, Fort Pershing)
2. For each proc-gen faction:
   a. Roll archetype from weighted table:
      - Puritan (religious zealot)           25%
      - Military (authoritarian remnant)     25%
      - Commune (cooperative survivors)      20%
      - Cult (anomaly-worshippers)           15%
      - Traders (resource-focused)           15%
   b. Generate name from archetype naming patterns (see docs/lore/naming-conventions.md)
   c. Roll disposition toward player: Neutral
   d. Assign home node (unoccupied node in Band 1-2)
   e. Generate 2-3 named NPCs (leader + lieutenant(s))
   f. Roll faction traits (1-2):
      - Aggressive / Defensive / Isolationist / Expansionist
      - Thaumic-friendly / Thaumic-hostile
      - Resource-rich / Resource-poor
3. Store in SaveGame as faction roster
```

### Hardcoded Factions

| Faction | Role | Home |
|---------|------|------|
| **Survivors** | Player faction | Shelter node |
| **Michael's Host** | Angelic military, hostile | Landmark node or Band 2 |
| **Fort Pershing** | Human military remnant, wary | Band 1-2 |

### Phase 2 Faction Additions
- Event-spawned factions (new groups emerge from narrative triggers)
- Faction reputation system (actions shift faction disposition)
- Faction quests (requests, demands, trade offers)

### Phase 3 Faction Additions
- Song of Syx pre-game history (factions have backstory: founding, wars, alliances, betrayals)
- Dynamic faction behavior (wage wars, form alliances, collapse)
- Faction diplomacy (player brokers deals between factions)

---

## Loot Generation

### MVP: Weighted Tables

Loot is rolled from weighted tables per context:

| Context | Table | Quality |
|---------|-------|---------|
| Dungeon room (Loot type) | Room loot table | Scales with floor depth |
| Enemy drop | Enemy drop table | Based on enemy tier |
| Scavenge (overworld Ruins) | Scavenge table | Band difficulty |
| Raid loot (raider drops) | Raider drop table | Raid difficulty |

### Quality Tiers

| Tier | Label | Drop Weight (base) |
|------|-------|--------------------|
| 0 | Junk | 40% |
| 1 | Common | 30% |
| 2 | Uncommon | 20% |
| 3 | Rare | 8% |
| 4 | Exceptional | 2% |

Depth/difficulty modifiers shift weights toward higher tiers.

### Loot Categories (MVP)

| Category | Examples |
|----------|----------|
| Resources | Food, water, scrap, medicine, ammo |
| Equipment | Melee weapons, ranged weapons, armor pieces |
| Consumables | Medkits, grenades, rations |
| Quest items | Keycards, data drives, faction tokens |

---

## Determinism Contracts

Every generation system must satisfy:

1. **Same seed → same output.** Always.
2. **No floating-point divergence.** Use integer math for generation decisions where possible.
3. **Platform-independent.** ChaCha8Rng produces identical output on all platforms.
4. **Separable.** Each subsystem uses its own derived seed — regenerating weather doesn't change dungeon layout.
5. **Saveable.** Only the world seed + player choices need to be saved. The world can be regenerated.

See also: [overworld.md](overworld.md) for travel mechanics, [colony.md](colony.md) for shelter layout generation, [combat.md](combat.md) for encounter generation, [phase-roadmap.md](phase-roadmap.md) for generation scope per phase.
