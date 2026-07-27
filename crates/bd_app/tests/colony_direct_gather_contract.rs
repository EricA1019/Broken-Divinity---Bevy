//! Turn-based direct-gathering contracts.
//!
//! Authority: GDD "Minimum colony foundation" and D-22.
//! These tests intentionally distinguish direct emergency gathering from the
//! station-backed gather/carry/refine recipes.

use std::collections::HashSet;

use bd_core::{
    colony::survivors::SurvivorTask,
    components::{Position, ResourceNodeType},
    signals::PoolKind,
    spatial::GameMode,
};
use bd_test_support::FoundationDriver;
use bevy_ecs::entity::Entity;
use serde::Deserialize;

const DIRECT_GATHER_WORK_TURNS: usize = 3;

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct DirectGatherDefinition {
    id: String,
    source_id: String,
    output_pool: PoolKind,
    output_amount: u32,
    work_turns: u32,
}

fn colony_driver(seed: u64) -> FoundationDriver {
    let mut driver = FoundationDriver::new(seed);
    driver.start_colony().expect("Foundation colony must start");
    driver
}

fn named_survivor(driver: &mut FoundationDriver, name: &str) -> Entity {
    driver
        .survivor_by_name(name)
        .unwrap_or_else(|| panic!("missing stable survivor `{name}`"))
}

fn wait_once(driver: &mut FoundationDriver, step: &str) {
    let player = driver.player().expect("player must exist");
    driver
        .submit_action_and_advance_result_frame(step, player, "ability.wait", None, None)
        .unwrap_or_else(|error| panic!("{step}: {error}"));
}

fn wait_until_turn(driver: &mut FoundationDriver, target_turn: u64) {
    while driver.summary().turn < target_turn {
        wait_once(driver, "advance direct-gather fixture turn");
    }
    assert_eq!(driver.summary().turn, target_turn);
}

fn assign_direct_gathering(driver: &mut FoundationDriver, survivor: Entity, kind: PoolKind) {
    let action_id = match kind {
        PoolKind::Supplies => "ability.gather_supplies",
        PoolKind::Materials => "ability.gather_materials",
        PoolKind::WildPlants => "ability.gather_plants",
        unsupported => panic!("unsupported direct-gather fixture pool: {unsupported:?}"),
    };
    let player = driver.player().expect("player must exist");
    driver
        .submit_action_and_advance_result_frame(
            &format!("assign direct {kind:?} gathering"),
            player,
            action_id,
            None,
            Some(survivor),
        )
        .expect("direct gathering assignment must resolve");
    assert_eq!(
        driver.survivor_task(survivor),
        Some(SurvivorTask::Gathering(kind))
    );
}

fn matching_node_type(kind: PoolKind) -> ResourceNodeType {
    match kind {
        PoolKind::Supplies => ResourceNodeType::WaterSource,
        PoolKind::Materials => ResourceNodeType::Trees,
        PoolKind::WildPlants => ResourceNodeType::WildPlants,
        unsupported => panic!("unsupported direct-gather fixture pool: {unsupported:?}"),
    }
}

fn place_at_matching_work_tile(
    driver: &mut FoundationDriver,
    survivor: Entity,
    kind: PoolKind,
) -> Position {
    let matching_type = matching_node_type(kind);
    let nodes = driver.resource_nodes_with_state();
    let target = nodes
        .iter()
        .find_map(|(_, node_kind, position, depleted)| {
            (*node_kind == matching_type && !depleted).then_some(*position)
        })
        .unwrap_or_else(|| panic!("fixture requires a non-depleted {matching_type:?}"));
    let map = driver.outpost_map();
    let mut occupied = nodes
        .iter()
        .map(|(_, _, position, _)| *position)
        .collect::<HashSet<_>>();
    for entity in driver.survivors() {
        if entity != survivor
            && let Some(position) = driver.position(entity)
        {
            occupied.insert(position);
        }
    }
    if let Some(player) = driver.player()
        && let Some(position) = driver.position(player)
    {
        occupied.insert(position);
    }
    for station in driver.stations() {
        if let Some(position) = driver.position(station) {
            occupied.insert(position);
        }
    }
    let work_position = [
        Position {
            x: target.x,
            y: target.y - 1,
        },
        Position {
            x: target.x,
            y: target.y + 1,
        },
        Position {
            x: target.x - 1,
            y: target.y,
        },
        Position {
            x: target.x + 1,
            y: target.y,
        },
    ]
    .into_iter()
    .find(|candidate| map.is_walkable(candidate.x, candidate.y) && !occupied.contains(candidate))
    .unwrap_or_else(|| panic!("{matching_type:?} requires a free cardinal work tile"));
    driver
        .fixture_set_position(survivor, work_position)
        .expect("direct-gather work-position fixture must be valid");
    work_position
}

