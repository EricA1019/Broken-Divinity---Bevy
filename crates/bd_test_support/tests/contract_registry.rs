use bd_test_support::contract_registry::{
    ContractRegistry, RegistryReport, RegistryValidationContext, TestEvidence, ValidationCode,
};
use std::{collections::BTreeSet, path::PathBuf};

const PRIMARY: &str = "bd_tui::tests::outpost_help_explains_visible_resource_glyphs";

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate directory has a workspace parent")
        .parent()
        .expect("workspace has a project parent")
        .to_path_buf()
}

fn contract() -> String {
    format!(
        r#"(
            contracts: [(
                id: "VISUAL-HELP-001",
                title: "Outpost Help explains visible resource glyphs",
                scope: "FoundationRequired",
                authority_references: ["GDD.md#6-core-gameplay-structure"],
                player_outcome: "Help identifies each active shelter resource category.",
                primary_test: Some("{PRIMARY}"),
                supporting_tests: [],
                evidence_layers: ["Projection", "BufferLayout", "PTY"],
                profiles: ["Foundation", "Baseline80x24", "Compact60x20"],
                fixture_id: "outpost_help",
                owner_phase: 6,
                status: "Red",
                known_failure: Some("Resource categories are omitted from Help."),
            )],
        )"#
    )
}

fn valid_context() -> RegistryValidationContext {
    RegistryValidationContext::new(project_root())
        .with_known_tests([PRIMARY])
        .with_ignored_tests(std::iter::empty::<&str>())
        .with_visual_evidence(std::iter::empty::<&str>())
}

fn codes(source: &str, context: &RegistryValidationContext) -> BTreeSet<ValidationCode> {
    let registry = ContractRegistry::parse(source).expect("fixture registry should parse");
    registry
        .validate(context)
        .into_iter()
        .map(|issue| issue.code)
        .collect()
}

#[test]
fn valid_required_contract_passes_registry_validation() {
    let registry =
        ContractRegistry::parse(&contract()).expect("a valid RON contract registry should parse");

    let issues = registry.validate(&valid_context());

    assert!(issues.is_empty(), "unexpected registry issues: {issues:#?}");
}

#[test]
fn duplicate_primary_owner_is_rejected() {
    let first = contract();
    let record = first
        .split_once("contracts: [")
        .expect("fixture has contracts list")
        .1
        .rsplit_once("],")
        .expect("fixture has contracts-list suffix")
        .0
        .trim();
    let source = format!("(contracts: [{record}, {record}],)");

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::DuplicatePrimaryOwner));
}

#[test]
fn duplicate_contract_id_is_rejected() {
    let first = contract();
    let record = first
        .split_once("contracts: [")
        .expect("fixture has contracts list")
        .1
        .rsplit_once("],")
        .expect("fixture has contracts-list suffix")
        .0
        .trim();
    let source = format!("(contracts: [{record}, {record}],)");

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::DuplicateContractId));
}

#[test]
fn missing_authority_reference_is_rejected() {
    let source = contract().replace(
        r#"authority_references: ["GDD.md#6-core-gameplay-structure"]"#,
        r#"authority_references: ["missing-authority.md"]"#,
    );

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::MissingAuthority));
}

#[test]
fn authority_reference_outside_repository_is_rejected() {
    let source = contract().replace(
        r#"authority_references: ["GDD.md#6-core-gameplay-structure"]"#,
        r#"authority_references: ["../GDD.md#6-core-gameplay-structure"]"#,
    );

    assert!(
        codes(&source, &valid_context()).contains(&ValidationCode::AuthorityOutsideRepository),
        "authority files outside the repository make clean clones non-reproducible"
    );
}

#[test]
fn empty_authority_list_is_rejected() {
    let source = contract().replace(
        r#"authority_references: ["GDD.md#6-core-gameplay-structure"]"#,
        "authority_references: []",
    );

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::MissingAuthority));
}

#[test]
fn required_contract_without_primary_test_is_rejected() {
    let source = contract().replace(
        &format!(r#"primary_test: Some("{PRIMARY}")"#),
        "primary_test: None",
    );

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::MissingPrimaryTest));
}

#[test]
fn unknown_primary_test_is_rejected() {
    let context = valid_context().with_known_tests(std::iter::empty::<&str>());

    assert!(codes(&contract(), &context).contains(&ValidationCode::UnknownPrimaryTest));
}

#[test]
fn unknown_supporting_test_is_rejected() {
    let source = contract().replace(
        "supporting_tests: []",
        r#"supporting_tests: ["support::missing"]"#,
    );

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::UnknownSupportingTest));
}

