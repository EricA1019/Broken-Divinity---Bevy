# Colony

The shelter colony is one of Broken Divinity's two core loops (the other being dungeon exploration). It runs during `AppState::AtShelter`. At MVP, the colony is **minimal but physically present** — a walkable tilemap where the player moves around, places stations, and manages a small crew of survivors.

---

## Shelter Layout

### MVP: Walkable Compound

The shelter is a walkable tilemap using the same renderer as dungeons (hybrid ASCII walls + 16×16 sprites). This is NOT an abstract menu — the player physically walks around the shelter.

| Property | Value |
|----------|-------|
| Initial area | ~40×30 tiles |
| Structure | Walled compound with interior rooms + exterior perimeter |
| Generation | BSP variant: rectangular compound, 3-4 starting rooms (entrance, quarters, 1 empty), expandable |
| Rendering | Same tile system as dungeons — consistency, less work |
| Mode | Shelter turns are real-time (not turn-based). Switches to turn-based for raids. |

### Room Types

| Room | Starting? | Purpose |
|------|-----------|---------|
| Entrance | Yes | Gate to overworld. Raid entry point. |
| Quarters | Yes | Survivor housing. Determines survivor cap. |
| Workshop | Yes (empty) | Player places first station here. |
| Expansion zones | Designatable | Player queues construction to carve new rooms from walls. |
| Storage | Buildable | Resource stockpile room. Capacity scales with size. |
| Perimeter | Exists | Exterior walls, patrol routes, gate tiles. |

### Construction

Players expand the shelter by queuing construction tasks:

- **New rooms**: Carve from adjacent wall tiles. Costs scrap. Takes time (120 turns base, reduced by builders).
- **Station placement**: Place a station on valid floor tiles in an existing room. Costs scrap. Immediate (no build time for T1).
- **Wall repair**: After raids, damaged wall tiles can be repaired. Costs scrap.

### Phase 2 Additions
- Room specialization (dedicated medical wing, armory, etc.)
- Outdoor areas (garden plots, water collection)
- Aesthetic upgrades (floors, lighting) that affect survivor morale

### Phase 3 Additions
- Multi-building compound (separate structures connected by paths)
- Outpost system (limited satellite shelters at overworld nodes)
- Environmental hazards from weather (storms damage buildings, ashfall contaminates water)

---

## Stations

### MVP: 10 Station Types, T1 Only

All stations start at T1. T2/T3 upgrades exist in data but require Research Table progression (shallow at MVP).

| Station | Category | Primary Output | Worker Slots | Power Upkeep |
|---------|----------|----------------|-------------|--------------|
| **Workbench** | Production | Equipment repair (MVP), crafting (Phase 2) | 2 | 1 |
| **Cook** | Production | Food (requires Raw Meat) | 1 | 0 |
| **Purifier** | Production | Water (requires Dirty Water) | 1 | 1 |
| **AmmoPress** | Production | Ammo (requires Scrap) | 1 | 1 |
| **Generator** | Production | Power (infinite power vs MVP) | 1 | 0 |
| **ResearchTable** | Production | Tech unlocks (shallow at MVP) | 1 | 1 |
| **MedicalBay** | Service | Healing + medicine production | 1 | 1 |
| **Quarters** | Service | Survivor cap (3-5 at T1) | 0 | 0 |
| **SecurityCheckpoint** | Service | Defense rating for raids | 1 | 0 |
| **MilitiaTraining** | Service | Survivor combat readiness | 1 | 0 |

### Station Footprint
Most stations occupy 2×2 tiles. Placed on valid interior floor tiles (not on walls, doors, or occupied tiles).

### Production Rates
- Staffed stations produce at `rate × (workers_assigned / worker_slots)`.
- Understaffed stations produce proportionally less. Empty stations produce nothing.
- Power: Generator produces; other stations with power upkeep require it. No power = no production.

### Upgrade Path (Phase 2+)

| Tier | Requirement | Cost |
|------|-------------|------|
| T1 | Always available | Scrap |
| T2 | Research Table at T1+ | Scrap × 2 |
| T3 | Research Table at T2+ | Scrap × 3 + rare materials |