fn resource(driver: &FoundationDriver, kind: PoolKind) -> i32 {
    driver
        .resource_current(kind)
        .unwrap_or_else(|| panic!("missing colony resource pool {kind:?}"))
}

/// Contract: CONTENT-DIRECT-GATHER-001
///
/// Given: the Foundation content root.
/// When: direct gathering definitions are loaded as RON data.
/// Then: all three stable task/source/output mappings own positive timing and yield.
/// Must not change: behavior must not depend on a Rust-only task roster.
#[test]
fn foundation_direct_gather_rules_are_declared_in_content() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate directory must have a workspace parent")
        .parent()
        .expect("workspace must have a project parent")
        .join("content/colony_gather_tasks/foundation.ron");
    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "contract=CONTENT-DIRECT-GATHER-001 expected data-defined direct \
             gathering file {}: {error}",
            path.display()
        )
    });
    let definitions: Vec<DirectGatherDefinition> = ron::from_str(&source)
        .unwrap_or_else(|error| panic!("{} must parse: {error}", path.display()));
    let expected = [
        ("gather.supplies", "source.water", PoolKind::Supplies),
        ("gather.materials", "source.trees", PoolKind::Materials),
        (
            "gather.wild_plants",
            "source.wild_plants",
            PoolKind::WildPlants,
        ),
    ];

    assert_eq!(
        definitions.len(),
        expected.len(),
        "contract=CONTENT-DIRECT-GATHER-001 expected exactly one Foundation \
         definition per active direct-gather task"
    );
    for (id, source_id, output_pool) in expected {
        let definition = definitions
            .iter()
            .find(|definition| definition.id == id)
            .unwrap_or_else(|| panic!("missing direct-gather definition `{id}`"));
        assert_eq!(definition.source_id, source_id, "definition={id}");
        assert_eq!(definition.output_pool, output_pool, "definition={id}");
        assert_eq!(definition.output_amount, 1, "definition={id}");
        assert_eq!(
            definition.work_turns,
            u32::try_from(DIRECT_GATHER_WORK_TURNS).unwrap(),
            "definition={id}"
        );
    }
}

/// Contract: COLONY-DIRECT-GATHER-001
///
/// Given: a named survivor adjacent to a matching source.
/// When: three accepted Outpost worker ticks occur.
/// Then: the first two credit nothing and the third credits exactly one.
/// Must not change: output before completion or more than once per operation.
#[test]
fn direct_gather_requires_three_work_ticks_and_credits_once() {
    let mut driver = colony_driver(22_001);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    driver.fixture_set_colony_resource(PoolKind::Supplies, 5);
    place_at_matching_work_tile(&mut driver, survivor, PoolKind::Supplies);
    assign_direct_gathering(&mut driver, survivor, PoolKind::Supplies);
    let before = resource(&driver, PoolKind::Supplies);

    for completed in 1..DIRECT_GATHER_WORK_TURNS {
        wait_once(
            &mut driver,
            &format!("direct gather incomplete work tick {completed}"),
        );
        assert_eq!(
            resource(&driver, PoolKind::Supplies),
            before,
            "contract=COLONY-DIRECT-GATHER-001 case=incomplete \
             tick={completed} expected=no output"
        );
    }

    wait_once(&mut driver, "direct gather completion work tick");
    assert_eq!(
        resource(&driver, PoolKind::Supplies),
        before + 1,
        "contract=COLONY-DIRECT-GATHER-001 case=completion \
         expected=exactly one Supplies after three work ticks"
    );

    wait_once(&mut driver, "direct gather next-operation first tick");
    assert_eq!(
        resource(&driver, PoolKind::Supplies),
        before + 1,
        "contract=COLONY-DIRECT-GATHER-001 case=no-duplicate \
         expected=no second output before another complete operation"
    );
}

/// Contract: COLONY-DIRECT-GATHER-002
///
/// Given: direct gathering has only two of three required work ticks.
/// When: the second tick crosses a day boundary.
/// Then: no legacy day-boundary gathering output is credited.
/// Must not change: Materials before the third worker tick.
#[test]
fn day_boundary_does_not_credit_legacy_direct_gather_output() {
    let mut driver = colony_driver(22_002);
    wait_until_turn(&mut driver, 22);
    let survivor = named_survivor(&mut driver, "Survivor 2");
    driver.fixture_set_colony_resource(PoolKind::Materials, 5);
    place_at_matching_work_tile(&mut driver, survivor, PoolKind::Materials);
    assign_direct_gathering(&mut driver, survivor, PoolKind::Materials);
    let before = resource(&driver, PoolKind::Materials);

    wait_once(
        &mut driver,
        "direct gathering work tick before day boundary",
    );
    wait_once(
        &mut driver,
        "direct gathering work tick crossing day boundary",
    );
    driver.advance_day_resolution_frame();

    assert_eq!(driver.summary().day, 1, "fixture must cross one day");
    assert_eq!(
        resource(&driver, PoolKind::Materials),
        before,
        "contract=COLONY-DIRECT-GATHER-002 case=day-boundary \
         expected=no legacy Materials credit at two-of-three work ticks"
    );
    assert_eq!(
        driver
            .latest_daily_summary()
            .expect("crossed day must publish one summary")
            .gathered_materials,
        0,
        "the daily transaction must not report direct gathering output"
    );
}

