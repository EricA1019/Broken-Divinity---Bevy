use bd_core::{
    colony::{
        stations::{StationEffect, StationType},
        survivors::SurvivorTask,
    },
    components::{Position, ResourceNodeType},
    direction::Direction,
    session::RunOutcome,
    signals::PoolKind,
    spatial::FOUNDATION_DUNGEON_ID,
};
use bd_test_support::{FoundationDriver, foundation_content};

fn colony_driver(seed: u64) -> FoundationDriver {
    let mut driver = FoundationDriver::new(seed);
    driver.start_colony().unwrap();
    driver
}

fn wait_once(driver: &mut FoundationDriver, step: &str) {
    let player = driver.player().unwrap();
    driver
        .submit_action_and_advance_result_frame(step, player, "ability.wait", None, None)
        .unwrap();
}

fn advance_to_turn_23(driver: &mut FoundationDriver) {
    while driver.summary().turn < 23 {
        wait_once(driver, "advance to pre-boundary turn");
    }
}

fn cross_tactical_day_boundary(driver: &mut FoundationDriver) {
    advance_to_turn_23(driver);
    driver.enter_dungeon(FOUNDATION_DUNGEON_ID).unwrap();
    wait_once(driver, "cross Tactical day boundary");
    driver.advance_day_resolution_frame();
}

fn place_gatherers_at_source(
    driver: &mut FoundationDriver,
    kind: ResourceNodeType,
    gatherers: usize,
) {
    let target = driver
        .resource_nodes_with_state()
        .into_iter()
        .find_map(|(_, node_kind, position, _)| (node_kind == kind).then_some(position))
        .unwrap_or_else(|| panic!("missing {kind:?} source"));
    let map = driver.outpost_map();
    let work_tiles = [
        Position {
            x: target.x - 1,
            y: target.y,
        },
        Position {
            x: target.x + 1,
            y: target.y,
        },
        Position {
            x: target.x,
            y: target.y - 1,
        },
        Position {
            x: target.x,
            y: target.y + 1,
        },
    ]
    .into_iter()
    .filter(|position| map.is_walkable(position.x, position.y))
    .take(gatherers)
    .collect::<Vec<_>>();
    assert_eq!(
        work_tiles.len(),
        gatherers,
        "insufficient source work tiles"
    );
    for (name, position) in ["Mara", "Iven", "Tala"]
        .into_iter()
        .take(gatherers)
        .zip(work_tiles)
    {
        let survivor = driver
            .survivor_by_name(name)
            .unwrap_or_else(|| panic!("stable gatherer `{name}` must exist"));
        driver.fixture_set_position(survivor, position).unwrap();
    }
}

#[test]
fn tactical_day_boundary_applies_one_colony_transaction() {
    let mut driver = colony_driver(201);
    cross_tactical_day_boundary(&mut driver);

    let summary = driver
        .latest_daily_summary()
        .expect("a Tactical day boundary must publish the colony transaction");
    assert_eq!(summary.day, 1);
    assert_eq!(summary.food_consumed, 3);
    assert_eq!(summary.supplies_before - summary.supplies_after, 3);
    assert_eq!(
        driver.resource_current(PoolKind::Supplies),
        Some(summary.supplies_after)
    );
    assert_eq!(driver.last_daily_summary_count(), 1);
}

#[test]
fn tactical_day_boundary_survives_save_load_without_replay() {
    let mut original = colony_driver(202);
    advance_to_turn_23(&mut original);
    original.enter_dungeon(FOUNDATION_DUNGEON_ID).unwrap();
    let checkpoint = original.checkpoint().unwrap();
    let mut restored = FoundationDriver::from_checkpoint(&checkpoint).unwrap();

    wait_once(&mut original, "original Tactical boundary");
    wait_once(&mut restored, "restored Tactical boundary");
    original.advance_day_resolution_frame();
    restored.advance_day_resolution_frame();

    assert_eq!(
        original.latest_daily_summary(),
        restored.latest_daily_summary()
    );
    assert!(original.latest_daily_summary().is_some());
    assert_eq!(
        original.resource_current(PoolKind::Supplies),
        restored.resource_current(PoolKind::Supplies)
    );
    let supplies = restored.resource_current(PoolKind::Supplies);
    restored.advance_idle();
    assert_eq!(restored.resource_current(PoolKind::Supplies), supplies);
}