#[test]
fn ignored_required_primary_test_is_rejected() {
    let context = valid_context().with_ignored_tests([PRIMARY]);

    assert!(codes(&contract(), &context).contains(&ValidationCode::IgnoredRequiredTest));
}

#[test]
fn deferred_required_contract_is_rejected() {
    let source = contract().replace(r#"status: "Red""#, r#"status: "Deferred""#);

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::DeferredRequiredContract));
}

#[test]
fn retired_contract_cannot_remain_a_primary_owner() {
    let source = contract().replace(r#"status: "Red""#, r#"status: "Retired""#);

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::RetiredPrimaryOwner));
}

#[test]
fn required_contract_without_evidence_layers_is_rejected() {
    let source = contract().replace(
        r#"evidence_layers: ["Projection", "BufferLayout", "PTY"]"#,
        "evidence_layers: []",
    );

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::MissingEvidenceLayers));
}

#[test]
fn unknown_evidence_layer_is_rejected() {
    let source = contract().replace(
        r#"evidence_layers: ["Projection", "BufferLayout", "PTY"]"#,
        r#"evidence_layers: ["LooksGood"]"#,
    );

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::UnknownEvidenceLayer));
}

#[test]
fn unknown_profile_is_rejected() {
    let source = contract().replace(
        r#"profiles: ["Foundation", "Baseline80x24", "Compact60x20"]"#,
        r#"profiles: ["DeveloperLaptop"]"#,
    );

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::UnknownProfile));
}

#[test]
fn unknown_scope_is_rejected() {
    let source = contract().replace(
        r#"scope: "FoundationRequired""#,
        r#"scope: "MostlyRequired""#,
    );

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::UnknownScope));
}

#[test]
fn unknown_status_is_rejected() {
    let source = contract().replace(r#"status: "Red""#, r#"status: "AlmostGreen""#);

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::UnknownStatus));
}

#[test]
fn red_contract_requires_a_known_failure() {
    let source = contract().replace(
        r#"known_failure: Some("Resource categories are omitted from Help.")"#,
        "known_failure: None",
    );

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::MissingKnownFailure));
}

