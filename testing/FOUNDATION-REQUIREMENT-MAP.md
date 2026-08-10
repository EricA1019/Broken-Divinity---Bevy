# Broken Divinity Foundation Requirement-to-Test Map

**Status:** Active evidence index; this file is not design authority
**Authority:** `../GDD.md`, `../docs/authority/DECISIONS-TO-LOCK.md`,
`../docs/authority/MVP-SCENARIO.md`, and
`../docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md`
**Scope:** The locked Foundation only. Product P2/P3 and explicitly deferred
systems do not create active Foundation acceptance failures.

## Purpose

This map prevents test count from being mistaken for product coverage. Each
row owns one player outcome or one narrow invariant. A broad scenario may
support a row, but it does not replace missing atomic evidence.

Status meanings:

- **Green:** the named automated evidence currently passes.
- **Green unreviewed:** automated evidence passes, but required visual,
  compact-profile, or PTY review is incomplete.
- **Partial:** some layers pass, but the player outcome is not fully proved.
- **Red:** an active test reproduces a confirmed implementation defect.
- **Open:** the authoritative contract still needs a primary test.
- **Manual:** the contract requires real-terminal evidence and must not be
  inferred from a headless test.
- **Deferred:** excluded from Foundation by the GDD or a locked decision.

## 1. Shell and lifecycle

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| New Game reaches Outpost with one player authority | `foundation_scenario::clean_launch_reaches_colony`; `bd_app::tests::application_startup_has_exactly_one_player_authority` | Green | Production-key/title transition remains supporting evidence rather than the primary owner |
| Title Load does not start a new run | `application_tests::missing_title_load_is_atomic_visible_and_recoverable_by_new_game`; corrupt-save counterpart | Green | Physical F9 preserves the title run for both missing and corrupt slots |
| Missing/corrupt save feedback is recoverable | `application_tests` missing/corrupt load contracts; `bd_tui::lib::tests::title_displays_persistence_failures_at_both_supported_profiles` | Green unreviewed | Application recovery and both supported buffers pass; real-PTY failure review remains open |
| Quit requests one shutdown | `application_tests::quit_key_emits_exactly_one_application_exit` | Partial | Exactly one production exit passes; alternate-screen/cursor restoration remains the manual PTY gate |
| Alternate screen is restored | real PTY gate | Manual | Recorded PTY evidence at both profiles |
| Cursor is restored | real PTY gate | Manual | Recorded PTY evidence at both profiles |
| Stable idle UI does not redraw | `runtime_control::idle_unchanged_ui_does_not_draw_again` | Green | None at the invalidation layer |
| Resize redraws once and leaves no stale cells | no primary contract | Open | 60x20→80x24→60x20 buffer transition and PTY resize |

## 2. Input and controls

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| Configured bindings drive Help, footer, and action projection | `input_help::configured_binding_emits_expected_command`; related binding tests | Green | None at the semantic command layer |
| Every advertised Foundation control works in its advertised mode | `input_help::every_advertised_control_resolves_to_its_declared_command_in_context`; focused production workflows | Partial | All guidance routes exactly; a table-driven end-effect matrix for every command remains open |
| Unadvertised controls do not mutate gameplay | `input_help::foundation_controls_do_not_advertise_redundant_z_command` | Partial | Production-key state diff for unbound keys in each mode |
| Press/Repeat/Release policy is explicit | `press_repeat_release_policy::only_physical_press_mutates_exactly_once_for_every_foundation_control`; `phase6_input::build_interaction_is_a_paused_press_only_state_machine` | Green | Repeat and Release are inert and Press acts exactly once for movement, wait, rest, management, Help, Inventory, and quit; F5/F9 persistence remains the app-boundary PTY gate |
| Buffered input preserves order | `phase6_input::buffered_semantic_commands_resolve_in_input_order` | Green | None for the bounded gameplay queue |
| Queue overflow is bounded and visible | `phase6_input::buffered_input_is_bounded_and_reports_one_overflow_warning` | Green | None for the current capacity |
| First Outpost movement key cannot enter Build | `phase6_input::first_outpost_move_key_moves_once_without_opening_or_creating_build_state` | Green unreviewed | Production-key state diff is exact; final PTY observation remains supporting evidence |
| Modal input never leaks | `phase6_input::management_cancel_is_atomic_and_discards_modal_gameplay_input` | Green | Same-batch routing predicts modal ownership and discards uncommitted input |
| Footer, Help, action panel, configuration, and runtime agree | `input_help::every_advertised_control_resolves_to_its_declared_command_in_context`; configured-binding projection tests | Green | Every advertised command has one configured key and resolves to itself in Title, Outpost, Tactical, Build, and Game Over |

