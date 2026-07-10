# Playtest Report

**Date**: 2026-04-06
**Build**: main (uncommitted — no commits yet)
**Target**: Full codebase audit — all systems as-built after Gabriel intro implementation
**App**: `broken_divinity` (headless validation — BRP tools unavailable)
**Method**: Build + 98 tests + clippy (67 lib warnings) + deep code audit across 5 domains
**Codebase**: 59 Rust files, ~11,000 LOC

## Summary

The game compiles cleanly (1 dead-code warning), all 98 tests pass, and the core gameplay loop (menu → colony → overworld → dungeon → combat → save/load) is structurally sound. Three **high-severity** issues were found: stale combat resources on dungeon re-entry, potential turn-phase lock if enemies spawn mid-turn, and most UI panels violating the draw/process split convention. No panics, no data loss, no security issues.

---

## Build & Test Results

| Metric | Result |
|--------|--------|
| `cargo build` | ✅ Pass (1 warning: unused `split_horizontal` field in bsp.rs) |
| `cargo test` | ✅ 98/98 pass, 0 fail, 0 ignored |
| `cargo clippy` | ⚠️ 67 lib warnings + 1 bin warning |

### Clippy Warning Breakdown

| Category | Count | Severity |
|----------|-------|----------|
| Complex type (large Query tuples) | 13 | Low — Bevy-idiomatic |
| Collapsible `if` | 11 | Low — style |
| Too many function arguments (>7) | 6+3+2+1+1 = 13 | Medium — refactor candidate |
| `.as_ref().map(\|x\| &**x)` → `.as_deref()` | ~15 | Low — auto-fixable |
| `map(..).flatten()` → `.and_then()` | 3 | Low — auto-fixable |
| `.is_multiple_of()` / `Range::contains` | 2 | Low — auto-fixable |
| Dead field (`split_horizontal`) | 1 | Low |
| Loop counter variable | 1 | Low |

**40 of 67 are auto-fixable** via `cargo clippy --fix`.

---

## Findings

### [HIGH] Stale `ShootTarget` Entity on Dungeon Re-entry

