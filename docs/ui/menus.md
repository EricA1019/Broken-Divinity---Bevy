# Menus & System UI

This file covers screens that exist outside the three main game states (InGame, AtShelter, Overworld), or that overlay on top of them. Includes main menu, pause, settings, save/load, death screen, dialogue, and state transitions.

For shared patterns (palette, typography, keybinds), see [README.md](README.md).

---

## Main Menu `[MVP]`

`AppState::MainMenu`. First screen the player sees.

### Wireframe

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│                                                                      │
│                                                                      │
│                         B R O K E N                                  │
│                       D I V I N I T Y                                │
│                                                                      │
│                    "The heavens fell. We remain."                     │
│                                                                      │
│                                                                      │
│                        ▸ New Game                                    │
│                          Continue                                    │
│                          Load Game                                   │
│                          Settings                                    │
│                          Quit                                        │
│                                                                      │
│                                                                      │
│                                                                      │
│                                            v0.1.0  Seed: -------    │
└──────────────────────────────────────────────────────────────────────┘
```

### Elements

| Element | Detail | Phase |
|---------|--------|-------|
| Title | "BROKEN DIVINITY" — large text, `GOLD` color, centered | MVP |
| Tagline | Flavor text below title, `DIM_TEXT` | MVP |
| New Game | Opens new game setup | MVP |
| Continue | Loads most recent save (greyed if no saves exist) | MVP |
| Load Game | Opens save slot browser | MVP |
| Settings | Opens settings panel | MVP |
| Quit | Exits application | MVP |
| Version | Bottom-right corner, `DIM_TEXT` | MVP |
| Background | Dark, atmospheric — static or minimal ambient animation | MVP |

### Menu Navigation

- `↑` / `↓` or `k` / `j` to navigate menu items
- `Enter` or `Space` to select
- `Esc` from any sub-menu returns to main menu
- Mouse click also works on all items

---

## New Game Setup `[MVP]`

Sub-screen of Main Menu. Centered modal.

### Wireframe

```
┌── New Game ──────────────────────────┐
│                                       │
│ Character Name: [Brother Marcus    ] │
│                                       │
│ Background: Shelter Founder           │
│   (You led the survivors here.        │
│    +5 Persuasion, +5 Mechanical)      │
│                                       │
│ Difficulty:                           │
│   ○ Merciful  (forgiving resource    │
│                drain, weaker enemies) │
│   ● Standard  (balanced)             │
│   ○ Punishing (scarce resources,     │
│                aggressive AI)         │
│   ○ Ironman   (Punishing + no save   │
│                scumming) [Phase 2]    │
│                                       │
│ World Seed: [______________]         │
│ (Leave blank for random)              │
│                                       │
│ [▸ Begin]  [Cancel]                  │
└───────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Character name | Text input field, default "Survivor" | MVP |
| Background | Fixed at "Shelter Founder" for MVP | MVP |
| Background selection | Multiple backgrounds with different starting bonuses | Phase 3 |
| Species selection | Human / Angelic / Demonic | Phase 3 |
| Difficulty selector | Radio buttons: Merciful / Standard / Punishing | MVP |
| Ironman mode | Fourth difficulty option — no manual saves | Phase 2 |
| World seed | Text input, blank = random. Displays generated seed after start. | MVP |
| Begin button | Creates world, transitions to `Loading` → `AtShelter` | MVP |
| Cancel button | Returns to main menu | MVP |

---

## Pause Menu `[MVP]`

Triggered by `Esc` when no panel/targeting is active. Overlays on current game state.

### Wireframe

