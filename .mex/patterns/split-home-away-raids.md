---
name: split-home-away-raids
description: Separate at-home raid handling from away auto-resolution while preserving colony state across Colony/Overworld handoffs.
---

# Split Home vs Away Raids

Use this when colony raids need different behavior depending on whether the player stays at the shelter or leaves, and the current colony scene tears down survivors/stations on exit.

## Goals

- Leaving the shelter during an active raid should resolve it as an away event, not silently drop or reset the raid.
- Survivor and station state must survive Colony -> Overworld/Dungeon -> Colony handoffs.
- Save/quit while away must preserve the queued colony state and the deferred raid summary.

## Steps

1. **Cache colony state before scene teardown**
   - Capture live survivors and stations into handoff resources during `OnExit(AppState::Colony)` before cleanup despawns them.
   - Reuse the existing save-adapter structs where possible so the runtime handoff and disk snapshot stay aligned.

2. **Expand the colony save boundary, not the gameplay modules**
   - Add save adapters for any missing shelter entities (for example stations) in `src/core/save.rs`.
   - Keep `#[serde(default)]` on every new persisted field.
   - When autosaving away from Colony, fall back to the pending handoff resources because there are no live colony entities to query.

3. **Restore colony entities from pending data on re-entry**
   - Make colony setup use pending station/survivor data when present and only spawn the bootstrap defaults when no pending data exists.
   - Keep restoration compatible with both a live runtime handoff and a loaded save file.

4. **Resolve the away raid at the exit point**
   - Trigger away auto-resolution from the actual Colony -> Overworld transition path while the live survivor/station queries still exist.
   - Apply losses/resources immediately, remove `ActiveRaid`, and insert a queued `PendingRaidReport` instead of logging the final outcome right away.

5. **Deliver the report on return**
   - On `OnEnter(AppState::Colony)`, surface the queued away-raid summary once and clear the pending resource.
   - Ensure autosave runs after report delivery if you want the cleared state to be what lands on disk.

6. **Prove the handoff**
   - Add tests for the save adapters, colony handoff cache, and setup restoration.
   - Add a BRP smoke that mutates a station or assignment away from defaults, abandons a live raid, returns to Colony, and confirms both the deferred report and the preserved colony state.

## Gotchas

- Colony teardown destroys the evidence you need for away auto-resolution. Resolve the raid before leaving Colony, not after entering Overworld.
- Query-based autosave from Overworld/Dungeon will see zero colony entities. Without the pending handoff fallback, away-raid outcomes vanish on save.
- Default shelter bootstrap stations can accidentally overwrite saved/runtime stations if setup does not treat pending station data as authoritative.

## Verify

- [ ] Colony exit caches live survivors and stations before cleanup
- [ ] Colony re-entry restores pending survivors/stations instead of respawning defaults
- [ ] Save/load preserves station state and queued away-raid reports with serde defaults
- [ ] Leaving during an active raid removes `ActiveRaid` and creates exactly one deferred report
- [ ] Returning to Colony logs the queued away-raid summary once and clears the pending resource
- [ ] Tests and BRP smoke both confirm colony state survives the round trip
