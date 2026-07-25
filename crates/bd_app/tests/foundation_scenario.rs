//! Canonical Foundation MVP acceptance tests.
//!
//! These tests intentionally drive the production `BdFoundationPlugin`
//! through `TransitionIntent` and `ActionIntent`. They are the ordered recovery
//! queue: a failure names the first unsupported player-visible step and must
//! not be bypassed with direct ECS mutation.

use bd_core::{direction::Direction, session::RunOutcome, signals::PoolKind, spatial::GameMode};
use bd_test_support::FoundationDriver;

const FIXED_DUNGEON: &str = "dungeon.foundation";

fn colony_driver() -> FoundationDriver {
    let mut driver = FoundationDriver::new(0);
    driver
        .start_colony()
        .expect("clean launch → colony must use the legal transition");
    driver
}

fn dungeon_driver() -> FoundationDriver {
    let mut driver = colony_driver();
    driver
        .enter_dungeon(FIXED_DUNGEON)
        .expect("colony → fixed dungeon must use the legal transition");
    driver
}

#[test]
fn clean_launch_reaches_colony() {
    let mut driver = colony_driver();
    let summary = driver.summary();

    assert_eq!(summary.mode, GameMode::Outpost);
    assert_eq!(summary.survivors, 3);
    assert_eq!(summary.session_phase, GameMode::Outpost);
}

#[test]
fn foundation_app_does_not_register_deferred_systems() {
    let driver = FoundationDriver::new(0);

    assert!(
        driver.deferred_resources_present().is_empty(),
        "foundation registered deferred resources: {:?}",
        driver.deferred_resources_present()
    );
}

#[test]
fn fixed_dungeon_loads_without_procgen() {
    let mut driver = dungeon_driver();
    let summary = driver.summary();

    assert_eq!(summary.mode, GameMode::Tactical);
    assert_eq!(summary.dungeon_id.as_deref(), Some(FIXED_DUNGEON));
    assert_eq!(summary.map_size, (8, 6));
    assert_eq!(summary.hostiles, 1);
    assert_eq!(summary.loose_items, 1);
    assert!(
        !summary
            .trace_events
            .iter()
            .any(|entry| entry.to_ascii_lowercase().contains("procgen")),
        "foundation transition invoked procgen: {:?}",
        summary.trace_events
    );
}

#[test]
fn canonical_colony_setup_uses_actions() {
    let mut driver = colony_driver();
    let player = driver
        .player()
        .expect("colony setup step: a player actor must exist in the shelter");

    driver
        .expect_action(
            "colony setup: build station",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    let station = driver
        .first_station()
        .expect("colony setup step: build action must create a station");
    let survivor = driver
        .first_survivor()
        .expect("colony setup step: starter survivor must exist");

    driver
        .expect_station_assignment_action(
            "colony setup: assign survivor to station",
            player,
            survivor,
            station,
        )
        .unwrap();

    assert_eq!(driver.summary().assigned_survivors, 1);
}

#[test]
fn canonical_dungeon_run_uses_actions() {
    let mut driver = dungeon_driver();
    let player = driver.player().expect("dungeon must contain the player");
    let hostile = driver
        .first_hostile()
        .expect("fixed dungeon must contain its hostile encounter");

    driver
        .expect_action(
            "dungeon run: explore",
            player,
            "ability.move",
            Some(Direction::East),
            None,
        )
        .unwrap();
    driver
        .approach_and_defeat("dungeon run: defeat hostile", hostile)
        .unwrap();
    driver
        .approach_and_pick_up("dungeon run: pick up healing item")
        .unwrap();

    assert_eq!(driver.summary().hostiles, 0);
    assert_eq!(driver.summary().carried_items, 1);
}

#[test]
fn canonical_extraction_applies_loot_once() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_defeat_first_hostile("extraction: defeat hostile")
        .unwrap();
    driver
        .approach_and_pick_up("extraction: collect loot")
        .unwrap();
    driver
        .move_player_to_exit("extraction: reach exit")
        .unwrap();
    driver
        .extract("extraction: explicit extract action")
        .unwrap();

    let once = driver.summary();
    assert_eq!(once.outcome, RunOutcome::Extracted);
    assert_eq!(once.storage_items, 1);
    assert_eq!(once.extracted_loot, 1);

    driver.advance_idle();
    let twice = driver.summary();
    assert_eq!(twice.storage_items, 1, "loot must not be applied twice");
    assert_eq!(twice.extracted_loot, 1);
}

