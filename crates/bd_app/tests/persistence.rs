//! Phase 3 acceptance tests for complete, atomic, deterministic persistence.

use std::path::PathBuf;

use bd_core::{
    colony::stations::StationType, direction::Direction, session::RunOutcome, spatial::EntityScope,
};
use bd_test_support::FoundationDriver;

const FIXED_DUNGEON: &str = "dungeon.foundation";

fn colony_driver() -> FoundationDriver {
    let mut driver = FoundationDriver::new(41);
    driver.start_colony().unwrap();
    driver
}

fn dungeon_driver() -> FoundationDriver {
    let mut driver = colony_driver();
    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    driver
}

fn build_and_assign(driver: &mut FoundationDriver) {
    let player = driver.player().unwrap();
    driver
        .expect_action(
            "persistence fixture: build",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    let station = driver
        .station_by_type(bd_core::colony::stations::StationType::Stove)
        .unwrap();
    driver.fixture_complete_construction(station);
    let survivor = driver.first_survivor().unwrap();
    driver.fixture_assign_station(survivor, station).unwrap();
}

fn temp_save_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("bd-persistence-{label}-{}", std::process::id()))
}

#[test]
fn save_load_colony_preserves_survivors_stations_and_assignments() {
    let mut driver = colony_driver();
    build_and_assign(&mut driver);
    let before = driver.summary();
    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();
    let after = driver.summary();

    assert_eq!(after.survivors, before.survivors);
    assert_eq!(after.stations, before.stations);
    assert_eq!(after.assigned_survivors, before.assigned_survivors);
    assert_eq!(after.resource_nodes, before.resource_nodes);
}

#[test]
fn station_catalog_identity_survives_save_load() {
    let mut driver = colony_driver();
    driver.fixture_select_station(StationType::Workshop);
    let player = driver.player().unwrap();
    driver
        .expect_action(
            "build Workshop identity fixture",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    let before_types = driver.station_types();
    assert!(before_types.contains(&StationType::Workshop));
    assert!(before_types.contains(&StationType::Custom(1)));

    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();

    assert_eq!(driver.station_types(), before_types);
}

#[test]
fn save_load_dungeon_preserves_player_enemy_item_and_scope() {
    let mut driver = dungeon_driver();
    let before = driver.summary();
    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();
    let after = driver.summary();

    assert_eq!(after.player_position, before.player_position);
    assert_eq!(after.player_health, before.player_health);
    assert_eq!(after.hostiles, 1);
    assert_eq!(after.loose_items, 1);
    assert_eq!(driver.scope_count(EntityScope::RunPersistent), 1);
    assert_eq!(driver.scope_count(EntityScope::DungeonTransient), 3);
}

#[test]
fn save_load_post_extraction_does_not_reapply_loot() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_defeat_first_hostile("persistence extraction fixture")
        .unwrap();
    let item = driver.first_loose_item().unwrap();
    driver.fixture_pick_up(item).unwrap();
    driver
        .return_to_colony("persistence extraction fixture")
        .unwrap();
    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();
    driver.advance_idle();

    let summary = driver.summary();
    assert_eq!(summary.outcome, RunOutcome::Extracted);
    assert_eq!(summary.storage_items, 1);
    assert_eq!(summary.extracted_loot, 1);
}

#[test]
fn save_load_defeat_preserves_outcome() {
    let mut driver = dungeon_driver();
    driver.wait_for_player_defeat("persistence defeat").unwrap();
    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();

    let summary = driver.summary();
    assert_eq!(summary.outcome, RunOutcome::Defeated);
    assert_eq!(summary.mode, bd_core::spatial::GameMode::GameOver);
}

#[test]
fn restored_dungeon_can_continue_to_defeat() {
    let mut driver = dungeon_driver();
    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();
    driver
        .wait_for_player_defeat("restored dungeon defeat")
        .unwrap();

    let summary = driver.summary();
    assert_eq!(summary.outcome, RunOutcome::Defeated);
    assert_eq!(summary.mode, bd_core::spatial::GameMode::GameOver);
}

#[test]
fn load_rebuilds_outpost_entity_references() {
    let mut driver = colony_driver();
    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();

    assert!(driver.outpost_party_references_are_valid());
}

