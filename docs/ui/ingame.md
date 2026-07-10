# InGame UI — Dungeon Exploration & Combat

All elements in this file are rendered during `AppState::InGame`. Systems are gated to `InGame` and relevant `TurnState` sub-states.

For shared patterns (palette, typography, keybinds, sanity distortion), see [README.md](README.md).

---

## Screen Layout

### Primary Wireframe (AwaitingInput)

```
┌─────────────────────────────────────────────────────────────────────┐
│ HP ████████░░ 74/100  SAN ██████████ 85/100  AP: 2/2  🔫 12/60    │ ← Vitals Bar
│ [Wounded 3t] [Stunned 1t]                          Floor 2 / 4    │ ← Status + Depth
├──────────────────────────────────────────────────┬──────────────────┤
│                                                  │                 │
│                                                  │  (Panel area)   │
│                                                  │                 │
│                                                  │  Inventory,     │
│              MAP VIEWPORT                        │  Character,     │
│              (tiles + sprites)                   │  Abilities      │
│                 @                                │  etc.           │
│              Fog of war at edges                 │                 │
│                                                  │  Opens on       │
│                                                  │  toggle key.    │
│                                                  │  ~40% width     │
│                                                  │  max.           │
│                                                  │                 │
├──────────────────────────────────────────────────┴──────────────────┤
│ > You descend into the medical wing. The air smells of antiseptic. │ ← Game Log
│ > The feral angel spots you!                                       │   (4-6 lines)
│ > You hit the feral angel for 18 damage. (Wound applied)           │
│ > Press [m] for full log                                           │
└─────────────────────────────────────────────────────────────────────┘
```

### Layout Rules

- **Vitals bar**: `TopBottomPanel::top` — always visible, never hidden by panels
- **Game log**: `TopBottomPanel::bottom` — 4-6 lines visible, scrollable with `m` for full view
- **Map viewport**: Center — fills all remaining space
- **Side panel area**: `SidePanel::right` — appears only when a panel is toggled open, max ~40% screen width
- **No panel occludes the vitals bar or game log** — they dock at edges, panels fill the middle-right

---

## Vitals Bar `[MVP]`

Docked to top edge. Single horizontal strip. Always visible.

### Elements (left to right)

| Element | Display | Threshold Behavior | Phase |
|---------|---------|--------------------|-------|
| HP bar | `HP ████░░ 74/100` | Normal → Warning (50%) → Critical pulsing (25%) | MVP |
| Sanity bar | `SAN ██████ 85/100` | Normal → Desaturated (50%) → Flickering (25%) | MVP |
| Action Points | `AP: 2/2` | Bright when full, dim when spent | MVP |
| Ammo | `🔫 12/60` (clip/total) | `WARNING_AMBER` when clip empty, `ERROR_RED` when total = 0 | MVP |
| Active status effects | `[Wounded 3t] [Stunned 1t]` | Color-coded badges with remaining duration | MVP |
| Armor status | `[Armor: Intact]` or `[Armor: Broken]` | `BONE` when intact, `ERROR_RED` when broken | MVP |
| Floor depth | `Floor 2 / 4` | Right-aligned | MVP |

### Status Effect Badges

| Effect | Color | Icon/Text | Phase |
|--------|-------|-----------|-------|
| Wounded | `ERROR_RED` | `[Wound ×2 3t]` (stacks + duration) | MVP |
| Stunned | `WARNING_AMBER` | `[Stunned 1t]` | MVP |
| Bleeding | `ERROR_RED` | `[Bleed 4t]` | Phase 2 |
| Burning | Orange `#e08040` | `[Burn 3t]` | Phase 2 |
| Frightened | `SAN_BLUE` | `[Fear 2t]` | Phase 2 |
| Corrupted | Purple `#8a5ac4` | `[Corrupt 5t]` | Phase 2 |

---

## Game Log `[MVP]`

Docked to bottom edge. Shows last 4-6 messages. Full log available via `m` key.

### Message Priority & Styling