- **Observed**: `ShootTarget(pub Option<Entity>)` and `BumpAttackTarget(pub Option<Position>)` are initialized once via `init_resource()` in `dungeon::plugin()` and never reset on `OnExit(AppState::Dungeon)`.
- **Expected**: Combat resources should reset to `None` when leaving a dungeon.
- **Risk**: If `ShootTarget` held a `Some(entity_id)` from the previous dungeon and a new dungeon reuses that entity ID (Bevy recycles IDs), the ranged system could fire at the wrong entity on frame 1 of re-entry.
- **ECS State**: `ShootTarget` consumed via `.take()` each frame, so the window is narrow but real.
- **File**: [src/game/dungeon/spawn.rs](src/game/dungeon/spawn.rs#L551) (`cleanup_dungeon`)
- **Suggested Fix**: Add to `cleanup_dungeon`:
  ```rust
  commands.insert_resource(melee::BumpAttackTarget::default());
  commands.insert_resource(ranged::ShootTarget::default());
  ```

### [HIGH] Turn Phase Can Lock in `EnemyTurn`

- **Observed**: `advance_turn_phase()` stays in `EnemyTurn` until `enemies.iter().all(|b| b.remaining == 0)`. If an entity with `Enemy` + `ActionBudget` is spawned mid-turn (e.g., hallucination spawn during `PlayerTurn` before `WorldTick` resets budgets), it may enter `EnemyTurn` with `remaining > 0` but never get its budget consumed by `enemy_ai_turn()` if it doesn't meet that system's query filters.
- **Expected**: Turn phase should always advance within one frame.
- **Risk**: Soft-lock — player stuck waiting forever. Requires exact timing (mid-turn spawn with non-zero budget not consumed by AI).
- **File**: [src/core/turn.rs](src/core/turn.rs#L101-L105)
- **Suggested Fix**: Add a max-iterations guard or force-zero budgets at the start of `EnemyTurn`:
  ```rust
  TurnPhase::EnemyTurn => {
      let enemies_done = enemies.iter().all(|b| b.remaining == 0);
      if enemies_done { next_phase.set(TurnPhase::WorldTick); }
  }
  // Add: timeout counter or unconditional advance after N frames
  ```

### [HIGH] Most UI Panels Mutate State in Draw Functions

- **Observed**: 6 of 10 UI panels perform game-logic mutations directly in their draw function instead of deferring to a process system via an action resource.
- **Expected**: Convention requires draw (EguiPrimaryContextPass, read-only) → action resource → process (Update, mutations).
- **Panels violating convention**:

  | Panel | Mutation in Draw |
  |-------|-----------------|
  | [menu.rs](src/ui/menu.rs) | `next_state.set()`, `commands.insert_resource(WorldSeed)`, `exit.write()` |
  | [perk_choice_panel.rs](src/ui/perk_choice_panel.rs) | `player_perks.unlock()`, `pending.pop_next()`, `log.push()` |
  | [overworld_panel.rs](src/ui/overworld_panel.rs) | `commands.insert_resource(SaveAndQuitRequested)` |
  | [colony_panel.rs](src/ui/colony_panel.rs) | `commands.insert_resource(SaveAndQuitRequested)` |
  | [gameover.rs](src/ui/gameover.rs) | `next_state.set(AppState::Menu)` |
  | [inventory_panel.rs](src/ui/inventory_panel.rs) | `open.0 = false` (panel close in draw) |

- **Only [gabriel_dialogue_panel.rs](src/ui/gabriel_dialogue_panel.rs) follows the full convention.**
- **Risk**: Mutation ordering unpredictable when draw and logic share the same frame phase. Can cause subtle one-frame-lag bugs or UI desyncs.
- **Suggested Fix**: Incrementally migrate each panel to draw/process split. Priority: `perk_choice_panel.rs` (mutates multiple systems) > `menu.rs` (state transitions) > rest.

### [MEDIUM] `TurnPhase` Not Reset on Dungeon Exit

- **Observed**: `TurnPhase` is a Bevy `States` resource initialized once at startup. No `OnExit(AppState::Dungeon)` handler resets it to `AwaitingInput`.
- **Expected**: Re-entering a dungeon should always start in `AwaitingInput`.
- **Risk**: If player exits during `PlayerTurn` or `EnemyTurn` (e.g., save-and-quit), re-entering dungeon could start mid-turn.
- **File**: [src/core/turn.rs](src/core/turn.rs#L81) (definition), [src/game/dungeon/mod.rs](src/game/dungeon/mod.rs#L27) (no reset)
- **Suggested Fix**: Add `OnExit(AppState::Dungeon)` handler to reset phase.

### [MEDIUM] Gabriel Companion Fallback Can Spawn Duplicates

- **Observed**: In `process_gabriel_dialogue_action()`, if `gabriel_q.single_mut()` fails (no Gabriel entity found), the fallback spawns a **new** Gabriel at the player's position.
- **Expected**: Should log an error and bail — Gabriel should always exist when dialogue completes.
- **Risk**: If Gabriel's entity is despawned by a timing edge case before dialogue completes, a duplicate companion spawns with fresh stats.
- **File**: [src/ui/gabriel_dialogue_panel.rs](src/ui/gabriel_dialogue_panel.rs#L100-L107)
- **Suggested Fix**: Change fallback from spawn to `warn!()` + return.

### [MEDIUM] `SelectedDestination` Overwritten on Load

- **Observed**: `restore_persistent_run_resources()` unconditionally inserts `SelectedDestination::default()`, even when loading into Overworld with pending travel state.
- **Expected**: Should preserve travel context when loading an overworld save.
- **File**: [src/core/save.rs](src/core/save.rs#L302)
- **Suggested Fix**: Only reset if no `TravelState` loaded.

### [LOW] UI Schedule Inconsistency

- **Observed**: Only `gabriel_dialogue_panel` draws in `EguiPrimaryContextPass`. All other panels draw in `Update`.
- **Expected**: All draw systems should use `EguiPrimaryContextPass` for consistent Z-ordering and frame timing.
- **Risk**: Gabriel's dialogue panel renders in an earlier phase than overlapping panels (perk choice, HUD), causing inconsistent layering.
- **File**: [src/main.rs](src/main.rs) (system registration)
- **Suggested Fix**: Migrate draw systems to `EguiPrimaryContextPass` as panels get refactored.

### [LOW] `PlayerSnapshot` Lifecycle Ambiguity

- **Observed**: `PlayerSnapshot` is created on colony→overworld transition and consumed by `setup_dungeon`. If a player loads a save before consumption, or exits dungeon→overworld without consumption, the snapshot persists.
- **Expected**: Snapshot should be explicitly cleared after consumption.
- **Risk**: Stale snapshot could be consumed on unexpected re-entry.
- **File**: [src/main.rs](src/main.rs) (creation), [src/game/dungeon/spawn.rs](src/game/dungeon/spawn.rs) (consumption)

### [LOW] Dead BSP Field

- **Observed**: `BspNode::Split.split_horizontal` is never read.
- **File**: [src/game/dungeon/bsp.rs](src/game/dungeon/bsp.rs#L65)
- **Suggested Fix**: Remove field or use it in rendering/debug visualization.

---

## System-by-System Audit Summary

### State Transitions
| Transition | Status | Notes |
|-----------|--------|-------|
| Menu → Colony (New Game) | ✅ | Seed + resources init, autosave on entry |
| Menu → * (Load Game) | ✅ | Full restore with PendingLoad queue |
| Colony → Overworld | ✅ | PlayerSnapshot bridge, cleanup on exit |
| Overworld → Dungeon | ✅ | DungeonState pre-inserted with node context |
| Dungeon floor transitions | ✅ | Theme preserved, Gabriel respawned, entities cleaned |
| Dungeon → Overworld (stairs up from floor 1) | ✅ | Cleanup runs, snapshot restored |
| Any → Menu (Save & Quit) | ✅ | `reset_run_state_for_menu()` clears all |
| Any → GameOver (player death) | ✅ | `check_player_death` properly gated |

### Save/Load
| Area | Status | Notes |
|------|--------|-------|
| Player round-trip | ✅ | Stats, inventory, equipment, sanity, perks all preserved |
| Dungeon round-trip | ✅ | Floor, theme, seed, story_tag, origin_node_id preserved |
| Overworld round-trip | ✅ | Graph, factions, position, travel state preserved |
| Colony round-trip | ✅ | Shelter seed, 5 resource types preserved |
| Gabriel round-trip | ✅ | encounter_completed, joined flags preserved |
| Backward compat (v2→v3) | ✅ | `normalize_loaded()` backfills story tags, syncs derived fields |
| Test coverage | ✅ | 4 round-trip tests (colony, overworld, dungeon, legacy migration) |

### Gabriel Encounter
| Check | Status | Notes |
|-------|--------|-------|
| Trigger gating | ✅ | Floor==2 && GabrielIntro tag && not completed && not joined |
| Dialogue state machine | ✅ | All states reachable, all paths converge to Accept |
| Input gating during dialogue | ✅ | 5/5 input systems check `is_active()` |
| Companion persistence across floors | ✅ | Despawn + respawn on every floor if `joined` |
| Entity cleanup on dungeon exit | ✅ | `cleanup_dungeon` despawns Gabriel entities |
| Save round-trip | ✅ | GabrielState flags restored, story tags preserved |

### Combat & AI
| Check | Status | Notes |
|-------|--------|-------|
| Damage overflow | ✅ | `.max(1)` floor, HP checked via `is_dead()` |
| AI pathfinding | ✅ | A* with occupied-tile avoidance, returns None safely |
| Query safety | ✅ | All queries use `single_mut()` with early return or `get_mut()` |
| Dead entity access | ✅ | Buffered despawn via Commands prevents mid-iteration panic |
| Gabriel excluded from hostile targeting | ✅ | Gabriel lacks `Enemy` component entirely |
| Skill XP cap | ✅ | Capped at level 10 |
| Status stacking | ✅ | Clamped to `max_stacks()` |

---

## Performance

- **Build time**: 7.3s incremental (optimized+debuginfo profile)
- **Test suite**: <0.01s (98 tests)
- **Clippy**: 11.9s
- **Entity count**: Not measurable without BRP (headless)
- **FPS**: Not measurable without BRP (headless)

---

## Verdict: NEEDS ATTENTION

The core architecture is solid and all gameplay paths are structurally complete. The 3 high-severity findings (stale combat resources, turn-phase lock potential, UI convention violations) are real but unlikely to cause player-visible issues in normal play — they affect edge-case re-entry timing, unlikely spawn patterns, and code maintainability rather than immediate crashes. The 98/98 test pass and clean build confirm the codebase is in good shape for continued development.

**Recommended next actions (priority order)**:
1. Reset `BumpAttackTarget` + `ShootTarget` + `TurnPhase` in `cleanup_dungeon` — 5 min fix
2. Remove Gabriel fallback spawn, replace with warning log — 2 min fix
3. Run `cargo clippy --fix` for the 40 auto-fixable warnings — 1 min
4. Migrate `perk_choice_panel` and `menu` to draw/process split — larger but convention-critical
5. Live Bevy MCP playtest when BRP tools are available — validates rendering, input, and frame timing
