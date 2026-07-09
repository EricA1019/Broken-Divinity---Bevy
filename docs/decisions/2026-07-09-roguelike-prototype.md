# Decision: Standalone Roguelike Prototype (Phase 23)

**Date**: 2026-07-09  
**Status**: Accepted

## Context

Phase 23 validates kernel reusability by extending the game with roguelike prototype content — a boss enemy, a second floor, a new tile type, and deterministic seed validation.

## Decision

### Content additions

| Addition | Details |
|---|---|
| `Tile::Water` | Non-walkable tile; renders as `~` (VisualToken::Water) |
| `blueprint.crypt_lord` | Boss enemy (30 HP, 3 AP) |
| `LocationTemplate::crypt()` | Harder 2nd-floor template (larger, skeleton + boss spawns) |

### No game-over or summary screen

The plan's "no-polish rule" explicitly warns against adding game-specific UI: "no unique UI polish, no special balance pass, no extra content beyond acceptance scope, no prototype-only architecture." Game-over detection and summary screens would be prototype-only UI. The kernel is validated by:
- Boss blueprint works through factory
- 2nd procgen template works through same pipeline
- New tile type works through same rendering pipeline
- Deterministic seed test passes

### Fixed-seed test

`prototype_fixed_seed_deterministic_run` validates:
- Ruin seed determinism (same seed → same tiles)
- Crypt Lord blueprint exists in registry
- Crypt template generates rooms
- Water tile is not walkable

## Consequences

- **Positive**: Reusability proven — boss, 2nd floor, new tile all use existing kernel systems.
- **Positive**: No prototype-only hacks added.
- **Positive**: 157 total tests (86 bd_core).
- **Negative**: Game-over detection not wired (player death / all enemies defeated).
- **Negative**: Summary screen not built (end-of-run stats).