#[test]
fn every_legal_day_boundary_has_one_summary() {
    let mut outpost = colony_driver(203);
    advance_to_turn_23(&mut outpost);
    wait_once(&mut outpost, "Outpost boundary");
    outpost.advance_day_resolution_frame();
    assert_eq!(outpost.last_day_advanced_count(), 1);
    let outpost_summary = outpost
        .latest_daily_summary()
        .expect("Outpost boundary must publish one summary");
    outpost.advance_idle();
    assert_eq!(outpost.latest_daily_summary(), Some(outpost_summary));

    let mut tactical = colony_driver(204);
    cross_tactical_day_boundary(&mut tactical);
    assert_eq!(tactical.last_day_advanced_count(), 1);
    assert!(
        tactical.latest_daily_summary().is_some(),
        "Tactical boundary must publish one summary"
    );
}

#[test]
fn fixed_shelter_has_every_reachable_gathering_target() {
    let mut driver = colony_driver(214);
    let kinds = driver.resource_node_kinds();

    for required in [
        ResourceNodeType::Trees,
        ResourceNodeType::WaterSource,
        ResourceNodeType::WildPlants,
    ] {
        assert!(
            kinds.contains(&required),
            "fixed shelter is missing the {required:?} gathering target"
        );
    }
    assert!(
        driver.all_resource_nodes_reachable_from_shelter_spawn(),
        "every fixed-shelter resource node must be reachable from the named return spawn"
    );
}

#[test]
fn next_day_forecast_excludes_direct_worker_tick_output() {
    let mut driver = colony_driver(301);
    driver.fixture_set_colony_resource(PoolKind::Supplies, 5);
    place_gatherers_at_source(&mut driver, ResourceNodeType::WaterSource, 1);
    let survivor = driver
        .survivor_by_name("Mara")
        .expect("stable Supplies gatherer must exist");
    let player = driver.player().expect("Foundation player must exist");
    driver
        .submit_action_and_advance_result_frame(
            "assign direct Supplies gathering",
            player,
            "ability.gather_supplies",
            None,
            Some(survivor),
        )
        .expect("direct gathering assignment must resolve");
    wait_once(&mut driver, "direct gathering work tick 1");
    wait_once(&mut driver, "direct gathering work tick 2");

    let forecast = driver.colony_forecast();
    assert_eq!(
        forecast.gathered_supplies, 0,
        "contract=VISUAL-COLONY-WORK-003 expected=next-day forecast excludes \
         direct worker-tick output actual={forecast:?}"
    );
}

#[test]
fn every_buildable_station_catalog_entry_has_an_implemented_effect() {
    let catalog = foundation_content().stations;
    let storage = catalog
        .iter()
        .find(|station| station.station_type == StationType::Storage)
        .expect("Storage remains defined for later compatibility");
    assert!(!storage.buildable);
    assert_eq!(
        storage.unavailable_reason.as_deref(),
        Some("No Foundation effect yet")
    );
    for station in catalog {
        if station.buildable {
            assert!(
                station.effect != StationEffect::Disabled,
                "{} is offered for payment without a represented effect",
                station.label
            );
        }
    }
}

#[test]
fn management_targets_a_named_survivor_and_task() {
    let mut driver = colony_driver(206);
    let selected = driver
        .survivor_by_name("Iven")
        .expect("selected survivor must exist");
    let untouched = driver
        .survivor_by_name("Mara")
        .expect("untouched survivor must exist");
    let player = driver.player().unwrap();

    driver
        .submit_action_and_advance_result_frame(
            "assign Iven to Supplies",
            player,
            "ability.gather_supplies",
            None,
            Some(selected),
        )
        .expect("named task selection must resolve");

    assert!(matches!(
        driver.survivor_task(selected),
        Some(SurvivorTask::Gathering(PoolKind::Supplies))
    ));
    assert!(matches!(
        driver.survivor_task(untouched),
        Some(SurvivorTask::Idle)
    ));
}

