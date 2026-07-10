# Overworld

The overworld is the connective tissue between shelter and dungeons. It runs during `AppState::Overworld`. At MVP, the overworld is **functional** — a node graph the player navigates between, with tile-based walking segments, weather, and basic encounters.

---

## Map Structure

### Node Graph + Tile Walking

The overworld is a **graph of nodes** connected by Delaunay-triangulated roads. Each node represents a location (shelter, dungeon entrance, ruins, crossroads). Between nodes, the player walks on a scrolling tile map showing terrain.

| Component | Description |
|-----------|-------------|
| **Nodes** | Named locations on the world map. Each has a type + properties. |
| **Roads** | Delaunay paths between nodes. Travel time = distance. |
| **Tile walking** | Between nodes, the player walks through terrain tiles. Not a free-roam map — path-constrained movement along roads. |
| **Fog of war** | Unvisited nodes are hidden. Nodes discovered through travel or information sources. |

### Node Types (MVP)

| Type | Count | Details |
|------|-------|---------|
| **Shelter** | 1 | Player's home base. Starting node. |
| **Dungeon** | 3-5 | Entrances to dungeon instances (Urban Decay, Underground, Military at MVP) |
| **Ruins** | 2-3 | Small scavenging sites — flat loot with light danger |
| **Crossroads** | 2-3 | Travel junctions. Encounter chance. |
| **Landmark** | 1-2 | Quest-relevant narrative sites, faction signals, or lore hooks |

### Travel

Travel between nodes is **NOT instant**. It costs real resources:

| Resource | Cost |
|----------|------|
| Time | Turns pass while traveling. Shelter production continues but player can't intervene. A universal tick system keeps time synced across overworld, dungeon, and shelter. |
| Food/Water | 1 each per day of travel |
| Danger | Random encounter roll per road segment |

Travel time creates **tension**: you can't just yo-yo between shelter and dungeon. Longer trips require more supplies and carry more risk.

### Encounter Chance

Each road segment has an encounter probability:

| Road Proximity | Encounter Chance |
|----------------|-----------------|
| Near shelter | 5% per segment |
| Mid-range | 15% per segment |
| Far | 25% per segment |

Encounters are simple at MVP:
- **Hostile**: 2-4 enemies, standard combat on a small generated tile map
- **Scavenge**: Abandoned supplies (small loot)
- **Nothing**: Movement continues

### Phase 2 Additions
- Faction territory zones (color-coded on map, affect encounters and dialogue)
- Patrols (faction units moving between nodes, can be ambushed or avoided)
- Rest stops (camp overnight to recover needs, but risk night encounters)
- Named NPC encounters on roads (traders, refugees, deserters)

### Phase 3 Additions
- Living world (factions wage wars, trade caravans move between nodes, territories shift)
- Dynamic events at nodes (node destroyed by faction war, new node created by settlers)
- Expeditions (send survivor teams to nodes while player dungeon-dives)

---

## Weather

### MVP: 8 Weather Types

Weather is rolled per overworld travel day. It affects travel and combat but NOT the shelter interior at MVP.

| Weather | Effect | Weight |
|---------|--------|--------|
| **Clear** | No effect (baseline) | 30% |
| **Overcast** | No mechanical effect. Atmospheric. | 20% |
| **Rain** | Visibility −1 tile, ranged accuracy −10% in encounters | 15% |
| **Heavy Rain** | Visibility −2 tiles, ranged accuracy −20%, travel speed −25% | 5% |
| **Fog** | Visibility −3 tiles, encounter detection range halved | 10% |
| **Dust Storm** | Visibility −2, ranged −15%, travel speed −25% | 5% |
| **Ashfall** | Atmospheric + sanity pressure (−1 per travel day) | 10% |
| **Anomaly Storm** | Travel blocked until storm passes (1-2 days). High sanity pressure. | 5% |

### Weather Roll
Weather is rolled using the overworld RNG seed: `weather_rng = derive_weather_rng(world_seed, day)`. This makes weather deterministic per seed but unpredictable to the player.

### Forecast
The player can see **current** weather. No forecast system at MVP (Phase 2: weather forecasting via Research Table upgrade).

### Phase 2 Weather Additions
- Temperature (cold/heat affect needs)
- Weather affects shelter exterior (storms damage walls)
- Weather patterns (multi-day weather fronts, not independent daily rolls)

---

## Overworld Map Generation

See [procgen.md](procgen.md) for detailed generation algorithms.

Quick summary:
- Nodes placed using seeded Poisson disk sampling in difficulty bands (easy near shelter, hard at edges)
- Roads via Delaunay triangulation with edge culling (minimum spanning tree + ~30% random edges)
- Dungeon type determined by node zone: Urban Decay near ruins, Underground in cave-marked tiles, Military near military landmarks
- Shelter always starts at map center

---

## Hell and Heaven Zones

> **Phase 3 content.** Documented here for vision only.

At Phase 3, the overworld includes corrupted zones:

| Zone | Source | Effect |
|------|--------|--------|
| **Hell Scar** | Demonic incursion sites (post-Sundering) | High sanity drain, demonic enemies, thaumic loot |
| **Celestial Ruin** | Angelic crash points | Radiation-like damage, angelic enemies, divine artifacts |

These are the endgame exploration areas. They're extremely dangerous and contain the most powerful (and most corrupting) equipment.

---

## MVP Overworld Loop

```
At shelter:
  └──→ Check weather
  └──→ Pack supplies (food/water for travel)
  └──→ Exit shelter → overworld node graph
       └──→ Choose destination node
       └──→ Walk along road (tile segments)
            ├── Encounter? → mini-combat or scavenge
            ├── Weather effects active
            └── Supplies consumed per day
       └──→ Arrive at destination
            ├── Dungeon? → Enter dungeon (AppState::Dungeon)
            ├── Ruins? → Scavenge event
            ├── Landmark? → Narrative event or lore hook
            └── Crossroads? → Choose next node
  └──→ Eventually return to shelter with loot
```

The overworld transforms resource management from abstract numbers into **felt experience**. Travel costs food and water. Weather adds randomness. Encounters add danger. The player shouldn't just teleport between dungeon and shelter — the journey IS part of the game.

See also: [colony.md](colony.md) for shelter base, [combat.md](combat.md) for encounter combat rules, [procgen.md](procgen.md) for map generation, [phase-roadmap.md](phase-roadmap.md) for Phase 2/3 overworld features.
