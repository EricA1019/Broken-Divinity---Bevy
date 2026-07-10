# Broken Divinity — Game Design Document

## Overview

**Genre**: Post-apocalyptic religious horror roguelike RPG
**Engine**: Rust + Bevy 0.18
**Rendering**: Hybrid ASCII glyphs (dungeon tiles) + 16×16 sprites (entities)
**Setting Year**: 2026 (33 years post-Sundering)
**Setting Location**: Deliberately unnamed/ambiguous — a composite post-industrial wasteland
**Tone**: Religious horror + post-apocalyptic desperation. Not heroic fantasy, not grimdark nihilism. Beauty exists in ruin. Hope is fragile but present.

---

## Core Pillars

1. **Every run is different** — Proc-gen factions, dungeons, overworld, and events ensure no two campaigns feel the same.
2. **The supernatural has a cost** — Engaging with magic, angels, or demons always extracts a psychological price (sanity).
3. **Survival is the victory condition** — The player is not a hero. They are a survivor making hard choices.
4. **Information is power** — The truth about The Sundering is the deepest reward, gated behind faction trust and dungeon exploration.

---

## Game Loop

A **Universal Tick System** keeps time globally synced across the overworld, dungeons, and the shelter.

```
Shelter Phase (real-time)
  └─ Assign survivors to stations, manage resources, upgrade, trade
  └─ Time passes — events fire, raids happen, factions shift

Overworld Phase (hybrid node + tile)
  └─ Choose destination from world graph (node-based fast travel)
  └─ Explore off-road via tile walking for discovery
  └─ Weather, encounters, time pressure during travel

Dungeon Phase (turn-based)
  └─ BSP-generated rooms, themed to location
  └─ d100 skill-based combat, speed-based action budgets
  └─ Raid Exposure sanity clock — push deeper for better loot, risk your mind
  └─ Extract with loot or lose it all
```

---

## Player Character

### Background System
The player chooses a background at campaign start. Backgrounds define starting skills, equipment, and opening narrative. Planned backgrounds:

- **Shelter Founder** — starting a new settlement from scratch
- **Exiled Survivor** — cast out from another settlement, arriving alone
- **Scavenger** — nomad who discovered a defensible location
- **Military Deserter** — left the Remnant armed forces, combat-skilled but friendless

MVP ships with one background. Full system is post-MVP.

### Species
- **Human** — MVP playable. No innate magic. Can learn thaumaturgy with equipment/training.
- **Angelic** (Post-MVP) — Celestial-touched. Innate Celestial abilities. Nonbinary, feminine-presenting. Revered by Puritans, feared by others.
- **Demonic** (Post-MVP) — Infernal-touched. Innate Infernal abilities. Gendered like humans. Hunted universally.

See [docs/lore/species.md](lore/species.md) for full details.

---

## Combat System

### d100 Skill Check
```
target_number = skill_level + 25 + modifiers − target_dv
success:  roll (1–100) ≤ target_number
critical: roll ≤ raw_skill_level AND success
```

### Damage Formula
```
base_damage = weapon_base + skill_level − target_ar   (min 1)
variance    = ±20% random
crit_mult   = 2.0× (if critical)
magic_reduc = −(target_md / 2) if damage_type is supernatural
```

### Damage Types (6)
**Physical** (resisted by AR): Ballistic, Slash, Blunt
**Supernatural** (resisted by MD): Celestial, Infernal, Thaumic

### Speed & Action Budget
Each entity gets `speed` actions per round. Speed 1 = 1 action, Speed 2 = 2 actions. Entities act in speed-descending order. When all entities exhaust their budgets → WorldTurn phase → tick effects, reset budgets.

### Armor Durability
Armor provides full AR until durability reaches 0, then AR drops to 0 until repaired at shelter. Durability loss is based on actual damage taken (post-armor reduction).

### Status Effects (MVP)
| Effect | Damage/Turn | Special |
|--------|------------|---------|
| Wounded | 3 + hp.max/10 | DoT |
| Stunned | 0 | Skip next action |

