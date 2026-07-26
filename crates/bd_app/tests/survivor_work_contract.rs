//! Foundation survivor movement and physical-work contracts.
//!
//! Authority: GDD §6/§8, D-18, and THC-01 through THC-03.
//! Each test owns one observable rule. Assignment is durable intent; movement,
//! activity, and production must still be proven independently.

use std::collections::HashSet;

use bd_core::{
    colony::{resources::pool_for_node, stations::StationType, survivors::SurvivorTask},
    components::{Position, Tile},
    direction::Direction,
    signals::PoolKind,
};
use bd_test_support::FoundationDriver;
use bevy_ecs::entity::Entity;

const FOUNDATION_DUNGEON: &str = "dungeon.foundation";
const MOVEMENT_BUDGET: usize = 48;

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
        .expect_action(step, player, "ability.wait", None, None)
        .unwrap_or_else(|error| panic!("{step}: {error}"));
}

fn wait_until_turn(driver: &mut FoundationDriver, target_turn: u64) {
    while driver.summary().turn < target_turn {
        wait_once(driver, "advance colony turn");
    }
    assert_eq!(driver.summary().turn, target_turn);
}

fn build_station(driver: &mut FoundationDriver, station_type: StationType) -> Entity {
    let player = driver.player().expect("player must exist");
    driver.fixture_select_station(station_type);
    driver
        .expect_action(
            "build physical-work station",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .expect("station build must resolve through the production action");
    driver
        .stations()
        .into_iter()
        .find(|station| {
            driver.station_type(*station) == Some(station_type)
                && driver.position(*station).is_some()
        })
        .expect("built station must exist")
}

fn assign_station(driver: &mut FoundationDriver, survivor: Entity, station: Entity) {
    let player = driver.player().expect("player must exist");
    driver.fixture_select_station_assignment(station);
    driver
        .expect_action(
            "assign named station worker",
            player,
            "ability.assign_station",
            None,
            Some(survivor),
        )
        .expect("station assignment must resolve through the production action");
    assert!(
        matches!(
            driver.survivor_task(survivor),
            Some(SurvivorTask::AssignedTo(bits)) if bits == station.to_bits()
        ),
        "assignment must retain the selected station identity"
    );
}

fn assign_gathering(driver: &mut FoundationDriver, survivor: Entity, kind: PoolKind) {
    let action_id = match kind {
        PoolKind::Supplies => "ability.gather_supplies",
        PoolKind::Materials => "ability.gather_materials",
        PoolKind::WildPlants => "ability.gather_plants",
        other => panic!("unsupported Foundation gathering kind: {other:?}"),
    };
    let player = driver.player().expect("player must exist");
    driver
        .expect_action(
            "assign named gatherer",
            player,
            action_id,
            None,
            Some(survivor),
        )
        .expect("gathering assignment must resolve through the production action");
}

fn manhattan(left: Position, right: Position) -> i32 {
    (left.x - right.x).abs() + (left.y - right.y).abs()
}

fn farthest_interior_position_from(
    targets: &[Position],
    map_width: i32,
    map_height: i32,
) -> Position {
    [
        Position { x: 1, y: 1 },
        Position {
            x: map_width - 2,
            y: 1,
        },
        Position {
            x: 1,
            y: map_height - 2,
        },
        Position {
            x: map_width - 2,
            y: map_height - 2,
        },
    ]
    .into_iter()
    .max_by_key(|candidate| {
        targets
            .iter()
            .map(|target| manhattan(*candidate, *target))
            .min()
            .unwrap_or(i32::MAX)
    })
    .expect("candidate set is non-empty")
}

fn cardinal_work_positions(
    target: Position,
    map_width: i32,
    map_height: i32,
) -> impl Iterator<Item = Position> {
    [
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
    .filter(move |candidate| {
        candidate.x > 0
            && candidate.y > 0
            && candidate.x < map_width - 1
            && candidate.y < map_height - 1
    })
}

#[test]
fn new_assignment_does_not_move_during_paused_confirmation() {
    let mut driver = colony_driver(701);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    let station = build_station(&mut driver, StationType::Stove);
    let before = driver.position(survivor).expect("survivor has a position");

    assign_station(&mut driver, survivor, station);

    assert_eq!(
        driver.position(survivor),
        Some(before),
        "paused assignment must not grant a movement step"
    );
}

#[test]
fn idle_survivor_does_not_move_on_accepted_outpost_turns() {
    let mut driver = colony_driver(717);
    let survivor = named_survivor(&mut driver, "Survivor 2");
    let start = Position { x: 8, y: 8 };
    driver
        .fixture_set_position(survivor, start)
        .expect("idle-position fixture must be valid");

    for step in 0..8 {
        wait_once(&mut driver, &format!("idle survivor turn {step}"));
    }

    assert_eq!(
        driver.position(survivor),
        Some(start),
        "Idle is a stable activity; accepted time alone must not move the survivor"
    );
}

#[test]
fn next_outpost_turn_moves_worker_exactly_one_cardinal_step() {
    let mut driver = colony_driver(702);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    let station = build_station(&mut driver, StationType::Stove);
    let target = driver.position(station).expect("station has a position");
    let before = driver.position(survivor).expect("survivor has a position");
    assign_station(&mut driver, survivor, station);

    wait_once(&mut driver, "move assigned worker once");
    let after = driver
        .position(survivor)
        .expect("survivor remains positioned");

    assert_eq!(
        manhattan(before, after),
        1,
        "one accepted Outpost turn grants exactly one cardinal step"
    );
    assert!(
        manhattan(after, target) < manhattan(before, target),
        "the step must reduce distance to the assigned station"
    );
}

#[test]
fn idle_render_frames_do_not_move_assigned_survivors() {
    let mut driver = colony_driver(703);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    let station = build_station(&mut driver, StationType::Stove);
    assign_station(&mut driver, survivor, station);
    let before = driver.position(survivor).expect("survivor has a position");

    for _ in 0..10 {
        driver.advance_idle();
    }

    assert_eq!(
        driver.position(survivor),
        Some(before),
        "render/scheduler frames without accepted time must not move workers"
    );
}

#[test]
fn tactical_turns_do_not_move_colony_survivors() {
    let mut driver = colony_driver(704);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    let station = build_station(&mut driver, StationType::Stove);
    assign_station(&mut driver, survivor, station);
    driver
        .enter_dungeon(FOUNDATION_DUNGEON)
        .expect("Foundation dungeon entry must resolve");
    let before = driver
        .position(survivor)
        .expect("survivor remains persisted");

    wait_once(&mut driver, "advance one tactical turn");

    assert_eq!(
        driver.position(survivor),
        Some(before),
        "Tactical time must not move colony-scoped survivors"
    );
}

#[test]
fn worker_uses_pathfinding_around_a_wall_blocker() {
    let mut driver = colony_driver(715);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    let station = build_station(&mut driver, StationType::Stove);
    let start = Position { x: 5, y: 2 };
    driver
        .fixture_set_position(survivor, start)
        .expect("movement fixture must be valid");
    driver.fixture_set_outpost_tile(Position { x: 4, y: 2 }, Tile::Wall);
    assign_station(&mut driver, survivor, station);

    wait_once(&mut driver, "path around one wall");
    let after = driver.position(survivor).expect("survivor has a position");

    assert_eq!(
        manhattan(start, after),
        1,
        "a reachable detour must produce one cardinal movement step"
    );
    assert_ne!(
        after, start,
        "worker must path around a blocker instead of repeatedly walking into it"
    );
}

#[test]
fn unreachable_worker_stays_put_and_reports_a_specific_blocked_reason() {
    let mut driver = colony_driver(716);
    let survivor = named_survivor(&mut driver, "Survivor 2");
    let station = build_station(&mut driver, StationType::Stove);
    let start = Position { x: 8, y: 8 };
    driver
        .fixture_set_position(survivor, start)
        .expect("blocked fixture must be valid");
    for wall in [
        Position { x: 7, y: 8 },
        Position { x: 9, y: 8 },
        Position { x: 8, y: 7 },
        Position { x: 8, y: 9 },
    ] {
        driver.fixture_set_outpost_tile(wall, Tile::Wall);
    }
    let log_count = driver.log_messages().len();
    assign_station(&mut driver, survivor, station);
    assert_eq!(driver.position(survivor), Some(start));
    let messages = driver.log_messages();
    let new_count = messages.len().saturating_sub(log_count);
    let feedback = messages
        .into_iter()
        .take(new_count)
        .collect::<Vec<_>>()
        .join("\n");
    for required in ["Survivor 2", "Blocked", "Stove"] {
        assert!(
            feedback.contains(required),
            "blocked feedback must identify `{required}`:\n{feedback}"
        );
    }

    let blocked_log_count = driver.log_messages().len();
    wait_once(&mut driver, "retain unreachable worker");
    assert_eq!(driver.position(survivor), Some(start));
    let duplicate_feedback = driver.log_messages();
    let duplicate_count = duplicate_feedback.len().saturating_sub(blocked_log_count);
    assert!(
        !duplicate_feedback
            .into_iter()
            .take(duplicate_count)
            .any(|message| message.contains("Blocked")),
        "an unchanged Blocked state must not spam another transition message"
    );
}

#[test]
fn station_worker_never_enters_the_station_tile() {
    let mut driver = colony_driver(705);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    let station = build_station(&mut driver, StationType::Stove);
    let station_position = driver.position(station).expect("station has a position");
    assign_station(&mut driver, survivor, station);

    for step in 0..MOVEMENT_BUDGET {
        wait_once(&mut driver, &format!("station approach step {step}"));
        assert_ne!(
            driver.position(survivor),
            Some(station_position),
            "a station is a blocking work target, not a survivor destination"
        );
    }
}

#[test]
fn station_worker_stops_cardinally_adjacent_to_target() {
    let mut driver = colony_driver(706);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    let station = build_station(&mut driver, StationType::Stove);
    let station_position = driver.position(station).expect("station has a position");
    assign_station(&mut driver, survivor, station);

    for step in 0..MOVEMENT_BUDGET {
        wait_once(&mut driver, &format!("reach station work tile {step}"));
    }

    assert_eq!(
        manhattan(
            driver.position(survivor).expect("survivor has a position"),
            station_position,
        ),
        1,
        "a station worker must settle on a cardinally adjacent work tile"
    );
}

#[test]
fn gatherer_never_enters_a_resource_node_tile() {
    let mut driver = colony_driver(707);
    let survivor = named_survivor(&mut driver, "Survivor 3");
    let nodes = driver.resource_nodes_with_state();
    let water_positions = nodes
        .iter()
        .filter(|(_, kind, _, depleted)| !depleted && pool_for_node(*kind) == PoolKind::Supplies)
        .map(|(_, _, position, _)| *position)
        .collect::<Vec<_>>();
    assert!(!water_positions.is_empty(), "fixture needs a Water Source");
    assign_gathering(&mut driver, survivor, PoolKind::Supplies);

    for step in 0..MOVEMENT_BUDGET {
        wait_once(&mut driver, &format!("resource approach step {step}"));
        let position = driver.position(survivor).expect("survivor has a position");
        assert!(
            !water_positions.contains(&position),
            "resource nodes are blocking work targets, not survivor destinations"
        );
    }
}

#[test]
fn assigned_survivors_never_stack_on_one_tile() {
    let mut driver = colony_driver(708);
    let station = build_station(&mut driver, StationType::Stove);
    for name in ["Survivor 1", "Survivor 2"] {
        let survivor = named_survivor(&mut driver, name);
        assign_station(&mut driver, survivor, station);
    }

    for step in 0..MOVEMENT_BUDGET {
        wait_once(&mut driver, &format!("collision reservation step {step}"));
        let positions = ["Survivor 1", "Survivor 2"]
            .into_iter()
            .map(|name| {
                let survivor = named_survivor(&mut driver, name);
                driver.position(survivor).expect("survivor has a position")
            })
            .collect::<Vec<_>>();
        let unique = positions.iter().copied().collect::<HashSet<_>>();
        assert_eq!(
            unique.len(),
            positions.len(),
            "movement must reserve destinations before moving the next survivor"
        );
    }
}

#[test]
fn rest_and_individual_waits_produce_the_same_worker_position() {
    let mut waits = colony_driver(709);
    let mut rest = colony_driver(709);
    for driver in [&mut waits, &mut rest] {
        let survivor = named_survivor(driver, "Survivor 1");
        let station = build_station(driver, StationType::Stove);
        assign_station(driver, survivor, station);
    }

    while waits.summary().day == 0 {
        wait_once(&mut waits, "individual worker movement turn");
    }
    let rest_player = rest.player().expect("player must exist");
    rest.expect_action(
        "rest with assigned worker",
        rest_player,
        "ability.rest_until_next_day",
        None,
        None,
    )
    .expect("rest must resolve");

    let waits_survivor = named_survivor(&mut waits, "Survivor 1");
    let rest_survivor = named_survivor(&mut rest, "Survivor 1");
    assert_eq!(
        waits.position(waits_survivor),
        rest.position(rest_survivor),
        "Rest must replay the same logical worker steps as individual waits"
    );
}

#[test]
fn assigned_but_enroute_station_worker_produces_nothing() {
    let mut driver = colony_driver(710);
    wait_until_turn(&mut driver, 23);
    let survivor = named_survivor(&mut driver, "Survivor 3");
    let station = build_station(&mut driver, StationType::Stove);
    assert_eq!(
        driver.summary().turn,
        23,
        "paused construction must preserve turn 23"
    );
    assign_station(&mut driver, survivor, station);
    assert!(
        manhattan(
            driver.position(survivor).expect("survivor has a position"),
            driver.position(station).expect("station has a position"),
        ) > 1,
        "worker must begin outside physical work range"
    );

    wait_once(&mut driver, "resolve en-route station boundary");
    let summary = driver
        .latest_daily_summary()
        .expect("day boundary must publish a summary");

    assert_eq!(summary.staffed_stations, 0);
    assert_eq!(summary.station_supplies_produced, 0);
}

#[test]
fn assigned_but_enroute_bed_worker_recovers_no_mood() {
    let mut driver = colony_driver(724);
    wait_until_turn(&mut driver, 23);
    let survivor = named_survivor(&mut driver, "Survivor 3");
    let bed = build_station(&mut driver, StationType::Bed);
    driver.fixture_set_entity_pool(survivor, PoolKind::Mood, 50);
    let mood_before = driver
        .entity_pool_current(survivor, PoolKind::Mood)
        .expect("survivor has Mood");
    assert!(
        manhattan(
            driver.position(survivor).expect("survivor has a position"),
            driver.position(bed).expect("bed has a position"),
        ) > 2,
        "worker must remain outside physical work range after the boundary step"
    );
    assign_station(&mut driver, survivor, bed);

    wait_once(&mut driver, "resolve en-route Bed boundary");

    assert_eq!(
        driver.entity_pool_current(survivor, PoolKind::Mood),
        Some(mood_before),
        "an assigned worker must reach the Bed before receiving its effect"
    );
}

#[test]
fn adjacent_station_worker_produces_once() {
    let mut driver = colony_driver(711);
    wait_until_turn(&mut driver, 23);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    let station = build_station(&mut driver, StationType::Stove);
    let station_position = driver.position(station).expect("station has a position");
    let work_position = Position {
        x: station_position.x + 1,
        y: station_position.y,
    };
    driver
        .fixture_set_position(survivor, work_position)
        .expect("work-position fixture must be valid");
    assign_station(&mut driver, survivor, station);

    wait_once(&mut driver, "resolve adjacent station boundary");
    let summary = driver
        .latest_daily_summary()
        .expect("day boundary must publish a summary");

    assert_eq!(driver.position(survivor), Some(work_position));
    assert_eq!(summary.staffed_stations, 1);
    assert_eq!(summary.station_supplies_produced, 3);
}

#[test]
fn assigned_but_enroute_gatherer_produces_nothing() {
    let mut driver = colony_driver(712);
    wait_until_turn(&mut driver, 23);
    let survivor = named_survivor(&mut driver, "Survivor 2");
    let targets = driver
        .resource_nodes_with_state()
        .into_iter()
        .filter(|(_, kind, _, depleted)| !depleted && pool_for_node(*kind) == PoolKind::Supplies)
        .map(|(_, _, position, _)| position)
        .collect::<Vec<_>>();
    let (width, height) = driver.summary().map_size;
    let far_position = farthest_interior_position_from(&targets, width, height);
    driver
        .fixture_set_position(survivor, far_position)
        .expect("far-position fixture must be valid");
    assign_gathering(&mut driver, survivor, PoolKind::Supplies);
    let distance_before = targets
        .iter()
        .map(|target| manhattan(far_position, *target))
        .min()
        .expect("fixture needs a Supplies node");
    assert!(distance_before > 2, "gatherer fixture must be en route");

    wait_once(&mut driver, "resolve en-route gathering boundary");
    let summary = driver
        .latest_daily_summary()
        .expect("day boundary must publish a summary");

    assert_eq!(summary.gathering_units, 0);
    assert_eq!(summary.gathered_supplies, 0);
}

#[test]
fn adjacent_matching_gatherer_produces_once() {
    let mut driver = colony_driver(718);
    wait_until_turn(&mut driver, 23);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    let nodes = driver.resource_nodes_with_state();
    let occupied = nodes
        .iter()
        .map(|(_, _, position, _)| *position)
        .collect::<HashSet<_>>();
    let (_, _, target, _) = nodes
        .iter()
        .find(|(_, kind, _, depleted)| !depleted && pool_for_node(*kind) == PoolKind::Supplies)
        .copied()
        .expect("fixture needs a non-depleted Supplies node");
    let (width, height) = driver.summary().map_size;
    let work_position = cardinal_work_positions(target, width, height)
        .find(|candidate| !occupied.contains(candidate))
        .expect("Supplies node needs one adjacent work tile");
    driver
        .fixture_set_position(survivor, work_position)
        .expect("matching work-position fixture must be valid");
    assign_gathering(&mut driver, survivor, PoolKind::Supplies);

    wait_once(&mut driver, "resolve matching gathering boundary");
    let summary = driver
        .latest_daily_summary()
        .expect("day boundary must publish a summary");

    assert_eq!(summary.gathering_units, 1);
    assert_eq!(summary.gathered_supplies, 1);
}

#[test]
fn gatherer_at_wrong_node_type_produces_nothing() {
    let mut driver = colony_driver(719);
    wait_until_turn(&mut driver, 23);
    let survivor = named_survivor(&mut driver, "Survivor 2");
    let nodes = driver.resource_nodes_with_state();
    let supplies_targets = nodes
        .iter()
        .filter(|(_, kind, _, depleted)| !depleted && pool_for_node(*kind) == PoolKind::Supplies)
        .map(|(_, _, position, _)| *position)
        .collect::<Vec<_>>();
    let occupied = nodes
        .iter()
        .map(|(_, _, position, _)| *position)
        .collect::<HashSet<_>>();
    let (width, height) = driver.summary().map_size;
    let wrong_work_position = nodes
        .iter()
        .filter(|(_, kind, _, depleted)| !depleted && pool_for_node(*kind) == PoolKind::Materials)
        .flat_map(|(_, _, target, _)| cardinal_work_positions(*target, width, height))
        .filter(|candidate| !occupied.contains(candidate))
        .max_by_key(|candidate| {
            supplies_targets
                .iter()
                .map(|target| manhattan(*candidate, *target))
                .min()
                .unwrap_or(i32::MAX)
        })
        .expect("fixture needs a work tile beside the wrong node type");
    assert!(
        supplies_targets
            .iter()
            .all(|target| manhattan(wrong_work_position, *target) > 1),
        "wrong-node fixture must not also be adjacent to a Supplies node"
    );
    driver
        .fixture_set_position(survivor, wrong_work_position)
        .expect("wrong-node fixture must be valid");
    assign_gathering(&mut driver, survivor, PoolKind::Supplies);

    wait_once(&mut driver, "resolve wrong-node gathering boundary");
    let summary = driver
        .latest_daily_summary()
        .expect("day boundary must publish a summary");

    assert_eq!(summary.gathering_units, 0);
    assert_eq!(summary.gathered_supplies, 0);
}

#[test]
fn zero_supply_recovery_remains_reachable_with_physical_gathering() {
    let mut driver = colony_driver(723);
    wait_until_turn(&mut driver, 23);
    driver.fixture_set_colony_resource(PoolKind::Supplies, 0);
    let nodes = driver.resource_nodes_with_state();
    let occupied = nodes
        .iter()
        .map(|(_, _, position, _)| *position)
        .collect::<HashSet<_>>();
    let (width, height) = driver.summary().map_size;
    let mut work_positions = nodes
        .iter()
        .filter(|(_, kind, _, depleted)| !depleted && pool_for_node(*kind) == PoolKind::Supplies)
        .flat_map(|(_, _, target, _)| cardinal_work_positions(*target, width, height))
        .filter(|candidate| !occupied.contains(candidate))
        .collect::<Vec<_>>();
    work_positions.sort_by_key(|position| (position.y, position.x));
    work_positions.dedup();
    assert!(
        work_positions.len() >= 3,
        "fixed shelter needs three distinct physical Supplies work positions; found {work_positions:?}"
    );

    for (name, work_position) in ["Survivor 1", "Survivor 2", "Survivor 3"]
        .into_iter()
        .zip(work_positions)
    {
        let survivor = named_survivor(&mut driver, name);
        driver
            .fixture_set_position(survivor, work_position)
            .expect("zero-Supplies work fixture must be valid");
        assign_gathering(&mut driver, survivor, PoolKind::Supplies);
    }

    wait_once(&mut driver, "resolve zero-Supplies recovery boundary");
    let summary = driver
        .latest_daily_summary()
        .expect("day boundary must publish a summary");

    assert_eq!(summary.supplies_before, 0);
    assert_eq!(summary.food_consumed, 0);
    assert_eq!(summary.gathering_units, 3);
    assert_eq!(summary.gathered_supplies, 3);
    assert_eq!(summary.supplies_after, 3);
}

#[test]
fn blocked_station_worker_produces_nothing() {
    let mut driver = colony_driver(720);
    wait_until_turn(&mut driver, 23);
    let survivor = named_survivor(&mut driver, "Survivor 2");
    let station = build_station(&mut driver, StationType::Stove);
    let start = Position { x: 8, y: 8 };
    driver
        .fixture_set_position(survivor, start)
        .expect("blocked-production fixture must be valid");
    for wall in [
        Position { x: 7, y: 8 },
        Position { x: 9, y: 8 },
        Position { x: 8, y: 7 },
        Position { x: 8, y: 9 },
    ] {
        driver.fixture_set_outpost_tile(wall, Tile::Wall);
    }
    assign_station(&mut driver, survivor, station);

    wait_once(&mut driver, "resolve blocked station boundary");
    let summary = driver
        .latest_daily_summary()
        .expect("day boundary must publish a summary");

    assert_eq!(driver.position(survivor), Some(start));
    assert_eq!(summary.staffed_stations, 0);
    assert_eq!(summary.station_supplies_produced, 0);
}

#[test]
fn forecast_excludes_enroute_worker_output() {
    let mut driver = colony_driver(713);
    let survivor = named_survivor(&mut driver, "Survivor 2");
    let targets = driver
        .resource_nodes_with_state()
        .into_iter()
        .filter(|(_, kind, _, depleted)| !depleted && pool_for_node(*kind) == PoolKind::Supplies)
        .map(|(_, _, position, _)| position)
        .collect::<Vec<_>>();
    let (width, height) = driver.summary().map_size;
    let far_position = farthest_interior_position_from(&targets, width, height);
    driver
        .fixture_set_position(survivor, far_position)
        .expect("far-position fixture must be valid");
    assign_gathering(&mut driver, survivor, PoolKind::Supplies);
    assert!(
        targets
            .iter()
            .all(|target| manhattan(far_position, *target) > 1),
        "forecast fixture must not start in physical work range"
    );

    let forecast = driver.colony_forecast();

    assert_eq!(
        forecast.gathered_supplies, 0,
        "forecast must not promise output from an en-route worker"
    );
}

#[test]
fn rest_and_individual_waits_produce_the_same_daily_resources() {
    let mut waits = colony_driver(721);
    let mut rest = colony_driver(721);
    for driver in [&mut waits, &mut rest] {
        let survivor = named_survivor(driver, "Survivor 1");
        let station = build_station(driver, StationType::Stove);
        assign_station(driver, survivor, station);
    }

    while waits.summary().day == 0 {
        wait_once(&mut waits, "individual daily-resource turn");
    }
    let rest_player = rest.player().expect("player must exist");
    rest.expect_action(
        "rest daily-resource transaction",
        rest_player,
        "ability.rest_until_next_day",
        None,
        None,
    )
    .expect("rest must resolve");

    assert_eq!(
        waits.latest_daily_summary(),
        rest.latest_daily_summary(),
        "Rest and equivalent waits must run the same physical-work transaction"
    );
    for kind in [
        PoolKind::Supplies,
        PoolKind::Materials,
        PoolKind::WildPlants,
        PoolKind::Faith,
    ] {
        assert_eq!(
            waits.resource_current(kind),
            rest.resource_current(kind),
            "Rest diverged from individual waits for {kind:?}"
        );
    }
}

#[test]
fn load_does_not_immediately_move_or_produce_for_assigned_worker() {
    let mut driver = colony_driver(722);
    let survivor = named_survivor(&mut driver, "Survivor 1");
    let station = build_station(&mut driver, StationType::Stove);
    assign_station(&mut driver, survivor, station);
    let before_position = driver.position(survivor);
    let before_supplies = driver.resource_current(PoolKind::Supplies);
    let before_summary = driver.latest_daily_summary();
    let checkpoint = driver.checkpoint().expect("checkpoint must serialize");

    driver
        .restore_checkpoint(&checkpoint)
        .expect("checkpoint must restore");
    driver.advance_idle();

    let restored_survivor = named_survivor(&mut driver, "Survivor 1");
    assert_eq!(driver.position(restored_survivor), before_position);
    assert_eq!(driver.resource_current(PoolKind::Supplies), before_supplies);
    assert_eq!(driver.latest_daily_summary(), before_summary);
}

#[test]
fn save_load_preserves_the_next_deterministic_worker_step() {
    let mut original = colony_driver(714);
    let survivor = named_survivor(&mut original, "Survivor 1");
    let station = build_station(&mut original, StationType::Stove);
    assign_station(&mut original, survivor, station);
    let checkpoint = original.checkpoint().expect("checkpoint must serialize");
    let mut restored =
        FoundationDriver::from_checkpoint(&checkpoint).expect("checkpoint must restore");

    wait_once(&mut original, "original deterministic worker step");
    wait_once(&mut restored, "restored deterministic worker step");

    let original_survivor = named_survivor(&mut original, "Survivor 1");
    let restored_survivor = named_survivor(&mut restored, "Survivor 1");
    assert_eq!(
        original.position(original_survivor),
        restored.position(restored_survivor),
        "save/load must derive the same next movement step"
    );
}