## 3. Build workflow

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| Opening, selection, navigation, and cancellation are paused | `phase6_input::build_interaction_is_a_paused_press_only_state_machine` | Green | Selection-navigation visual transition |
| Selection shows complete cost/effect/availability | `bd_tui::lib::tests::compact_build_selection_shows_complete_selected_effect`; existing selection layout tests | Green unreviewed | Complete visual evidence and PTY review |
| Placement retains selected name, cost, and effect | `bd_tui::lib::tests::build_placement_exposes_selected_station_name_cost_and_effect` | Green unreviewed | Automated profile proof passes; final PTY observation remains required |
| Initial placement preview and confirmation target agree | `phase6_input::entering_build_placement_starts_on_a_visible_adjacent_candidate` | Green | One `BuildInteraction` owns the selected station, visible cursor, validation, and submitted target |
| Placement cursor moves cumulatively without moving the player | `phase6_input::build_placement_cursor_moves_cumulatively_without_moving_the_player` | Green | None |
| Distant confirmation places at the absolute preview coordinate | `phase6_input::distant_build_confirmation_places_at_the_absolute_preview_coordinate` | Green | None |
| Valid and invalid previews differ semantically | `bd_tui::lib::tests::invalid_build_preview_explains_egress_rejection` | Partial | Semantic token, style, ASCII fallback, and transition evidence |
| Invalid preview exposes a typed reason | `invalid_build_preview_explains_egress_rejection` | Green unreviewed | Compact profile and PTY review |
| Invalid confirmation is atomic and remains correctable | `phase6_input::invalid_build_confirmation_keeps_preview_active_and_is_atomic`; `denied_build_resolution_returns_to_correctable_placement` | Green | Preview rejection and later core denial both preserve a correctable `Placing` transaction without payment/time/entity mutation |
| Accepted build pays and advances exactly once | `foundation_stabilization::construction_deducts_authoritative_colony_supplies_once` | Green | None at the domain/action layer |
| Accepted placement preserves gate reachability | `colony_spatial_contract::every_accepted_station_placement_preserves_gate_reachability` | Green unreviewed | Existing-station blocker property matrix remains supporting unit evidence |
| Accepted placement retains one reachable adjacent work tile | `bd_core::colony::stations::tests::placement_rejects_station_without_a_reachable_adjacent_work_tile` | Green | None |

## 4. Task management and station staffing

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| `c` and `e` open distinct paused modes | `phase6_input::c_opens_paused_task_management_with_task_identity`; `e_opens_paused_station_staffing_with_station_identity` | Green unreviewed | Rendered transition evidence at both profiles |
| Each mode exposes only choices for its responsibility | `station_staffing_lists_station_assignments_not_gathering_tasks`; `task_management_lists_survivor_tasks_not_station_staffing_choices` | Green | None |
| Named stable survivor selection | `station_staffing_confirmation_changes_only_the_named_survivor_relationship` | Green | Survivors are ordered by stable unique name |
| Named stable station selection | same staffing confirmation contract | Green | Stations are ordered by label and physical position rather than entity bits |
| Open/navigation/confirmation/cancellation are paused | opening tests and existing confirmation test | Partial | Discrete navigation and Repeat/Release tests |
| Confirmation changes only the intended relationship | `station_staffing_confirmation_changes_only_the_named_survivor_relationship` | Green | Equivalent task-assignment state-diff contract |
| Cancellation is atomic and does not leak input | `management_cancel_is_atomic_and_discards_modal_gameplay_input` | Green | None |
| Assignment does not immediately move workers | `survivor_work_contract::new_assignment_does_not_move_during_paused_confirmation` | Green | None |
| Feedback names survivor, target, and resulting activity | `task_confirmation_emits_one_named_target_and_activity_result`; station counterpart | Green | One post-derivation result names the survivor, human assignment, physical target, and authoritative activity |
| Modal/footer controls agree at both profiles | `bd_tui::lib::tests::management_modal_and_footer_controls_agree_at_supported_profiles` | Green unreviewed | Both buffers show selection/confirm/cancel only; final PTY review remains |