### Abilities (MVP)
| Ability | Type | Cost | Effect |
|---------|------|------|--------|
| Attack | Melee | 1 action | Bump-to-attack |
| Shoot | Ranged | 1 action + 1 ammo | Explicit targeting, loud noise |
| First Aid | Utility | 1 action + 1 med | Heal HP (skill-scaled) |
| Sprint | Movement | 1 action | Move 2 tiles in 1 action |

---

## Sanity System (Dual)

### Raid Exposure (Short-Term)
- Ticking clock during dungeon runs
- Accumulates from: combat, anomalies, enemy abilities, environmental hazards
- Resets on returning to shelter
- Creates per-run tension: "get out before you lose it"

### Long-Term Erosion (Persistent)
- Cumulative across the entire campaign
- Erodes from traumatic events, cumulative exposure, NPC loss
- Does NOT fully reset — represents chronic psychological damage
- Creates campaign-level dread and recovery gameplay

### Effects
Low sanity triggers hallucination spawns, unreliable perception, and gameplay debuffs.

See [docs/lore/sanity.md](lore/sanity.md) for narrative context.

---

## Settlement System

### Home Base
One primary settlement. Build and upgrade stations, assign survivors, manage resource production/consumption.

### Outposts
Claimable limited-function positions at strategic overworld locations. Outposts provide trade routes, early warning, or resource extraction but cannot replace the home base.

### Stations
Workstations where survivors perform tasks: crafting, medical, cooking, repair, sentry duty, research. Each station consumes and produces specific resources.

### Survivor Workforce
Survivors are NPCs with skills, needs (hunger, morale, health), and personalities. Assigned to stations or sent on expeditions. They can die.

### Raids
Settlement attacks scale with campaign time and settlement visibility. Defense requires planning: sentries, walls, stockpiled ammo.

---

## Economy

### MVP Resources
A reduced core set for MVP (expanding post-MVP):

- **Food** — consumed daily by survivors
- **Water** — consumed daily
- **Scrap** — building material for upgrades and repairs
- **Medicine** — healing, First Aid ability fuel
- **Ammo** — universal for MVP (future: per-caliber)

Post-MVP additions planned: Fuel, thaumic components, trade goods.

---

## Overworld

### Structure
Hybrid node-based graph + tile walking:
- **Nodes**: Named locations (settlements, dungeon entrances, faction territories, points of interest) connected by roads (Delaunay network)
- **Fast travel**: Click a connected node, time passes proportionally
- **Tile exploration**: Walk off-road between nodes to discover hidden locations

### Hell/Heaven Zones
Permanent overworld zones where the Veil has fully collapsed:
- **Infernal zones**: Visibly corrupted terrain. Demons are native. Extremely dangerous, unique resources.
- **Celestial zones**: Radiant ruins, petrified figures, blinding light. Angels manifest. Rapid sanity drain.
- Visible on the overworld map. The player chooses whether to risk entering.

### Weather & Time
- Weather rolls affect travel (visibility, exposure, faction activity)
- Time passes at shelter in real-time (events accumulate)
- Dungeons are turn-based
- Overworld travel consumes time proportionally — timing is a resource

---

## Faction System

### Architecture
Factions are a **mix of hardcoded archetypes and proc-gen groups**, inspired by Caves of Qud.

**Five Archetypes:**

| Archetype | Magic Affinity | Style |
|-----------|---------------|-------|
| Puritan/Angel | Celestial | Theocratic, angel-backed, authoritarian |
| Infernal/Demon | Infernal | Corruptors, invaders, hierarchical by power |
| Thaumic/Occultist | Thaumic | Knowledge seekers, equipment-based magic |
| Conventional/Military | None | Disciplined remnants, conventional arms |
| Independent/Survivor | None (scavenged) | Pragmatic settlement-builders |

**Hardcoded Factions (always present):**

| Faction | Archetype | Notes |
|---------|-----------|-------|
| **Survivors** | Independent | Player faction |
| **Michael's Host** | Puritan/Angel | Led by Archangel Michael. Militant theocracy. |
| **The Court of Irkalla** | Infernal/Demon | Sumerian-rooted demon court. Invasion beachhead. |
| **The Lethean Circle** | Thaumic/Occultist | Stabilized-rift coven. Knowledge hoarders. |
| **Fort Pershing Garrison** | Conventional/Military | Bunker-based remnant unit. Chain of command intact. |

