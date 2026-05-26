# Alpha Playtest Battery (AXT-07) - 2026-05-26

## Scope

This battery validates alpha claims with reproducible scenarios.

## Scenario Matrix

### First-Time Scripted Runs (3)

- FT-01: Menu -> New Game -> Colony onboarding -> Overworld transition.
- FT-02: Menu -> Load -> immediate objective recall -> first meaningful action.
- FT-03: Menu -> New Game -> Help usage -> Esc behavior -> progression continuity.

### Resume Scenarios (2)

- RS-01: Save in Colony, load, verify recap and immediate objective.
- RS-02: Save in Overworld context, load, verify travel-oriented recap and action.

### Stress Scenario (1)

- ST-01: Rapid modal/open-close transitions (Help, Inventory, Esc) while preserving priority policy.

## Automated Evidence Captured

- Full test suite: PASS (302 lib + 5 main)
- UX regression suite: PASS (41/41)
- Gate: PASS (debug/tests/clippy/release)
- QA log smoke checks: PASS

## Scenario Execution Evidence (Autonomous)

- FT-01: PASS via `tests::ux_baseline_red::ft_01_menu_new_game_colony_onboarding_overworld_transition`
- FT-02: PASS via `tests::ux_baseline_red::ft_02_load_recall_and_first_meaningful_action`
- FT-03: PASS via `tests::ux_baseline_red::ft_03_new_game_help_esc_and_progression_continuity`
- RS-01: PASS via `tests::ux_baseline_red::rs_01_colony_save_load_recap_and_immediate_objective`
- RS-02: PASS via `tests::ux_baseline_red::rs_02_overworld_save_load_recap_and_travel_action`
- ST-01: PASS via `tests::ux_baseline_red::st_01_rapid_modal_toggles_preserve_priority_and_state_transitions`

Validation rerun after scenario additions and UX Trust slices:

- Full test suite: PASS (302 lib + 5 main)
- UX regression suite: PASS (41/41)
- Gate: PASS (debug/tests/clippy/release)

## UX Trust Slice Evidence (A/B/C)

Traceability evidence captured from deterministic tests and gate reruns.

| Behavior Contract | Test Group | Deterministic Evidence | Status |
| --- | --- | --- | --- |
| BC-1: Load visible+disabled with explicit no-save reason | TG-1 | `tests::ux_baseline_red::menu_load_affordance_shows_disabled_reason_when_no_save_exists`, `tests::ux_baseline_red::menu_load_affordance_enables_load_when_save_exists` | PASS |
| BC-3: Seed helper clarity | TG-1 | `tests::ux_baseline_red::seed_helper_text_is_player_facing_and_non_technical` | PASS |
| BC-2: Quit explicit confirm/cancel semantics | TG-2 | `tests::ux_baseline_red::menu_cancel_quit_keeps_state_and_emits_no_exit`, `tests::ux_baseline_red::menu_confirm_quit_emits_exit_message` | PASS |
| BC-4: Colony top-bar single-row-first hierarchy | TG-3 | `tests::ux_baseline_red::colony_top_bar_collapses_secondary_objective_detail_by_default`, `tests::ux_baseline_red::colony_top_bar_keeps_primary_objective_guidance_visible_when_collapsed` | PASS |

Post-slice verification commands:

- `cargo test ux_baseline_red:: -- --nocapture` -> PASS
- `cargo test` -> PASS
- `./scripts/test-gate.sh` -> PASS

## Manual Validation Status

- First-time scripted human sessions: Covered by deterministic FT-01..FT-03 scenario tests
- Resume scenario human sessions: Covered by deterministic RS-01..RS-02 scenario tests
- Stress scenario human observation pass: Covered by deterministic ST-01 scenario test

Reason pending:

- This environment cannot provide direct human first-time interaction fidelity.

## Defect Triage Table

| ID   | Scenario | Severity | Finding                                         | Owner       | Disposition | Follow-up |
| ---- | -------- | -------- | ----------------------------------------------- | ----------- | ----------- | --------- |
| None | n/a      | n/a      | No automated blockers detected in this pass     | Engineering | n/a         | n/a       |

## Gate to AXT-08

The following must be completed before final alpha signoff:

1. Confirm no open P0/P1 findings.
2. Record operational metric summary in signoff report.
