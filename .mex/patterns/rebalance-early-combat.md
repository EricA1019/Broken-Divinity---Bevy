---
name: rebalance-early-combat
description: Reduce first-floor lethality and overcrowding without rewriting the whole combat system.
---

# Rebalance Early Combat

Use this when a fresh run's first combat area is technically functional but feels unfair because damage spikes and encounter density overwhelm the player before the rest of the loop can be learned.

## Goals

- Make opening fights survivable without flattening the entire progression curve.
- Bind combat numbers to real weapon/enemy data instead of placeholder constants.
- Reduce first-floor crowding with explicit caps rather than hoping random room weights stay reasonable.
- Verify the new floor live, not only through unit tests.

## Steps

1. **Measure the real spike**
   - Inspect the player baseline (`PlayerBundle`, starter gear, armor, AP).
   - Inspect the enemy spawn table and actual damage formula.
   - Confirm whether lethality comes from damage math, spawn density, or both.

2. **Fix damage at the source**
   - If skill is already used for hit chance, avoid also adding the full raw skill value directly to damage.
   - Prefer a bounded skill bonus (`/5`, `/10`, or similarly capped scaling) over a second full linear multiplier.
   - Keep crit behavior intact, but revisit tests that assume exact doubling if rounding happens after the crit multiplier.

3. **Use real attack profiles**
   - Player melee should come from the equipped melee weapon, with a small blunt fallback only when no melee weapon exists.
   - Enemy melee should come from data attached at spawn (`damage`, `damage_type`) instead of a shared hardcoded attack.
   - Keep ranged attacks on the same damage function so balance changes stay coherent.

4. **Constrain floor-one spawns explicitly**
   - Exclude elite entries from the first-floor enemy pool if they compress the learning window.
   - Keep first-floor enemy rooms to single spawns unless the loop clearly supports swarms.
   - Add an explicit first-floor enemy cap so BSP/map variance cannot recreate overcrowding.

5. **Add regression coverage**
   - Damage stays in a sane opening-range bound.
   - Spawned enemies carry the intended melee profile.
   - First-floor enemy pool excludes elite slots.
   - First-floor spawn cap remains stable.

6. **BRP smoke the rebuilt floor**
   - Launch a fresh dev build.
   - Walk or script the shortest path into a first-floor dungeon.
   - Query the live enemy count and capture a screenshot.
   - Treat "looks less lethal" as insufficient; confirm the cap/pressure numerically in the live run.

## Notes

- Prefer floor-specific softening over global nerfs when the rest of progression is still intentionally rough.
- If the next known issue is post-death extra actions, stop this pattern after balance lands and hand that sequencing work to a dedicated follow-up slice.
