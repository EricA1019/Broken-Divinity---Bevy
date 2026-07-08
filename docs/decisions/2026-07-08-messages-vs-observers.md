# Decision: Bevy Messages vs Observers

## Problem

Bevy 0.18 provides two signal mechanisms:
1. **Messages** (`bevy_ecs::message`) — buffered, pull-based, processed in order
2. **Observers** (`bevy_ecs::observer`) — push-based, triggered immediately on mutation

We needed to choose one as the consistent pattern for the kernel's signal pipeline.

## Options tested

- **Messages**: `MessageReader`/`MessageWriter` — explicit read/write in systems, ordered processing, good for multi-stage pipelines
- **Observers**: `Trigger`/`Observer` — automatic triggering, fire-and-forget, good for reactive side effects

## Accept criteria

- Supports multi-stage pipeline (Intent → Validate → Cost → Effect → Result)
- Traceable — every signal must be debuggable
- Predictable ordering — same inputs produce same signal order
- Works with bevy_ratatui's `KeyMessage` (already Messages-based)

## Reject criteria

- Implicit triggering that's hard to trace
- Non-deterministic ordering
- Requires external crate for basic functionality

## Result

**Use Bevy Messages exclusively for the kernel signal pipeline.**

Observers may be considered later for tightly scoped immediate reactions (e.g., "when an entity is spawned, auto-attach a component"), but never for the core gameplay pipeline.

## Reason

1. Messages are pull-based — systems explicitly read what they need, when they need it
2. Messages process in order — critical for multi-stage pipelines where validation must happen before mutation
3. Messages are traceable — every read/write is explicit in system code
4. bevy_ratatui already uses Messages (`KeyMessage`) — consistency
5. Observers would make it harder to guarantee that validation runs before cost compilation before mutation

## Follow-up work

None. This decision is documented and locked.
