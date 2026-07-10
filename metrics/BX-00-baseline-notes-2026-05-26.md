# BX-00 Baseline Notes (2026-05-26)

## Scope lock
- Ticket: BX-00 Baseline Capture and Scope Freeze
- Rule: red tests only, no UX behavior implementation changes

## Baseline branch status snapshot
- Captured via `git status --short` before BX-00 test additions.
- Workspace was already dirty before this ticket.

## Baseline playtest artifacts
Existing screenshots in `metrics/` used as baseline set:
- `playtest_menu_01.png`
- `playtest_after_newgame_02.png`
- `playtest_colony_help_03.png`
- `playtest_colony_after_esc_04.png`
- `playtest_overworld_05.png`
- `playtest_colony_06.png`
- `playtest_raid_modal_07.png`
- `playtest_after_savequit_08.png`
- `playtest_after_load_09.png`
- `playtest_gate_attempt_10.png`
- `playtest_gate_attempt2_11.png`

## BX-00 red test runs
Command run twice:
- `cargo test ux_baseline_red -- --nocapture`

Result (both runs):
- 5 tests failed consistently:
  - `enter_off_gate_emits_guidance`
  - `help_does_not_overlap_raid_modal`
  - `escape_closes_topmost_blocking_layer_first`
  - `load_emits_concise_recap`
  - `stale_entity_brp_request_is_graceful`

## Full gate baseline
Command:
- `./scripts/test-gate.sh`

Result summary:
- Build (debug): PASS
- Tests: FAIL (251 passed, 5 failed)
- Clippy: PASS
- Build (release): PASS
- Final: `GATE FAILED — do not distribute`

## Notes
- Failure profile matches BX-00 intent: codify known UX defects as reproducible red tests before BX-01.