#[test]
fn staffing_targets_a_named_survivor_and_station() {
    let mut driver = colony_driver(207);
    let player = driver.player().unwrap();
    driver.fixture_select_station(StationType::Stove);
    driver
        .submit_action_and_advance_result_frame(
            "build first station fixture through production action",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    driver
        .submit_action_and_advance_result_frame(
            "move to safe second station footprint",
            player,
            "ability.move",
            Some(Direction::South),
            None,
        )
        .unwrap();
    driver.fixture_select_station(StationType::Altar);
    driver
        .submit_action_and_advance_result_frame(
            "build second station fixture through production action",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    let survivor = driver
        .survivor_by_name("Iven")
        .expect("selected survivor must exist");
    let selected_station = driver
        .station_by_type(StationType::Altar)
        .expect("selected Altar must exist");
    driver.fixture_complete_construction(selected_station);
    driver.fixture_select_station_assignment(selected_station);

    driver
        .submit_action_and_advance_result_frame(
            "assign named survivor to selected station",
            player,
            "ability.assign_station",
            None,
            Some(survivor),
        )
        .unwrap();

    assert!(
        matches!(
            driver.survivor_task(survivor),
            Some(SurvivorTask::AssignedTo(bits)) if bits == selected_station.to_bits()
        ),
        "staffing must preserve the explicitly selected station"
    );
}

#[test]
fn bed_restores_only_its_assigned_worker_by_the_catalog_amount() {
    let mut driver = colony_driver(216);
    let player = driver.player().unwrap();
    driver.fixture_select_station(StationType::Bed);
    driver
        .submit_action_and_advance_result_frame(
            "build Bed",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    let survivor = driver
        .survivor_by_name("Iven")
        .expect("Bed worker must exist");
    let untouched = driver
        .survivor_by_name("Mara")
        .expect("untouched survivor must exist");
    let bed = driver.station_by_type(StationType::Bed).unwrap();
    driver.fixture_complete_construction(bed);
    driver.fixture_select_station_assignment(bed);
    driver
        .submit_action_and_advance_result_frame(
            "assign Bed worker",
            player,
            "ability.assign_station",
            None,
            Some(survivor),
        )
        .unwrap();
    driver.fixture_set_entity_pool(survivor, PoolKind::Mood, 50);
    let mood_before = driver
        .entity_pool_current(survivor, PoolKind::Mood)
        .unwrap();
    let untouched_before = driver
        .entity_pool_current(untouched, PoolKind::Mood)
        .unwrap();
    advance_to_turn_23(&mut driver);
    wait_once(&mut driver, "Bed recovery boundary");
    driver.advance_day_resolution_frame();

    assert_eq!(
        driver.entity_pool_current(survivor, PoolKind::Mood),
        Some(mood_before + bd_core::colony::survivors::MOOD_REST_BONUS)
    );
    assert_eq!(
        driver.entity_pool_current(untouched, PoolKind::Mood),
        Some(untouched_before)
    );
}

#[test]
fn each_catalog_station_effect_applies_once_when_staffed() {
    let cases = [
        (StationType::Stove, PoolKind::Supplies, 3),
        (StationType::Altar, PoolKind::Faith, 2),
        (StationType::Workshop, PoolKind::Materials, 2),
    ];
    for (offset, (station_type, pool_kind, amount)) in cases.into_iter().enumerate() {
        let mut driver = colony_driver(220 + offset as u64);
        let player = driver.player().unwrap();
        driver.fixture_select_station(station_type);
        driver
            .submit_action_and_advance_result_frame(
                "build catalog station",
                player,
                "ability.build",
                Some(Direction::East),
                None,
            )
            .unwrap();
        let survivor = driver
            .survivor_by_name("Mara")
            .expect("station worker must exist");
        let station = driver.station_by_type(station_type).unwrap();
        driver.fixture_complete_construction(station);
        driver.fixture_select_station_assignment(station);
        driver
            .submit_action_and_advance_result_frame(
                "staff catalog station",
                player,
                "ability.assign_station",
                None,
                Some(survivor),
            )
            .unwrap();
        advance_to_turn_23(&mut driver);
        let before = driver.resource_current(pool_kind).unwrap();
        let forecast = driver.colony_forecast();
        wait_once(&mut driver, "catalog station day");
        driver.advance_day_resolution_frame();
        let after = driver.resource_current(pool_kind).unwrap();
        let expected_delta = if pool_kind == PoolKind::Supplies {
            forecast.supplies_net
        } else {
            amount
        };
        assert_eq!(
            after - before,
            expected_delta,
            "{station_type:?} did not apply its catalog effect exactly once"
        );
    }
}

#[test]
fn disabled_storage_rejection_is_atomic() {
    let mut driver = colony_driver(217);
    let player = driver.player().unwrap();
    let supplies_before = driver.resource_current(PoolKind::Supplies);
    let stations_before = driver.stations().len();
    driver.fixture_select_station(StationType::Storage);

    let reason = driver
        .expect_denied_action(
            "reject disabled Storage",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();

    assert!(format!("{reason:?}").contains("No Foundation effect yet"));
    assert_eq!(driver.resource_current(PoolKind::Supplies), supplies_before);
    assert_eq!(driver.stations().len(), stations_before);
}

#[test]
fn new_dungeon_preserves_last_completed_run() {
    let mut driver = colony_driver(208);
    driver.enter_dungeon(FOUNDATION_DUNGEON_ID).unwrap();
    driver
        .defeat_all_hostiles("clear extraction route")
        .unwrap();
    driver.move_player_to_exit("reach extraction").unwrap();
    driver.extract("complete first run").unwrap();
    assert_eq!(driver.summary().outcome, RunOutcome::Extracted);

    driver.enter_dungeon(FOUNDATION_DUNGEON_ID).unwrap();

    assert_eq!(
        driver.summary().last_completed_outcome,
        RunOutcome::Extracted,
        "starting an active run must not erase completed-run history"
    );
}

#[test]
fn active_and_completed_run_state_survive_save_load_independently() {
    let mut driver = colony_driver(218);
    driver.enter_dungeon(FOUNDATION_DUNGEON_ID).unwrap();
    driver
        .defeat_all_hostiles("clear first extraction")
        .unwrap();
    driver
        .move_player_to_exit("reach first extraction")
        .unwrap();
    driver.extract("complete first extraction").unwrap();
    driver.enter_dungeon(FOUNDATION_DUNGEON_ID).unwrap();
    let checkpoint = driver.checkpoint().unwrap();
    let mut restored = FoundationDriver::from_checkpoint(&checkpoint).unwrap();
    let summary = restored.summary();

    assert_eq!(summary.outcome, RunOutcome::None);
    assert_eq!(summary.last_completed_outcome, RunOutcome::Extracted);
}

#[test]
fn extraction_uses_shelter_return_spawn() {
    let mut driver = colony_driver(209);
    let shelter_spawn = driver.summary().player_position.unwrap();
    driver.enter_dungeon(FOUNDATION_DUNGEON_ID).unwrap();
    driver
        .defeat_all_hostiles("clear extraction route")
        .unwrap();
    driver.move_player_to_exit("reach extraction").unwrap();
    driver.extract("return to shelter").unwrap();

    assert_eq!(driver.summary().player_position, Some(shelter_spawn));
}

#[test]
fn defeat_restart_uses_the_same_shelter_return_spawn() {
    let mut driver = colony_driver(219);
    let shelter_spawn = driver.summary().player_position;
    driver.enter_dungeon(FOUNDATION_DUNGEON_ID).unwrap();
    driver
        .approach_and_attack_first_hostile("engage defeat run")
        .unwrap();
    driver
        .wait_for_player_defeat("complete ordinary defeat")
        .unwrap();
    driver
        .request_transition("restart to title", bd_core::spatial::GameMode::Title, None)
        .unwrap();
    driver.start_colony().unwrap();

    assert_eq!(driver.summary().player_position, shelter_spawn);
    assert_eq!(driver.player_count(), 1);
    assert_eq!(
        driver.summary().last_completed_outcome,
        RunOutcome::Defeated,
        "restart must preserve the latest completed run result"
    );
}

#[test]
fn canonical_feedback_contains_no_duplicate_results() {
    let mut driver = colony_driver(210);
    for _ in 0..3 {
        driver.advance_idle();
    }
    let before = driver.log_messages().len();
    let player = driver.player().unwrap();
    driver
        .submit_action_and_advance_result_frame(
            "build one Stove",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    let messages = driver.log_messages();
    let emitted = &messages[..messages.len() - before];

    assert_eq!(
        emitted.len(),
        1,
        "one accepted build should have one player-facing result: {emitted:?}"
    );
    assert!(
        emitted[0].contains("Stove"),
        "build result must name the constructed station: {emitted:?}"
    );
}

#[test]
fn extraction_emits_one_compact_canonical_result() {
    let mut driver = colony_driver(211);
    driver.enter_dungeon(FOUNDATION_DUNGEON_ID).unwrap();
    driver
        .defeat_all_hostiles("clear canonical feedback run")
        .unwrap();
    driver
        .move_player_to_exit("reach canonical feedback exit")
        .unwrap();
    let before = driver.log_messages().len();
    driver.extract("resolve canonical extraction").unwrap();
    let after = driver.log_messages();
    let new_messages = &after[..after.len() - before];
    let extraction_results = new_messages
        .iter()
        .filter(|message| {
            message.contains("Extracted")
                || message.contains("extract from")
                || message.contains("return to the outpost")
        })
        .collect::<Vec<_>>();

    assert_eq!(extraction_results.len(), 1);
    assert_eq!(
        extraction_results[0].as_str(),
        "Extracted; loot secured: 0."
    );
}
