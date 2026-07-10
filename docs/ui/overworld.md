# Overworld UI — World Map & Travel

All elements in this file are rendered during `AppState::Overworld`. Uses `apply_theme()` (gold accents, dungeon context).

For shared patterns (palette, typography, keybinds), see [README.md](README.md).
For overworld mechanics, see [gameplay/overworld.md](../gameplay/overworld.md).

---

## Screen Layout

### Primary Wireframe (Node Selection)

```
┌──────────────────────────────────────────────────────────────────────┐
│ ☀ Clear │ Day 14 │ Location: Forest Crossroads │ Food: 12 Water: 8  │ ← Status Bar
├───────────────────────────────────────────────────┬──────────────────┤
│                                                   │                  │
│                    ■ Ruins of Malkov              │ ── Destination ──│
│                   / \                             │                  │
│                  /   \                            │ Shattered Labs   │
│     ▲ Shelter  /     ● Crossroads                │ Type: Dungeon    │
│      \        /       |                          │ Theme: Medical   │
│       \      /        |                          │ Difficulty: ██░  │
│        \    /         |                          │ Distance: 3 days │
│         \  /          |                          │ Cost: -3 food    │
│          ●───────────[★] Shattered Labs          │       -3 water   │
│                                                   │                  │
│     ▲ = Shelter  ● = Crossroads                  │ Weather on route:│
│     ■ = Ruins    ★ = Dungeon                     │ Rain (day 2)     │
│     ◆ = Landmark ? = Undiscovered                │ -1 visibility    │
│                                                   │                  │
│     [●] = Current location (highlighted)          │ [Enter] Travel   │
│                                                   │ [Esc] Cancel     │
├───────────────────────────────────────────────────┴──────────────────┤
│ > You arrive at Forest Crossroads. The road forks east and north.    │
│ > Weather forecast: Rain expected tomorrow.                          │
└──────────────────────────────────────────────────────────────────────┘
```

### Layout Rules

- **Status bar**: `TopBottomPanel::top` — weather, day, location name, portable resources (food/water only)
- **World map**: Left side (~65% width) — node graph with roads
- **Node info panel**: `SidePanel::right` (~35% width) — details of selected/hovered node
- **Game log**: `TopBottomPanel::bottom` — travel events, arrivals, weather changes
- Arrow keys / vi-keys navigate between connected nodes on the map. `Space`/`Enter` confirms travel.

---

## Status Bar `[MVP]`

Top-docked strip. Always visible in overworld.

| Element | Display | Phase |
|---------|---------|-------|
| Weather icon + name | `☀ Clear` / `🌧 Rain` / `🌫 Fog` / `💨 Dust Storm` / `🌋 Ashfall` / `⚡ Storm` | MVP |
| Day counter | `Day 14` | MVP |
| Current location | `Location: Forest Crossroads` | MVP |
| Portable food | `Food: 12` — `WARNING_AMBER` if < 5 | MVP |
| Portable water | `Water: 8` — `WARNING_AMBER` if < 5 | MVP |
| Player HP | `HP: 74/100` (compact) | MVP |
| Sanity | `SAN: 85` (compact, no bar) | MVP |

---

## World Map `[MVP]`

The map renders nodes and roads as a graph, NOT a tile grid. Each node is a styled icon at a fixed position, with lines connecting adjacent nodes.

### Node Type Display

| Type | Icon | Color | Phase |
|------|------|-------|-------|
| Shelter | `▲` | `SUCCESS_GREEN` | MVP |
| Dungeon | `★` | `ERROR_RED` | MVP |
| Ruins | `■` | `AMBER` | MVP |
| Crossroads | `●` | `BONE` | MVP |
| Landmark | `◆` | `GOLD` | MVP |
| Undiscovered | `?` | `DIM_TEXT` | MVP |
| Hell Zone | `★` with red glow | Crimson | Phase 3 |
| Heaven Zone | `★` with gold glow | Cold gold | Phase 3 |

### Current Location

Current node is highlighted with a brighter color and a pulsing ring animation.

### Road Rendering

Roads drawn as lines between connected nodes:

| Road State | Line Style | Phase |
|------------|-----------|-------|
| Traversable | Solid line, `DIM_TEXT` | MVP |
| Selected route | Solid line, `GOLD` | MVP |
| Dangerous (difficulty) | Dashed line, `WARNING_AMBER` | MVP |
| Blocked | Dotted line, `ERROR_RED` | Phase 2 |
| Faction-controlled | Colored by faction | Phase 2 |

### Fog of War

| Discovery State | Display |
|----------------|---------|
| Visited | Full node icon + name label |
| Scouted (adjacent to visited) | Node icon visible, name visible, details available |
| Undiscovered | `?` icon, no name, greyed road if connecting to known node |

### Difficulty Banding

Nodes farther from shelter are harder. Visual cues:

