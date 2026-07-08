# Decision: Pathfinding & Visibility Adapters (Phase 13)

**Date**: 2026-07-08  
**Status**: Accepted

## Context

Phase 13 adds pathfinding and field-of-view to the BD kernel. We need:
1. An A* pathfinder that respects `BlocksMovement` entities and wall tiles.
2. A visibility provider that computes which tiles are visible from an origin.
3. Both must hide the underlying crate behind adapter traits so we can swap implementations later.

## Decision

### Crate: `pathfinding`

Use the `pathfinding` crate (v5) for A*. It is well-maintained, pure Rust, and the `astar` function's API (`neighbors`, `heuristic`, `success`) maps cleanly onto our adapter trait.

### Trait design

```rust
pub trait Pathfinder {
    fn find_path(&self, map: &SmokeMap, start: Position, goal: Position, blocked: &HashSet<Position>) -> Option<Vec<Position>>;
}

pub trait VisibilityProvider {
    fn visible_tiles(&self, map: &SmokeMap, origin: Position, radius: u32, blocked: &HashSet<Position>) -> HashSet<Position>;
}
```

- `blocked` is a `HashSet<Position>` of entity-occupied tiles (not walls — walls come from `SmokeMap::is_walkable`).
- The adapter structs (`AStarPathfinder`, `BresenhamVisibility`) are unit structs with `Default`.

### Visibility: square bounding box

`BresenhamVisibility` iterates a square `[-radius, radius]` in both axes, then applies Bresenham line-of-sight on each tile. This means the effective visibility region is Chebyshev (square), not Manhattan or Euclidean. This is a conscious simplification for Phase 13.

### Integration

No ECS systems use these yet — they are pure library code. Systems that consume them (movement overlay, FOV update, enemy AI) come in later phases.

## Alternatives considered

| Alternative | Reason rejected |
|---|---|
| Inline A* from scratch | Unnecessary; `pathfinding` crate is mature and tested |
| `bracket-pathfinding` | Tied to bracket-lib; we want no rendering dependency |
| Circular FOV (Euclidean) | Premature — square FOV is simpler and sufficient for early phases |
| Shadowcasting | Overkill for Phase 13; can be swapped behind `VisibilityProvider` later |

## Consequences

- **Positive**: Adapter traits let us swap pathfinding and FOV algorithms without touching consumers.
- **Positive**: Zero new ECS systems — no schedule impact.
- **Neutral**: `pathfinding` crate is a new workspace dependency (transitive: `deprecation-rs`, `num-traits`, etc.).
- **Negative**: Square FOV is less realistic than circular; will be revisited when gameplay demands it.
