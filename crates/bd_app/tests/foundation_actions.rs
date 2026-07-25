//! Phase 4 acceptance tests for the unified Foundation action pipeline.

use bd_core::{
    colony::survivors::SurvivorTask,
    direction::Direction,
    signals::{DenialReason, PoolKind},
};
use bd_test_support::FoundationDriver;

const FIXED_DUNGEON: &str = "dungeon.foundation";

fn colony_driver() -> FoundationDriver {
    let mut driver = FoundationDriver::new(53);
    driver.start_colony().unwrap();
    driver
}

fn dungeon_driver() -> FoundationDriver {
    let mut driver = colony_driver();
    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    driver
}

#[test]
fn terminal_attack_maps_to_quick_attack() {
    assert_eq!(
        bd_tui::commands::command_action_id(bd_tui::commands::UiCommand::Attack),
        Some("ability.quick_attack")
    );
}

#[test]
fn pickup_resolves_through_action_pipeline() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_defeat_first_hostile("pickup pipeline fixture")
        .unwrap();
    driver
        .approach_and_pick_up("pickup pipeline acceptance")
        .unwrap();

    let summary = driver.summary();
    assert_eq!(
        summary.carried_items,
        1,
        "pickup logs: {:?}",
        driver.log_messages()
    );
    assert!(
        summary
            .replay_intents
            .iter()
            .any(|record| record.action_id == "ability.pickup" && record.target.is_some())
    );
}

#[test]
fn station_assignment_resolves_through_action_pipeline() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    driver
        .expect_action(
            "station pipeline: build",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    let survivor = driver.first_survivor().unwrap();
    let station = driver.first_station().unwrap();
    driver
        .expect_station_assignment_action("station pipeline: assign", player, survivor, station)
        .unwrap();

    assert!(matches!(
        driver.survivor_task(survivor),
        Some(SurvivorTask::AssignedTo(_))
    ));
}

#[test]
fn survivor_task_assignment_resolves_through_action_pipeline() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    let survivor = driver.first_survivor().unwrap();
    driver
        .expect_action(
            "task pipeline: gather",
            player,
            "ability.assign_gathering",
            None,
            Some(survivor),
        )
        .unwrap();

    assert!(matches!(
        driver.survivor_task(survivor),
        Some(SurvivorTask::Gathering)
    ));
}

#[test]
fn rejected_foundation_action_has_typed_reason() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    let reason = driver
        .expect_denied_action(
            "typed denial",
            player,
            "ability.move",
            Some(Direction::West),
            None,
        )
        .unwrap();

    assert_eq!(reason, DenialReason::BlockedTile);
}

#[test]
fn rejected_action_has_no_partial_mutation() {
    let mut driver = dungeon_driver();
    let player = driver.player().unwrap();
    let item = driver.first_loose_item().unwrap();
    let before = driver.summary();
    let reason = driver
        .expect_denied_action(
            "pickup rejection",
            player,
            "ability.pickup",
            None,
            Some(item),
        )
        .unwrap();
    let after = driver.summary();

    assert_eq!(reason, DenialReason::OutOfRange);
    assert_eq!(after.turn, before.turn);
    assert_eq!(after.loose_items, before.loose_items);
    assert_eq!(after.carried_items, before.carried_items);
    assert_eq!(
        driver.pool_current(PoolKind::ActionPoints),
        Some(3),
        "rejected pickup must not spend AP"
    );
}

#[test]
fn accepted_action_advances_time_once() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    let before = driver.summary();
    driver
        .expect_action("accepted turn", player, "ability.wait", None, None)
        .unwrap();
    let after = driver.summary();

    assert_eq!(after.turn, before.turn + 1);
}

#[test]
fn rejected_action_does_not_advance_time() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    let before = driver.summary();
    driver
        .expect_denied_action(
            "rejected turn",
            player,
            "ability.move",
            Some(Direction::West),
            None,
        )
        .unwrap();

    assert_eq!(driver.summary().turn, before.turn);
}

#[test]
fn replay_record_contains_action_parameters() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    driver
        .expect_action(
            "typed replay",
            player,
            "ability.move",
            Some(Direction::East),
            None,
        )
        .unwrap();

    let record = driver.summary().replay_intents.last().cloned().unwrap();
    assert_eq!(record.action_id, "ability.move");
    assert_eq!(record.direction, Some(Direction::East));
    assert_eq!(record.actor, player.to_bits());
    assert_eq!(record.target, None);
}

#[test]
fn replay_includes_pickup_and_colony_actions() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    let survivor = driver.first_survivor().unwrap();
    driver
        .expect_action(
            "replay: survivor task",
            player,
            "ability.assign_gathering",
            None,
            Some(survivor),
        )
        .unwrap();
    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    driver
        .approach_and_defeat_first_hostile("replay: combat fixture")
        .unwrap();
    driver.approach_and_pick_up("replay: pickup").unwrap();

    let replay = driver.summary().replay_intents;
    assert!(
        replay
            .iter()
            .any(|record| record.action_id == "ability.assign_gathering")
    );
    assert!(
        replay
            .iter()
            .any(|record| record.action_id == "ability.pickup")
    );
}