| Band | Map Region | Tint |
|------|-----------|------|
| Easy | Near shelter | Normal colors |
| Medium | Mid-distance | Slightly desaturated |
| Hard | Distant | Darker, red-tinted roads |

---

## Node Info Panel `[MVP]`

Right-side panel. Shows details of the currently selected (cursor-highlighted) node.

### Dungeon Node

```
┌── Shattered Labs ────────────────┐
│ Type: Dungeon                     │
│ Theme: Medical                    │
│ Difficulty: ██░░░ Medium          │
│ Floors: 3                         │
│                                   │
│ Known hostiles: Feral Angels,     │
│   Corrupted Medics                │
│                                   │
│ ── Travel Cost ────────────────  │
│ Distance: 3 days                  │
│ Food: -3  Water: -3              │
│ Current supply: ✓ Sufficient     │
│                                   │
│ ── Route Weather ──────────────  │
│ Day 1: Clear                      │
│ Day 2: Rain (-1 visibility)      │
│ Day 3: Clear                      │
│                                   │
│ [Enter] Begin Travel              │
│ [Esc] Back                        │
└───────────────────────────────────┘
```

### Ruins Node

```
┌── Abandoned Depot ───────────────┐
│ Type: Ruins                       │
│ Scavenge quality: Medium          │
│ Danger level: Low                 │
│                                   │
│ ── Travel Cost ────────────────  │
│ Distance: 1 day                   │
│ Food: -1  Water: -1              │
│ Current supply: ✓ Sufficient     │
│                                   │
│ [Enter] Travel & Scavenge         │
│ [Esc] Back                        │
└───────────────────────────────────┘
```

### Info Panel Elements

| Element | Detail | Phase |
|---------|--------|-------|
| Node name | Large heading in `GOLD` | MVP |
| Node type | Dungeon / Ruins / Crossroads / Landmark | MVP |
| Theme (dungeons) | Medical, Urban Decay, Military, etc. | MVP |
| Difficulty rating | Bar + label (Easy/Medium/Hard/Extreme) | MVP |
| Floor count (dungeons) | Number of dungeon levels | MVP |
| Known hostiles | Enemy types previously encountered here (empty on first visit) | MVP |
| Travel cost | Days + food/water consumption | MVP |
| Supply check | `✓ Sufficient` in `SUCCESS_GREEN` or `✗ Insufficient` in `ERROR_RED` | MVP |
| Route weather | Day-by-day weather forecast along the route | MVP |
| Travel button | `[Enter] Begin Travel` | MVP |
| Faction presence | Which faction controls this area | Phase 2 |
| Quest marker | Active quest associated with this node | Phase 2 |
| Corruption level | Hell/Heaven zone intensity | Phase 3 |

---

## Weather System `[MVP]`

### Weather Types & Display

| Weather | Icon | Visibility Effect | Accuracy Effect | Sanity Effect | Phase |
|---------|------|-------------------|-----------------|---------------|-------|
| Clear | ☀ | None | None | None | MVP |
| Overcast | ☁ | None | None | None | MVP |
| Rain | 🌧 | -1 tile | -10% ranged | None | MVP |
| Heavy Rain | 🌧🌧 | -2 tiles | -20% ranged | None | MVP |
| Fog | 🌫 | -3 tiles | -15% ranged | None | MVP |
| Dust Storm | 💨 | -2 tiles | -20% ranged | -1/day | MVP |
| Ashfall | 🌋 | -1 tile | -10% ranged | -2/day | MVP |
| Anomaly Storm | ⚡ | -4 tiles | -30% ranged | -5/day | Phase 2 |

### Weather Panel (`w` key)

Expanded weather details:

```
┌── Weather ───────────────────────┐
│ Current: 🌧 Rain                 │
│                                   │
│ Effects:                          │
│   Visibility: -1 tile range      │
│   Ranged accuracy: -10%          │
│   Travel speed: ×1.2 (slower)    │
│   Sanity impact: None            │
│                                   │
│ Regional forecast:                │
│   Tomorrow: Heavy Rain            │
│   Day after: Clearing             │
│                                   │  ← Phase 2 expanded forecast
│ [Esc] Close                       │
└───────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Current weather | Icon + name + active effects | MVP |
| Effect list | Visibility, accuracy, travel speed, sanity penalties | MVP |
| Regional forecast | Multi-day prediction | Phase 2 |
| Weather history | Past 5 days of weather | Phase 3 |

---

## Travel Mode `[MVP]`

When the player confirms travel to a destination node, the overworld enters travel mode.

### Travel Wireframe

```
┌──────────────────────────────────────────────────────────────────────┐
│ 🌧 Rain │ Day 15 │ Traveling to: Shattered Labs │ Food: 11 Water: 7 │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                     Forest Crossroads ──── Shattered Labs            │
│                           ●═══════●═══════★                          │
│                           ✓       ▸       ○                          │
│                         Day 1   Day 2   Day 3                        │
│                        (done)  (now)   (remaining)                   │
│                                                                      │
│                     Travel Progress: ██████░░░░ 2 / 3 days          │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│ > Day 15: You continue east through the rain.                        │
│ > The road is muddy. Travel is slower than expected.                 │
│ > You consume 1 food and 1 water.                                    │
└──────────────────────────────────────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Travel header | Destination name, remainging distance | MVP |
| Route visualization | Source → destination with segment markers | MVP |
| Segment status | Done (✓), Current (▸), Remaining (○) | MVP |
| Progress bar | Days completed / total days | MVP |
| Daily consumption log | Food/water consumed per day | MVP |
| Continue/rest choice | End of each day: continue or camp (rest recovers HP, costs extra food) | MVP |

