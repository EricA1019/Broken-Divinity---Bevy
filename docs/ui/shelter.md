# Shelter UI — Colony Management

All elements in this file are rendered during `AppState::AtShelter`. The shelter screen uses `apply_bunker_theme()` (steel accents, cooler tones).

For shared patterns (palette, typography, keybinds, sanity distortion), see [README.md](README.md).
For shelter mechanics, see [gameplay/colony.md](../gameplay/colony.md).

---

## Screen Layout

### Primary Wireframe

```
┌──────────────────────────────────────────────────────────────────────┐
│ 🏠 Shelter │ Day 14  │ Food: 45 (+3/d)  Scrap: 120 (-2/d)         │ ← Resource Bar
│ Survivors: 4/5       │ Meds: 8  Ammo: 30  Water: 22 (+1/d)         │
├───────────────────────────────────────┬──────────────────────────────┤
│                                       │ ┌── Management ───────────┐ │
│                                       │ │[1]Surv [2]Res [3]Stat   │ │
│         SHELTER TILEMAP               │ │[4]Build [5]Res [6]Work  │ │
│         (walkable compound)           │ │[7]Raid                  │ │
│                                       │ ├──────────────────────────┤ │
│    ┌─────┐  ┌─────┐  ┌─────┐        │ │                          │ │
│    │Quart│  │Work │  │Stor │        │ │  (Active tab content)    │ │
│    │  S  │  │shop │  │age  │        │ │                          │ │
│    └─────┘  └─────┘  └─────┘        │ │                          │ │
│         S = Survivor sprite          │ │                          │ │
│    ┌─────┐  ┌─────┐                 │ │                          │ │
│    │Cook │  │Gener│                 │ │                          │ │
│    │Stat │  │ator │                 │ │                          │ │
│    └─────┘  └─────┘                 │ │                          │ │
│                                       │ │                          │ │
│    ═══ Perimeter Wall ═══            │ └──────────────────────────┘ │
├───────────────────────────────────────┴──────────────────────────────┤
│ > A survivor complains about the food supply.                        │ ← Game Log
│ > Construction complete: Storage Room.                                │
│ > Day 14 production: +3 food, +1 water, -2 scrap (maintenance).     │
└──────────────────────────────────────────────────────────────────────┘
```

### Layout Rules

- **Resource bar**: `TopBottomPanel::top` — always visible, shows all 5 resources with deltas + survivor count + day counter
- **Shelter tilemap**: Left side (~55% width) — walkable view of the compound, same bevy_ecs_tilemap renderer as dungeons
- **Management window**: `SidePanel::right` (~45% width) — single egui window with tab bar
- **Game log**: `TopBottomPanel::bottom` — shelter-specific log (construction, production, survivor events)
- Toggle the management window with `s` key. Closing it gives full-width tilemap view.

---

## Resource Bar `[MVP]`

Top-docked strip. Always visible while in shelter.

### Elements

| Element | Display | Color Behavior | Phase |
|---------|---------|----------------|-------|
| Shelter label | `🏠 Shelter` | `STEEL` | MVP |
| Day counter | `Day 14` | `BONE` | MVP |
| Food | `Food: 45 (+3/d)` | Delta: `SUCCESS_GREEN` if positive, `ERROR_RED` if negative | MVP |
| Water | `Water: 22 (+1/d)` | Same delta coloring | MVP |
| Scrap | `Scrap: 120 (-2/d)` | Same delta coloring | MVP |
| Medicine | `Meds: 8` | `WARNING_AMBER` if < 5 | MVP |
| Ammo | `Ammo: 30` | `WARNING_AMBER` if < 10 | MVP |
| Survivor count | `Survivors: 4/5` | `WARNING_AMBER` if at capacity | MVP |
| Morale indicator | `Morale: Good` | Color-coded by level | Phase 2 |

### Resource Icons

Each resource displays with an icon from `ShelterHudIcons` texture set (with emoji fallback if textures not loaded):

| Resource | Icon | Fallback |
|----------|------|----------|
| Food | `food` texture | 🍖 |
| Water | `water` texture | 💧 |
| Scrap | `scrap` texture | ⚙️ |
| Medicine | `meds` texture | 💊 |
| Ammo | `ammo` texture | 🔫 |