```
┌──────────────────────────────────────────────────────────────────────┐
│                          (game dimmed)                                │
│                                                                      │
│                      ┌── Paused ──────────┐                          │
│                      │                     │                          │
│                      │  ▸ Resume           │                          │
│                      │    Save Game        │                          │
│                      │    Load Game        │                          │
│                      │    Settings         │                          │
│                      │    Help             │                          │
│                      │    Quit to Menu     │                          │
│                      │    Quit to Desktop  │                          │
│                      │                     │                          │
│                      └─────────────────────┘                          │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Background dim | Semi-transparent dark overlay on the game | MVP |
| Resume | Closes pause menu, returns to game | MVP |
| Save Game | Opens save slot selector | MVP |
| Load Game | Opens load slot selector | MVP |
| Settings | Opens settings panel | MVP |
| Help | Opens keybind reference | MVP |
| Quit to Menu | Confirms, then returns to `MainMenu` | MVP |
| Quit to Desktop | Confirms, then exits application | MVP |

All items navigable with arrow keys + Enter, or mouse click.

---

## Save/Load Interface `[MVP]`

Shared between main menu (Load Game) and pause menu (Save/Load).

### Save Slot Browser

```
┌── Save / Load ───────────────────────┐
│                                       │
│ ┌─ Slot 1 ─────────────────────────┐ │
│ │ Brother Marcus  │ Day 47         │ │
│ │ Floor 3/4 (Urban Decay)          │ │
│ │ HP: 74/100  Survivors: 4        │ │
│ │ Saved: 2026-04-05 14:32          │ │
│ │ Seed: 48291067                    │ │
│ └──────────────────────────────────┘ │
│ ┌─ Slot 2 ─────────────────────────┐ │
│ │ Sister Vera  │ Day 12            │ │
│ │ At Shelter                        │ │
│ │ HP: 100/100  Survivors: 3       │ │
│ │ Saved: 2026-04-03 09:15          │ │
│ └──────────────────────────────────┘ │
│ ┌─ Slot 3 (empty) ────────────────┐ │
│ │ [Empty Slot]                      │ │
│ └──────────────────────────────────┘ │
│                                       │
│ [Save] [Load] [Delete] [Cancel]      │
└───────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Save slot cards | Character name, day, current location, HP, survivor count, timestamp, seed | MVP |
| Save button | Writes current game to selected slot (overwrite confirmation if occupied) | MVP |
| Load button | Loads selected save (confirmation dialog) | MVP |
| Delete button | Removes save (confirmation dialog) | MVP |
| Empty slot indicator | "Empty Slot" text in `DIM_TEXT` | MVP |
| Max slots | 5 save slots at MVP | MVP |
| Auto-save slot | Separate always-present auto-save slot (read-only for load) | MVP |
| Multiple slots per run | Expanded slot count | Phase 2 |

### Confirmation Dialogs

All destructive save operations show a confirmation:

```
┌── Overwrite Save? ────────────┐
│                                │
│ This will replace:             │
│ "Brother Marcus — Day 47"     │
│                                │
│ [▸ Confirm]  [Cancel]         │
└────────────────────────────────┘
```

---

## Settings `[MVP]`

Accessible from main menu and pause menu. Tabbed panel.

### Wireframe

```
┌── Settings ──────────────────────────┐
│ [Audio] [Display] [Controls] [Game]  │
├──────────────────────────────────────┤
│                                       │
│ ── Audio ──────────────────────────  │
│ Master Volume   ████████░░ 80%       │
│ Music Volume    ██████░░░░ 60%       │
│ SFX Volume      ████████░░ 80%      │
│ Ambiance Volume ██████████ 100%      │
│                                       │
│ [Apply]  [Reset Defaults]            │
└───────────────────────────────────────┘
```

### Settings Tabs

#### Audio Tab `[MVP]`

| Setting | Control | Default |
|---------|---------|---------|
| Master volume | Slider 0-100% | 80% |
| Music volume | Slider 0-100% | 60% |
| SFX volume | Slider 0-100% | 80% |
| Ambiance volume | Slider 0-100% | 100% |

#### Display Tab `[MVP]`

| Setting | Control | Default |
|---------|---------|---------|
| Fullscreen | Toggle | Off |
| Resolution | Dropdown | Native |
| VSync | Toggle | On |
| UI Scale | Slider 80-150% | 100% |
| Font size | Dropdown: Small / Normal / Large | Normal |
| Show FPS | Toggle (dev) | Off |

#### Controls Tab `[MVP]`

| Setting | Control | Default |
|---------|---------|---------|
| Keybind remapping | List of actions with current key + "Rebind" button | Standard layout |
| Mouse sensitivity | Slider | 100% |
| Edge scrolling | Toggle | Off |
| Camera follow speed | Slider: Snap / Fast / Smooth | Smooth |

