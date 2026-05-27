# AXT-00 Baseline Scorecard (2026-05-26)

## Header
- run_id: AXT00-20260527-REDUCED-RUNTIME-01
- timestamp_utc: 2026-05-27T17:42:02Z
- observer_id: copilot-gpt-5.4
- build_ref: local-main-reduced-runtime

## Entries
| scenario_id | metric | result | notes |
| --- | --- | --- | --- |
| S01_MENU_TO_COLONY | onboarding_clarity | 8.6 | Current reduced-runtime harness run reached Colony from Menu with deterministic seed handoff; score carried via the existing proxy-scoring model from `docs/tech/alpha-playtest-battery-2026-05-26.md`. |
| S02_COLONY_TO_OVERWORLD | control_discoverability | 8.4 | Current reduced-runtime harness run advanced Colony to Overworld through the active primary action; score carried via the existing proxy-scoring model from `docs/tech/alpha-playtest-battery-2026-05-26.md`. |
| S03_OVERWORLD_TO_DUNGEON | navigation_predictability | 8.2 | Current reduced-runtime harness run advanced Overworld to Dungeon and exposed the high-risk recap surface; score carried via the existing proxy-scoring model from `docs/tech/alpha-playtest-battery-2026-05-26.md`. |
| S04_DUNGEON_TO_COLONY_RETURN | overall_first_session_confidence | 8.5 | Current reduced-runtime harness run completed the active Menu -> Colony -> Overworld -> Dungeon -> Colony loop; score carried via the existing proxy-scoring model from `docs/tech/alpha-playtest-battery-2026-05-26.md`. |
| S05_SAVE_LOAD_COLONY | save_load_continuity | 8.4 | Current reduced-runtime harness run validated Colony save-state mapping plus load recap copy. |
| S06_SAVE_LOAD_OVERWORLD | save_load_continuity | 8.4 | Current reduced-runtime harness run validated Overworld save-state mapping plus recap availability. |
| S07_SAVE_LOAD_DUNGEON | save_load_continuity | 8.4 | Current reduced-runtime harness run validated Dungeon save-state mapping plus recap availability. |
| S08_MODAL_STRESS_TOGGLE | error_edge_case_trust | 8.3 | Current reduced-runtime harness run validated critical modal priority under rapid help-toggle stress; score carried via the existing proxy-scoring model from `docs/tech/alpha-playtest-battery-2026-05-26.md`. |

## Metric Summary
| metric | threshold | current | status | evidence |
| --- | --- | --- | --- | --- |
| onboarding_clarity | >= 8.0 | 8.6 | PASS | `S01_MENU_TO_COLONY`, `docs/tech/alpha-playtest-battery-2026-05-26.md` |
| control_discoverability | >= 8.0 | 8.4 | PASS | `S02_COLONY_TO_OVERWORLD`, `tests/axt03_primary_cta.rs` |
| navigation_predictability | >= 8.0 | 8.2 | PASS | `S03_OVERWORLD_TO_DUNGEON`, `tests/runtime_flow_contracts.rs` |
| ui_hierarchy_readability | >= 7.8 | 8.2 | PASS | `src/ui/readability.rs`, readability snapshots in `src/ui/menu.rs` and `src/ui/colony_panel.rs`, `first_time_flow_readability_baseline_not_met` in `src/tests.rs`, carried-forward quantified proxy scoring from `docs/tech/alpha-playtest-battery-2026-05-26.md` |
| feedback_quality | >= 8.0 | 8.4 | PASS | `tests/axt04b_feedback_semantics.rs`, menu feedback in `tests/menu_runtime_contracts.rs`, carried-forward quantified proxy scoring from `docs/tech/alpha-playtest-battery-2026-05-26.md` |
| error_edge_case_trust | >= 8.0 | 8.3 | PASS | `S08_MODAL_STRESS_TOGGLE`, `tests/axt02_escape_semantics.rs` |
| goal_clarity_progression_cues | >= 8.2 | 8.5 | PASS | `objective_prompt`/CTA coverage, recap coverage, carried-forward quantified proxy scoring from `docs/tech/alpha-playtest-battery-2026-05-26.md` |
| overall_first_session_confidence | >= 8.0 | 8.5 | PASS | `S04_DUNGEON_TO_COLONY_RETURN`, `docs/tech/alpha-playtest-battery-2026-05-26.md` |

## Gate Notes
- ux_baseline_red_executed_tests: 11
- open_p0_count: 0 (no automated harness failures elevated to P0 in run `AXT00-20260527-REDUCED-RUNTIME-01`)
- open_p1_count: 0 (no automated harness failures elevated to P1 in run `AXT00-20260527-REDUCED-RUNTIME-01`)
- automated_scenario_evidence: captured for S01-S08 via `cargo test --test ux_baseline_red_harness -- --nocapture`
- scoring_method: criteria-based proxy scoring remains the accepted methodology for this environment because direct first-time human GUI fidelity is unavailable; current numeric values are carried forward only where the underlying evidence model remains green in the reduced runtime path
- metrics_met: true (all Alpha threshold metrics meet or exceed the current thresholds in the Metric Summary table)
- waived_defects_with_mitigation: none