| Priority | Examples | Styling |
|----------|----------|---------|
| **Critical** | "You are dying!" / "Armor destroyed!" | `ERROR_RED`, bold. Optional map flash. |
| **Combat** | "You hit Raider for 12 damage." | `BONE`, normal weight |
| **Background** | "You hear distant footsteps." | `DIM_TEXT` |
| **Tutorial** | "Press [f] to fire at enemies." | `GOLD`, extended display, then dims |
| **Stealth** | "You strike from behind! (stealth crit)" | `SUCCESS_GREEN` |

### Deduplication

Consecutive identical messages compress: `"The cultist misses. (×3)"`

### Separate Channels

- **Narrative log**: Prose style — "You descend into the ruins."
- **Combat results**: Structured — roll numbers, damage, status applied
- These render in the same log strip but with different text styling. In the expanded `m` view, they can optionally be filtered by channel.

---

## Map Viewport `[MVP]`

Center of screen, fills remaining space between vitals bar (top), game log (bottom), and optional side panel (right).

### Tile Rendering

Three Z-layers via bevy_ecs_tilemap + Bevy sprites:

| Layer | Z | Content |
|-------|---|---------|
| Tilemap | 0 | Floor, wall, door, stairs (ASCII glyphs → tile sprites) |
| Items | 1 | Weapons, consumables, armor on the ground (16×16 sprites) |
| Entities | 2-3 | Enemies (z=2), player (z=3) — always on top |

### FOV States

| Visibility | Rendering |
|------------|-----------|
| **Visible** | Full color, entity sprites shown |
| **Revealed** (seen before, not currently visible) | Desaturated grey, no entity sprites |
| **Hidden** (never seen) | Black / not rendered |

### Camera

- Default zoom: ~40×30 tiles visible
- Zoom in: ~20×15 tiles (detail, sprites clearer)
- Zoom out: ~60×45 tiles (tactical overview)
- Smooth lerp follow on player
- Scroll wheel or `+`/`-` to zoom

### ASCII Glyph Table

| Char | Meaning | Color Direction |
|------|---------|-----------------|
| `.` | Floor | Theme floor tint |
| `#` | Wall | Theme wall tint |
| `+` | Closed door | Rust `#6a4a3a` |
| `-` / `\|` | Open door | Slightly lighter rust |
| `<` | Stairs up | `GOLD` |
| `>` | Stairs down | `GOLD` |
| `~` | Liquid/water | Blue `#3a5a8a` |
| `^` | Trap (visible) | `WARNING_AMBER` |
| `*` | Targeting cursor | Pulsing `GOLD` |

---

## Targeting Mode `[MVP]`

Activated by `f` (fire) or abilities that require target selection.

### Wireframe (Targeting Active)

```
┌─────────────────────────────────────────────────────────────────────┐
│ HP ████████░░ 74/100  SAN ██████████ 85/100  AP: 2/2  🔫 12/60    │
│ TARGETING: Shoot │ Range: 8 │ Tab: next │ Esc: cancel              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│                       . . . . . . . .                               │
│                     . . . .[R]. . . .    R = Raider (cover: Half)   │
│                     . . . . . . . . .        To-hit: 55%           │
│                     . . . .[@]. . . .        Damage: 8-15          │
│                     . . . . . . . . .                               │
│                       . . . . . . .     [*] = targeting cursor      │
│                                         [ ] = range ring            │
├─────────────────────────────────────────────────────────────────────┤
│ > Targeting Raider. Cover: Half (-20%). Base hit: 75%. Final: 55%. │
└─────────────────────────────────────────────────────────────────────┘
```

### Targeting Elements

| Element | Display | Phase |
|---------|---------|-------|
| Targeting header | Replaces status line: ability name, range, controls | MVP |
| Valid target highlights | Colored overlay on targetable enemy tiles | MVP |
| Range indicator | Faded ring at max range distance | MVP |
| Targeting cursor `*` | Pulsing `GOLD` on selected target tile | MVP |
| Cover indicator | `None` / `Half (-20%)` / `Full (-40%)` next to target | MVP |
| To-hit preview | Calculated hit chance shown before confirming | MVP |
| Damage range | Min-max damage preview | MVP |
| Tab cycling | Cycles through valid targets in distance order | MVP |
| Esc cancel | Returns to `AwaitingInput` without spending a turn | MVP |