**Proc-Gen Factions (10-20 per world, seeded at world-gen):**
Generated with Caves of Qud-depth traits. Events can spawn or destroy factions during play.

See [docs/lore/factions.md](lore/factions.md) for detailed trait tables and generation rules.

### Reputation
Per-faction reputation from hostile to allied. Actions shift reputation: helping/attacking members, completing quests, trading, dungeon choices. High reputation unlocks trade, recruits, faction-gated narrative truths, and safe passage.

---

## Dungeon Generation

### BSP Room Generation
Binary Space Partitioning generates rooms connected by corridors. Size, shape, and density vary by theme.

### 11 Themes
Urban Decay, Religious, Underground, Corporate, Medical, Industrial, Scientific, Military, Infernal, Celestial, Reality-warped.

See [docs/lore/dungeon-themes.md](lore/dungeon-themes.md) for flavor details.

### Anomalies
Supernatural distortions — reality glitches, temporal echoes, gravity shifts. High risk, high reward. Spike Raid Exposure.

---

## Permadeath & Game Modes

### The Rimworld Loop & Win Condition
The game operates in an endless loop format akin to Rimworld, challenging players to survive as long as possible. The game "ends" in failure only when the player character dies, but you can recover from survivor/colony deaths if the player character lives. There is no hard win condition at MVP — survival is the objective.

Plans for three modes (MVP ships with one):

1. **Full Permadeath** — settlement and all progress lost on player death
2. **Settlement Persists** — player character dies, settlement continues with a new character
3. **Roguelite** — meta-progression carries over partially between runs

---

## Narrative

### The Sundering Mystery
The central narrative is discovering what caused The Sundering in 1993. The truth is layered and faction-gated:

- **Dungeon lore fragments**: Journals, inscriptions, pre-Sundering terminals
- **NPC quest chains**: Faction-specific NPCs reveal pieces
- **Faction reputation gates**: Deeper truths require higher standing with specific factions
- Each faction has a version of the truth. Some are wrong. Some are partially right.

### Always-Generated Characters
- **Gabriel** — Archangel. First companion, encountered in a scripted first-dungeon event. Anchors the Sundering questline. Nonbinary (they/them), feminine-presenting.
- **Michael** — Archangel. Leader of Michael's Host. Always present in the world.

### Endgame
Two parallel paths:
1. **Sandbox survival** — ignore the narrative, play endlessly, build your settlement, raid dungeons
2. **Discover the truth** — piece together the cause of The Sundering through exploration, faction relationships, and sacrifice. Multiple endings based on what you learn and who you trust.

---

## Detailed Gameplay Documentation

For in-depth mechanical breakdowns with MVP vs Phase 2/3 scope tagging, see `docs/gameplay/`:

| Document | Covers |
|----------|--------|
| [gameplay/phase-roadmap.md](gameplay/phase-roadmap.md) | MVP scope, Phase 2/3 plans, dependency chain, "NOT in MVP" list |
| [gameplay/combat.md](gameplay/combat.md) | d100 system, cover, damage, armor, status effects, abilities, shelter raids |
| [gameplay/colony.md](gameplay/colony.md) | Walkable shelter, stations, resources, survivors, needs, raid defense |
| [gameplay/overworld.md](gameplay/overworld.md) | Node graph, tile walking, weather, encounters, hell/heaven zones |
| [gameplay/procgen.md](gameplay/procgen.md) | Seed architecture, BSP dungeons, overworld gen, faction gen, loot tables |
| [gameplay/progression.md](gameplay/progression.md) | Skills, perks, equipment tiers, station upgrades, sanity, narrative gates |

This GDD provides the **overview**. The gameplay docs provide the **implementation-ready detail**.

---

## Tech Cap

No technology beyond **1993**. The Sundering destroyed global infrastructure.

**Exceptions:**
- Pre-Sundering thaumic-augmented devices (rare relics)
- Crude post-Sundering fabrications from scrap
- No internet, no GPS, no cell phones, no modern logistics