---

## Encounters `[MVP]`

During travel, encounters trigger on road segments. The encounter UI temporarily replaces the travel view.

### Encounter Trigger

```
┌──────────────────────────────────────────────────────────────────────┐
│                        ⚠ ENCOUNTER ⚠                                 │
│                                                                      │
│  Three figures emerge from the treeline ahead.                       │
│  They look hostile.                                                  │
│                                                                      │
│  [F]ight  [R]un  [N]egotiate                                        │
│                                                                      │
│  Running costs 1 extra food and may fail.                            │
│  Negotiation uses Persuasion skill (35).                             │
└──────────────────────────────────────────────────────────────────────┘
```

### Encounter Types & Resolution

| Type | Display | Resolution | Phase |
|------|---------|------------|-------|
| Hostile | Red alert banner, enemy count/type | Generates mini-dungeon combat map | MVP |
| Scavenge | Amber banner, location description | Loot roll + display | MVP |
| Nothing | Brief text ("The road is quiet.") | Auto-dismiss after 1 turn | MVP |
| Trader | Blue banner, trader inventory | Trade interface | Phase 2 |
| Faction patrol | Faction-colored banner | Diplomacy check or combat | Phase 2 |
| Quest event | Gold banner, narrative text | Branching dialogue choices | Phase 2 |

### Scavenge Result

```
┌── Scavenge Results ──────────────┐
│                                   │
│ You search the abandoned truck.   │
│                                   │
│ Found:                            │
│   ⚙️ Scrap: 15                   │
│   💊 Medicine: 2                  │
│   🔫 Ammo: 8                     │
│                                   │
│ [Space] Continue                  │
└───────────────────────────────────┘
```

### Combat Encounter

When a hostile encounter triggers combat, a small tile map is generated for the fight. This uses the **same InGame combat UI** (vitals bar, targeting, game log) but on a smaller map (~20×15). After combat resolves, the player returns to travel mode.

| Element | Detail | Phase |
|---------|--------|-------|
| Encounter type banner | Color-coded by type, brief description | MVP |
| Choice buttons | Fight / Run / Negotiate (where applicable) | MVP |
| Skill check display | Relevant skill shown for non-combat options | MVP |
| Mini-map combat | Small generated map, full combat rules | MVP |
| Scavenge result panel | Itemized loot with icons | MVP |
| Retreat option | Flee mid-combat at a cost | MVP |

---

## Arrival `[MVP]`

When the player reaches a destination node:

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│                   You arrive at Shattered Labs.                      │
│                                                                      │
│         An abandoned medical complex. The lights still flicker.      │
│                                                                      │
│              [E]nter Dungeon    [C]amp (rest + save)                 │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

| Node Type | Options |
|-----------|---------|
| Dungeon | Enter Dungeon / Camp / Return to Map |
| Ruins | Scavenge / Camp / Return to Map |
| Crossroads | Choose next road / Camp / Return to Map |
| Landmark | Interact (quest/event) / Camp / Return to Map |
| Shelter | Enter Shelter (transitions to `AtShelter`) |

---

## Phase Summary

### MVP
- Status bar (weather, day, location, portable resources, HP, sanity)
- World node map (5 node types, roads, fog of war, difficulty banding)
- Node info panel (type, theme, difficulty, floors, travel cost, supply check, route weather)
- Weather display (8 types with combat/travel effects)
- Travel mode (progress bar, segment markers, daily consumption)
- Encounters (hostile combat / scavenge / nothing, choice buttons)
- Arrival screen (node-type-specific options)
- Game log (travel events)

### Phase 2 Additions
- Faction territory color-coding on roads/nodes
- Weather forecast (multi-day prediction)
- Anomaly Storm weather type
- Trader encounters with trade UI
- Faction patrol encounters with diplomacy
- Quest event encounters with branching dialogue
- Blocked roads
- Faction presence in node info
- Quest markers on nodes

### Phase 3 Additions
- Hell/Heaven zone markers (crimson/gold glowing nodes)
- Corruption level display for corrupted zones
- Weather history
- Expedition tracking (dispatched survivor teams)
- Advanced route planning (multi-stop itinerary)