#[test]
fn load_rejects_missing_relationship_reference() {
    let mut driver = dungeon_driver();
    let item = driver.first_loose_item().unwrap();
    driver
        .approach_and_defeat_first_hostile("relationship fixture")
        .unwrap();
    driver.fixture_pick_up(item).unwrap();
    let corrupt = driver
        .checkpoint_with_missing_relationship()
        .expect("fixture snapshot");
    let before = driver.summary();

    assert!(driver.restore_checkpoint(&corrupt).is_err());
    assert_eq!(
        driver.summary(),
        before,
        "failed load must leave live state unchanged"
    );
}

#[test]
fn manual_slot_replaces_atomically() {
    let save_dir = temp_save_dir("atomic");
    let mut driver = colony_driver();
    let first = driver.save_manual_slot(&save_dir).unwrap();
    let player = driver.player().unwrap();
    driver
        .expect_action("manual slot: advance", player, "ability.wait", None, None)
        .unwrap();
    let second = driver.save_manual_slot(&save_dir).unwrap();

    assert_eq!(first, second);
    assert!(!save_dir.join("manual-slot.ron.tmp").exists());
    let mut restored = FoundationDriver::new(0);
    restored.load_manual_slot(&save_dir).unwrap();
    assert_eq!(restored.summary().turn, driver.summary().turn);
    let _ = std::fs::remove_dir_all(save_dir);
}

#[test]
fn latest_state_does_not_depend_on_turn_within_day() {
    let save_dir = temp_save_dir("day-boundary");
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    for _ in 0..23 {
        driver
            .expect_action("day boundary", player, "ability.wait", None, None)
            .unwrap();
    }
    driver.save_manual_slot(&save_dir).unwrap();
    driver
        .expect_action("day boundary", player, "ability.wait", None, None)
        .unwrap();
    driver.save_manual_slot(&save_dir).unwrap();

    let mut restored = FoundationDriver::new(0);
    restored.load_manual_slot(&save_dir).unwrap();
    let summary = restored.summary();
    assert_eq!((summary.day, summary.turn), (1, 0));
    let _ = std::fs::remove_dir_all(save_dir);
}

#[test]
fn combat_rng_continues_after_load() {
    let mut uninterrupted = dungeon_driver();
    uninterrupted
        .approach_and_attack_first_hostile("rng: first attack")
        .unwrap();
    let checkpoint = uninterrupted.checkpoint().unwrap();
    let mut restored = FoundationDriver::from_checkpoint(&checkpoint).unwrap();

    uninterrupted
        .approach_and_attack_first_hostile("rng: uninterrupted attack")
        .unwrap();
    restored
        .approach_and_attack_first_hostile("rng: restored attack")
        .unwrap();

    assert_eq!(
        uninterrupted.first_hostile_health(),
        restored.first_hostile_health()
    );
}

#[test]
fn same_snapshot_and_actions_match_after_process_restart() {
    let mut driver = dungeon_driver();
    let checkpoint = driver.checkpoint().unwrap();
    let mut left = FoundationDriver::from_checkpoint(&checkpoint).unwrap();
    let mut right = FoundationDriver::from_checkpoint(&checkpoint).unwrap();
    let left_player = left.player().unwrap();
    let right_player = right.player().unwrap();
    left.expect_action(
        "restart determinism",
        left_player,
        "ability.move",
        Some(Direction::East),
        None,
    )
    .unwrap();
    right
        .expect_action(
            "restart determinism",
            right_player,
            "ability.move",
            Some(Direction::East),
            None,
        )
        .unwrap();

    assert_eq!(left.summary(), right.summary());
}

#[test]
fn load_does_not_reapply_production_progression_or_virtues() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_attack_first_hostile("no duplicate progression")
        .unwrap();
    let before = driver.summary();
    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();
    driver.advance_idle();
    let after = driver.summary();

    assert_eq!(after.melee_skill, before.melee_skill);
    assert_eq!(
        driver.pool_current(bd_core::signals::PoolKind::Thumos),
        Some(1)
    );
    assert_eq!(after.storage_items, before.storage_items);
}