---

## Shelter Tilemap `[MVP]`

Left side of the screen. Same rendering system as dungeons (bevy_ecs_tilemap).

### Tile Types

| Char | Meaning | Color |
|------|---------|-------|
| `#` | Perimeter wall | Steel grey |
| `.` | Interior floor | Dark concrete |
| `+` | Door | Rust |
| `S` | Station footprint | Highlighted floor |
| `C` | Construction site | Pulsing yellow |

### Overlays

| Overlay | Display | Phase |
|---------|---------|-------|
| Survivor sprites | 16×16 sprites at assigned positions with task icon above | MVP |
| Task indicator | Small icon above survivor: ⚒ (working), 🔨 (building), 💤 (resting), 🛡 (patrolling), ❓ (idle) | MVP |
| Room labels | Room name floats above room center | MVP |
| Damage indicators | Wall sections tinted red if damaged by raid | MVP |
| Construction highlights | Tiles under construction pulse `WARNING_AMBER` | MVP |

### Interaction

Players can walk the shelter with movement keys (same as dungeon). Interacting (`Space`) while adjacent to a station opens its detail in the management window tab.

---

## Management Window `[MVP]`

Single egui `Window` with a tab bar. Toggled with `s` key. Tabs switchable with `1`-`7` number keys.

### Tab Bar

```
┌─────────────────────────────────────────────────────────┐
│ [1]Survivors [2]Resources [3]Stations [4]Build          │
│ [5]Research  [6]Workbench [7]Raid Prep                  │
└─────────────────────────────────────────────────────────┘
```

Tab 7 (Raid Prep) only appears when a raid is imminent — hidden otherwise.

---

### Tab 1: Survivors `[MVP]`

```
┌── Survivors ─────────────────────────┐
│ ┌─ Marcus (Former Soldier) ────────┐ │
│ │ Task: Working @ Armory           │ │
│ │ HP: ██████████ 100/100           │ │
│ │ Hunger: ████████░░ 72/100       │ │
│ │ Thirst: ██████░░░░ 58/100       │ │
│ │ Rest:   █████████░ 85/100       │ │
│ │ [Reassign ▾] [View Details]      │ │
│ └──────────────────────────────────┘ │
│ ┌─ Elena (Former Medic) ──────────┐ │
│ │ Task: Working @ Medical Bay      │ │
│ │ HP: ██████████ 100/100           │ │
│ │ Hunger: ██████████ 94/100       │ │
│ │ Thirst: ████████░░ 80/100       │ │
│ │ Rest:   ███░░░░░░░ 28/100  ⚠   │ │
│ │ [Reassign ▾] [View Details]      │ │
│ └──────────────────────────────────┘ │
│ ┌─ (2 more survivors...) ─────────┐ │
│                                       │
│ Idle survivors: 0                     │
└───────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Survivor cards | Scrollable list, one card per survivor | MVP |
| Name + background trait | Header of each card, trait in parentheses | MVP |
| Current task | Station name or status (idle/resting/building) | MVP |
| HP bar | Full health bar | MVP |
| Needs bars (3) | Hunger, Thirst, Rest — each 0-100, `⚠` warning at 35 | MVP |
| Reassign dropdown | Dropdown menu: station list + Idle / Rest / Patrol / Build | MVP |
| View Details button | Opens expanded view with skill bonuses, history | MVP |
| Skills display | Survivor-specific skills affecting work efficiency | Phase 2 |
| Morale bar | Individual morale level | Phase 2 |
| Equipment slots | Assign gear from pool | Phase 2 |
| Relationship indicators | Friendships, rivalries | Phase 3 |

### Needs Warning Behavior

| Need Level | Display |
|------------|---------|
| > 50% | Normal color (`HUNGER_AMBER`, `THIRST_CYAN`, `SAN_BLUE`) |
| 35-50% | Bar turns brighter, `⚠` icon appears |
| < 35% | `ERROR_RED` bar, `⚠⚠` double warning |
| 0% | Critical — survivor begins taking HP damage. Card border turns `ERROR_RED`. |

---

### Tab 2: Resources `[MVP]`

```
┌── Resources ─────────────────────────┐
│ ── Stockpile ──────────────────────  │
│ 🍖 Food     45   +8 prod  -5 cons  = +3/day  │
│ 💧 Water    22   +4 prod  -3 cons  = +1/day  │
│ ⚙️ Scrap    120  +0 prod  -2 maint = -2/day  │
│ 💊 Medicine  8   +2 prod  -0 cons  = +2/day  │
│ 🔫 Ammo     30   +0 prod  -0 cons  =  0/day  │
│                                       │
│ ── Storage Capacity ───────────────  │
│ Food     45 / 100  ████░░░░░░       │
│ Water    22 / 80   ██░░░░░░░░       │
│ Scrap   120 / 200  ██████░░░░       │
│ Medicine  8 / 30   ██░░░░░░░░       │
│ Ammo     30 / 50   ██████░░░░       │
│                                       │
│ ── Alerts ─────────────────────────  │
│ ⚠ Food will run out in 15 days      │
│ ⚠ Scrap declining — no production   │
└───────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Resource rows | Icon, name, quantity, production, consumption, net delta | MVP |
| Delta coloring | Positive = `SUCCESS_GREEN`, negative = `ERROR_RED`, zero = `DIM_TEXT` | MVP |
| Storage capacity bars | Current / max with progress bar | MVP |
| Resource alerts | Predictive warnings (e.g., "will run out in X days") | MVP |
| Trade interface | Buy/sell with visiting traders | Phase 2 |
| Resource history graph | Trend line of resource levels over time | Phase 3 |

