# AXT-00 Baseline Evidence (2026-05-26)

## Objective
Freeze baseline behavior before AXT implementation and capture reproducible evidence.

## Commands Run
- cargo test ux_baseline_red:: -- --nocapture
- ./scripts/test-gate.sh

## Results
- UX regression suite: PASS
  - 20 passed
  - 0 failed
- Quality gate: PASS
  - Build (debug): PASS
  - Tests: PASS
  - Clippy (-D warnings): PASS
  - Build (release): PASS

## Baseline Playtest Artifact Set
The existing visual baseline set for first-session flow remains:
- metrics/playtest_menu_01.png
- metrics/playtest_after_newgame_02.png
- metrics/playtest_colony_help_03.png
- metrics/playtest_colony_after_esc_04.png
- metrics/playtest_overworld_05.png
- metrics/playtest_colony_06.png
- metrics/playtest_raid_modal_07.png
- metrics/playtest_after_savequit_08.png
- metrics/playtest_after_load_09.png
- metrics/playtest_gate_attempt_10.png
- metrics/playtest_gate_attempt2_11.png

## Baseline Scorecard Snapshot
Current first-session baseline from latest structured UX report:
- Onboarding clarity: 7.5
- Control discoverability: 8.0
- Navigation predictability: 8.0
- UI hierarchy/readability: 7.0
- Feedback quality: 8.0
- Error/edge-case trust: 8.0
- Goal clarity/progression cues: 8.0
- Overall first-session confidence: 7.8

## Delta Target Reference
Alpha metric targets are tracked in:
- PLAN-2026-05-26-ALPHA-READINESS.md

## Notes
- This baseline confirms project is currently gate-green before AXT-01+ implementation.
- No gameplay code modifications were performed as part of AXT-00 evidence capture.