#[test]
fn accepted_visual_contract_requires_completed_matrix_evidence() {
    let source = contract().replace(r#"status: "Red""#, r#"status: "Accepted""#);

    assert!(codes(&source, &valid_context()).contains(&ValidationCode::MissingVisualEvidence));
}

#[test]
fn accepted_visual_contract_passes_with_completed_matrix_evidence() {
    let source = contract()
        .replace(r#"status: "Red""#, r#"status: "Accepted""#)
        .replace(
            r#"known_failure: Some("Resource categories are omitted from Help.")"#,
            "known_failure: None",
        );
    let context = valid_context().with_visual_evidence(["outpost_help"]);

    assert!(codes(&source, &context).is_empty());
}

#[test]
fn unknown_registry_field_fails_to_parse() {
    let source = contract().replace(
        r#"known_failure: Some("Resource categories are omitted from Help."),"#,
        r#"known_failure: Some("Resource categories are omitted from Help."), invented: true,"#,
    );

    let error = ContractRegistry::parse(&source).expect_err("unknown metadata must be rejected");

    assert!(
        error.to_string().contains("invented"),
        "parse error should name the unknown field: {error}"
    );
}

#[test]
fn validation_issues_are_deterministically_sorted() {
    let source = contract()
        .replace(
            r#"authority_references: ["GDD.md#6-core-gameplay-structure"]"#,
            r#"authority_references: ["../z-missing.md", "../a-missing.md"]"#,
        )
        .replace(
            r#"profiles: ["Foundation", "Baseline80x24", "Compact60x20"]"#,
            r#"profiles: ["UnknownZ", "UnknownA"]"#,
        )
        .replace(
            r#"evidence_layers: ["Projection", "BufferLayout", "PTY"]"#,
            r#"evidence_layers: ["UnknownZ", "UnknownA"]"#,
        );
    let registry = ContractRegistry::parse(&source).expect("fixture registry should parse");

    let first = registry.validate(&valid_context());
    let second = registry.validate(&valid_context());

    assert_eq!(first, second);
    assert!(
        first.windows(2).all(|pair| pair[0] <= pair[1]),
        "issues are not sorted: {first:#?}"
    );
}

#[test]
fn text_and_json_reports_use_the_same_explicit_evidence_totals() {
    let registry =
        ContractRegistry::parse(&contract()).expect("a valid RON contract registry should parse");
    let evidence = [
        TestEvidence::passed(PRIMARY),
        TestEvidence::failed("support::known_red"),
        TestEvidence::ignored("diagnostic::pty"),
    ];

    let report = RegistryReport::from_registry(&registry, &evidence);
    let text = report.to_text();
    let json: serde_json::Value =
        serde_json::from_str(&report.to_json().expect("report should serialize"))
            .expect("report JSON should parse");

    assert!(text.contains("listed: 3"), "{text}");
    assert!(text.contains("green_unreviewed: 0"), "{text}");
    assert!(text.contains("not_implemented: 0"), "{text}");
    assert!(text.contains("passed: 1"), "{text}");
    assert!(text.contains("failed: 1"), "{text}");
    assert!(text.contains("ignored: 1"), "{text}");
    assert_eq!(json["tests"]["listed"], 3);
    assert_eq!(json["tests"]["passed"], 1);
    assert_eq!(json["tests"]["failed"], 1);
    assert_eq!(json["tests"]["ignored"], 1);
    assert_eq!(json["contracts"]["green_unreviewed"], 0);
    assert_eq!(json["contracts"]["not_implemented"], 0);
}

#[test]
fn seeded_registry_maps_current_foundation_contract_batches() {
    let registry_path = project_root()
        .join("testing")
        .join("foundation-contracts.ron");
    let registry =
        ContractRegistry::load(&registry_path).expect("seeded Foundation registry should load");
    let known_tests = [
        "bd_tui::lib::tests::outpost_80x24_viewport_keeps_player_visible_at_far_shelter_edge",
        "bd_tui::lib::tests::outpost_60x20_viewport_keeps_player_visible_at_far_shelter_edge",
        "bd_tui::lib::tests::compact_viewport_projects_resource_next_to_far_edge_player",
        "bd_tui::lib::tests::station_and_resource_cells_have_distinct_resolved_styles",
        "bd_tui::lib::tests::compact_build_selection_shows_complete_selected_effect",
        "bd_tui::lib::tests::build_selection_and_placement_fit_both_supported_profiles",
        "bd_tui::lib::tests::invalid_build_preview_explains_egress_rejection",
        "bd_tui::lib::tests::distant_build_preview_drives_the_viewport_at_both_supported_profiles",
        "bd_tui::lib::tests::station_staffing_uses_a_distinct_modal_title",
        "bd_tui::lib::tests::compact_station_staffing_keeps_each_wrapped_station_status_inside_the_modal",
        "bd_tui::lib::tests::outpost_help_explains_visible_resource_glyphs",
        "bd_app::survivor_work_contract::rest_and_individual_waits_produce_the_same_worker_position",
        "bd_app::survivor_work_contract::new_assignment_does_not_move_during_paused_confirmation",
        "bd_app::survivor_work_contract::next_outpost_turn_moves_worker_exactly_one_cardinal_step",
        "bd_app::survivor_work_contract::idle_render_frames_do_not_move_assigned_survivors",
        "bd_app::survivor_work_contract::idle_survivor_does_not_move_on_accepted_outpost_turns",
        "bd_app::survivor_work_contract::tactical_turns_do_not_move_colony_survivors",
        "bd_app::survivor_work_contract::rest_and_individual_waits_produce_the_same_daily_resources",
        "bd_app::survivor_work_contract::worker_uses_pathfinding_around_a_wall_blocker",
        "bd_app::survivor_work_contract::unreachable_worker_stays_put_and_reports_a_specific_blocked_reason",
        "bd_app::survivor_work_contract::assigned_survivors_never_stack_on_one_tile",
        "bd_app::survivor_work_contract::station_worker_never_enters_the_station_tile",
        "bd_app::survivor_work_contract::station_worker_stops_cardinally_adjacent_to_target",
        "bd_app::survivor_work_contract::gatherer_never_enters_a_resource_node_tile",
        "bd_app::survivor_work_contract::assigned_but_enroute_station_worker_produces_nothing",
        "bd_app::survivor_work_contract::adjacent_station_worker_produces_once",
        "bd_app::survivor_work_contract::blocked_station_worker_produces_nothing",
        "bd_app::survivor_work_contract::assigned_but_enroute_gatherer_produces_nothing",
        "bd_app::survivor_work_contract::adjacent_matching_gatherer_produces_once",
        "bd_app::survivor_work_contract::gatherer_at_wrong_node_type_produces_nothing",
        "bd_app::survivor_work_contract::zero_supply_recovery_remains_reachable_with_physical_gathering",
        "bd_app::survivor_work_contract::forecast_excludes_enroute_worker_output",
        "bd_app::survivor_work_contract::save_load_preserves_the_next_deterministic_worker_step",
        "bd_app::survivor_work_contract::load_does_not_immediately_move_or_produce_for_assigned_worker",
        "bd_app::phase6_input::c_opens_paused_task_management_with_task_identity",
        "bd_app::phase6_input::e_opens_paused_station_staffing_with_station_identity",
        "bd_app::phase6_input::station_staffing_lists_station_assignments_not_gathering_tasks",
        "bd_app::phase6_input::task_management_lists_survivor_tasks_not_station_staffing_choices",
        "bd_app::phase6_input::management_cancel_is_atomic_and_discards_modal_gameplay_input",
        "bd_app::phase6_input::station_staffing_confirmation_changes_only_the_named_survivor_relationship",
        "bd_app::phase6_input::entering_build_placement_starts_on_a_visible_adjacent_candidate",
        "bd_app::phase6_input::build_menu_sixth_number_key_selects_the_sixth_data_driven_station",
        "bd_app::phase6_input::build_placement_cursor_moves_cumulatively_without_moving_the_player",
        "bd_app::phase6_input::distant_build_confirmation_places_at_the_absolute_preview_coordinate",
        "bd_app::phase6_input::invalid_build_confirmation_keeps_preview_active_and_is_atomic",
        "bd_app::phase6_input::altar_and_idle_survivor_remain_distinct_without_color",
        "bd_app::phase6_input::workshop_and_water_source_remain_distinct_without_color",
        "bd_app::phase6_input::staffed_and_unstaffed_station_have_distinct_ascii_projection",
        "bd_tui::lib::tests::build_placement_exposes_selected_station_name_cost_and_effect",
        "bd_tui::lib::tests::rendered_outpost_help_contains_every_foundation_legend_at_supported_profiles",
        "bd_app::colony_spatial_contract::every_accepted_station_placement_preserves_gate_reachability",
        "bd_app::colony_spatial_contract::second_corner_station_is_rejected_before_it_traps_the_player",
        "bd_core::colony::stations::tests::placement_rejects_station_without_a_reachable_adjacent_work_tile",
        "bd_data::loader::tests::foundation_colony_chains_are_complete_and_cross_referenced",
        "bd_data::loader::tests::colony_source_rejects_a_missing_raw_resource_reference",
        "bd_data::loader::tests::colony_recipe_rejects_non_positive_amounts",
        "bd_data::loader::tests::colony_recipe_rejects_non_positive_work_turns",
        "bd_data::loader::tests::buildable_station_rejects_zero_construction_work",
        "bd_data::loader::tests::colony_recipe_invalid_reference_matrix_names_the_recipe_and_file",
        "bd_data::loader::tests::colony_recipe_rejects_source_input_mismatch",
        "bd_data::loader::tests::colony_recipe_rejects_output_without_finished_pool_mapping",
        "bd_app::colony_node_generation_contract::configured_source_counts_own_new_colony_node_coverage",
        "bd_app::colony_node_generation_contract::node_planner_satisfies_spatial_invariants_across_map_fixtures",
        "bd_app::colony_node_generation_contract::impossible_node_layout_is_typed_and_atomic",
        "bd_app::colony_node_generation_contract::named_128_seed_profile_preserves_complete_reachable_spaced_layouts",
        "bd_app::colony_node_generation_contract::same_seed_repeats_layout_and_seed_matrix_preserves_invariants",
        "bd_app::colony_node_generation_contract::source_file_order_does_not_change_seeded_node_ownership_or_positions",
        "bd_app::colony_node_generation_contract::persisted_node_layout_is_restored_without_regeneration",
        "bd_app::colony_production_route_contract::one_survivor_completes_the_pilot_source_to_station_route",
        "bd_app::colony_production_route_contract::pilot_recipe_transition_matrix_performs_one_operation_per_worker_tick",
        "bd_app::colony_production_route_contract::missing_or_unreachable_targets_block_without_losing_cargo_or_output",
        "bd_app::colony_production_route_contract::absent_worker_tick_is_a_no_op",
        "bd_app::colony_production_route_contract::scheduler_frames_without_accepted_time_do_not_advance_logistics",
        "bd_app::colony_production_route_contract::tactical_turns_do_not_advance_colony_logistics",
        "bd_app::colony_production_route_contract::pilot_recipe_conserves_raw_input_and_only_refining_creates_finished_output",
        "bd_app::colony_production_route_contract::reassigning_a_carrying_worker_deposits_raw_cargo_and_cancels_logistics",
        "bd_app::colony_production_route_contract::carrying_checkpoint_preserves_recipe_stage_and_raw_cargo",
        "bd_app::colony_production_route_contract::checkpoint_round_trip_preserves_every_logistics_stage",
        "bd_app::colony_production_route_contract::every_configured_recipe_obeys_the_same_gather_and_refine_transition",
        "bd_app::colony_production_route_contract::fixture_fourth_chain_needs_no_new_transition_branch",
        "bd_app::colony_production_route_contract::two_survivors_complete_different_chains_without_stacking_or_duplicate_credit",
        "bd_app::colony_production_route_contract::two_workers_share_one_station_work_tile_without_stacking_or_duplicate_credit",
        "bd_app::colony_production_route_contract::carrying_checkpoint_preserves_the_next_deterministic_worker_tick",
        "bd_app::colony_production_route_contract::complete_colony_workflow_replays_deterministically_from_player_actions",
        "bd_app::colony_production_route_contract::configured_work_turns_gate_gather_and_refine_yields_exactly_once",
        "bd_app::colony_production_route_contract::partial_work_progress_survives_checkpoint_without_free_yield",
        "bd_app::colony_construction_contract::accepted_build_is_a_paid_non_operational_construction_site",
        "bd_app::colony_construction_contract::idle_survivors_travel_to_and_complete_construction_without_stealing_assigned_workers",
        "bd_app::colony_construction_contract::render_frames_and_save_load_do_not_grant_construction_work",
        "bd_app::phase6_input::explicit_gather_assignment_overrides_pending_automatic_construction",
        "bd_app::phase6_input::placed_construction_site_has_distinct_map_and_progress_feedback",
        "bd_app::phase6_input::processing_assignment_selects_named_survivor_station_and_recipe_while_paused",
        "bd_app::phase6_input::production_key_workflow_assigns_travels_gathers_refines_and_reports",
        "bd_app::phase6_input::deterministic_production_key_fuzz_preserves_colony_invariants",
        "bd_tui::lib::tests::colony_worker_recipe_stage_target_and_cargo_are_visible_at_supported_profiles",
        "bd_app::foundation_actions::valid_fixed_dungeon_movement_changes_one_cardinal_tile",
        "bd_app::foundation_actions::fixed_dungeon_wall_movement_is_typed_and_atomic",
        "bd_app::foundation_actions::extraction_away_from_fixed_exit_is_typed_and_atomic",
    ];
    let context = RegistryValidationContext::new(
        project_root()
            .canonicalize()
            .expect("workspace root should resolve"),
    )
    .with_known_tests(known_tests)
    .with_ignored_tests(std::iter::empty::<&str>())
    .with_visual_evidence(std::iter::empty::<&str>());

    let issues = registry.validate(&context);

    assert!(
        issues.is_empty(),
        "seeded registry contains invalid contracts: {issues:#?}"
    );
    assert_eq!(
        registry.contracts.len(),
        46,
        "the registry must own every contract in the current visual, worker, management, build, spatial, and dungeon batches"
    );
    assert_eq!(
        registry
            .contracts
            .iter()
            .filter(|contract| {
                contract.id.starts_with("VISUAL-") && contract.status == "GreenUnreviewed"
            })
            .count(),
        13,
        "all thirteen registered visual contracts are green but still require review evidence"
    );
    assert_eq!(
        registry
            .contracts
            .iter()
            .filter(|contract| { contract.id.starts_with("VISUAL-") && contract.status == "Red" })
            .count(),
        0,
        "no registered presentation contract remains red after remediation"
    );
    assert_eq!(
        registry
            .contracts
            .iter()
            .filter(|contract| contract.id.starts_with("COLONY-") && contract.status == "Red")
            .count(),
        0,
        "no registered physical-worker contract remains red after remediation"
    );
    assert_eq!(
        registry
            .contracts
            .iter()
            .filter(|contract| contract.id.starts_with("COLONY-")
                && contract.status == "GreenUnreviewed")
            .count(),
        19,
        "all nineteen colony contracts are green but unreviewed"
    );
    assert_eq!(
        registry
            .contracts
            .iter()
            .filter(|contract| { contract.id.starts_with("INPUT-") && contract.status == "Red" })
            .count(),
        0,
        "no registered management/build input contract remains red after remediation"
    );
    assert_eq!(
        registry
            .contracts
            .iter()
            .filter(|contract| {
                contract.id.starts_with("DUNGEON-") && contract.status == "GreenUnreviewed"
            })
            .count(),
        2,
        "the new fixed-dungeon movement and extraction guards are green but unreviewed"
    );
}