---

## Resources

### MVP: 5 Core Resources

| Resource | Produced By | Consumed By | Notes |
|----------|------------|-------------|-------|
| **Food** | CookingStation, dungeon loot, scavenging | Survivors (1/day each) | Run out = starvation penalties → death |
| **Water** | WaterPurifier, dungeon loot | Survivors (1/day each) | Run out = dehydration penalties → death |
| **Scrap** | Dungeon loot, scavenging, salvaging | Construction, repairs, station builds | Primary building currency |
| **Medicine** | MedicalBay, dungeon loot | First Aid ability, survivor healing | Scarce — every use matters |
| **Ammo** | AmmoPress, dungeon loot | Shoot ability (1 per shot) | Universal at MVP |

### Resource Flow
```
Dungeon loot ──┐
Scavenging ────┤
               ├──→ Shelter stockpile ──→ Consumption (survivors, combat, building)
Production ────┘     (visible in UI)       (tracked per-turn delta)
```

### Delta Display
The settlement panel shows **net flow** per resource: `+3 food/day, −5 food/day = −2 net`. The player should always know if they're gaining or losing.

### Phase 2 Additions
- Fuel (Generator station, overworld travel)
- Thaumic components (crafting supernatural equipment)
- Trade goods (barter with factions)

---

## Survivors

### MVP: Minimal Identity

Survivors are NPCs who live at the shelter. At MVP they are **functional workers**, not characters with deep personality. The colony-management layer is thin — assign them to stations and keep them alive.

| Attribute | MVP | Phase 2 | Phase 3 |
|-----------|-----|---------|---------|
| Name | Yes | Yes | Yes |
| Species | Human only | Human only | Human, Angelic, Demonic |
| Background trait | 1 trait (affects work efficiency slightly) | Expanded traits | Full personality system |
| Skills | Same `SkillId` system as player, lower levels | Skill growth over time | Deep skill progression |
| Needs | Hunger, thirst, rest | + morale, medical | + social, comfort |
| Equipment | None | Random equip from available pool | Player-assignable loadouts |
| Relationships | None | None | Friendships, rivalries, romance |
| Mental state | None | Basic moods | Mental breaks, coping behaviors |
| Combat stats | Minimal (HP, basic damage) | Moderate | Full RPG character |
| Commandable | Never | Never | Never |

### Survivor Cap

| Quarters Tier | Cap | Phase |
|---------------|-----|-------|
| T1 | 3-5 | **MVP** |
| T2 | 8-12 | Phase 2 |
| T3 | 15-20 | Phase 3 |

### Needs System (MVP)

3 needs, each 0-100 (start at 100, decay over time):

| Need | Decay Rate | Recovery | Critical Threshold | Critical Effect |
|------|-----------|----------|--------------------|----------------|
| Hunger | −1 per tick | +3 per tick at CookingStation output | 35 | Work penalty → starvation damage |
| Thirst | −1 per tick | +3 per tick at WaterPurifier output | 35 | Work penalty → dehydration damage |
| Rest | −1 per tick | +4 per tick in Quarters | 35 | Work penalty → collapse |

Critical needs override all other task assignments — a starving survivor will abandon their station to eat.

### Task Assignment (MVP)

Survivors perform one task at a time. The player assigns tasks through the UI.

| Task | Behavior |
|------|----------|
| **Working(StationType)** | Go to assigned station, produce resources |
| **Construction** | Go to build site, reduce remaining build turns |
| **Resting** | Go to Quarters, recover rest/health |
| **Patrolling** | Walk perimeter, contribute to defense rating |
| **Idle** | No assignment — wanders, eats/drinks if needed |

### AI Behavior (MVP)

Survivors are not directly controllable. They follow simple priority logic:

```
1. Critical need? → Address it (eat, drink, rest)
2. Assigned task? → Go to station/site, perform task
3. No assignment? → Idle wander
```

No utility scoring, no sticky bonus, no travel cost optimization at MVP. These are Phase 2 additions from the colony-management skill.

### Survivor Death

Survivors can die from:
- Starvation/dehydration (needs hit 0)
- Raid combat (HP reaches 0)
- Unhealed wounds (future: medical system)

