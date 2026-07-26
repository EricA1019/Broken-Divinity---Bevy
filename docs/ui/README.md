# UI Design Specification

> **RECONCILE — NOT IMPLEMENTATION AUTHORITY**
> Current runtime direction comes from [Kernel.md](../../Kernel.md) and the
> [Foundation Recovery Plan](../FOUNDATION-RECOVERY-PLAN.md). This
> document may describe superseded egui or graphical assumptions.

This directory defines every user-facing screen, panel, HUD element, and interaction for Broken Divinity. Each file covers one major game state. This README defines shared foundations that apply everywhere.

**Phase tags**: Every element is tagged `[MVP]`, `[Phase 2]`, or `[Phase 3]`. MVP ships first. Phase 2/3 elements are designed now to avoid layout rework later, but are not implemented until their phase.

**Companion docs**: [gameplay/](../gameplay/) defines *what* the mechanics do. These docs define *how the player sees and interacts with* those mechanics. [tech/architecture.md](../tech/architecture.md) defines the rendering pipeline and egui implementation patterns.

---

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [File Index](#file-index)
3. [Color Palette](#color-palette)
4. [Typography](#typography)
5. [Theming](#theming)
6. [Master Keybind Table](#master-keybind-table)
7. [Panel System](#panel-system)
8. [Escape Cascade](#escape-cascade)
9. [Sanity Distortion System](#sanity-distortion-system)
10. [Feedback Principles](#feedback-principles)

---

## Design Philosophy

Three rules govern all UI decisions:

1. **The map is king.** The tile viewport is never fully occluded. Every panel is either edge-docked, corner-anchored, or a toggleable overlay that the player explicitly opens.

2. **Show only what's actionable.** If missing a piece of information for one turn won't cause a bad decision, it's contextual (modal, collapsible, or state-gated) — not always-visible.

3. **Visuals announce, the log confirms.** Color changes, glyph flashes, and bar shifts communicate outcomes *before* the message log. If the player reads the log to know whether they hit, the visuals have failed.

---

## File Index

| File | AppState | Covers |
|------|----------|--------|
| [ingame.md](ingame.md) | `InGame` | Dungeon HUD, combat, targeting, game log, inventory, character sheet, sanity effects |
| [shelter.md](shelter.md) | `AtShelter` | Tabbed management window (7 tabs), shelter tilemap, raid prep, post-raid report |
| [overworld.md](overworld.md) | `Overworld` | World node map, travel, weather, encounters, node info |
| [menus.md](menus.md) | All / None | Main menu, pause, settings, save/load, death screen, dialogue, transitions |

---

## Color Palette

Dark and oppressive baseline. Bright colors are reserved for danger signals and supernatural effects.

### UI Chrome & Backgrounds

| Constant | Hex | Use |
|----------|-----|-----|
| `PANEL_BG` | `#1a1a1a` | Panel and HUD backgrounds |
| `PANEL_BG_LIGHTER` | `#2a2a2a` | Inset backgrounds, bar backgrounds, input fields |
| `PANEL_BORDER` | `#3a3a3a` | Panel borders, dividers |

### Stat Bars

| Constant | Hex | Use |
|----------|-----|-----|
| `HP_RED` | `#c43c3c` | Health bars |
| `SAN_BLUE` | `#4a7ab5` | Sanity bars |
| `HUNGER_AMBER` | `#c4943c` | Hunger bars |
| `THIRST_CYAN` | `#3ca5c4` | Thirst bars |
| `AP_GOLD` | `#d4a843` | Action point indicators |

### Text

| Constant | Hex | Use |
|----------|-----|-----|
| `BONE` | `#d4c9a8` | Primary text, stat labels |
| `STEEL` | `#a0aab4` | Shelter-context primary text |
| `DIM_TEXT` | `#707070` | Secondary/muted text, inactive elements |
| `GOLD` | `#d4a843` | Primary accent, headings, dungeon context |
| `AMBER` | `#c4943c` | Secondary accent |

### Feedback

| Constant | Hex | Use |
|----------|-----|-----|
| `SUCCESS_GREEN` | `#5ab55a` | Positive deltas, heals, successful actions |
| `ERROR_RED` | `#e05050` | Negative deltas, damage taken, critical warnings |
| `WARNING_AMBER` | `#d4a843` | Warnings, low resource alerts |

### Map Elements

| Element | Color Direction |
|---------|----------------|
| Walls | Dark grey `#2a2a2a` – near-black |
| Floors | Muted dark brown `#3a3530` |
| Doors | Rust/oxidized metal `#6a4a3a` |
| Player | Slightly brighter than surroundings — stands out |
| Enemies | Faction-coded (see [lore/factions.md](../lore/factions.md)) |
| Anomalies | Sickly green `#5a8a3a` / void purple `#6a3a8a` |

### Alert State Colors

| State | Color | Indicator |
|-------|-------|-----------|
| Unaware | — | No indicator |
| Suspicious | `#d4d43c` (yellow) | `?` above sprite |
| Alert | `#e05050` (red) | `!` above sprite |

---

## Typography

Three font size tiers. All text uses a monospace font for roguelike aesthetic.

| Constant | Size | Use |
|----------|------|-----|
| `CAPTION` | 11px | Tooltips, secondary info, compact labels |
| `STAT` | 13px | Stat bar labels, values, HUD text |
| `HEADING` | 16px | Section headings, panel titles |

**Emphasis rules:**
- `.strong()` for important values (current HP number, ammo count)
- `.color(palette::DIM_TEXT)` for labels and secondary info
- `.color(palette::GOLD)` for headings and emphasis in dungeon context
- `.color(palette::STEEL)` for headings in shelter context

---

## Theming

Two theme contexts, applied at the start of every draw system:

### Dungeon Theme (`apply_theme`)
- Panel fill: `PANEL_BG`
- Accent color: `GOLD`
- Text color: `BONE`
- Used in: `InGame`, `Overworld`, `MainMenu`, `GameOver`

### Shelter Theme (`apply_bunker_theme`)
- Panel fill: `PANEL_BG` (slightly cooler tint)
- Accent color: `STEEL`
- Text color: `STEEL`
- Used in: `AtShelter`

Both call `apply_unified_theme(ctx, UiContext)` internally. Widget state colors (hover, active, focused) derive from the accent.

### Dungeon Theme Overlays

Each dungeon theme adjusts the map tile palette but NOT the HUD/panel chrome. The HUD remains consistent; the *map viewport* shifts color:

| Theme | Floor Tint | Wall Tint | Atmosphere |
|-------|-----------|-----------|------------|
| Urban Decay | Grey-brown | Dark grey | Dust particles |
| Underground | Dark earth | Near-black | Oppressive darkness |
| Military | Cold grey | Steel | Sterile, angular |
| Medical | Yellowed white | Off-white | Clinical, decayed |
| Religious | Warm stone | Dark wood | Wrong-colored light |
| Corporate | Beige | Glass-grey | Eerily preserved |
| Infernal | Deep crimson | Sickly orange | Heat shimmer `[Phase 2]` |
| Celestial | Cold gold | White radiance | Harmonic glow `[Phase 2]` |
| Reality-Warped | Shifting | Unstable | Geometry lies `[Phase 3]` |

---

## Master Keybind Table

### Movement (All movement states)

| Key | Alt Key | Action |
|-----|---------|--------|
| `h` / `←` | Numpad 4 | Move west |
| `j` / `↓` | Numpad 2 | Move south |
| `k` / `↑` | Numpad 8 | Move north |
| `l` / `→` | Numpad 6 | Move east |
| `y` | Numpad 7 | Move northwest |
| `u` | Numpad 9 | Move northeast |
| `b` | Numpad 1 | Move southwest |
| `n` | Numpad 3 | Move southeast |
| `.` | Numpad 5 | Wait one turn |

Both vi-keys and numpad are always active simultaneously.

### Actions (InGame — AwaitingInput)

| Key | Action | Phase |
|-----|--------|-------|
| `g` / `,` | Pick up item | MVP |
| `d` | Drop item | MVP |
| `r` | Reload weapon | MVP |
| `f` | Fire / ranged attack (enters targeting) | MVP |
| `a` | Use ability (opens ability picker) | MVP |
| `>` | Descend stairs | MVP |
| `<` | Ascend stairs | MVP |
| `x` | Examine tile (look mode) | MVP |
| `Tab` | Cycle targeting to next valid target | MVP |
| `Space` | Confirm action / interact | MVP |

### Panels (InGame)

| Key | Panel | Phase |
|-----|-------|-------|
| `i` | Inventory (grid) | MVP |
| `@` | Character sheet (skills, perks, stats) | MVP |
| `c` | Crafting `[Phase 2]` | Phase 2 |
| `m` | Message log (expanded view) | MVP |
| `?` | Help / keybind reference | MVP |

### Panels (AtShelter)

| Key | Panel / Tab | Phase |
|-----|-------------|-------|
| `s` | Shelter management window (toggles) | MVP |
| `1`-`7` | Switch to tab 1-7 within shelter window | MVP |
| `m` | Message log | MVP |
| `?` | Help | MVP |

### System (All States)

| Key | Action | Phase |
|-----|--------|-------|
| `Esc` | Back / cancel / pause (see Escape Cascade) | MVP |
| `F5` | Quick save | MVP |
| `F9` | Quick load | MVP |
| `F1` | Toggle FPS / debug overlay (dev only) | Dev |

### Overworld

| Key | Action | Phase |
|-----|--------|-------|
| Movement keys | Navigate between nodes | MVP |
| `Space` / `Enter` | Enter selected node / confirm travel | MVP |
| `w` | View weather details | MVP |
| `m` | Message log | MVP |
| `?` | Help | MVP |

---

## Panel System

All panels use the egui draw/process split pattern (see [tech/architecture.md](../tech/architecture.md)):

- **Draw system** (runs in `EguiPrimaryContextPass`): Reads game state, renders egui widgets, writes to `*UiAction` resource. Never mutates game state.
- **Process system** (runs in `Update`): Reads `*UiAction`, mutates game state. All ECS mutations happen here.

### Panel Navigation State

```
UiPanel enum:
  Inventory, CharacterSheet, Crafting, Abilities,
  MessageLog, Help, ShelterManagement, PauseMenu,
  NodeInfo, WeatherDetail
```

`UiNavState` resource tracks which panel is open. At most one primary panel open at a time (opening a new one closes the previous). The HUD vitals bar is always visible regardless of panel state.

### Panel Rules

1. Panels never fully occlude the map viewport — they dock to edges or use ~40% screen width max
2. Every panel has `Esc` to close (see Escape Cascade)
3. Panel state is cleared on AppState transition (closing inventory when entering overworld)
4. Panels respect the pause state — no panel interactions when `PauseMenuOpen` is true

---

## Escape Cascade

Each `Esc` press backs out exactly one layer:

```
Targeting mode     → Cancel targeting, return to AwaitingInput
Open sub-panel     → Close the sub-panel (inventory, character sheet, etc.)
Pending action     → Clear pending action
Nothing open       → Open pause menu
Pause menu open    → Close pause menu, resume game
```

This is a strict priority chain. If targeting is active, `Esc` cancels targeting and does NOT open the pause menu.

---

## Sanity Distortion System

Sanity distortion is the *unreliable narrator* — the UI itself becomes untrustworthy at high Raid Exposure. These effects are **separate from gameplay debuffs** (those are mechanical). These are *perceptual lies*.

### Distortion Tiers `[MVP]`

| Exposure Range | Tier | UI Effects |
|----------------|------|------------|
| 0-25% (Steady) | Clean | UI is accurate. All displays truthful. |
| 25-50% (Unsettled) | Subtle | Minimap flickers occasionally. Enemy count on the HUD may be off by ±1. Stat bar values jitter by ±1 for a single frame then correct. Item descriptions in inventory occasionally show a different item's text for one frame. |
| 50-75% (Shaken) | Significant | False enemy blips appear on the minimap (shown for 2-3 turns, then vanish). Stat displays fluctuate visibly (±5-10% on bars). Messages appear in the game log that didn't happen ("You hear footsteps behind you." with no source). Tile tints shift subtly in peripheral vision. |
| 75-90% (Breaking) | Severe | The UI actively lies. False item pickups announced in the log. Phantom ally sprites appear on the map. The map shows doors/passages that don't exist (rendering a walkable tile as a wall or vice versa for 1-2 turns). Health bar may show a different value than true HP. Enemy sprites swap appearances. |
| 90-100% (Broken) | Terminal | Run ends — player evacuates or is lost. Screen distortion intensifies as a transition effect. |

### Implementation Notes

- Distortion effects are **cosmetic overlays** — they do not change actual game state. A false door in the map is a rendering lie, not a collision lie.
- The player should **never be 100% certain** whether what they're seeing is real at Shaken+ tiers.
- At Subtle tier, effects should be rare enough to create doubt but not annoyance.
- At Significant tier, the player should actively question the UI.
- At Severe tier, the player should distrust everything and rely on memory/intuition.
- Long-Term Erosion (the persistent sanity track) does NOT distort UI — it changes available dialogue, unlocks risky abilities, and affects endings. `[Phase 2]`

---

## Feedback Principles

### Visual Before Verbal

1. **Bar shifts** — HP/Sanity bars change immediately on damage/drain
2. **Color flash** — Tile under damaged entity flashes briefly
3. **Glyph animation** — Attack direction indicator on the target tile
4. **Then** the game log confirms: "You hit the Raider for 12 damage."

### Threshold Warnings

Stat bars shift color at critical thresholds to signal danger before the number reaches zero:

| Bar | Normal Color | Warning Threshold | Warning Color | Critical Threshold | Critical Color |
|-----|-------------|-------------------|---------------|--------------------|----------------|
| HP | `HP_RED` | 50% | Brighter red | 25% | Pulsing `ERROR_RED` |
| Sanity | `SAN_BLUE` | 50% (Unsettled) | Desaturated blue | 25% (Shaken) | Flickering purple-blue |
| Hunger | `HUNGER_AMBER` | 35% | Brighter amber | 15% | Pulsing `WARNING_AMBER` |
| Thirst | `THIRST_CYAN` | 35% | Brighter cyan | 15% | Pulsing `WARNING_AMBER` |

### Message Deduplication

Consecutive identical log messages compress:

> ~~"The cultist misses. The cultist misses. The cultist misses."~~
> "The cultist misses. (×3)"

Track repeat counts in `GameLog` when pushing consecutive identical messages.

### Stealth Feedback

Stealth outcomes report context in the log:
- "You strike the feral angel from behind! (stealth critical)"
- "The bandit hears your gunshot and turns toward you."
- Never reveal information the player cannot perceive (e.g., exact offscreen alert state transitions).

### Damage Number Style

Floating damage numbers appear briefly above the target entity:
- White for normal damage
- `ERROR_RED` for critical hits
- `SUCCESS_GREEN` for healing
- Numbers drift upward and fade over ~0.5s
- Stacks when multiple damage events occur on the same target in one turn

---

## Inline Capacity Indicators

Surface key counts directly on toggle buttons — avoid forcing panels open just to check state:

| Element | Inline Display | Example |
|---------|---------------|---------|
| Inventory button | Items / capacity | `Inv 6/20` |
| Ammo display | Clip / total | `12/60` |
| AP display | Remaining / max | `2/2 AP` |
| Construction queue | Count | `Build (3)` |
| Research | Current name | `Research: Water Purification` |