#### Game Tab `[MVP]`

| Setting | Control | Default |
|---------|---------|---------|
| Auto-save interval | Dropdown: Every 10 turns / 25 turns / 50 turns / Off | 25 turns |
| Message log size | Dropdown: 100 / 500 / 1000 lines | 500 |
| Combat animation speed | Dropdown: Instant / Fast / Normal | Normal |
| Confirm destructive actions | Toggle | On |
| Tooltip delay | Slider: 0-2s | 0.5s |
| Tutorial messages | Toggle | On (first run), Off (subsequent) |

---

## Death Screen `[MVP]`

`AppState::GameOver`. Shown after permadeath.

### Wireframe

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│                                                                      │
│                           Y O U   D I E D                            │
│                                                                      │
│                    "The silence claims another."                      │
│                                                                      │
│              ─────────────────────────────────────                    │
│                                                                      │
│              Cause of death: Bled out from wounds                    │
│              Location: Shattered Labs, Floor 3                       │
│              Day: 47                                                  │
│                                                                      │
│              ── Run Statistics ──                                     │
│              Turns survived:     1,247                                │
│              Enemies killed:     38                                   │
│              Floors explored:    11                                   │
│              Survivors rescued:  4                                    │
│              Highest skill:      Medical (52)                        │
│              Dungeons cleared:   2                                    │
│              Raids survived:     3                                    │
│                                                                      │
│              ── Best Run ──                                           │
│              Previous best: 2,891 turns (Sister Vera)                │
│                                                                      │
│                                                                      │
│              [▸ New Game]  [Main Menu]  [Quit]                       │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Death header | "YOU DIED" — large, `ERROR_RED`, centered | MVP |
| Flavor text | Random death quote, `DIM_TEXT` | MVP |
| Cause of death | Specific: bled out, starvation, sanity break, overwhelmed in combat | MVP |
| Location | Where death occurred (dungeon name + floor, overworld node, shelter) | MVP |
| Day | How many days the run lasted | MVP |
| Run statistics | Turns, enemies killed, floors explored, survivors rescued, highest skill, dungeons cleared, raids survived | MVP |
| Best run comparison | Previous best turns + character name | MVP |
| New Game button | Goes directly to new game setup | MVP |
| Main Menu button | Returns to main menu | MVP |
| Quit button | Exits application | MVP |
| Roguelite unlock display | Meta-progression unlocks earned this run | Phase 3 |
| Run history log | Scrollable list of past runs | Phase 3 |

---

## NPC Dialogue `[MVP]`

Dialogue with NPCs (primarily Gabriel at MVP) uses a bottom-anchored text panel.

### Wireframe

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│                          (game map visible)                           │
│                                                                      │
│                           NPC sprite                                 │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│ ┌── Gabriel ─────────────────────────────────────────────────────┐  │
│ │                                                                 │  │
│ │  "You've been chosen, whether you wanted it or not. The        │  │
│ │   Sundering left scars deeper than the landscape. Something   │  │
│ │   stirs in the ruins to the east. I can feel it."             │  │
│ │                                                                 │  │
│ │  1. "What do you mean, chosen?"                                │  │
│ │  2. "Tell me about the eastern ruins."                         │  │
│ │  3. "I don't have time for riddles."                          │  │
│ │  4. [Leave]                                                    │  │
│ │                                                                 │  │
│ └─────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Speaker name | NPC name as panel header, `GOLD` color | MVP |
| Dialogue text | Narrative prose, typewriter effect (optional), `BONE` | MVP |
| Response choices | Numbered list, selectable with `1`-`4` keys or click | MVP |
| Leave option | Always present as the last choice | MVP |
| Skill checks in choices | "[Persuasion 40] Try to convince them" — greyed if skill too low | Phase 2 |
| Faction reputation gates | Choices locked behind faction standing | Phase 2 |
| Erosion-gated choices | Dark options available only at high long-term erosion | Phase 2 |
| Dialogue portrait | NPC portrait next to their name | Phase 3 |

### Dialogue Flow