---

## Enemy Display `[MVP]`

### On-Map Indicators

| Element | Display | Phase |
|---------|---------|-------|
| Enemy sprite | 16×16 sprite at tile position (z=2) | MVP |
| Facing indicator | Small chevron overlaid showing FOV direction | Phase 2 |
| Alert: Unaware | No indicator | MVP |
| Alert: Suspicious | Yellow `?` floating above sprite | MVP |
| Alert: Alert | Red `!` floating above sprite | MVP |

### Enemy Info (on hover/examine)

When the player uses `x` (examine) or hovers over a visible enemy:

```
┌─────────────────────┐
│ Feral Angel          │
│ HP ██████░░░ 62/100  │
│ Alert: Suspicious    │
│ Armor: Intact        │
│ [Wounded] [Stunned]  │
│ Melee / Aggressive   │
└─────────────────────┘
```

Shown as a tooltip `Area` anchored near the examined tile. Dismisses on next action or `Esc`.

| Info Field | Phase |
|------------|-------|
| Name | MVP |
| HP bar | MVP |
| Alert state | MVP |
| Armor status | MVP |
| Active status effects | MVP |
| Behavior tag (Melee/Ranged) | MVP |
| Damage type | Phase 2 |
| Resistances | Phase 2 |

---

## Toggleable Panels

All panels open in the right `SidePanel` area (~40% screen width). Opening one closes any other open panel. `Esc` closes the current panel.

### Inventory (`i` key) `[MVP]`

Grid-based layout with item sprites in cells.

```
┌── Inventory (6/20) ──────────────┐
│ ┌────┬────┬────┬────┬────┐       │
│ │ 🗡 │ 🔫 │ 💊 │ 🛡 │    │       │
│ ├────┼────┼────┼────┼────┤       │
│ │ 📦 │ 🔑 │    │    │    │       │
│ ├────┼────┼────┼────┼────┤       │
│ │    │    │    │    │    │       │
│ └────┴────┴────┴────┴────┘       │
│                                   │
│ ▸ Salvaged Rifle                 │
│   Damage: 8-15  Range: 8        │
│   Ammo: 12/30   Condition: 85%  │
│   [U]se  [D]rop  [E]quip        │
└───────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Grid cells | 5 columns, rows scale with capacity. 16×16 item sprites. | MVP |
| Capacity header | `Inventory (6/20)` — inline count | MVP |
| Item detail panel | Below grid — name, stats, actions when an item is selected | MVP |
| Item actions | `[U]se`, `[D]rop`, `[E]quip` — single-key from within inventory | MVP |
| Item categories / tabs | Filter by type (Weapons, Armor, Consumables, Misc) | Phase 2 |
| Drag-and-drop equip | Drag from grid to equipment slot | Phase 2 |
| Item comparison | Side-by-side when hovering equippable vs. current | Phase 2 |

### Character Sheet (`@` key) `[MVP]`

```
┌── Character ─────────────────────┐
│ Name: Brother Marcus              │
│ Level: 3  XP: 450/600            │
│                                   │
│ ── Skills ──────────────────────  │
│ Melee      45  ████████░░        │
│ Ranged     38  ███████░░░        │
│ Medical    52  █████████░        │
│ Stealth    25  █████░░░░░        │
│ Scavenging 41  ████████░░        │
│ Mechanical 30  ██████░░░░        │
│ Tech       20  ████░░░░░░        │
│ Persuasion 35  ███████░░░        │
│                                   │
│ ── Perks ────────────────────   │
│ ▸ Steady Hands (+5 Ranged)      │
│ ▸ Field Medic (First Aid +1 HP) │
│                                   │
│ ── Equipment ───────────────    │
│ Weapon: Salvaged Rifle           │
│ Armor:  Scrap Vest [Intact]      │
│ Trinket: (empty)                  │
└───────────────────────────────────┘
```

| Section | Detail | Phase |
|---------|--------|-------|
| Name, level, XP bar | Current progression | MVP |
| 8 skills with values + bars | Clickable for detail tooltip | MVP |
| Perk list | Acquired perks with effects | MVP |
| Equipment slots | Weapon, Armor, Trinket (Trinket in Phase 2) | MVP |
| Perk tree browser | Visual tree with locked/available/owned | Phase 2 |
| Faction reputation | Bars per known faction | Phase 2 |
| Long-term erosion indicator | Personality shift tracking | Phase 2 |

### Ability Picker (`a` key) `[MVP]`

Compact list of available abilities with AP cost and status.

```
┌── Abilities ─────────────────────┐
│ [1] Attack     1 AP  ✓ Ready     │
│ [2] Shoot      1 AP  ✓ Ready     │
│ [3] First Aid  1 AP  2 Med left  │
│ [4] Sprint     1 AP  ✓ Ready     │
│                                   │
│ [5] Aimed Shot 2 AP  ✓ Ready     │  ← Phase 2
│ Press number to select            │
└───────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Ability list | Name, AP cost, ready/cooldown/resource status | MVP |
| Number keys to select | `1`-`4` for quick activation | MVP |
| Expanded ability list | More abilities from perk unlocks | Phase 2 |
| Ability tooltips | Hover for detailed description + formula | Phase 2 |

