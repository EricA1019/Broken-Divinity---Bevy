//! Phase 5 acceptance tests for progression, virtues, and Foundation factions.

use bd_core::{content::FoundationDisposition, signals::PoolKind};
use bd_test_support::{FoundationDriver, foundation_content};

fn dungeon_driver() -> FoundationDriver {
    let mut driver = FoundationDriver::new(67);
    driver.start_colony().unwrap();
    driver.enter_dungeon("dungeon.foundation").unwrap();
    driver
}

#[test]
fn quick_attack_improves_melee() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_attack_first_hostile("melee gain")
        .unwrap();
    assert_eq!(driver.summary().melee_skill, 1);
}

#[test]
fn quick_attack_expresses_thumos() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_attack_first_hostile("thumos gain")
        .unwrap();
    assert_eq!(driver.pool_current(PoolKind::Thumos), Some(1));
}

fn used_item_driver() -> FoundationDriver {
    let mut driver = dungeon_driver();
    driver
        .approach_and_defeat_first_hostile("medicine fixture")
        .unwrap();
    let item = driver.first_loose_item().unwrap();
    driver.fixture_pick_up(item).unwrap();
    let player = driver.player().unwrap();
    driver
        .expect_action(
            "medicine action",
            player,
            "ability.use_item",
            None,
            Some(item),
        )
        .unwrap();
    driver
}

#[test]
fn use_item_improves_medicine() {
    let mut driver = used_item_driver();
    assert_eq!(driver.summary().medicine_skill, 1);
}

#[test]
fn use_item_expresses_temperance() {
    let mut driver = used_item_driver();
    assert_eq!(driver.pool_current(PoolKind::Temperance), Some(1));
}

#[test]
fn rejected_action_grants_no_progression() {
    let mut driver = dungeon_driver();
    let player = driver.player().unwrap();
    let enemy = driver.first_hostile().unwrap();
    driver
        .expect_denied_action(
            "rejected progression",
            player,
            "ability.quick_attack",
            None,
            Some(enemy),
        )
        .unwrap();
    assert_eq!(driver.summary().melee_skill, 0);
    assert_eq!(driver.pool_current(PoolKind::Thumos), Some(0));
}

#[test]
fn player_has_all_six_virtues_and_kleos() {
    let mut driver = dungeon_driver();
    for virtue in [
        PoolKind::Temperance,
        PoolKind::Justice,
        PoolKind::Prudence,
        PoolKind::Fortitude,
        PoolKind::Thumos,
        PoolKind::Metis,
        PoolKind::Kleos,
    ] {
        assert_eq!(driver.pool_current(virtue), Some(0), "missing {virtue:?}");
    }
}

#[test]
fn generic_enemy_defeat_grants_fortitude_but_not_kleos() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_defeat_first_hostile("virtue defeat")
        .unwrap();
    assert_eq!(driver.pool_current(PoolKind::Fortitude), Some(5));
    assert_eq!(driver.pool_current(PoolKind::Kleos), Some(0));
}

#[test]
fn progression_survives_save_load() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_attack_first_hostile("saved gain")
        .unwrap();
    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();
    assert_eq!(driver.summary().melee_skill, 1);
    assert_eq!(driver.pool_current(PoolKind::Thumos), Some(1));
}

#[test]
fn two_foundation_factions_load_with_typed_disposition() {
    let content = foundation_content();
    assert_eq!(content.factions.len(), 2);
    assert_eq!(
        content.factions[0].disposition,
        FoundationDisposition::Hostile
    );
    assert_eq!(
        content.factions[1].disposition,
        FoundationDisposition::Neutral
    );
}

#[test]
fn target_hostility_uses_faction_disposition() {
    let content = foundation_content();
    assert!(bd_core::factions::foundation_is_hostile(
        &content,
        "faction.placeholder_a"
    ));
    assert!(!bd_core::factions::foundation_is_hostile(
        &content,
        "faction.placeholder_b"
    ));
}

#[test]
fn hostile_faction_drives_enemy_ai() {
    let mut driver = dungeon_driver();
    let enemy = driver.first_hostile().unwrap();
    let before = driver.position(enemy).unwrap();
    let player = driver.player().unwrap();
    driver
        .expect_action("hostile AI", player, "ability.wait", None, None)
        .unwrap();
    assert_ne!(driver.position(enemy), Some(before));
}
