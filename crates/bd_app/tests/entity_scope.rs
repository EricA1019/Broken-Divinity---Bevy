//! Phase 2 acceptance tests for explicit entity scope and location continuity.

use bd_core::{
    direction::Direction,
    spatial::{EntityScope, GameMode},
};
use bd_test_support::FoundationDriver;

const FIXED_DUNGEON: &str = "dungeon.foundation";

fn colony_driver() -> FoundationDriver {
    let mut driver = FoundationDriver::new(0);
    driver.start_colony().expect("start colony");
    driver
}

fn build_station(driver: &mut FoundationDriver) -> bevy_ecs::entity::Entity {
    let player = driver.player().expect("shelter player must exist");
    driver
        .submit_action_and_advance_result_frame(
            "scope fixture: build station",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    let station = driver
        .station_by_type(bd_core::colony::stations::StationType::Stove)
        .expect("build must create Stove");
    driver.fixture_complete_construction(station);
    station
}

#[test]
fn built_station_survives_dungeon_round_trip() {
    let mut driver = colony_driver();
    let stations_before = driver.summary().stations;
    let station = build_station(&mut driver);
    assert_eq!(
        driver.entity_scope(station),
        Some(EntityScope::ColonyPersistent)
    );

    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    driver
        .return_to_colony("scope: return with built station")
        .unwrap();

    assert!(driver.entity_exists(station));
    assert_eq!(driver.summary().stations, stations_before + 1);
}

#[test]
fn station_assignment_survives_dungeon_round_trip() {
    let mut driver = colony_driver();
    let station = build_station(&mut driver);
    let survivor = driver
        .survivor_by_name("Survivor 1")
        .expect("named starter survivor");
    driver
        .fixture_assign_station(survivor, station)
        .expect("legacy assignment setup must use the production assignment system");
    assert_eq!(driver.summary().assigned_survivors, 1);

    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    driver
        .return_to_colony("scope: return with assignment")
        .unwrap();

    assert!(driver.entity_exists(station));
    assert!(driver.entity_exists(survivor));
    assert_eq!(driver.summary().assigned_survivors, 1);
}

#[test]
fn resource_nodes_survive_dungeon_round_trip() {
    let mut driver = colony_driver();
    let nodes = driver.resource_nodes();
    assert!(!nodes.is_empty());
    assert!(
        nodes
            .iter()
            .all(|node| { driver.entity_scope(*node) == Some(EntityScope::ColonyPersistent) })
    );

    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    driver
        .return_to_colony("scope: return with resource nodes")
        .unwrap();

    assert!(nodes.iter().all(|node| driver.entity_exists(*node)));
    assert_eq!(driver.summary().resource_nodes, nodes.len());
}

#[test]
fn colony_entities_do_not_block_dungeon_queries() {
    let mut driver = colony_driver();
    let station = build_station(&mut driver);
    assert_eq!(
        driver.position(station),
        Some(bd_core::components::Position { x: 2, y: 1 })
    );

    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    let player = driver.player().expect("dungeon player");
    driver
        .submit_action_and_advance_result_frame(
            "scope: dungeon movement ignores colony station",
            player,
            "ability.move",
            Some(Direction::East),
            None,
        )
        .unwrap();

    assert_eq!(
        driver.position(player),
        Some(bd_core::components::Position { x: 2, y: 1 })
    );
}

#[test]
fn dungeon_enemy_is_removed_on_extraction() {
    let mut driver = colony_driver();
    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    let hostile = driver.first_hostile().expect("fixed hostile");
    assert_eq!(
        driver.entity_scope(hostile),
        Some(EntityScope::DungeonTransient)
    );

    driver
        .return_to_colony("scope: extract without combat")
        .unwrap();

    assert!(!driver.entity_exists(hostile));
    assert_eq!(driver.scope_count(EntityScope::DungeonTransient), 0);
}

#[test]
fn uncollected_dungeon_loot_is_removed() {
    let mut driver = colony_driver();
    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    let item = driver.first_loose_item().expect("fixed healing item");
    assert_eq!(
        driver.entity_scope(item),
        Some(EntityScope::DungeonTransient)
    );

    driver.return_to_colony("scope: leave loot behind").unwrap();

    assert!(!driver.entity_exists(item));
    assert_eq!(driver.summary().storage_items, 0);
}

#[test]
fn player_survives_location_cleanup() {
    let mut driver = colony_driver();
    let player = driver.player().expect("shelter player");
    assert_eq!(
        driver.entity_scope(player),
        Some(EntityScope::RunPersistent)
    );

    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    driver.return_to_colony("scope: preserve player").unwrap();

    assert!(driver.entity_exists(player));
    assert_eq!(driver.summary().mode, GameMode::Outpost);
}

#[test]
fn carried_loot_reaches_extraction_resolver() {
    let mut driver = colony_driver();
    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    driver
        .approach_and_defeat_first_hostile("scope fixture: clear encounter")
        .unwrap();
    let item = driver.first_loose_item().expect("fixed healing item");
    driver
        .fixture_pick_up(item)
        .expect("legacy pickup setup must use the production inventory system");
    assert_eq!(driver.summary().carried_items, 1);

    driver
        .return_to_colony("scope: transfer carried loot")
        .unwrap();

    assert_eq!(driver.summary().storage_items, 1);
    assert!(!driver.entity_exists(item));
}

#[test]
fn cleanup_is_idempotent() {
    let mut driver = colony_driver();
    driver.enter_dungeon(FIXED_DUNGEON).unwrap();
    driver.return_to_colony("scope: first cleanup").unwrap();
    let after_cleanup = driver.summary();

    driver.advance_idle();
    driver.advance_idle();

    let after_idle = driver.summary();
    assert_eq!(after_idle.survivors, after_cleanup.survivors);
    assert_eq!(after_idle.resource_nodes, after_cleanup.resource_nodes);
    assert_eq!(
        driver.scope_count(EntityScope::DungeonTransient),
        0,
        "idle schedules must not repeat cleanup"
    );
    assert!(
        !driver
            .log_messages()
            .iter()
            .any(|message| message.contains("invalid entity")),
        "cleanup emitted an invalid-entity warning"
    );
}
