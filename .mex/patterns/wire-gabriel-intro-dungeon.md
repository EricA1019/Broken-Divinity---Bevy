---
name: wire-gabriel-intro-dungeon
description: Pattern for tagging the first dungeon, carrying dungeon-site narrative context into runtime/save data, and staging Gabriel's scripted floor-2 encounter plus ghost companion join.
last_updated: 2026-04-06
---

# Task: Wire Gabriel Intro Dungeon

## Use When
- A dungeon needs node-specific narrative behavior instead of generic floor generation.
- The closest overworld dungeon must become the Gabriel intro site.
- A scripted dungeon encounter should unlock Gabriel as a non-blocking companion.

## Steps
1. Extend the overworld graph node model with an optional story tag and ensure graph generation backfills the closest dungeon as `GabrielIntro`.
2. Carry dungeon-site metadata into runtime before switching to `AppState::Dungeon`; in this codebase that means inserting a `DungeonState` resource from overworld arrival instead of letting dungeon setup guess.
3. Persist dungeon origin metadata and Gabriel progression state in the save schema with `#[serde(default)]`, and normalize older saves by backfilling story tags onto restored overworld graphs.
4. Keep dungeon site seeds stable per overworld node so returning to the same dungeon uses the same base seed instead of the global world seed directly.
5. When entering floor 2 of the tagged dungeon, reserve one room for the scripted encounter before generic room content spawns.
6. Spawn Gabriel as a separate companion entity that does not use the hostile enemy marker, so movement/targeting code does not treat them as a normal enemy.
7. Gate normal dungeon input while Gabriel dialogue is open, then flip Gabriel into active companion mode and persist that joined state once the player accepts.
8. Re-spawn Gabriel on later floors and on dungeon save/load restores using the persisted join flag rather than re-running the intro.

## Gotchas
- If the dungeon entry flow does not preserve the overworld node id, floor-2 narrative hooks will be impossible to target cleanly and future node-specific dungeons will repeat the same bug.
- Runtime snapshot spawns must override the saved map position when entering a different scene, or the player can inherit stale shelter/dungeon coordinates.
- The Gabriel intro room must be excluded from generic room-content spawning or enemies/hazards can overwrite the scripted scene.
- Keep Gabriel out of hostile `Enemy` queries. Ghost behavior is easier to maintain by giving Gabriel their own marker and only querying hostile enemies explicitly.

## Verify
- `cargo build -p broken_divinity`
- `cargo test -p broken_divinity`
- Confirm the closest dungeon receives the `GabrielIntro` story tag deterministically.
- Confirm floor 2 of that dungeon opens Gabriel dialogue before normal actions can continue.
- Confirm Gabriel stays joined after the encounter and survives save/load plus later floor transitions.