/// Contract: COLONY-DIRECT-GATHER-003
///
/// Given: a direct gatherer is adjacent to its source.
/// When: render frames and Tactical-mode frames occur before Outpost work.
/// Then: only accepted Outpost worker ticks count toward completion.
/// Must not change: resource output during non-worker frames.
#[test]
fn render_and_tactical_frames_do_not_advance_direct_gathering() {
    let mut driver = colony_driver(22_003);
    let survivor = named_survivor(&mut driver, "Survivor 3");
    driver.fixture_set_colony_resource(PoolKind::WildPlants, 5);
    place_at_matching_work_tile(&mut driver, survivor, PoolKind::WildPlants);
    assign_direct_gathering(&mut driver, survivor, PoolKind::WildPlants);
    let before = resource(&driver, PoolKind::WildPlants);

    driver.update_frames(12);
    driver
        .request_transition(
            "enter Tactical mode without a colony worker tick",
            GameMode::Tactical,
            Some("dungeon.foundation"),
        )
        .expect("Tactical fixture transition must resolve");
    driver.update_frames(12);
    driver
        .return_to_colony("return from Tactical isolation fixture")
        .expect("Outpost return must resolve");
    driver.update_frames(12);
    assert_eq!(resource(&driver, PoolKind::WildPlants), before);

    for tick in 1..=DIRECT_GATHER_WORK_TURNS {
        wait_once(
            &mut driver,
            &format!("accepted direct-gather worker tick {tick}"),
        );
    }
    assert_eq!(
        resource(&driver, PoolKind::WildPlants),
        before + 1,
        "contract=COLONY-DIRECT-GATHER-003 expected only three accepted \
         Outpost worker ticks to complete gathering"
    );
}

/// Contract: PERSIST-DIRECT-GATHER-001
///
/// Given: a direct gatherer has completed two of three work ticks.
/// When: the colony is saved, restored, and advances one worker tick.
/// Then: exactly one resource is credited.
/// Must not change: output during save/load or the configured task.
#[test]
fn partial_direct_gather_progress_survives_save_load_without_free_output() {
    let mut original = colony_driver(22_004);
    let survivor = named_survivor(&mut original, "Survivor 1");
    original.fixture_set_colony_resource(PoolKind::Supplies, 5);
    place_at_matching_work_tile(&mut original, survivor, PoolKind::Supplies);
    assign_direct_gathering(&mut original, survivor, PoolKind::Supplies);
    wait_once(&mut original, "direct gather progress tick one");
    wait_once(&mut original, "direct gather progress tick two");
    let before = resource(&original, PoolKind::Supplies);

    let checkpoint = original.checkpoint().expect("checkpoint must serialize");
    let mut restored =
        FoundationDriver::from_checkpoint(&checkpoint).expect("checkpoint must restore");
    let restored_survivor = named_survivor(&mut restored, "Survivor 1");
    assert_eq!(resource(&restored, PoolKind::Supplies), before);
    assert_eq!(
        restored.survivor_task(restored_survivor),
        Some(SurvivorTask::Gathering(PoolKind::Supplies))
    );

    wait_once(&mut restored, "restored direct gather completion tick");
    assert_eq!(
        resource(&restored, PoolKind::Supplies),
        before + 1,
        "contract=PERSIST-DIRECT-GATHER-001 expected restored 2/3 progress \
         to complete on the next accepted worker tick"
    );
}

