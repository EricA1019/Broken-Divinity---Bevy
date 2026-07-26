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
| Title/new game | `title_new_game` | C,Y,G,P | `bd_tui::lib::tests::title_wordmark_is_complete_and_centered_at_both_supported_profiles` | n/a | Green | Green | Green | n/a | Green | Green | Green | D-02/D-16; GDD 8 | Green unreviewed: responsive wordmark and controls pass automated and PTY checks; full snapshot review remains open |
| Clean shelter overview | `outpost_clean` | S,C,Y,G,P | `visual_scene_contract::canonical_outpost_scene_matches_all_visual_representations` | Open | Green | Open | Open | n/a | Green | Green | Open | THC-04/05; GDD 6/8 | Green unreviewed: player-presence buffer checks pass at both profiles |
| Help/context legend | `outpost_help` | S,C,Y,G,P | `bd_tui::lib::tests::rendered_outpost_help_contains_every_foundation_legend_at_supported_profiles` | Green | Green | Open | Green | n/a | Green | Green | Green | THC-05; GDD 8 | Green unreviewed: complete entries render without ellipsis at both profiles and passed PTY inspection; resolved-style review remains open |
| Build selection | `build_selection` | S,C,Y,G,T,P | `visual_transition_contract::build_selection_changes_only_authorized_visual_regions` | Open | Green | Open | Open | Open | Open | Green | Open | THC-01/06; GDD 6/8 | Green unreviewed: complete selected effect is present at 60x20 |
| Valid build placement | `build_valid` | S,C,Y,G,T,P | `bd_tui::lib::tests::build_placement_exposes_selected_station_name_cost_and_effect` | Green | Green | Open | Green | Green | Green | Green | Green | THC-04/05/06 | Green unreviewed: one transaction owns selection/cursor/submission and complete details render at both profiles; resolved-style snapshot remains open |
| Invalid build placement | `build_invalid_egress` | S,C,Y,G,T,P | `bd_tui::lib::tests::invalid_build_preview_explains_egress_rejection`; `phase6_input::invalid_build_confirmation_keeps_preview_active_and_is_atomic` | Green | Green | Open | Open | Green | Green | Green | Partial | THC-04/05/06 | Green unreviewed: invalid preview remains correctable and atomic at both automated profiles; compact PTY denial observation remains open |
| Survivor task management | `task_management` | S,C,Y,G,T,P | `phase6_input::task_management_lists_survivor_tasks_not_station_staffing_choices`; management render tests | Green | Green | Open | Green | Green | Green | Green | Green | THC-01/02/06 | Green unreviewed: paused task-only reducer and complete modal pass both profiles; resolved-style snapshots remain open |
| Station staffing | `station_staffing` | S,C,Y,G,T,P | `phase6_input::station_staffing_lists_station_assignments_not_gathering_tasks`; staffing render tests | Green | Green | Open | Green | Green | Green | Green | Green | THC-01/03/06 | Green unreviewed: station-only choices, distinct title/cancel key, and staffed glyph pass; resolved-style snapshots remain open |
| Worker idle→en-route→working | `worker_progression` | S,C,Y,T,P | `survivor_work_contract`; `test_harness_contract` | Green | Open | Open | n/a | Open | Open | Open | Partial | THC-02/03/05 | Domain Green; complete activity canvas/style transition evidence remains open |
| Worker blocked | `worker_blocked` | S,C,Y,G,P | `survivor_work_contract::unreachable_worker_stays_put_and_reports_a_specific_blocked_reason` | Green | Open | Open | Open | n/a | Open | Open | Open | THC-02/03/05 | Domain Green with transition-only specific feedback; visual scene evidence remains open |
| Off-screen assigned target | `target_offscreen` | S,C,Y,G,T,P | `bd_tui::lib::tests::assigned_offscreen_target_has_a_directional_edge_indicator` | Green | Green | Open | Green | Green | Green | Green | Partial | THC-04/05 | Green unreviewed: directional edge indicator passes both buffers and compact PTY; resolved style and baseline PTY remain open |
| Adverse/zero-Supplies state | `outpost_zero_supplies` | S,C,Y,G,P | `visual_scene_contract::zero_supplies_scene_exposes_recovery_action` | Open | Open | Open | Open | n/a | Open | Open | Open | D-17; GDD 6 | Not implemented |
| Day summary | `day_summary` | S,C,Y,G,P | `visual_scene_contract::day_summary_matches_authoritative_delta` | Open | Open | Open | Open | n/a | Open | Open | Open | THC-03/06; GDD 6 | Not implemented |
| Save/load success and failure | `persistence_feedback` | C,Y,G,T,P | `visual_transition_contract::persistence_feedback_preserves_coherent_screen` | n/a | Open | Open | Open | Open | Open | Open | Open | D-09/D-16; GDD 8 | Not implemented |
| Dungeon exploration/combat/loot | `dungeon_core_loop` | S,C,Y,G,T,P | `visual_transition_contract::dungeon_core_loop_matches_visual_contract` | Open | Open | Open | Open | Open | Open | Open | Open | D-01/D-02; GDD 3/8 | Not implemented |
| Extraction and game over | `terminal_outcomes` | C,Y,G,T,P | `visual_transition_contract::terminal_outcomes_expose_complete_next_action` | n/a | Open | Open | Open | Open | Open | Open | Open | D-08/D-16; GDD 3/8 | Not implemented |