## 5. Viewport and visual language

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| Player remains visible at supported profiles | far-edge buffer tests; `screens::tests::every_shelter_position_projects_inside_supported_viewports` | Green unreviewed | Every coordinate projects and rendered edge cases pass; exact-one-player all-position canvas proof and PTY review remain |
| Build cursor remains visible at supported profiles | `bd_tui::lib::tests::distant_build_preview_drives_the_viewport_at_both_supported_profiles` | Green unreviewed | Final PTY observation remains required |
| Every layer uses one viewport transform | far-edge tests plus `map_projection_uses_one_semantic_visual_list` | Green unreviewed | One semantic visual list and one viewport projection path are implemented; full layer-by-layer snapshot review remains |
| Assigned off-screen targets remain discoverable | `bd_tui::lib::tests::assigned_offscreen_target_has_a_directional_edge_indicator` | Green unreviewed | Direction is proven at both profiles; target-name/distance aggregation remains a later polish gap |
| Active categories have a symbol/style/legend | semantic Help, symbol-registry, station-catalog, and resolved-style tests | Green unreviewed | Invalid-placement resolved-style snapshot remains open |
| Station/resource categories are distinct | `station_and_resource_cells_have_distinct_resolved_styles` | Green unreviewed | ASCII fallback remains red |
| Simultaneous categories remain distinct without color | Altar/survivor and Workshop/water tests; loader collision validation | Green unreviewed | Automated monochrome and validation proof passes; PTY review remains |
| Staffed and unstaffed stations are distinct | `staffed_and_unstaffed_station_have_distinct_ascii_projection` | Green unreviewed | Automated ASCII proof passes; PTY review remains |
| Worker activities are distinct | no visual primary contract | Open | Idle/EnRoute/Working/Blocked semantic and resolved observations |
| Player/survivor cannot be hidden by lower layers | no primary contract | Open | Co-location/layer matrix and illegal-overlap guard |
| Compact decisive text remains complete | compact build and management layout tests | Partial | Full canonical scene matrix and no-mid-word invariant |
| Rendered Help contains its complete legend | `rendered_outpost_help_contains_every_foundation_legend_at_supported_profiles` | Green unreviewed | Complete no-ellipsis buffers pass at 80x24 and 60x20; PTY review remains |
| Same state renders deterministically | visible frame fingerprint test | Partial | Full semantic/canvas/style observation equality |
| Modal close and resize leave no stale cells | no primary contract | Open | Before/after buffer diff |