Dead survivors are gone permanently. With only 3-5 at MVP, every loss is devastating.

---

## Raids

### MVP: Turn-Based Shelter Defense

Raids are the highest-stakes colony event. They use the same combat system as dungeons, played on the shelter map.

### Trigger
**Event-driven**, NOT timer-based. Raid probability increases with:
- Shelter visibility (more construction, more activity = more visible)
- Faction hostility (attacking faction members, refusing demands)
- Game progression (later = more dangerous world)

A raid can be **forecast** — subtle warnings in the game log ("Scout tracks spotted near the perimeter", "Faction X is mobilizing") give the player time to prepare.

### Schrodinger's Raid

If a raid triggers while the player is away (in the overworld or a dungeon), the raid **auto-resolves** based on the shelter's total defense rating. The player returns to a post-action report detailing casualties, stolen resources, and damaged stations.

### Raid Flow

```
1. Warning phase (several turns of foreshadowing in game log)
2. Raid declared → TRANSITION SCREEN
   ├── Player sees raider count, estimated strength
   ├── Player reassigns survivor combat presets
   ├── Player checks ammo/med stockpiles
   └── Player confirms "Ready" → shelter switches to turn-based
3. Turn-based combat on shelter map
   ├── Player controls their character
   ├── Survivors act autonomously per preset
   └── Raiders enter from perimeter, target stockpiles and stations
4. Resolution
   ├── Raiders retreat at 50%+ losses OR grab enough loot and flee
   ├── OR all raiders defeated
   └── Damage assessment: survivor casualties, wall damage, resource theft
5. Post-raid
   └── Repair phase, losses tallied, game log summary
```

### Survivor Combat Presets

| Preset | Behavior | Best For |
|--------|----------|----------|
| **Flee** | Run to interior, hide behind doors | Non-combatants, protect key workers |
| **Defend** | Hold position at assigned station, fight if engaged | Station guards |
| **Support** | Follow player at medium range, assist when player engages | Ranged-capable survivors |
| **Hold Gate** | Position at shelter entrance, block raider advance | Strong melee survivors |

The player sets these **before** the raid starts (transition screen). During combat, survivors execute their preset autonomously. The planning layer IS the strategy.

### Raider Behavior (MVP)

Raiders are simple AI:
1. Enter from perimeter gate tiles
2. Primary target: food/water storage rooms
3. Secondary target: defenseless stations (loot resources)
4. Fight defenders who block their path
5. Retreat when losses > 50% or looting complete
6. Flee toward perimeter exit

### Raid Scale

| Progression | Raider Count | Composition |
|-------------|-------------|-------------|
| Early | 3-4 | Human scavengers, light weapons |
| Mid | 4-5 | Mixed scavengers + armed raiders |
| Late | 5-6 | Armed raiders, possibly faction troops |

### Phase 2 Raid Additions
- Faction-specific raider types (Puritan zealots, Military deserters, demon-led warbands)
- Multi-wave raids (second wave arrives after first is repelled)
- Siege mechanics (raiders camp outside, starve you out)
- Raid rewards (captured raiders → recruit or interrogate)

---

## Colony Loop Summary

### The MVP Rhythm

```
Return from dungeon with loot
  └──→ Deposit resources into stockpile
  └──→ Repair armor at Workbench
  └──→ Check resource deltas (are we net positive or negative?)
  └──→ Adjust survivor assignments if needed
  └──→ Queue any construction (new room, new station)
  └──→ Rest, heal, eat
  └──→ Watch for raid warnings
  └──→ When ready: walk to shelter exit → overworld → next dungeon
```

The colony should feel like a **home base** — the safe place you return to between dangerous runs. It creates the rhythm: preparation → danger → return → preparation. At MVP it's functionally thin but physically present — you walk around it, you see survivors working, you notice when food is running low.

See also: [combat.md](combat.md) for raid combat mechanics, [overworld.md](overworld.md) for the world around the shelter, [progression.md](progression.md) for station upgrade gates, [phase-roadmap.md](phase-roadmap.md) for Phase 2/3 colony features.
