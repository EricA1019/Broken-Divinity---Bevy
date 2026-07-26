# UI Design Architecture

> **RECONCILE — NOT IMPLEMENTATION AUTHORITY**
> Current UI execution is governed by [Kernel.md](../../Kernel.md) and the
> [Foundation Recovery Plan](../FOUNDATION-RECOVERY-PLAN.md).

## Overview
This document outlines the UI architecture for Broken Divinity MVP using `bevy_egui`.
The game leans entirely into an endless Rimworld-style survival loop. The UI design must facilitate risk/reward decisions with slot-based inventory constraints, post-action reports from auto-resolving shelter raids, and a persistent sanity/action point tracking HUD during turn-based dungeon exploration.

## 1. UI Framework Integration
- **Crate**: `bevy_egui` 0.39.1
- **Rendering Schedule**: All immediate mode UI is drawn during the `EguiPrimaryContextPass` schedule. We strictly enforce a draw/process split (see `node-structure` and `egui-panel` skills).
- **State Synchronization**: Bevy's `AppState` (e.g., `AppState::Overworld`, `AppState::Dungeon`, `AppState::Colony`) dictates which UI systems run. Data is decoupled using Bevy Resources to hold cross-frame state (like `Inventory`, `PlayerStats`, `ShelterDefense`).

## 2. HUD Elements (Persistent Overlays)
*(Reference: `hud-ui` skill for edge-docked un-interactive/semi-interactive info)*
The HUD remains visible during Overworld and Dungeon exploration:
- **Sanity Meter (Raid Exposure)**: Bottom-center or top-left. Fills from 0 to 100 as the player engages in combat or encounters anomalies. Resets to 0 upon returning to the Shelter.
- **Action Points & Turn Counter**: Distinct pips indicating available AP for the current WorldTurn.
- **Health Bar**: Colored red/green to indicate thresholds (Sanity/Health).
- **Minimap / Radar**: Top-right corner showing discovered BSP rooms or Delaunay overworld nodes.

## 3. State-Driven Panels
*(Reference: `egui-panel` skill for modal, stateful interactions)*

### Dungeon UI
- **Action Bar**: Native Bevy Node UI (or bottom egui panel) with buttons for Move, Sprint (shows 3-turn cooldown), Attack, abilities.
- **Inventory Grid (Slot-based)**: Toggled via `I` or an on-screen button. Exactly 20 slots. Items stack up to defined limits. Encumbrance is ignored; slots are the hard risk/reward limit.

### Colony / Shelter Management UI
- **Shelter Overview**: Main center view of the 16x16 or chunked map, rendered by `bevy_ecs_tilemap`. UI overlays on top.
- **Resource Trackers**: Top UI bar showing current Food (requires Raw Meat), Water (requires Dirty Water), Ammo (requires Scrap), and infinite MVP Generator Power.
- **Assignment Board**: A side panel listing the 3-5 shelter survivors and dropdowns to assign them to the CookingStation, WaterPurifier, AmmoPress, etc.
- **Auto-Raid Post-Action Report**: An egui modal popup that immediately displays upon transitioning from Dungeon/Overworld to Colony if a "Schrodinger's Raid" occurred while away. Shows casualties, resources lost, and damage taken based on shelter autonomous defense stats.

### Overworld UI
- **Travel Log**: Scrollable panel logging days passed, universal ticks, and resources consumed/found.
- **Node Inspector**: Panel showing a preview of a Delaunay graph node's difficulty and expected theme before the player commits to traveling.

## 4. Dialogs, Modals, and Tooltips
- **Loot Interface**: Drag-and-drop or click-to-transfer interface bounded by the 20-slot inventory limit.
- **Lore Tooltips**: Hovering over enemies, items, or stations displays tooltips extracting data from `docs/lore/` definitions.
- **Confirmation Dialogs**: For abandoning a run, leaving a dungeon, or assigning a dangerously low-health survivor.

## 5. Input and Navigation
- **Keyboard + Mouse**: Standard for MVP. egui handles mouse interactions natively. Focus is explicitly managed so pressing `W` to move up doesn't accidentally type 'w' into a theoretical search box.
- **Gamepad**: (Phase 2/3 focus, basic mapping support built in).
- **Universal Tick Mapping**: The Turn sequence (Dungeon) or Real-Time sequence (Shelter) updates UI reactively without tight coupling.

## 6. Theming and Styling
- **Typography & Colors**: Dark theme default mimicking a terminal or grimdark aesthetic.
- **UI Scaling**: Scaled appropriately relative to monitor resolution using `bevy_egui`'s scale factor. Font rendering uses pre-loaded .ttf or .otf files.
- **Icons**: Texture atlases provided to egui via `bevy_egui` texture ID registration for item slots, skills, and survivor portraits.