## 6. Worker movement and physical production

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| Idle workers do not move | `idle_render_frames_do_not_move_assigned_survivors`; `idle_survivor_does_not_move_on_accepted_outpost_turns` | Green | None |
| Assignment becomes EnRoute without movement | `new_assignment_does_not_move_during_paused_confirmation` | Green | `WorkerActivity::EnRoute` is derived without moving during paused confirmation |
| One Outpost turn permits one cardinal step | `next_outpost_turn_moves_worker_exactly_one_cardinal_step` | Green | None |
| Tactical turns do not move colony workers | `tactical_turns_do_not_move_colony_survivors` | Green | None |
| Workers path around blockers | `worker_uses_pathfinding_around_a_wall_blocker` | Green | Existing A* adapter selects deterministic adjacent work paths |
| Reservations prevent stacking | `assigned_survivors_never_stack_on_one_tile` | Green | Stable-name movement order reserves each accepted destination |
| Workers stop adjacent to blocking targets | station/resource occupancy tests | Green | Fixtures remain blocking and workers stop cardinally adjacent |
| Arrival becomes Working | `station_worker_stops_cardinally_adjacent_to_target`; fingerprint activity checks | Green | Typed activity and physical evaluator agree |
| No route becomes Blocked with a specific reason | `unreachable_worker_stays_put_and_reports_a_specific_blocked_reason` | Green | Typed reason and transition-only feedback pass |
| EnRoute/Blocked station workers produce zero | `assigned_but_enroute_station_worker_produces_nothing`; `blocked_station_worker_produces_nothing` | Green | Shared physical-work evaluator rejects remote/blocked work |
| Adjacent station worker produces once | `adjacent_station_worker_produces_once` | Green | Shared physical-work evaluator credits once |
| EnRoute/Blocked gatherers produce zero | `assigned_but_enroute_gatherer_produces_nothing` | Green | Shared physical-work evaluator rejects remote gathering |
| Matching adjacent gatherer produces after configured work | `direct_gather_requires_three_work_ticks_and_credits_once` | Green | None |
| Wrong-node gatherer produces zero | `gatherer_at_wrong_node_type_produces_nothing` | Green | The adjacent physical node must match the durable task |
| Blocked station worker produces zero | `blocked_station_worker_produces_nothing` | Green | None |
| Rest equals equivalent individual turns | position and daily-resource equivalence tests | Green | Rest replays the same logical worker steps before each crossed day boundary |
| Save/load preserves deterministic next step | next-step, no-immediate-work, and fingerprint tests | Green unreviewed | Durable and derived state equality pass; complete visual snapshot review remains |
| Foundation source/recipe chains are data-defined and cross-referenced | `bd_data::loader::tests::foundation_colony_chains_are_complete_and_cross_referenced` | Green | Invalid references and non-positive amounts have narrow rejection tests |
| New-colony source counts come from content | `colony_node_generation_contract::configured_source_counts_own_new_colony_node_coverage` | Green | None |
| Node layouts are deterministic, separated, legal, and reachable | node spatial and seed contracts | Green | 128-seed and content-order profiles pass; both C8 PTY runs showed the generated colony fixtures |
| Save/load preserves generated source identity and coordinates | `colony_node_generation_contract::persisted_node_layout_is_restored_without_regeneration` | Green | None |
| A worker completes the timber source-to-processing route | `colony_production_route_contract::one_survivor_completes_the_pilot_source_to_station_route` | Green | Player-facing assignment and projection remain C6 work |
| Gather/refine transitions conserve raw input and finished output | transition-matrix and conservation contracts | Green | Same matrix must cover every D-20 recipe in C5 |
| Carrying save/load preserves recipe, stage, and cargo | `colony_production_route_contract::carrying_checkpoint_preserves_recipe_stage_and_raw_cargo` | Green | Every stage and next-tick continuation pass; carrying continuity passed the 80x24 PTY run |
| All configured recipes share one transition implementation | `colony_production_route_contract::every_configured_recipe_obeys_the_same_gather_and_refine_transition` | Green | A fixture-only fourth chain also passes without a gameplay branch |
| Concurrent different-chain workers do not stack or duplicate output | `colony_production_route_contract::two_survivors_complete_different_chains_without_stacking_or_duplicate_credit` | Green | Sole-work-tile contention also passes in C7 |
| Cancellation/reassignment preserves carried raw input | `colony_production_route_contract::reassigning_a_carrying_worker_deposits_raw_cargo_and_cancels_logistics` | Green | Raw cargo deposits atomically into the persisted `ColonyResources` owner |
| `e` assigns a named survivor to a named processor recipe while paused | `phase6_input::processing_assignment_selects_named_survivor_station_and_recipe_while_paused` | Green unreviewed | Passed at 80x24 and 60x20 PTY; owner acceptance remains |
| Worker recipe, stage, target/activity, and cargo remain visible | `bd_tui::lib::tests::colony_worker_recipe_stage_target_and_cargo_are_visible_at_supported_profiles` | Green unreviewed | Travel, gather, cargo, refine, completion, and load continuation passed both PTY profiles; owner acceptance remains |
| Production keys complete one source-to-station cycle | `phase6_input::production_key_workflow_assigns_travels_gathers_refines_and_reports` | Green unreviewed | Complete real-terminal workflow passed both profiles; owner acceptance remains |

## 7. Economy and day transaction

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| Every day boundary runs one transaction | `colony_day_cycle::day_advanced_emits_once`; `mvp_correction::every_legal_day_boundary_has_one_summary` | Green | None |
| Tactical and Outpost day boundaries agree | tactical day-boundary tests | Green | Exact normalized state equality would strengthen this |
| Food, station output, mood, and summary run once | `colony_day_cycle` and `mvp_correction` matrices | Green | Direct worker output is intentionally owned by accepted Outpost ticks, not this transaction |
| Summary equals authoritative delta | `colony_day_cycle::daily_summary_matches_resource_delta` | Green | Physical activity fields remain absent |
| Authoritative daily deltas remain visible | `bd_tui::lib::tests::day_summary_keeps_authoritative_deltas_visible_at_supported_profiles` | Green unreviewed | A structured Day result preserves all five deltas at both profiles; final PTY review remains |
| Next-day forecast excludes direct worker output | `mvp_correction::next_day_forecast_excludes_direct_worker_tick_output` | Green | None |
| Next worker completion and next-day upkeep are distinct | `phase6_input::colony_projection_separates_next_worker_result_from_next_day_upkeep` | Green unreviewed | Real-PTY visual review remains open |
| Zero-Supplies recovery is reachable | direct-gather recovery tests; `phase6_input::zero_supplies_overview_exposes_a_reachable_gathering_recovery_path` | Green unreviewed | Domain recovery and semantic guidance pass; complete rendered-scene and real-PTY review remain open |
| Storage rejects before payment | `mvp_correction::disabled_storage_rejection_is_atomic` | Green | None |
| Station catalog owns costs/effects/availability | catalog and loader tests | Green | Presentation still drops placement details |

