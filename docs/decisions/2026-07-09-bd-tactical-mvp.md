# Decision: BD Tactical MVP (Phase 18)

**Date**: 2026-07-09  
**Status**: Accepted

## Context

Phase 18 assembles all 17 prior phases into the first playable Broken Divinity tactical loop — a generated ruin with enemies, items, an exit, and save/load.

## Decision

### Procgen integration

The hardcoded `spawn_world` function is replaced with a procedural generation flow:

1. `generate_location(&template, seed)` creates a `LocationPlan`.
2. `SmokeMap::from_tiles(...)` replaces the map resource.
3. Player spawns at the entrance.
4. Enemies spawn on valid spawn zones.
5. Items are scattered across rooms.
6. Exit markers are placed at exit positions.

### Content pack: `BlueprintRegistry::phase18_defaults()`

Adds 6 blueprints to the existing 4:

| Blueprint | Type |
|---|---|
| `blueprint.skeleton` | Enemy (12 HP) |
| `blueprint.ally_warden` | Ally (25 HP) |
| `blueprint.sword` | Item |
| `blueprint.shield` | Item |
| `blueprint.smite_scroll` | Usable item |
| `blueprint.gold_pile` | Loot |

### Door tile

`Tile::Door` is added to the tile enum. Doors are walkable and render as `VisualToken::DoorClosed`. The entrance and exit positions are marked as doors on the map.

### `ExitTile` component

A marker component placed on the exit entity. Future systems can detect when the player steps on the exit position and trigger location transitions.

### `Pools::iter_mut()`

Added to allow mutable iteration over pools (needed by combat resolution and tests).

## Consequences

- **Positive**: The game now generates a procedural map with enemies and items at startup.
- **Positive**: 7 MVP integration tests validate the core loop.
- **Positive**: All content is in Rust fixtures (no RON yet, as per "stable shapes first" rule).
- **Negative**: Debug overlay (F1) not yet implemented — deferred.
- **Negative**: Win/lose checks are not wired into the game loop (EntityDefeated is emitted but not processed for game-over).
- **Neutral**: 142 total tests across the workspace.