### Full Message Log (`m` key) `[MVP]`

Expands the bottom game log into a scrollable panel covering ~50% of the screen height.

```
┌── Message Log ───────────────────────────────────────────────┐
│ Turn 47: You descend into the medical wing.                   │
│ Turn 47: The air smells of antiseptic and something worse.    │
│ Turn 48: A feral angel spots you from across the room!        │
│ Turn 48: The feral angel charges toward you.                  │
│ Turn 49: You hit the feral angel for 18 damage. (Wound ×1)   │
│ Turn 49: The feral angel claws at you. Miss.                  │
│ Turn 50: You shoot the feral angel for 12 damage.             │
│ Turn 50: The feral angel collapses. +25 XP.                   │
│                                                               │
│ [Esc] Close  [↑↓] Scroll  [f] Filter                        │
└──────────────────────────────────────────────────────────────┘
```

| Feature | Detail | Phase |
|---------|--------|-------|
| Turn-stamped entries | Each message prefixed with turn number | MVP |
| Scrollable | Arrow keys to scroll through history | MVP |
| Priority coloring | Same colors as compact log | MVP |
| Channel filter | Toggle: All / Combat / Narrative / Stealth | Phase 2 |

---

## Examine Mode (`x` key) `[MVP]`

Cursor-driven inspection of any visible tile. No turns consumed.

### Behavior

1. Press `x` — cursor appears on player tile
2. Move cursor with movement keys — cursor moves freely within visible area
3. Tile under cursor shows a tooltip:
   - **Empty floor**: "Floor (passable)"
   - **Wall**: "Wall (blocked)"
   - **Item**: Item name and primary stat
   - **Enemy**: Full enemy info panel (see Enemy Display above)
   - **Door**: "Closed Door — [Space] to open" / "Open Door"
   - **Stairs**: "Stairs Down — [>] to descend"
4. `Esc` or `x` again to exit examine mode

---

## Combat Feedback `[MVP]`

### Visual Sequence (on attack resolution)

1. **Attack direction indicator**: Brief flash on the line between attacker and target tiles
2. **Target tile flash**: Target tile background flashes `ERROR_RED` (damage) or `DIM_TEXT` (miss)
3. **Floating damage number**: Appears above target, drifts upward, fades
4. **Bar update**: HP bar on vitals strip updates immediately
5. **Status badge appears**: If Wounded/Stunned applied, badge appears on vitals bar
6. **Log message**: Confirms the outcome in text

### Floating Damage Numbers