---

### Tab 3: Stations `[MVP]`

```
┌── Stations ──────────────────────────┐
│ ┌─ Cooking Station (T1) ──────────┐ │
│ │ Workers: 1/2          ⚡ Powered │ │
│ │ Output: 4 food/day per worker    │ │
│ │ Current: 4 food/day (1 worker)   │ │
│ │ [Staff +] [Staff -] [Upgrade ▾]  │ │
│ └──────────────────────────────────┘ │
│ ┌─ Armory (T1) ───────────────────┐ │
│ │ Workers: 1/1          ⚡ Powered │ │
│ │ Output: Ammo crafting            │ │
│ │ Current: 2 ammo/day             │ │
│ │ [Staff +] [Staff -] [Upgrade ▾]  │ │
│ └──────────────────────────────────┘ │
│ ┌─ Generator (T1) ────────────────┐ │
│ │ Workers: 1/1          ⚡ Active  │ │
│ │ Output: Powers 3 stations       │ │
│ │ Fuel: 2 scrap/day               │ │
│ │ [Staff +] [Staff -] [Upgrade ▾]  │ │
│ └──────────────────────────────────┘ │
│                                       │
│ (scroll for more stations...)         │
└───────────────────────────────────────┘
```

### T1 Stations (MVP)

| Station | Workers | Output |
|---------|---------|--------|
| Cooking Station | 1-2 | Food/day |
| Water Purifier | 1-2 | Water/day |
| Armory | 1 | Ammo/day |
| Medical Bay | 1 | Medicine/day + heal injured |
| Generator | 1 | Powers other stations |
| Quarters | — | Survivor capacity |
| Storage Room | — | Resource capacity |
| Workbench | 1 | Repair equipment |
| Research Table | 1 | Unlock tech |
| Guard Post | 1 | Raid defense bonus |

| Element | Detail | Phase |
|---------|--------|-------|
| Station cards | Name, tier, staffing, power status, output | MVP |
| Staff +/- buttons | Assign/remove a survivor (opens survivor picker) | MVP |
| Power indicator | `⚡ Powered` (green) or `⚡ No Power` (red) | MVP |
| Upgrade dropdown | Shows available upgrades with cost/time | MVP |
| T2 station cards | Upgraded versions with improved output | Phase 2 |
| T3 station cards | End-game stations | Phase 3 |
| Station efficiency modifiers | Skill bonuses, morale effects on output | Phase 2 |

---

### Tab 4: Construction `[MVP]`

