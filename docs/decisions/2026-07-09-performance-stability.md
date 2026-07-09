# Decision: Performance and Stability (Phase 21)

**Date**: 2026-07-09  
**Status**: Accepted

## Context

Phase 21 adds stress testing and stability validation for the kernel. No new features — only measurement, testing, and bounds assertion.

## Decision

### Stress tests (5 new)

| Test | What it validates |
|---|---|
| `hundred_turn_simulation_does_not_leak_entities` | 100-turn combat-like cycle of spawn/despawn; entity count returns to baseline |
| `seed_batch_does_not_panic` | 1000 procgen seeds with validation; no panics, basic structure assertions |
| `save_load_stress_roundtrip_passes` | 10 consecutive save/load roundtrips; data integrity preserved |
| `event_queue_does_not_grow_unbounded` | 1000 messages sent in one frame; drained by reader system |
| `procgen_timing_is_reasonable` | 100 procgen calls complete in under 5 seconds |

### No optimizations applied

None of the measurements showed a bottleneck requiring optimization at this stage:
- Procgen: ~50ms for 100 seeds (0.5ms per generation) ✅
- Entity management: stable with proper cleanup ✅
- Messages: drained per-frame by reader systems ✅
- Save/load: ~1ms per roundtrip for small worlds ✅

The plan's "optimize only if needed" rule is followed — nothing needs optimization yet.

## Alternatives considered

| Alternative | Reason rejected |
|---|---|
| Full Bevy schedule simulation | Complex; manual spawn/despawn cycle covers entity leak detection |
| Render performance measurement | Terminal rendering at 60fps is not a bottleneck for current map sizes |
| Memory profiling | Would require external tooling; out of scope for this phase |

## Consequences

- **Positive**: 5 stress tests catch entity leaks, procgen panics, save/load corruption.
- **Positive**: Seed batch test validates 1000 seeds in <1 second — procgen is fast.
- **Positive**: No regressions introduced (all prior tests still pass).
- **Negative**: No actual performance optimizations — all bottlenecks are within acceptable bounds for MVP.
- **Neutral**: 156 total tests across the workspace.