## Direct visual invariants

| Invariant | Exact test | Owning phase | Status |
|---|---|---:|---|
| Foreground-only differences are observable | `style_observation_detects_foreground_only_change` | 4 | Not implemented |
| Modifier-only differences are observable | `style_observation_detects_modifier_only_change` | 4 | Not implemented |
| Layer/priority differences are observable | `semantic_observation_detects_layer_or_priority_change` | 4 | Not implemented |
| Panel overlap is rejected | `geometry_observation_detects_panel_overlap` | 4 | Not implemented |
| Closed modals leave no stale cell | `transition_observation_detects_stale_cell_after_modal_close` | 4 | Not implemented |
| Player appears exactly once inside viewport | `player_cell_is_present_exactly_once` | 4/5 | Not implemented |
| Rendering identical state is deterministic | `rendering_same_state_twice_produces_identical_visual_observation` | 4 | Not implemented |
| Fixture construction ignores ECS query order | `fixture_identity_does_not_depend_on_ecs_query_order` | 4 | Not implemented |
| Viewport pan preserves relative world positions | `viewport_pan_preserves_world_entity_relative_positions` | 5 | Not implemented |
| Resize round-trip leaves no stale cells | `resize_round_trip_leaves_no_stale_cells` | 4/5 | Not implemented |
| Every entity is visible or represented off-screen | `all_visible_entities_have_one_cell_or_one_offscreen_indicator` | 5 | Not implemented |
| Invalid placement remains distinct without color | `invalid_build_preview_is_distinct_in_ascii_fallback` | 6/8 | Not implemented |
| Worker presentation agrees with authoritative activity | `worker_visual_state_matches_authoritative_activity` | 7 | Not implemented |
| Survivor collision never removes a visible worker | `survivor_collision_never_removes_a_visible_worker` | 7/8 | Not implemented |
| Simultaneous categories have unique glyph/style pairs | `glyph_style_pairs_are_unique_for_simultaneously_visible_categories` | 8 | Green unreviewed: station and resource resolved styles differ at 80x24 |
| Fallback symbols remain unambiguous | Altar/Idle and Workshop/Water production projection tests plus loader collision tests | 8 | Green unreviewed: locked active pairs and validation pass; full visual-scene snapshot remains open |
| Staffed and unstaffed stations remain distinct without color | `staffed_and_unstaffed_station_have_distinct_ascii_projection` | 8 | Green unreviewed: explicit catalog-owned lowercase/uppercase states pass |
| Required detail never ends mid-word | `required_detail_text_never_ends_mid_word` | 4/9 | Not implemented |
| Save/load preserves projected visual state | `save_load_same_fingerprint_produces_same_visual_scene` | 7/10 | Not implemented |

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
