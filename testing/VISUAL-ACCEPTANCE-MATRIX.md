# Foundation Visual Acceptance Matrix

Status: Active; repaired automated contracts remain unaccepted pending required evidence
Authority: `../docs/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md`, Sections 5 and 7–17
Profiles: 80x24 baseline and 60x20 compact

Evidence codes:

- **S:** semantic projection
- **C:** plain terminal canvas
- **Y:** resolved style cells
- **G:** layout geometry
- **T:** before/after transition
- **P:** real PTY inspection

An `Accepted` row requires every listed evidence type at both listed profiles.
Substring checks are not snapshots and cannot close a row.

## Scene ledger

| Scene | Fixture ID | Required | Test target / exact test | S | C | Y | G | T | 80x24 | 60x20 | PTY | Authority | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Title/new game | `title_new_game` | C,Y,G,P | `ui_development_contract_tests::title_without_save_explains_why_load_is_unavailable`; responsive wordmark tests | n/a | Green | Green | Green | n/a | Green | Green | Green | D-02/D-16; GDD 8 | Green unreviewed: no-save availability and responsive wordmark pass; owner acceptance remains open |
| Clean shelter overview | `outpost_clean` | S,C,Y,G,P | `ui_development_contract_tests::outpost_map_is_the_largest_interactive_panel_at_supported_profiles` | Open | Green | Open | Green | n/a | Green | Green | Green | THC-04/05; GDD 6/8 | Green unreviewed: map hierarchy and non-overlap geometry pass at both profiles |
| Help/context legend | `outpost_help` | S,C,Y,G,P | `bd_tui::lib::tests::rendered_outpost_help_contains_every_foundation_legend_at_supported_profiles` | Green | Green | Open | Green | n/a | Green | Green | Green | THC-05; GDD 8 | Green unreviewed: complete entries render without ellipsis at both profiles and passed PTY inspection; resolved-style review remains open |
| Build selection | `build_selection` | S,C,Y,G,T,P | `ui_development_contract_tests::unaffordable_build_selection_explains_the_exact_shortage`; existing catalog tests | Green | Green | Open | Open | Open | Green | Green | Green | THC-01/06; GDD 6/8 | Green unreviewed: selected effect and exact shortage fit both profiles |
| Valid build placement | `build_valid` | S,C,Y,G,T,P | `bd_tui::lib::tests::build_placement_exposes_selected_station_name_cost_and_effect` | Green | Green | Open | Green | Green | Green | Green | Green | THC-04/05/06 | Green unreviewed: one transaction owns selection/cursor/submission and complete details render at both profiles; resolved-style snapshot remains open |
| Invalid build placement | `build_invalid_egress` | S,C,Y,G,T,P | `ui_development_contract_tests::valid_and_invalid_build_previews_differ_without_color`; atomic placement tests | Green | Green | Green | Open | Green | Green | Green | Green | THC-04/05/06 | Green unreviewed: denial, atomic correction, and color-independent preview distinction pass |
| Survivor task management | `task_management` | S,C,Y,G,T,P | `ui_development_contract_tests::task_management_exposes_survivor_task_and_confirm_stages`; paused-management tests | Green | Green | Open | Green | Green | Green | Green | Green | THC-01/02/06 | Green unreviewed: complete staged path, controls, and pause behavior pass |
| Station staffing | `station_staffing` | S,C,Y,G,T,P | `ui_development_contract_tests::station_staffing_exposes_survivor_station_recipe_and_confirm_stages`; paused-staffing tests | Green | Green | Open | Green | Green | Green | Green | Green | THC-01/03/06 | Green unreviewed: complete staged path, controls, and pause behavior pass |
| Worker idle→en-route→working | `worker_progression` | S,C,Y,T,P | `survivor_work_contract`; `test_harness_contract`; colony production route and projection contracts | Green | Green | Open | n/a | Green | Green | Green | Green | THC-02/03/05; D-20 | Green unreviewed: assignment, EnRoute, Working, cargo, refine, and completion pass both buffers and PTY; resolved-style evidence remains open |
| Worker blocked | `worker_blocked` | S,C,Y,G,P | `ui_development_contract_tests::blocked_worker_has_a_distinct_resolved_style_from_working_worker`; blocked projection tests | Green | Green | Green | Green | n/a | Green | Green | Open | THC-02/03/05; D-22 | Green unreviewed: target, reason, and distinct danger style pass |
| Direct gather progress | `direct_gather_progress` | S,C,Y,G,T,P | `phase6_input::direct_gather_assignment_projects_source_and_three_tick_progress`; `bd_tui::lib::tests::direct_gather_progress_raw_stockpile_and_split_forecast_fit_supported_profiles` | Green | Green | Open | Green | Green | Green | Green | Open | D-22; GDD Minimum colony foundation | Green unreviewed: zero-of-three and one-of-three progression fits both automated profiles; resolved-style and PTY review remain open |
| Human colony work labels | `colony_work_labels` | S,C,Y,G,P | `phase6_input::recipe_management_uses_human_resource_labels_not_content_ids`; worker layout test | Green | Green | Open | Green | n/a | Green | Green | Open | D-22 | Green unreviewed: recipe and resource IDs are replaced by human labels at both automated profiles; resolved-style and PTY review remain open |
| Worker/day forecast split | `colony_forecast_split` | S,C,Y,G,P | `phase6_input::colony_projection_separates_next_worker_result_from_next_day_upkeep`; `bd_tui::lib::tests::direct_gather_progress_raw_stockpile_and_split_forecast_fit_supported_profiles` | Green | Green | Open | Green | n/a | Green | Green | Open | D-22 | Green unreviewed: worker completion and day upkeep are distinct and fit both profiles; resolved-style and PTY review remain open |
| Raw stockpile | `colony_raw_stockpile` | S,C,Y,G,P | `phase6_input::nonzero_raw_stockpile_is_projected_with_a_human_label`; `bd_tui::lib::tests::direct_gather_progress_raw_stockpile_and_split_forecast_fit_supported_profiles` | Green | Green | Open | Green | n/a | Green | Green | Open | D-22 | Green unreviewed: nonzero raw stockpiles use human labels and fit both profiles; resolved-style and PTY review remain open |
| Off-screen assigned target | `target_offscreen` | S,C,Y,G,T,P | `ui_development_contract_tests::offscreen_assignment_names_target_and_distance_at_supported_profiles` | Green | Green | Open | Green | Green | Green | Green | Partial | THC-04/05 | Green unreviewed: direction, target identity, and numeric distance pass both profiles |
| Adverse/zero-Supplies state | `outpost_zero_supplies` | S,C,Y,G,P | `phase6_input::zero_supplies_overview_exposes_a_reachable_gathering_recovery_path` | Green | Open | Open | Open | n/a | Open | Open | Open | D-17/D-22; GDD 6 | Partial: semantic Travel denial and reachable Gather Supplies recovery pass; full rendered scene evidence remains open |
| Day summary | `day_summary` | S,C,Y,G,P | `bd_tui::lib::tests::day_summary_keeps_authoritative_deltas_visible_at_supported_profiles` | Green | Green | Open | Green | n/a | Green | Green | Open | THC-03/06; GDD 6 | Green unreviewed: a dedicated structured Day result keeps Supplies, Materials, Plants, Faith, and Food visible at both profiles; resolved style and PTY remain open |
| Decisive warning under routine traffic | `decisive_warning_routine_overflow` | S,C,Y,G,P | `ui_development_contract_tests::decisive_warning_survives_routine_log_overflow` | Green | Green | Open | Green | n/a | Green | Green | Open | UI-04; GDD 3/6 | Green unreviewed: latest decisive warning survives routine overflow |
| Save/load success and failure | `persistence_feedback` | C,Y,G,T,P | `visual_transition_contract::persistence_feedback_preserves_coherent_screen` | n/a | Open | Open | Open | Open | Open | Open | Partial | D-09/D-16; GDD 8 | PTY success/carrying continuation passes both profiles; failure and automated visual-transition evidence remain open |
| Dungeon exploration/combat/loot | `dungeon_core_loop` | S,C,Y,G,T,P | `ui_development_contract_tests::dungeon_status_distinguishes_carried_loot_and_extraction_readiness`; production-key workflow | Green | Green | Open | Open | Green | Green | Green | Green | D-01/D-02/D-10; GDD 3/8 | Green unreviewed: action denials, carried loot, and extraction readiness render from authoritative state |
| Extraction and game over | `terminal_outcomes` | C,Y,G,T,P | `visual_transition_contract::terminal_outcomes_expose_complete_next_action` | n/a | Open | Open | Open | Open | Open | Open | Partial | D-08/D-16; GDD 3/8 | Extraction and colony return passed both C8 PTY profiles; Game Over and automated visual-transition evidence remain open |