#[test]
fn canonical_defeat_awards_no_loot() {
    let mut driver = dungeon_driver();
    driver
        .wait_for_player_defeat("defeat: resolve normal enemy combat")
        .unwrap();

    let summary = driver.summary();
    assert_eq!(summary.mode, GameMode::GameOver);
    assert_eq!(summary.outcome, RunOutcome::Defeated);
    assert_eq!(summary.storage_items, 0);
    assert_eq!(summary.extracted_loot, 0);
}

#[test]
fn colony_to_title_restart_is_legal() {
    let mut driver = colony_driver();
    driver
        .request_transition("restart: colony → title", GameMode::Title, None)
        .unwrap();

    assert_eq!(driver.summary().mode, GameMode::Title);
}

#[test]
fn defeat_to_title_restart_is_legal() {
    let mut driver = dungeon_driver();
    driver
        .wait_for_player_defeat("restart: reach defeat")
        .unwrap();
    driver
        .request_transition("restart: defeat → title", GameMode::Title, None)
        .unwrap();

    assert_eq!(driver.summary().mode, GameMode::Title);
}

#[test]
fn canonical_progression_improves_one_skill() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_attack_first_hostile("progression: quick attack")
        .unwrap();

    assert!(
        driver.summary().melee_skill > 0,
        "successful quick attack must improve melee"
    );
}

#[test]
fn canonical_progression_emits_two_virtues() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_defeat_first_hostile("progression: combat and defeat")
        .unwrap();

    assert!(
        driver.pool_current(PoolKind::Thumos).unwrap_or_default() > 0,
        "quick attack must express Thumos"
    );
    assert!(
        driver.pool_current(PoolKind::Fortitude).unwrap_or_default() > 0,
        "surviving combat must express Fortitude"
    );
}

#[test]
fn canonical_colony_state_survives_round_trip() {
    let mut driver = colony_driver();
    let before = driver.summary();

    driver
        .enter_dungeon(FIXED_DUNGEON)
        .expect("round trip: colony → dungeon");
    driver
        .return_to_colony("round trip: dungeon → colony")
        .unwrap();

    let after = driver.summary();
    assert_eq!(after.survivors, before.survivors);
    assert_eq!(after.stations, before.stations);
    assert_eq!(after.resource_nodes, before.resource_nodes);
}

#[test]
fn canonical_save_load_resumes_state() {
    let mut driver = colony_driver();
    let before = driver.summary();
    let checkpoint = driver.checkpoint().expect("save must produce a checkpoint");

    driver
        .restore_checkpoint(&checkpoint)
        .expect("load must restore the production app");

    let after = driver.summary();
    assert_eq!(after.mode, before.mode);
    assert_eq!(after.survivors, before.survivors);
    assert_eq!(after.resource_nodes, before.resource_nodes);
}

#[test]
fn same_snapshot_and_actions_match() {
    let mut original = dungeon_driver();
    let checkpoint = original.checkpoint().expect("snapshot must save");
    let mut left =
        FoundationDriver::from_checkpoint(&checkpoint).expect("left branch must restore");
    let mut right =
        FoundationDriver::from_checkpoint(&checkpoint).expect("right branch must restore");

    let left_player = left.player().expect("left player must restore");
    let right_player = right.player().expect("right player must restore");
    left.expect_action(
        "determinism: left move",
        left_player,
        "ability.move",
        Some(Direction::East),
        None,
    )
    .unwrap();
    right
        .expect_action(
            "determinism: right move",
            right_player,
            "ability.move",
            Some(Direction::East),
            None,
        )
        .unwrap();

    assert_eq!(left.summary(), right.summary());
}