## 8. Fixed dungeon loop

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| Entry costs exactly two Supplies | `foundation_stabilization::dungeon_entry_deducts_colony_supplies_once` | Green | None |
| Entry denial is atomic | `dungeon_entry_denial_preserves_mode_turn_and_resources` | Green | None |
| Fixed content loads without procgen | `foundation_scenario::fixed_dungeon_loads_without_procgen` | Green | None |
| Entrance, hostile, loot, and exit are reachable | loader validation plus canonical action paths | Green | A single content reachability matrix would improve diagnostics |
| Legal movement changes one cardinal tile and one turn | `foundation_actions::valid_fixed_dungeon_movement_changes_one_cardinal_tile` | Green unreviewed | Rendered movement transition |
| Wall movement is typed and atomic | `foundation_actions::fixed_dungeon_wall_movement_is_typed_and_atomic` | Green | None |
| Default hostile survives one default attack | `foundation_scenario::canonical_combat_requires_more_than_one_attack` | Green | None |
| Enemy phase occurs once | tactical input and fatal-action tests | Partial | Exact one-response state-diff per accepted tactical action |
| Invalid attacks are atomic | progression and core action tests | Partial | Integrated no-target/out-of-range player-path matrix |
| Pickup is explicit and positional | `foundation_actions::pickup_resolves_through_action_pipeline`; rejected pickup atomicity | Green | Rendered pickup feedback |
| Extraction requires exit and explicit action | premature extraction test; `production_keys_complete_the_fixed_dungeon_loop_with_named_checkpoints` | Green | Production keys reach the exit and explicitly extract |
| Extracted loot applies once | `foundation_scenario::canonical_extraction_applies_loot_once` | Green | None |
| Colony state survives the run | `canonical_colony_state_survives_round_trip`; entity-scope tests | Green | Full normalized fingerprint |
| Defeat grants no loot | `canonical_defeat_awards_no_loot` | Green | None |
| Restart uses shelter return spawn | `mvp_correction::defeat_restart_uses_the_same_shelter_return_spawn` | Green | None |
| Full dungeon path is playable through production keys | `production_keys_complete_the_fixed_dungeon_loop_with_named_checkpoints`; `production_keys_complete_defeat_title_and_shelter_restart` | Green unreviewed | Extraction and defeat/restart paths pass with named state/visibility checkpoints; complete scene-style and PTY review remain |

## 9. Persistence

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| Clean Outpost round trip | `persistence_checkpoint_matrix` colony-idle projection; canonical persistence tests | Green | Full normalized fingerprint through checkpoint and manual-slot paths |
| Built station round trip | station catalog identity test | Partial | Position, cost state, effect, and visual equality together |
| Assigned worker round trip | colony assignment persistence test | Partial | Stable relationship identity rather than count |
| EnRoute checkpoint | deterministic next worker step test | Partial | Typed activity and projection equality |
| Working checkpoint | `persistence_checkpoint_matrix::every_projection_round_trips_the_full_fingerprint_through_checkpoint` (colony-working projection) | Green | Two-of-three direct-gather progress, Working activity, and durable task survive both checkpoint and manual-slot restore |
| Before/after day boundary checkpoints | `save_before_day_boundary...`; `save_after_day_boundary...` | Green | Fingerprint equality |
| Active dungeon checkpoint | `persistence::save_load_active_dungeon_preserves_foundation_fingerprint_and_scope_counts` | Green | Durable player, colony, worker-activity, resource, and entity-scope state survives the checkpoint |
| Carrying loot checkpoint | `persistence_checkpoint_matrix` dungeon-carrying-loot projection | Green | Player inventory survives checkpoint and manual-slot restore in a fresh runtime |
| Extracted checkpoint | post-extraction no-reapply test; `persistence_checkpoint_matrix` extracted projection | Green | Full fingerprint through checkpoint and manual-slot paths |
| Game Over checkpoint | defeat outcome test; `persistence_checkpoint_matrix` game-over projection | Green | Full fingerprint and rendered outcome equality |
| Failed load is atomic | missing/corrupt title-load application contracts; `load_rejects_missing_relationship_reference` | Green | Missing slot, corrupt file, and corrupt relationship all preserve live state |
| Deterministic next action survives load | RNG and same-snapshot tests | Green | Worker and visual continuation remain partial |
| Entity-independent Foundation fingerprint | `test_harness_contract` target | Green | Stable names/catalog identities/positions replace raw entity bits; transient build state is excluded |
| Save/load preserves projected visual state | `phase6_input::colony_checkpoint_round_trip_preserves_the_visible_projection`; Tactical counterpart | Partial | Stable screen/stats/map/actions/log projection equality passes; canvas and resolved-style equality remain open |

