# Decision: Save/Load/Replay Spikes (Phase 17)

**Date**: 2026-07-09  
**Status**: Accepted

## Context

Phase 17 adds save/load and replay capability to the BD kernel. Key challenges:
- Bevy's `Entity` type is not serializable (runtime-internal IDs).
- Relationships (OwnedBy, ContainedIn, EquippedBy, etc.) reference `Entity` values.
- Need deterministic replay for testing.

## Decision

### Save format: RON (human-readable, same as content)

Use `ron` for save files — the same crate already used for content loading. This gives us:
- Human-readable save files for debugging.
- Familiar tooling (same RON parser/emitter as content).
- No additional serialization dependencies beyond `serde` + `ron`.

### Entity ID mapping: `SaveId(u64)`

Each entity is assigned a `SaveId` (sequential `u64`) during save. On load, entities are spawned in order and a `HashMap<SaveId, Entity>` is built. Relationship components reference entities by `SaveId` in the save file and are resolved to `Entity` during load.

### Snapshot structure: `RunSnapshot`

The save file contains:
- Version metadata (`save_version`, `content_version`)
- Map data (width, height, tile grid)
- Entity data list (components, pools, statuses, relationships, inventory)
- Game log (for context on reload)
- Seed (for procgen reproducibility)

### IntentReplayLog

A simple serializable record of intent strings, keyed by seed. Enables fixed-seed deterministic replay for testing.

### Serde derives on components

All ECS component types that don't contain `Entity` now derive `Serialize`/`Deserialize`. Types containing `Entity` (relationship markers, status source) are serialized via `SaveId` in the snapshot, not directly.

### Not spiked: `bevy_save`, `moonshine-save`

Per the plan's spike rules, these were considered but the custom RON-serde approach was chosen because:
1. It keeps save/load under our control (save boundaries are game-specific).
2. RON is already a dependency for content loading.
3. The entity ID mapping is straightforward.
4. No additional crate complexity.

## Alternatives considered

| Alternative | Reason rejected |
|---|---|
| `bevy_save` | Requires `bevy_scene`; entity ID mapping not transparent |
| `moonshine-save` | Addresses Bevy 0.17, not 0.18; migration uncertainty |
| Bincode/binary format | Harder to debug; RON's readability wins for development |
| JSON | Text format but noisier than RON; RON already in deps |

## Consequences

- **Positive**: Save files are human-readable RON.
- **Positive**: Full roundtrip save/load tested (11 unit tests).
- **Positive**: `IntentReplayLog` enables deterministic replay testing.
- **Neutral**: Custom serialization code (~600 lines) to handle entity ID mapping.
- **Negative**: `save_world` requires `&mut World` for the entity query.
- **Negative**: Not wired into the game loop yet — save/load is library code, not triggered by keypress.
