---
name: wire-save-restoration
description: "Integrating save/load into app-state entry and runtime transitions using PendingLoad, PlayerSnapshot, and a menu-entry reset."
triggers:
  - "load restoration"
  - "wire save load"
  - "load game menu"
  - "save and quit"
  - "pending load"
  - "player snapshot"
edges:
  - target: context/architecture.md
    condition: to place seed and runtime bridge resources in the correct tier
  - target: context/conventions.md
    condition: to verify state-gated systems and tier-safe imports
  - target: patterns/expand-save-schema.md
    condition: if the nested save schema is not already in place
last_updated: 2026-04-06
---

# Wire Save Restoration

## Context

Load `context/architecture.md` and `context/conventions.md`. If the save format is still flat or missing `PendingLoad`, load `patterns/expand-save-schema.md` first. Read `src/core/save.rs`, the menu UI, and each target state's `OnEnter` setup system before changing any wiring.

## Steps

1. Keep disk IO in `src/core/save.rs` and move any shared procgen/run resource needed by gameplay systems down to `resources.rs` rather than importing from UI modules.
2. Add a menu-entry reset system that clears long-lived run resources when returning to `AppState::Menu`, but reinitializes always-present resources like `GameTime`, `GameLog`, `LoreJournal`, and `SelectedDestination` instead of removing them outright.
3. Add a runtime bridge resource for player state when no player entity exists in the current state. In this project that bridge is `PlayerSnapshot`, because Overworld uses resources/UI but not a live player entity.
4. In the menu `Load Game` flow, call `load_game()`, insert the saved `WorldSeed`, seed `PlayerSnapshot`, queue the full `PendingLoad`, and route into a gameplay state via a save-to-runtime state mapper.
5. In each target state's `OnEnter` setup system, consume `PendingLoad` only when it matches that state, restore persistent resources from the save, then spawn the player from the save or runtime snapshot. If there is no pending load, preserve existing live resources and only initialize missing ones.
6. On transitions that despawn the player entity, capture the player into `PlayerSnapshot` before setting `NextState`.
7. Add a `SaveAndQuitRequested` resource and a handler system that serializes the current run, then transitions to `AppState::Menu`. Let the menu-entry reset handle cleanup instead of clearing state in UI code.
8. Test build and run the focused save tests, then the full suite if state-entry logic changed.

## Gotchas

- Overworld has no live player entity, so any save path that only queries `With<Player>` will silently fail there unless it falls back to `PlayerSnapshot`.
- `setup_shelter` must not blindly insert `ShelterResources::new_game()` on every entry or travel/load will wipe the stockpile.
- Returning to Menu should not `remove_resource::<GameTime>()`, `GameLog`, or `LoreJournal`; many systems assume those resources exist after startup.
- If a save restores into Dungeon, also restore persistent overworld resources from the file so surfacing later still has a valid world map and factions.
- Keep `PendingLoad` one-shot. Long-lived cross-state continuity belongs in runtime resources like `PlayerSnapshot`, not in the load queue.

## Verify

- [ ] Shared resources used by gameplay modules live in `resources.rs`, not in UI modules
- [ ] Menu reset is handled by an `OnEnter(AppState::Menu)` system
- [ ] `Load Game` queues `PendingLoad` and routes into a gameplay state, not back into `Menu`
- [ ] Colony, Overworld, and Dungeon `OnEnter` systems restore from `PendingLoad` without breaking fresh-game setup
- [ ] Player state survives Colony → Overworld → Dungeon → Overworld/Colony transitions via `PlayerSnapshot`
- [ ] `Save & Quit` is state-gated and returns cleanly to Menu without deleting the save
- [ ] Focused save tests and a full build/test pass

## Debug

- If Load Game lands in the wrong scene, inspect the save-to-runtime state mapping first.
- If the player resets after travel, check whether the transition captured `PlayerSnapshot` before the current state cleanup despawned the player entity.
- If travel or other Menu → Overworld loads fail, verify `ShelterResources` and `PlayerSnapshot` were restored even though Overworld has no player entity.
- If a new run reuses old overworld data, the menu-entry reset is missing a persistent resource.