## Direct visual invariants

| Invariant | Exact test | Owning phase | Status |
|---|---|---:|---|
| Foreground-only differences are observable | `visual_observation_detects_glyph_foreground_modifier_and_geometry_changes` | UI0 | Green unreviewed |
| Modifier-only differences are observable | `visual_observation_detects_glyph_foreground_modifier_and_geometry_changes` | UI0 | Green unreviewed |
| Layer/priority differences are observable | `semantic_observation_detects_layer_or_priority_change` | 4 | Not implemented |
| Panel overlap is rejected | `supported_outpost_panel_rectangles_never_overlap` | UI0/UI2 | Green unreviewed |
| Closed modals leave no stale cell | `closed_management_modal_leaves_the_same_canvas_as_a_clean_overview` | UI0/UI7 | Green unreviewed |
| Player appears exactly once inside viewport | `player_cell_is_present_exactly_once` | 4/5 | Not implemented |
| Rendering identical state is deterministic | `identical_fixture_has_identical_canvas_and_resolved_styles` | UI0 | Green unreviewed |
| Fixture construction ignores ECS query order | `fixture_identity_does_not_depend_on_ecs_query_order` | 4 | Not implemented |
| Viewport pan preserves relative world positions | `screens::tests::viewport_pan_preserves_relative_world_positions` | 5 | Green unreviewed: edge/center cases pass at both production-derived map sizes |
| Resize round-trip leaves no stale cells | `resize_round_trip_returns_to_the_original_canvas_and_styles` | UI0/UI7 | Green unreviewed: buffer round trip passes; real PTY resize remains |
| Every entity is visible or represented off-screen | `all_visible_entities_have_one_cell_or_one_offscreen_indicator` | 5 | Not implemented |
| Invalid placement remains distinct without color | `valid_and_invalid_build_previews_differ_without_color` | UI1/UI3 | Green unreviewed: invalid placement uses a distinct danger glyph and style |
| Worker presentation agrees with authoritative activity | `worker_visual_state_matches_authoritative_activity` | 7 | Not implemented |
| Survivor collision never removes a visible worker | `survivor_collision_never_removes_a_visible_worker` | 7/8 | Not implemented |
| Simultaneous categories have unique glyph/style pairs | `glyph_style_pairs_are_unique_for_simultaneously_visible_categories` | 8 | Green unreviewed: station and resource resolved styles differ at 80x24 |
| Fallback symbols remain unambiguous | Altar/Idle and Workshop/Water production projection tests plus loader collision tests | 8 | Green unreviewed: locked active pairs and validation pass; full visual-scene snapshot remains open |
| Staffed and unstaffed stations remain distinct without color | `staffed_and_unstaffed_station_have_distinct_ascii_projection` | 8 | Green unreviewed: explicit catalog-owned lowercase/uppercase states pass |
| Required detail never ends mid-word | `required_detail_text_never_ends_mid_word` | 4/9 | Not implemented |
| Save/load preserves projected visual state | `phase6_input::colony_checkpoint_round_trip_preserves_the_visible_projection`; Tactical counterpart | 7/10 | Partial: stable screen/stats/map/actions/log projections match; canvas and resolved-style observations remain open |