| Type | Color | Example |
|------|-------|---------|
| Normal damage | `BONE` | `-12` |
| Critical hit | `ERROR_RED`, bold | `-24!` |
| Healing | `SUCCESS_GREEN` | `+8` |
| Miss | `DIM_TEXT` | `Miss` |
| Blocked (armor) | `STEEL` | `Blocked` |

Numbers drift upward ~24px over 0.5s and fade to transparent.

---

## Hunger & Thirst `[MVP]`

Not shown as bars on the main vitals strip (to keep it compact). Instead:

- **Healthy**: No indicator shown (principle: don't show what's not actionable)
- **Warning threshold (35%)**: `[Hungry]` or `[Thirsty]` badge appears on the status line in `WARNING_AMBER`
- **Critical threshold (15%)**: Badge turns `ERROR_RED`, text changes to `[Starving]` / `[Dehydrated]`
- **Penalty active**: Tooltip on the badge shows the roll penalty (`−5`, `−15`, or `−30`)

This follows the "show only what's actionable" principle — hunger/thirst only appear when they're about to matter.

---

## Sanity Distortion Effects (InGame) `[MVP]`

These effects apply *only during InGame* and scale with Raid Exposure. See [README.md](README.md) for the full tier table.

### Subtle Tier (25-50% Exposure)

- Vitals bar: Sanity number briefly shows ±1 from true value (single frame, then corrects)
- Map: No effect
- Log: No effect
- Frequency: ~1 per 10 turns

### Significant Tier (50-75%)

- Vitals bar: HP and Sanity bars fluctuate ±5-10% visually (true value unchanged underneath)
- Map: False enemy blips appear for 2-3 turns on revealed-but-not-visible tiles, then disappear
- Log: Phantom messages inject: "You hear skittering behind you.", "Something moves in the corner of your eye."
- Enemy count on any "enemies visible" display may show +1
- Frequency: ~1 per 3-5 turns

### Severe Tier (75-90%)

- Vitals bar: HP bar shows a different value than true HP (±15-20%). Player cannot trust the bar.
- Map: False doors appear (walkable tiles rendered as walls, or walls rendered as doors — for 1-2 turns). Enemy sprites swap appearances (a rat shows as a demon, a demon shows as a rat).
- Log: "You found a medkit!" announced when nothing was picked up. "Ally approaching from the east." with no ally present.
- Inventory: Item icons may temporarily show as different items (visual only — selecting still uses the real item)
- All effects increase in frequency

---

## Phase Summary

### MVP
- Vitals bar (HP, Sanity, AP, Ammo, Armor, Status effects, Floor depth)
- Game log (4-6 lines, priority coloring, deduplication)
- Map viewport (FOV, tiles, sprites, camera zoom)
- Targeting mode (cursor, range ring, cover display, to-hit preview)
- Enemy indicators (alert state: `?` / `!`, examine tooltip)
- Inventory (grid-based, item detail, use/drop/equip)
- Character sheet (skills, perks, equipment)
- Ability picker (list, number-key selection)
- Full message log (scrollable, turn-stamped)
- Examine mode (cursor inspection)
- Combat feedback (visual sequence, floating numbers)
- Hunger/thirst badges (threshold-triggered)
- Sanity distortion (3 tiers: Subtle, Significant, Severe)

### Phase 2 Additions
- Enemy facing indicator (FOV chevron)
- Additional status effects (Bleeding, Burning, Frightened, Corrupted)
- Inventory categories/tabs, drag-and-drop equip, item comparison
- Perk tree browser (visual tree)
- Faction reputation display
- Long-term erosion indicator
- Expanded abilities from perk unlocks
- Ability tooltips with formulas
- Message log channel filter
- Enemy damage type and resistance display
- Stealth detection UI (noise radius indicator)

### Phase 3 Additions
- Species-specific UI elements (Angelic/Demonic visual overlays)
- Reality-Warped dungeon theme (geometry distortion)
- Advanced camera effects (screen shake, chromatic aberration at high exposure)