## 10. Progression, factions, and data-driven content

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| Quick Attack improves Melee once | `progression_factions::quick_attack_improves_melee` | Green | None |
| Quick Attack expresses Thumos once | `quick_attack_expresses_thumos` | Green | None |
| Combat survival expresses Fortitude | `generic_enemy_defeat_grants_fortitude_but_not_kleos` | Green | Exact mapping remains Foundation representative only |
| Item use improves Medicine and Temperance | item-use progression tests | Green | Item pickup fixture still uses a helper after action-based movement |
| Rejected actions grant no progression | `rejected_action_grants_no_progression` | Green | Ranged/Repair rejection matrix not required |
| Melee, Ranged, Repair, and Medicine records exist | loader required-ID and link validation | Green | Ranged and Repair lack player-facing Foundation controls; content linkage is the current locked minimum |
| Six virtues plus Kleos exist | `player_has_all_six_virtues_and_kleos` | Green | Exact balance/mapping is deferred |
| Exactly two placeholder factions load from data | `two_foundation_factions_load_with_typed_disposition` | Green | None |
| A third valid faction needs no Rust branch | loader extensibility test | Green | Registry ownership should be added during full migration |
| Hostility uses disposition | faction and enemy-AI tests | Green | None |
| Invalid content reports source and IDs | loader validation tests | Partial | Table-driven file/record diagnostic matrix |
| Active symbols reject ambiguous simultaneous categories | monochrome presentation tests and `bd_data` collision-validation tests | Green | Validation names colliding active categories and rejects both glyph and fallback collisions |
| Station catalog extends without a Rust branch | sixth-station content test | Green | None |

## 11. Deferred boundary

These goals belong to the product vision but must not be converted into active
Foundation failures:

| Product goal | Foundation disposition | Current guard |
|---|---|---|
| Procedural dungeon generation | Deferred | `fixed_dungeon_loads_without_procgen`; legacy procgen tests remain non-acceptance evidence |
| Raids and colony events | Deferred | `foundation_app_does_not_register_deferred_systems` |
| Sanity | Deferred | same deferred-system isolation contract |
| Full overworld travel/weather | Deferred | fixed direct dungeon entry is the Foundation path |
| Theology-driven mechanics | Deferred | representative virtue hooks only |
| Faction reputation/diplomacy and final canon | Deferred | two placeholder factions only |
| Deeper narrative/investigation | Deferred | no Foundation acceptance dependency |

## 12. Next test-authoring queue

The following missing contracts are the highest-value additions after the
current red implementation batch. They are ordered by how much false
confidence they currently permit:

1. complete persistence checkpoint matrix using the normalized fingerprint,
   including Working, carrying-loot, extracted, and Game Over projections —
   closed by `PERSIST-MATRIX-001` (`persistence_checkpoint_matrix`);
2. PTY completion for the now-green semantic/canvas/style/geometry visual
   observation infrastructure;
3. close `VISUAL-COLONY-WORK-006` and `VISUAL-LANGUAGE-004` for target
   distance and blocked-worker style;
4. exact-one-player and all-layer canvas matrices across every shelter position;
5. management selection-navigation and result-transition style/PTY evidence;
6. all-advertised-control end-effect matrix and explicit Press/Repeat/Release
   policy beyond Build — Outpost controls closed by `INPUT-POLICY-001`;
   combat-mode controls (attack, guard, extract, pickup) remain;