## Snapshot review record

No snapshots are accepted yet. For each future change, append:

```text
Date:
Fixture/profile:
Snapshot names:
Contract intentionally changed:
Unexpected regions changed: no/yes (explain):
Reviewer:
Result:
```

Never bulk-accept pending snapshots. Canvas, style, semantic, and geometry
snapshots are reviewed separately.

## Red acceptance evidence — 2026-07-26

Command:

```bash
cargo test -p bd_tui --lib -- --test-threads=1
```

Result: 45 passed and 8 failed. Compilation and test harness execution
succeeded. The intentional failures are:

| Test | Confirmed failure |
|---|---|
| `outpost_80x24_viewport_keeps_player_visible_at_far_shelter_edge` | Player count is zero at `(38,28)` |
| `outpost_60x20_viewport_keeps_player_visible_at_far_shelter_edge` | Player count is zero at `(38,28)` |
| `compact_viewport_projects_resource_next_to_far_edge_player` | Adjacent resource is absent with the far-edge player |
| `station_and_resource_cells_have_distinct_resolved_styles` | Both categories resolve to identical Cyan style |
| `compact_build_selection_shows_complete_selected_effect` | Selected effect is clipped mid-sentence |
| `invalid_build_preview_explains_egress_rejection` | Preview shows controls but no rejection reason |
| `station_staffing_uses_a_distinct_modal_title` | Staffing renders as `Colony Management` |
| `outpost_help_explains_visible_resource_glyphs` | Help omits Trees, Water Source, and Wild Plants |