/// Contract: COLONY-DIRECT-GATHER-004
///
/// Given: a survivor has two ticks of partial direct-gather work.
/// When: the player assigns Idle and then starts gathering again.
/// Then: the replacement operation starts from zero.
/// Must not change: resources during cancellation or before three new ticks.
#[test]
fn reassignment_clears_partial_direct_gather_progress_without_output() {
    let mut driver = colony_driver(22_005);
    let survivor = named_survivor(&mut driver, "Survivor 2");
    driver.fixture_set_colony_resource(PoolKind::Materials, 5);
    place_at_matching_work_tile(&mut driver, survivor, PoolKind::Materials);
    assign_direct_gathering(&mut driver, survivor, PoolKind::Materials);
    wait_once(&mut driver, "cancelled direct gather tick one");
    wait_once(&mut driver, "cancelled direct gather tick two");
    let before = resource(&driver, PoolKind::Materials);

    let player = driver.player().expect("player must exist");
    driver
        .submit_action_and_advance_result_frame(
            "replace gathering with Idle",
            player,
            "ability.assign_idle",
            None,
            Some(survivor),
        )
        .expect("Idle reassignment must resolve");
    assert_eq!(resource(&driver, PoolKind::Materials), before);

    assign_direct_gathering(&mut driver, survivor, PoolKind::Materials);
    for tick in 1..DIRECT_GATHER_WORK_TURNS {
        wait_once(
            &mut driver,
            &format!("replacement direct gather incomplete tick {tick}"),
        );
        assert_eq!(
            resource(&driver, PoolKind::Materials),
            before,
            "contract=COLONY-DIRECT-GATHER-004 expected reset progress"
        );
    }
    wait_once(&mut driver, "replacement direct gather completion tick");
    assert_eq!(resource(&driver, PoolKind::Materials), before + 1);
}

/// Supporting matrix for COLONY-DIRECT-GATHER-001.
#[test]
fn every_foundation_direct_gather_task_uses_the_same_three_tick_rule() {
    let cases = [
        ("supplies", PoolKind::Supplies),
        ("materials", PoolKind::Materials),
        ("wild-plants", PoolKind::WildPlants),
    ];

    for (case_id, kind) in cases {
        let mut driver = colony_driver(22_100 + case_id.len() as u64);
        let survivor = named_survivor(&mut driver, "Survivor 1");
        driver.fixture_set_colony_resource(kind, 5);
        let work_position = place_at_matching_work_tile(&mut driver, survivor, kind);
        assign_direct_gathering(&mut driver, survivor, kind);
        let before = resource(&driver, kind);

        for tick in 1..=DIRECT_GATHER_WORK_TURNS {
            wait_once(
                &mut driver,
                &format!("{case_id} direct gathering work tick {tick}"),
            );
        }

        assert_eq!(
            resource(&driver, kind),
            before + 1,
            "contract=COLONY-DIRECT-GATHER-001 case={case_id} \
             source={:?} work_position={work_position:?} expected_delta=1",
            matching_node_type(kind)
        );
    }
}

/// Supporting recovery case for COLONY-DIRECT-GATHER-001.
#[test]
fn zero_supplies_recovers_after_three_worker_ticks_without_waiting_for_day_end() {
    let mut driver = colony_driver(22_006);
    let survivor = named_survivor(&mut driver, "Survivor 3");
    driver.fixture_set_colony_resource(PoolKind::Supplies, 0);
    place_at_matching_work_tile(&mut driver, survivor, PoolKind::Supplies);
    assign_direct_gathering(&mut driver, survivor, PoolKind::Supplies);

    for tick in 1..=DIRECT_GATHER_WORK_TURNS {
        wait_once(
            &mut driver,
            &format!("zero-Supplies direct recovery tick {tick}"),
        );
    }

    assert_eq!(
        resource(&driver, PoolKind::Supplies),
        1,
        "zero-Supplies recovery must complete before the day boundary"
    );
    assert_eq!(driver.summary().day, 0);
}

/// Supporting schedule equivalence for COLONY-DIRECT-GATHER-003.
#[test]
fn rest_and_equivalent_individual_turns_preserve_direct_gather_results() {
    let mut individual = colony_driver(22_007);
    wait_until_turn(&mut individual, 21);
    let survivor = named_survivor(&mut individual, "Survivor 1");
    place_at_matching_work_tile(&mut individual, survivor, PoolKind::Materials);
    assign_direct_gathering(&mut individual, survivor, PoolKind::Materials);
    let checkpoint = individual.checkpoint().expect("fixture must checkpoint");
    let mut rested = FoundationDriver::from_checkpoint(&checkpoint).expect("fixture must restore");

    for tick in 1..=3 {
        wait_once(
            &mut individual,
            &format!("individual turn to day boundary {tick}"),
        );
    }
    let player = rested.player().expect("rested player must exist");
    rested
        .submit_action_and_advance_result_frame(
            "Rest until the same day boundary",
            player,
            "ability.rest_until_next_day",
            None,
            None,
        )
        .expect("Rest must resolve");

    assert_eq!(
        rested.fingerprint(),
        individual.fingerprint(),
        "contract=COLONY-DIRECT-GATHER-003 Rest must replay the same direct \
         gathering transitions as equivalent individual turns"
    );
}
