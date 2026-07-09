# Known Issues — Broken Divinity Kernel

**Date**: 2026-07-09  
**Phase**: 24 (Production-Ready Gate)

This document tracks known limitations, unimplemented features, and design gaps. Issues are not blockers for the production gate — they represent follow-up work.

## Critical (no known critical issues)

All 157 tests pass. Content validation passes. Release build runs.

## Gameplay gaps

### Game-over/win condition not wired
- **Impact**: Player can die (HP reaches 0) but no game-over screen or restart flow. All enemies can be defeated but no victory condition.
- **Root cause**: `EntityDefeated` message is emitted but no system reads it to transition to a summary screen.
- **Fix**: Add a system in `BdSet::ResultEmission` that checks `EntityDefeated` and transitions to a "Game Over" or "Victory" game mode.

### Save/load not triggered by keypress
- **Impact**: Save/load works (tested via library API) but no keybinding triggers it in-game.
- **Root cause**: Phase 17 focused on the serialization infrastructure; UI binding was deferred.
- **Fix**: Add `Ctrl+S` → `save_world()`, `Ctrl+L` → `load_world()` in `map_input_to_intents`.

### 2nd floor (Crypt) not accessible from game loop
- **Impact**: `LocationTemplate::crypt()` generates a harder 2nd floor with skeletons and a Crypt Lord boss, but no in-game transition from the ruin to the crypt.
- **Root cause**: Phase 19 travel system has `TravelNode` entries but no floor-descend mechanic.
- **Fix**: Add floor counter in a resource; when player reaches exit in tactical mode, increment floor and generate next location.

### Summary/end-of-run screen not built
- **Impact**: No end-of-run stats (enemies killed, items collected, turns taken).
- **Root cause**: No game-over detection means no trigger for a summary screen.
- **Fix**: Build after game-over system is wired.

## UI limitations

### Debug overlay is basic
- **Impact**: F1 debug screen shows the game log in reverse — helpful but not a structured entity inspector or SignalTrace viewer.
- **Root cause**: Phase 20 scope was reduced to "log viewer" instead of "entity inspector".
- **Fix**: Add structured SignalTrace display with filter-by-stage and entity selector.

### Movement keys hardcoded in input mapper
- **Impact**: WASD and arrow keys are hardcoded in `map_input_to_intents`. The `KeyBindingConfig` is loaded from config and used for the help line, but the actual key matching still uses hardcoded constants.
- **Root cause**: Phase 16 built the key binding config and help line but didn't refactor the input mapper to read from it.
- **Fix**: Replace `KeyCode::Char('w')` matches with lookup from `KeyBindingConfig` resource.

### Inventory equipping not wired
- **Impact**: Inventory screen shows items but `EquipIntent`/`UseItemIntent` are not triggered by keypresses in the inventory view.
- **Root cause**: Phase 12 built the inventory intents and processing systems, but the TUI input mapper never added keys for equip/use/drop.
- **Fix**: Add key bindings for equip (Enter), use (u), drop (d) when inventory screen is active.

## Content gaps

### Water tile unreachable in generated maps
- **Impact**: `Tile::Water` exists with `VisualToken::Water` but is never placed by the current procgen (only Floor, Wall, Door are placed).
- **Root cause**: Phase 23 added the tile type but didn't update procgen to use it.
- **Fix**: Add water pools to `place_rooms` or create a new `place_water` generation stage.

### Only 3 tile types used in generation
- **Impact**: Floor, Wall, Door are placed. Water is defined but not generated. No other tile types exist.
- **Root cause**: Procgen V1 keeps it simple.
- **Fix**: Add more tile types and generation rules in a future phase.

## Technical debt

### RON content files use // comments
- **Impact**: RON files for symbols and themes use `//` comments (valid RON) but are less readable than the original `#` comments.
- **Root cause**: `#` is not valid RON comment syntax — it conflicts with RON's `#![enable(...)]` attribute syntax.
- **Fix**: Use `//` consistently (already done) or switch to YAML/TOML for content.

### Pools component uses `pools` private field
- **Impact**: External code must use `Pools::iter()` / `Pools::iter_mut()` / `Pools::get()` instead of direct field access.
- **Root cause**: Intentional encapsulation. The field is private to enforce access through methods.
- **Note**: This is intentional, not a bug. Documented for awareness.