This table is historical before-state evidence. The tests remained active and
were repaired without ignoring them. Current automated results are recorded
below; no row is accepted from substring evidence alone.

## Automated repair evidence — 2026-07-26

Command:

```bash
cargo test -p bd_tui --lib -- --test-threads=1
```

Result: 53 passed, 0 failed, 0 ignored. The eight formerly red tests are green.
The corresponding seven registry contracts are `GreenUnreviewed`.

This does not change any scene row to `Accepted`. Required semantic, canvas,
resolved-style, geometry, transition, terminal-profile, and PTY observations
remain open wherever the scene ledger says `Open`; those cells must be updated
only from evidence at that exact layer.

## Responsive title repair evidence — 2026-07-26

- Replaced the malformed hand-authored ASCII logo with one responsive
  Ratatui-centered wordmark and text hierarchy.
- `title_wordmark_is_complete_and_centered_at_both_supported_profiles` proves
  one complete `BROKEN DIVINITY` wordmark with exact horizontal centering at
  80x24 and 60x20.
- `title_wordmark_has_distinct_accent_style_at_both_supported_profiles`
  verifies the resolved Cyan and bold title style at both profiles.
- The existing title contract continues to prove the begin prompt, load and
  quit controls, and absence of out-of-context movement controls.
- Real PTY inspection at both profiles showed the complete centered wordmark,
  `FOUNDATION BUILD`, begin prompt, version, status, and title controls without
  clipping or malformed fragments.
- The row remains `Green unreviewed`, not `Accepted`, until its complete
  canvas/style snapshot receives the review required by this matrix.

## Holistic GDD sweep evidence — 2026-07-26

- `rendered_outpost_help_contains_every_foundation_legend_at_supported_profiles`
  proves a semantic/rendering disagreement: all resource entries exist in the
  Help model, but none reaches the visible 80x24 canvas.
- `build_placement_exposes_selected_station_name_cost_and_effect` fails at the
  first missing detail because the placement banner contains controls only.
- Production-input contracts prove the default preview is not adjacent and an
  invalid confirmation closes the preview.
- Production projection contracts prove monochrome collisions for
  Altar/survivor and Workshop/Water Source, and prove staffing does not alter a
  station's ASCII state.
- These rows are Red, not snapshot candidates. Do not accept or regenerate
  visual evidence until the structural and semantic failures are repaired.