7. content-invalid diagnostic matrix with exact file and record ownership;
8. real PTY lifecycle, repeat, resize, and restoration evidence.

Do not replace these with a single broad “MVP scenario passes” test. Each item
must retain discrete atomic owners, with the broad scenario serving only as
workflow evidence.

## 13. Turn-based work and construction

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| Recipe data owns positive gather/refine work turns | `colony_recipe_rejects_non_positive_work_turns` | Green | Owner balance review |
| No resource appears before configured work completes | `configured_work_turns_gate_gather_and_refine_yields_exactly_once` | Green | None |
| Placement creates a paid non-operational site | `accepted_build_is_a_paid_non_operational_construction_site` | Green | None |
| Idle survivors travel and contribute construction work | `idle_survivors_travel_to_and_complete_construction_without_stealing_assigned_workers` | Green | Multi-site priority policy remains deferred |
| Assigned workers are never stolen | same construction workflow primary | Green | None |
| Explicit gathering immediately releases an automatic builder | `explicit_gather_assignment_overrides_pending_automatic_construction` | Green | None |
| Render/save/load grant no construction work | `render_frames_and_save_load_do_not_grant_construction_work` | Green | None |
| Site and progress are player-visible | `placed_construction_site_has_distinct_map_and_progress_feedback` | Green | Real-terminal owner review |

## 14. Direct gathering coherence and feedback

| Requirement piece | Primary evidence | Status | Remaining review |
|---|---|---|---|
| Direct gathering rules are data-defined | `foundation_direct_gather_rules_are_declared_in_content` | Green | Balance values remain owner-tunable content |
| Three adjacent worker ticks produce exactly one resource | `direct_gather_requires_three_work_ticks_and_credits_once` | Green | None |
| All three Foundation gathering tasks share the rule | `every_foundation_direct_gather_task_uses_the_same_three_tick_rule` | Green | None |
| Day advancement grants no legacy duplicate | `day_boundary_does_not_credit_legacy_direct_gather_output` | Green | None |
| Render and Tactical frames grant no work | `render_and_tactical_frames_do_not_advance_direct_gathering` | Green | None |
| Rest equals equivalent individual turns | `rest_and_equivalent_individual_turns_preserve_direct_gather_results` | Green | None |
| Partial progress survives save/load | `partial_direct_gather_progress_survives_save_load_without_free_output` | Green | None |
| Reassignment clears partial work | `reassignment_clears_partial_direct_gather_progress_without_output` | Green | None |
| Zero Supplies recovers before day end | `zero_supplies_recovers_after_three_worker_ticks_without_waiting_for_day_end` | Green | None |
| `c` workflow shows source and progress | `direct_gather_assignment_projects_source_and_three_tick_progress` | Green unreviewed | Real-PTY visual review remains open |
| Recipe choices use human labels | `recipe_management_uses_human_resource_labels_not_content_ids` | Green unreviewed | Real-PTY visual review remains open |
| Worker result and day upkeep are distinct | `colony_projection_separates_next_worker_result_from_next_day_upkeep` | Green unreviewed | Real-PTY visual review remains open |
| Nonzero raw stockpiles are visible | `nonzero_raw_stockpile_is_projected_with_a_human_label` | Green unreviewed | Real-PTY visual review remains open |
| Blocked direct gathering names target and reason | `blocked_direct_gatherer_projects_target_and_actionable_reason` | Green unreviewed | Real-PTY visual review remains open |

## 15. Foundation UI improvement contracts

