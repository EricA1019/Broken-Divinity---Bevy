# RCV-07 UX Baseline Harness and Evidence Protocol (2026-05-26)

## Scope
Defines executable UX baseline harness requirements for AXT validation.

## Test Suite Location
- Integration suite namespace: `tests/ux_baseline_red_harness.rs`
- Required command: `cargo test ux_baseline_red:: -- --nocapture`

## Scenario IDs
1. `S01_MENU_TO_COLONY`
2. `S02_COLONY_TO_OVERWORLD`
3. `S03_OVERWORLD_TO_DUNGEON`
4. `S04_DUNGEON_TO_COLONY_RETURN`
5. `S05_SAVE_LOAD_COLONY`
6. `S06_SAVE_LOAD_OVERWORLD`
7. `S07_SAVE_LOAD_DUNGEON`
8. `S08_MODAL_STRESS_TOGGLE`

## Non-Zero Execution Gate
- The `ux_baseline_red` suite is valid only if executed test count is greater than zero.
- A run with zero matching tests is an automatic gate failure.

## Scripted Run Protocol
1. First-time battery: run `S01` through `S04` in sequence.
2. Resume battery: run `S05` through `S07` from persisted snapshots.
3. Stress battery: run `S08` with rapid state/modal transitions.
4. Record run IDs and timestamps in scorecard artifacts.

## Scorecard Schema Contract
Required fields per scored item:
1. `run_id`
2. `scenario_id`
3. `metric`
4. `observer_id`
5. `result`
6. `notes`

Required summary table for Alpha signoff:
1. one row per Alpha threshold metric
2. threshold value
3. current scored value
4. pass/fail status
5. evidence references

Environment note:
1. When direct human first-time interaction fidelity is unavailable, criteria-based proxy scoring is allowed if the evidence source, run id, and carry-forward rationale are explicit in the scorecard.

## Exit Check
- Harness suite path is defined.
- Scenario IDs are fixed.
- Non-zero execution rule is explicit.
- Scorecard schema and run protocol are fixed.