```
┌── Construction Queue ────────────────┐
│ ┌─ New Room: Quarters ────────────┐  │
│ │ Cost: 30 scrap                   │  │
│ │ Workers: 2 assigned              │  │
│ │ Progress: ████████░░ 80/120 turns│  │
│ │ ETA: ~20 turns                   │  │
│ │ [+Worker] [-Worker] [Cancel]     │  │
│ └──────────────────────────────────┘  │
│                                       │
│ ── Available Projects ──────────────  │
│ ▸ Storage Room     (40 scrap)        │
│ ▸ Guard Post       (25 scrap)        │
│ ▸ Expand Quarters  (50 scrap)        │
│ ▸ Reinforce Wall   (20 scrap)        │
│                                       │
│ [Start Selected]                      │
└───────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Active construction card | Project name, cost, workers, progress bar, ETA | MVP |
| Worker +/- buttons | More workers = faster, but pulled from other duties | MVP |
| Cancel button | Refunds partial resources | MVP |
| Available projects list | Selectable, shows cost. Greyed if insufficient resources. | MVP |
| Start button | Begins construction, deducts resources | MVP |
| Multi-queue | Queue multiple projects (execute sequentially) | Phase 2 |
| Blueprint unlocks | New projects available from research | Phase 2 |

---

### Tab 5: Research `[MVP]`

```
┌── Research ──────────────────────────┐
│ ── Current Research ───────────────  │
│ ▸ Water Purification (T1)           │
│   Cost: 15 scrap                     │
│   Progress: ██████░░░░ 60/100       │
│   Researcher: Elena                  │
│                                       │
│ ── Available ──────────────────────  │
│ ▸ Improved Cooking (T1)    10 scrap │
│ ▸ Scrap Recycling (T1)     20 scrap │
│ ▸ Basic Defenses (T1)      15 scrap │
│                                       │
│ ── Locked ─────────────────────────  │
│ 🔒 Advanced Medicine (T2)           │
│    Requires: Medical Bay T2          │
│ 🔒 Power Grid (T2)                  │
│    Requires: Generator T2            │
│                                       │
│ [Start Research]                      │
└───────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Current research card | Name, tier, cost, progress bar, assigned researcher | MVP |
| Available research list | Selectable — meets prerequisites, has resources | MVP |
| Locked research list | Shows prerequisites that aren't met | MVP |
| Start button | Begins research, assigns free researcher | MVP |
| Tech tree visualization | Graph view of tech dependencies | Phase 2 |
| T3 research | End-game technologies | Phase 3 |

---

### Tab 6: Workbench `[MVP]`

```
┌── Workbench ─────────────────────────┐
│ ── Repair ─────────────────────────  │
│ Select item to repair:               │
│ ▸ Salvaged Rifle    85% → 100%      │
│   Cost: 5 scrap │ Time: 3 turns     │
│ ▸ Scrap Vest        Broken → 100%   │
│   Cost: 12 scrap │ Time: 8 turns    │
│                                       │
│ [Repair Selected]                     │
│                                       │
│ ── Crafting ─────────────── [Ph.2]  │
│ 🔒 Requires: Crafting research       │
└───────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Repair item list | Equipment with current durability, cost, time | MVP |
| Durability display | Percentage or bar, "Broken" for 0% | MVP |
| Repair button | Starts repair, deducts scrap | MVP |
| Crafting recipes | Create new items from materials | Phase 2 |
| Blueprint system | Unlock recipes through research/scavenging | Phase 2 |

---

### Tab 7: Raid Prep `[Contextual — MVP]`

Only visible when a raid is imminent. Appears as the rightmost tab with a pulsing `ERROR_RED` indicator.

```
┌── ⚠ RAID INCOMING ──────────────────┐
│                                       │
│ ── Threat Assessment ──────────────  │
│ Raiders: ~8-12  Faction: Puritan     │
│ Estimated strength: MODERATE          │
│ Approach: South perimeter             │
│                                       │
│ ── Assign Survivors ───────────────  │
│ Marcus     [Defend ▾]  ← Wall duty  │
│ Elena      [Support ▾] ← Medical    │
│ Jonas      [Hold Gate ▾]             │
│ Kai        [Flee ▾]    ← Evacuate   │
│                                       │
│ ── Resources Available ────────────  │
│ Ammo: 30 (enough for ~15 shots)      │
│ Meds: 8  (enough for ~4 treatments)  │
│ Wall HP: ████████░░ 80%              │
│                                       │
│ [▸ DEFEND]  [✕ Abandon Shelter]      │
└───────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Threat assessment | Raider count range, faction, strength rating, direction | MVP |
| Survivor assignment | Dropdown per survivor: Flee / Defend / Support / Hold Gate | MVP |
| Resource summary | Ammo count + shot estimate, medicine + treatment estimate | MVP |
| Wall HP | Current wall durability bar | MVP |
| Defend button | Confirms and begins raid combat | MVP |
| Abandon option | Evacuate — lose shelter, flee with survivors + portable resources | MVP |
| Siege UI | Extended multi-wave raid with preparation phases | Phase 2 |