```
Approach NPC → Press [Space]
    │
    ├── Dialogue panel opens (bottom ~40% of screen)
    │   Map still visible above
    │
    ├── NPC text displays
    ├── Player choices appear
    │
    ├── Player selects choice (number key or click)
    │   ├── NPC responds with new text + new choices
    │   └── Loop until conversation ends
    │
    └── [Leave] or end of dialogue → Panel closes, returns to game
```

---

## State Transitions `[MVP]`

Transitions between AppStates use brief fade effects to prevent jarring context switches.

### Transition Types

| From | To | Transition | Duration |
|------|-----|-----------|----------|
| `MainMenu` | `Loading` | Fade to black | 0.5s |
| `Loading` | `AtShelter` | Fade from black | 0.5s |
| `AtShelter` | `Overworld` | Fade to black → fade in | 0.8s total |
| `Overworld` | `InGame` | Fade to black + "Entering [Dungeon Name]..." text → fade in | 1.2s total |
| `InGame` | `Overworld` | Fade to black + "Returning to the surface..." → fade in | 1.0s total |
| `Overworld` | `AtShelter` | Fade to black → fade in | 0.8s total |
| `InGame` | `DeathTransition` | Slow fade to red → death screen | 1.5s |
| `DeathTransition` | `GameOver` | Fade from red to death screen | 0.5s |

### Loading Screen `[MVP]`

Shown during `AppState::Loading` while world generation runs.

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│                                                                      │
│                                                                      │
│                         B R O K E N                                  │
│                       D I V I N I T Y                                │
│                                                                      │
│                      Generating world...                             │
│                      ████████░░░░░░░░ 52%                            │
│                                                                      │
│                      Seed: 48291067                                  │
│                                                                      │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

| Element | Detail | Phase |
|---------|--------|-------|
| Title | Same as main menu | MVP |
| Progress text | "Generating world..." / "Building shelter..." / "Placing nodes..." | MVP |
| Progress bar | Approximate % complete | MVP |
| Seed display | Shows the world seed being used | MVP |
| Flavor tips | Random gameplay tips while loading | Phase 2 |

---

## Help Overlay (`?` key) `[MVP]`

Toggleable keybind reference. Works in all game states.

### Wireframe

```
┌── Controls ──────────────────────────────────────────────────────┐
│                                                                   │
│ ── Movement ───────────     ── Actions ───────────               │
│ h/←  West                   g/,  Pick up                        │
│ j/↓  South                  d    Drop                           │
│ k/↑  North                  r    Reload                         │
│ l/→  East                   f    Fire (targeting)               │
│ y    Northwest               a    Ability                       │
│ u    Northeast               x    Examine                       │
│ b    Southwest               >/<  Stairs                        │
│ n    Southeast               .    Wait                          │
│                                                                   │
│ ── Panels ──────────────    ── System ────────────               │
│ i    Inventory               Esc  Back / Pause                  │
│ @    Character               F5   Quick save                    │
│ m    Message log             F9   Quick load                    │
│ s    Shelter mgmt            ?    This help                     │
│ w    Weather detail                                              │
│                                                                   │
│                              [Esc] Close                         │
└──────────────────────────────────────────────────────────────────┘
```

Semi-transparent overlay centered on screen. Dismisses on `Esc` or `?`.

---

## Phase Summary

### MVP
- Main menu (title, New Game, Continue, Load, Settings, Quit)
- New game setup (name, difficulty, seed)
- Pause menu (Resume, Save, Load, Settings, Help, Quit)
- Save/load interface (5 slots + auto-save, confirmation dialogs)
- Settings (Audio, Display, Controls, Game tabs)
- Death screen (cause, location, statistics, best run comparison)
- NPC dialogue (text + numbered choices, Gabriel encounter)
- State transitions (fades between all AppStates)
- Loading screen (progress bar, seed display)
- Help overlay (keybind reference)

### Phase 2 Additions
- Ironman difficulty mode
- Dialogue skill checks and faction gates
- Erosion-gated dark dialogue choices
- Loading screen flavor tips
- Expanded save slots

### Phase 3 Additions
- Background selection (multiple starting configurations)
- Species selection (Human / Angelic / Demonic)
- Roguelite unlock display on death screen
- Run history log
- Dialogue portraits