| Requirement piece | Primary evidence | Status | Required implementation |
|---|---|---|---|
| Visual observations detect glyph/style/geometry changes | `visual_observation_detects_glyph_foreground_modifier_and_geometry_changes` | Green unreviewed | PTY and owner review |
| Same-state rendering is deterministic | `identical_fixture_has_identical_canvas_and_resolved_styles` | Green unreviewed | PTY review |
| Shelter panels never overlap | `supported_outpost_panel_rectangles_never_overlap` | Green unreviewed | Continue through layout changes |
| Shelter map owns primary interactive area | `outpost_map_is_the_largest_interactive_panel_at_supported_profiles` | Green unreviewed | Automated layout passes at both profiles; PTY/owner review remains open |
| Off-screen target exposes direction/name/distance | `offscreen_assignment_names_target_and_distance_at_supported_profiles` | Green unreviewed | Automated projection passes; PTY/owner review remains open |
| Worker row exposes numeric target distance | `assigned_worker_row_names_target_and_numeric_distance` | Green unreviewed | Automated projection passes; PTY/owner review remains open |
| Valid/invalid previews differ without color | `valid_and_invalid_build_previews_differ_without_color` | Green unreviewed | InvalidSelection semantic token passes; PTY/owner review remains open |
| Unaffordable build explains exact shortage | `unaffordable_build_selection_explains_the_exact_shortage` | Green unreviewed | Automated shortage projection passes; PTY/owner review remains open |
| Task workflow exposes three stages | `task_management_exposes_survivor_task_and_confirm_stages` | Green unreviewed | Automated stage projection passes; PTY/owner review remains open |
| Staffing workflow exposes four stages | `station_staffing_exposes_survivor_station_recipe_and_confirm_stages` | Green unreviewed | Automated stage projection passes; PTY/owner review remains open |
| Blocked worker style differs from working | `blocked_worker_has_a_distinct_resolved_style_from_working_worker` | Green unreviewed | Danger-style blocked token passes; PTY/owner review remains open |
| Decisive warning survives routine overflow | `decisive_warning_survives_routine_log_overflow` | Green unreviewed | Automated overflow separation passes; PTY/owner review remains open |
| Dungeon denials render completely | `rendered_dungeon_denials_explain_attack_pickup_and_extraction` | Green unreviewed | PTY review |
| Dungeon status exposes carried loot/readiness | `dungeon_status_distinguishes_carried_loot_and_extraction_readiness` | Green unreviewed | Automated dungeon status projection passes; PTY/owner review remains open |
| Title explains unavailable Load | `title_without_save_explains_why_load_is_unavailable` | Green unreviewed | Automated title availability passes; PTY/owner review remains open |
| Closing management leaves no stale cells | `closed_management_modal_leaves_the_same_canvas_as_a_clean_overview` | Green unreviewed | PTY review |
| Supported-profile resize is deterministic | `resize_round_trip_returns_to_the_original_canvas_and_styles` | Green unreviewed | Real resize/PTY lifecycle evidence |
| Selected Cinder Rite identity is shared beyond the colony screen | `selected_cinder_rite_identity_frames_colony_and_reusable_screens` | Green unreviewed | Palette, double shell/modal chrome, muted single-rule panels, responsive HP/AP tracks, and shared mode/command ribbons pass both automated profiles; final owner review and complete placement/profile PTY remain |
| Supplies exposes pressure and next-day outlook | `provisions_show_stock_pressure_and_dawn_outlook_at_supported_profiles` | Green unreviewed | Structured exact gauge, pressure, dawn delta/result, and both automated profiles pass; PTY/owner review remains |
| Entering station/node range emits one nearby hint | `entering_adjacent_range_emits_one_deduplicated_nearby_hint` plus registered node/aggregation/re-entry supports | Green unreviewed | Production movement now emits one focused fact plus a count, preserves the complete deterministic target set, rearms on exit/re-entry, and keeps unbound Interact truthfully disabled; PTY/owner review remains |
| One Context presentation serves station/node/colonist | `nearby_station_context_is_complete_at_supported_profiles` plus registered category/state/action/final-composition supports | Green unreviewed | All 113 `bd_tui --lib` observers pass; final PTY/owner review remains open |

## 16. Developer-console input support

These rows support development tooling and do not expand Foundation product
scope. The C1 v2 handoff preserves six useful physical greens while keeping
two independently observed architecture gaps intentionally Red.

| Requirement piece | Primary evidence | Status | Missing proof |
|---|---|---|---|
| One registered reducer owns physical console editing | `console_input_contract::physical_console_editing_uses_the_registered_production_reducer` | Red | Physical editing is green, but C1 cannot close until the reducer has explicit schedule ownership and one causal submission path |
| Console-owned close keys never reach gameplay routing | `console_input_contract::escape_close_is_consumed_before_title_routing` plus Title/Outpost close cases and `console_capture_is_explicitly_ordered_before_gameplay_routing` | Red | Physical close cases pass, but Bevy still reports one unresolved conflict involving the reducer instead of an explicit dependency before gameplay routing |
| One physical line uses one typed submission path | `console_input_contract::one_physical_line_reaches_dispatch_exactly_once` | Red | The reducer emits `ConsoleCommand` and independently writes `ConsoleState.pending`; quarantining that competitor leaves zero dispatch results |
