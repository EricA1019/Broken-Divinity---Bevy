# Decision: Outpost, Travel, and Transitions V1 (Phase 19)

**Date**: 2026-07-09  
**Status**: Accepted

## Context

Phase 19 connects the tactical gameplay loop to a broader game structure — an outpost/shelter layer with resource management, travel between locations, and entity state isolation.

## Decision

### GameMode resource

A `#[derive(Resource)] enum GameMode { Outpost, Travel, Tactical }` tracks the current game state. It is mutated by `TransitionIntent` messages processed in `BdSet::IntentCollection`.

### Transition system

`process_transitions` handles:
1. **Tactical → Outpost**: Despawns all transient entities (combat enemies, summons, dropped items). Player and persistent entities survive.
2. **Outpost → Travel**: Sets mode to Travel; displays travel log.
3. **Travel → Tactical**: Sets mode to Tactical; displays entry log.

### Entity isolation

- `PersistentEntity` marker — survives transitions (player, party, permanent items)
- `TransientEntity` marker — despawned when leaving tactical mode
- Unmarked entities are also removed (safe default for procedurally spawned content)

### OutpostState resource

Two resource pools: `Supplies` (10/50) and `Morale` (50/100). Uses the existing `Pools` component for resource management, reusing the PoolDelta pipeline.

### TravelMap

A simple list of `TravelNode` structs with `id`, `name`, `travel_time`, and `location_template`. Two default destinations: Ancient Temple (3 turns) and Crypt of the Fallen (5 turns).

### Outpost screen

New screen definition with panels for party list, travel options, stats, and log. Accessible via 'r' key (return to outpost).

## Alternatives considered

| Alternative | Reason rejected |
|---|---|
| Full overworld map | Premature; simple travel node list is sufficient for V1 |
| ScreenState coupled to GameMode | Kept separate to allow flexible UI without forcing game state changes |
| Custom resource system | PoolDelta pipeline already exists; reusing it reduces new code |

## Consequences

- **Positive**: Outpost mode connects to tactical mode via travel transitions.
- **Positive**: Entity isolation prevents state leaks between modes.
- **Positive**: PoolDelta pipeline reused for outpost resources.
- **Neutral**: 5 tests verify transition behavior.
- **Negative**: Travel does not actually advance time yet (no turn counter in Travel mode).
- **Negative**: No production timers or outpost events in V1.
