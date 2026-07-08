# Decision: Procedural Location V1 (Phase 14)

**Date**: 2026-07-08  
**Status**: Accepted

## Context

Phase 14 adds procedural location generation to the BD kernel. We need:
1. A template-driven generator that creates `LocationPlan` values.
2. Seed-deterministic generation using a staged pipeline.
3. Plan validation before any entity spawning.
4. Integration with the entity factory for spawning.

## Decision

### RNG: `rand::rngs::StdRng` (ChaCha-based, seedable)

Use `StdRng` from `rand` crate with `SeedableRng::seed_from_u64(seed)`. This gives deterministic, reproducible outputs. The `rand` crate was already in the workspace dependency list. `rand_chacha` is not needed directly since `StdRng` wraps `ChaCha12Rng`.

### Graph: `petgraph::UnGraph` for room connectivity

Rooms become nodes, corridors become edges in an undirected graph. This lets us:
- Query connectivity with `kosaraju_scc` (which gives connected components in undirected graphs).
- Ensure all rooms are reachable by bridging disconnected components with corridors.

The `petgraph` crate was already in the workspace dependency list.

### Generation pipeline (6 stages)

1. **Layout**: Determine dimensions from template (random within range).
2. **Place rooms**: Rejection-sampling of non-overlapping rectangular rooms.
3. **Connect rooms**: L-shaped corridors between room centers; `kosaraju_scc` to ensure full connectivity.
4. **Paint tiles**: Floor tiles for rooms and corridors.
5. **Entrance/exits**: Entrance in first room, exits in last room.
6. **Spawn zones**: Walkable tiles in non-entrance rooms.

### Validation before spawning

`validate_plan()` checks:
- Entrance is walkable.
- At least one exit exists and is walkable.
- At least one room exists.
- Minimum walkable area (10% of map).
- Spawn zones are on walkable tiles.
- All rooms reachable from entrance (A* via Phase 13 `Pathfinder`).
- All exits reachable from entrance.

### Spawn integration

Not yet wired into ECS systems — the plan's `SpawnEntry` data exists but converting `LocationPlan` → `SpawnRequests` → `FactoryResolver` is a future phase. The `to_smoke_map()` method bridges plan validation with Phase 13's `Pathfinder`.

### `SmokeMap::from_tiles()` helper

Added a constructor to `SmokeMap` to create a map from a flat tile slice. This is needed for validation and will be reused for runtime location loading.

## Alternatives considered

| Alternative | Reason rejected |
|---|---|
| BSP dungeon generation | Premature for V1; simple random rooms are sufficient |
| Single-stage generation without plan | Violates "plan validates before spawning" principle |
| Custom room graph (no petgraph) | `petgraph` is already in workspace dep list; no need to reimplement |
| `rand_chacha::ChaCha8Rng` directly | `StdRng` from `rand` provides `RngExt` trait methods (`random_range`, `random`); `ChaCha8Rng` in `rand` 0.10 only implements `RngCore` |

## Consequences

- **Positive**: Seed determinism enables reproducible bugs and testing.
- **Positive**: Plan/validation separation prevents invalid locations from spawning entities.
- **Positive**: `petgraph` enables future graph algorithms (world maps, faction relations, etc.).
- **Neutral**: `petgraph` and `rand` are new crate dependencies for `bd_core`.
- **Negative**: Simple room placement may produce suboptimal layouts; BSP can replace it later.
- **Negative**: `rand` v0.10 uses `RngExt` trait (`random_range` method) instead of the familiar `Rng::gen_range` from v0.8 — a learning curve for maintainers.
