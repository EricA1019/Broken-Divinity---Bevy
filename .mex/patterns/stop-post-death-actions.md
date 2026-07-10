---
name: stop-post-death-actions
description: Halt enemy-turn processing immediately after a fatal player hit and queue GameOver once.
---

# Stop Post-Death Actions

Use this when combat technically reaches zero HP, but the current frame still allows more enemy actions, extra damage logs, or duplicate death handling before the app transitions to `GameOver`.

## Goals

- The hit that kills the player should be the last hostile action of that turn.
- `DeathSummary` and the death log should be created once.
- Existing universal death detection should stay compatible with non-combat deaths (travel attrition, hazards, status ticks).

## Steps

1. **Find every fatal-hit path in the active turn**
   - Check enemy melee, enemy ranged, and any companion/enemy-turn systems that can still act after the player reaches 0 HP.
   - Confirm whether the current GameOver transition is deferred to a separate always-on checker.

2. **Create a shared GameOver helper**
   - Centralize the death log + `DeathSummary` insertion + `NextState<AppState>::set(GameOver)` in one helper.
   - Reuse that helper from both the universal death checker and any combat systems that can kill the player directly.

3. **Short-circuit the active combat systems**
   - Early-return immediately if the player is already dead when the system begins.
   - After each hostile damage application (and any rescue mechanic like Second Wind), check again.
   - If the player is still dead, queue GameOver and return before later enemies, wound procs, or follow-up systems run.

4. **Guard against duplicate death handling**
   - If `DeathSummary` already exists, the universal death checker should do nothing.
   - Avoid double-logging `"You have been slain..."`.

5. **Regression-test the stop condition**
   - Add one test for fatal ranged enemy turns.
   - Add one test for fatal melee enemy turns.
   - Assert only one hostile-hit log is emitted and `DeathSummary` exists after the fatal action.

## Notes

- This pattern fixes sequencing, not balance. Run `rebalance-early-combat.md` first if the player is dying because the damage numbers themselves are unreasonable.
- Keep non-combat deaths on the universal checker so travel/status/hazard damage does not need to duplicate combat-specific logic.