---

## Post-Raid Report `[MVP]`

Modal overlay after raid resolves. Must be dismissed before resuming shelter mode.

```
┌── Raid Report ───────────────────────┐
│                                       │
│ RAID REPELLED                         │
│                                       │
│ ── Casualties ─────────────────────  │
│ Marcus: Wounded (15 HP remaining)    │
│ Jonas:  KIA                           │
│                                       │
│ ── Damage ─────────────────────────  │
│ South Wall: 80% → 45%                │
│ Cooking Station: Damaged (offline)   │
│                                       │
│ ── Losses ─────────────────────────  │
│ Ammo spent: 18                        │
│ Medicine used: 3                      │
│ Food stolen: 0                        │
│                                       │
│ ── Salvage ────────────────────────  │
│ Scrap recovered from raiders: 15     │
│                                       │
│           [Acknowledge]               │
└───────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Outcome header | "RAID REPELLED" or "SHELTER OVERRUN" in appropriate color | MVP |
| Casualty list | Survivor name + status (Wounded/KIA) | MVP |
| Damage list | Wall/station damage with before→after | MVP |
| Resource losses | Ammo, medicine, stolen food/scrap | MVP |
| Salvage | Resources gained from defeated raiders | MVP |
| Faction reputation change | Rep delta with raiding faction | Phase 2 |
| Morale impact | Morale change summary | Phase 2 |

---

## Shelter Interaction Flow

```
Enter AtShelter
    │
    ├── Walk around with movement keys (tilemap view)
    │
    ├── Press [s] → Management window opens
    │   ├── Press [1]-[7] → Switch tabs
    │   ├── Interact with tab content (assign, build, research, repair)
    │   └── Press [Esc] or [s] → Close management window
    │
    ├── Press [Space] adjacent to station → Opens relevant station in management window
    │
    ├── Raid imminent alert → Tab 7 (Raid Prep) appears with pulsing indicator
    │   ├── Assign survivors, review resources
    │   ├── Press [Defend] → Raid combat begins
    │   └── Raid resolves → Post-raid report modal
    │
    └── Walk to shelter exit (stairs/gate) → Press [>] → Transition to Overworld
```

---

## Phase Summary

### MVP
- Resource bar (5 resources + deltas + survivor count + day)
- Shelter tilemap (walkable, rooms, survivor sprites with task indicators)
- Tabbed management window with tabs 1-6 (Survivors, Resources, Stations, Construction, Research, Workbench)
- Tab 7: Raid Prep (contextual)
- Post-raid report modal
- Survivor cards (name, trait, task, HP, needs bars with warnings)
- Resource dashboard (stockpile, production/consumption, storage capacity, predictive alerts)
- Station management (all 10 T1 stations, staffing, power, output)
- Construction queue (projects, progress, worker assignment)
- Research (current + available + locked, tier prerequisites)
- Workbench repair (item list, durability, cost/time)
- Game log (shelter events)

### Phase 2 Additions
- Morale indicator (resource bar + individual survivor)
- Survivor skills, equipment slots, morale bar
- Station efficiency modifiers
- T2 station upgrades
- Multi-queue construction
- Blueprint unlocks
- Tech tree visualization
- Crafting at workbench
- Siege preparation UI
- Faction reputation changes in raid report
- Trade interface in Resources tab

### Phase 3 Additions
- Survivor relationships (friendships, rivalries, romance)
- Survivor personality traits + mood indicators
- Mental break states
- T3 station research/upgrades
- Expedition dispatch UI
- Resource history graphs
