---
name: expand-save-schema
description: "Evolving the JSON save layer with nested state, serde defaults, and backward-compatible legacy-field loading."
triggers:
  - "save schema"
  - "save/load"
  - "backward-compatible save"
  - "pending load"
edges:
  - target: context/conventions.md
    condition: to verify serde-default rules and tier boundaries
  - target: context/architecture.md
    condition: to identify which runtime resources belong in player, colony, overworld, and dungeon save sections
last_updated: 2026-04-06
---

# Expand Save Schema

## Context

Load `context/architecture.md` and `context/conventions.md`. Read `src/core/save.rs` first, then inspect only the components and resources that are actually being serialized: player bundle fields, colony stockpiles/state, overworld graph/travel, dungeon state, and the lore journal.

## Steps

1. Keep `SaveGame` as the single disk boundary inside `src/core/save.rs`.
2. Add nested `Save*State` structs in the save module instead of spreading new serde derives across unrelated gameplay files.
3. Put `#[serde(default)]` on every persisted field. If the runtime type lacks `Default`, wrap it in a save-specific adapter struct or store it as `Option<T>`.
4. Preserve old flat keys as load-only compatibility fields on `SaveGame` using `rename = "..."` plus `skip_serializing`.
5. Normalize after deserialize: map legacy flat data into the nested schema and fill any derived count hints from real stored collections.
6. Snapshot optional runtime resources separately for colony, overworld, dungeon, and lore so autosave can capture whichever section is currently present without forcing restoration wiring.
7. Add `load_game()`, `PendingLoad`, and a queue helper in the save module, but leave entity/resource restoration to a later integration pass.
8. Add roundtrip tests for each save flavor plus one legacy-load compatibility test.

## Gotchas

- Root-level aliases do not populate nested fields. Legacy flat support needs real root compatibility fields on `SaveGame`.
- Old saves may only contain counts, not full collections. Preserve those as hints instead of inventing missing inventory or lore entries.
- Avoid broad derive churn in gameplay modules when a save adapter struct in `save.rs` is enough.
- `cargo build` does not compile `#[cfg(test)]` code. Run a focused `cargo test` slice for the save module after adding roundtrip coverage.

## Verify

- [ ] Every new persisted field in `SaveGame` and the nested save structs has `#[serde(default)]` or a serde default function
- [ ] The schema is nested by domain: player, colony, overworld, dungeon
- [ ] Old flat save keys still deserialize through compatibility fields and normalization
- [ ] `PendingLoad`, `load_game()`, and the queue helper exist without app restoration wiring
- [ ] Autosave only snapshots state and writes the file; it does not restore anything
- [ ] Colony, overworld, dungeon, and legacy compatibility tests pass

## Debug

- If legacy saves stop loading, check that the compatibility fields still use the original flat key names with `rename = "..."`.
- If deserialization fails after adding a field, add `#[serde(default)]` or convert the field to a save-specific wrapper type.
- If tests pass but runtime sections are empty, inspect whether the relevant resource exists when autosave runs and keep that section optional instead of assuming presence.

## Update Scaffold

- [ ] Update `.mex/ROUTER.md` "Current Project State" if save/load status changed
- [ ] Update any `.mex/context/` files that are now out of date
- [x] Added `.mex/patterns/expand-save-schema.md` to `INDEX.md`