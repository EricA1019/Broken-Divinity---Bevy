---
name: gate-real-time-sim
description: "Adding or repairing real-time colony/overworld timers so simulation advances on explicit ticks instead of every Update frame."
triggers:
  - "timer"
  - "travel pacing"
  - "shelter tick"
  - "real-time simulation"
edges:
  - target: context/architecture.md
    condition: to confirm the state and tier boundary for the paced systems
  - target: context/conventions.md
    condition: to place shared timer resources correctly and verify state gating
last_updated: 2026-04-12
---

# Gate Real-Time Simulation

## Context

Load `context/architecture.md` and `context/conventions.md` first. Then read the relevant scheduler module (`src/game/colony/mod.rs`, `src/game/overworld/mod.rs`, or similar) plus `src/core/resources.rs` and `src/core/turn.rs`.

Broken Divinity uses `GameTime` as the universal clock. If a real-time system needs pacing, prefer ticking a shared Bevy `Timer` resource and advancing `GameTime.turn` on each finished interval instead of inventing ad hoc per-frame counters.

## Steps

1. **Define shared timers in Tier 0** — put reusable timer resources in `src/core/resources.rs` (`ColonyTickTimer`, `TravelDayTimer`, etc.). Keep them transient unless the exact elapsed state must survive save/load.
2. **Reset on entry and run reset** — add explicit timer resets on state entry, and also reset them in `reset_run_state_for_menu` so New Game and Menu returns do not inherit stale elapsed time.
3. **Tick before gating** — in the owning module, place `tick_*_timer` immediately before the paced systems in a single `.chain()` so the `run_if` predicate reads the same-frame timer state.
4. **Advance universal time first** — call `advance_game_time` before systems that log, consume interval-based recipes, or emit warnings keyed to the shared timeline.
5. **Keep local counters local** — preserve domain counters like `TravelState.day` or `RaidChance.ticks_since_last_raid` for mechanics, but use `GameTime.turn` for user-facing log chronology and shared tick math.
6. **Reset mid-state travel/action timers** — if pacing begins after a state is already active (for example, starting overworld travel while already in `AppState::Overworld`), reset the timer when inserting the state resource.
7. **Add regressions** — cover timer duration/finish behavior, `GameTime` advancement, log turn usage, and Menu/New Game reset behavior.

## Gotchas

- Separate `.add_systems(Update, ...)` registrations do not guarantee order. When the predicate depends on a timer updated that same frame, keep timer tick and paced systems in one ordered chain.
- `init_resource` does not overwrite stale run state. Use explicit reset systems for state entry and `reset_run_state_for_menu` for new-run cleanup.
- Timer resources belong in `src/core/resources.rs`, but the systems they gate usually stay in the feature module that owns the behavior.
- If tests start silently doing nothing after a signature change, check for new required resources such as `GameTime`.

## Verify

- [ ] Shared timers live in `src/core/resources.rs`
- [ ] Paced systems are still gated to the correct `AppState`
- [ ] Timer tick and paced systems are ordered in one chain
- [ ] `GameTime.turn` advances exactly once per paced simulation step
- [ ] Menu/New Game reset clears transient timer and raid/travel carryover
- [ ] Tests cover timer behavior plus at least one log/reset regression

## Debug

- **Simulation still runs every frame** — confirm the timer tick system and `run_if` predicate are in the same ordered chain.
- **Logs show stale turns** — verify `advance_game_time` runs before the systems writing to `GameLog`.
- **Travel/colony ticks fire instantly on re-entry** — reset the timer on state entry and when the travel/action resource is created.
- **`run_system_once` tests no-op** — insert the newly required resources (`GameTime`, timer resources, or the domain state resource) before running the system.

## Update Scaffold

- [ ] Update `.mex/ROUTER.md` "Current Project State" if what's working/not built has changed
- [ ] Update any `.mex/context/` files that are now out of date
- [ ] If this is a new task type without a pattern, create one in `.mex/patterns/` and add to `INDEX.md`
