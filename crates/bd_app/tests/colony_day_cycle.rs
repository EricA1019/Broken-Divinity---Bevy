//! Phase 6 acceptance tests for the exact-once colony day transaction.

use bd_core::{direction::Direction, signals::PoolKind};
use bd_test_support::FoundationDriver;

fn colony_driver() -> FoundationDriver {
    let mut driver = FoundationDriver::new(79);
    driver.start_colony().unwrap();
    driver
}

fn wait_to_next_day(driver: &mut FoundationDriver) {
    let starting_day = driver.summary().day;
    while driver.summary().day == starting_day {
        let player = driver.player().unwrap();
        driver
            .expect_action("advance colony day", player, "ability.wait", None, None)
            .unwrap();
    }
}

fn rest_to_next_day(driver: &mut FoundationDriver) {
    let player = driver.player().unwrap();
    driver
        .expect_action(
            "rest to next colony day",
            player,
            "ability.rest_until_next_day",
            None,
            None,
        )
        .unwrap();
}

fn build_station(driver: &mut FoundationDriver, staffed: bool) {
    let player = driver.player().unwrap();
    driver
        .expect_action(
            "daily cycle: build station",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    if staffed {
        let survivor = driver.first_survivor().unwrap();
        let station = driver.first_station().unwrap();
        driver
            .expect_station_assignment_action(
                "daily cycle: staff station",
                player,
                survivor,
                station,
            )
            .unwrap();
    }
}

#[test]
fn day_advanced_emits_once() {
    let mut driver = colony_driver();
    wait_to_next_day(&mut driver);
    assert_eq!(driver.last_day_advanced_count(), 1);
    driver.advance_idle();
    assert_eq!(driver.last_day_advanced_count(), 1);
}

#[test]
fn staffed_station_produces_once_per_day() {
    let mut driver = colony_driver();
    build_station(&mut driver, true);
    wait_to_next_day(&mut driver);
    let summary = driver.latest_daily_summary().unwrap();
    assert_eq!(summary.staffed_stations, 1);
    assert_eq!(summary.station_supplies_produced, 3);
    let after = driver.resource_current(PoolKind::Supplies).unwrap();
    driver.advance_idle();
    assert_eq!(driver.resource_current(PoolKind::Supplies), Some(after));
}

#[test]
fn unstaffed_station_does_not_produce() {
    let mut driver = colony_driver();
    build_station(&mut driver, false);
    wait_to_next_day(&mut driver);
    let summary = driver.latest_daily_summary().unwrap();
    assert_eq!(summary.staffed_stations, 0);
    assert_eq!(summary.station_supplies_produced, 0);
}

#[test]
fn survivors_consume_food_once_per_day() {
    let mut driver = colony_driver();
    wait_to_next_day(&mut driver);
    let summary = driver.latest_daily_summary().unwrap();
    assert_eq!(summary.food_consumed, 3);
    assert_eq!(summary.supplies_before - summary.supplies_after, 3);
    let after = summary.supplies_after;
    driver.advance_idle();
    assert_eq!(driver.resource_current(PoolKind::Supplies), Some(after));
}

#[test]
fn starvation_consequence_applies_once_per_day() {
    let mut driver = colony_driver();
    driver.fixture_set_colony_resource(PoolKind::Supplies, 0);
    let survivor = driver.first_survivor().unwrap();
    wait_to_next_day(&mut driver);
    let after = driver
        .entity_pool_current(survivor, PoolKind::Mood)
        .unwrap();
    assert_eq!(after, 90);
    assert_eq!(driver.latest_daily_summary().unwrap().starved_survivors, 3);
    driver.advance_idle();
    assert_eq!(
        driver.entity_pool_current(survivor, PoolKind::Mood),
        Some(after)
    );
}

#[test]
fn gathering_applies_once_per_day() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    let survivor = driver.first_survivor().unwrap();
    driver
        .expect_action(
            "daily cycle: assign gathering",
            player,
            "ability.assign_gathering",
            None,
            Some(survivor),
        )
        .unwrap();
    wait_to_next_day(&mut driver);
    let summary = driver.latest_daily_summary().unwrap();
    assert_eq!(summary.gathering_units, 1);
    let after = summary.materials_after + summary.wild_plants_after + summary.supplies_after;
    driver.advance_idle();
    let idle_total = driver.resource_current(PoolKind::Materials).unwrap()
        + driver.resource_current(PoolKind::WildPlants).unwrap()
        + driver.resource_current(PoolKind::Supplies).unwrap();
    assert_eq!(idle_total, after);
}

#[test]
fn daily_summary_matches_resource_delta() {
    let mut driver = colony_driver();
    build_station(&mut driver, true);
    wait_to_next_day(&mut driver);
    let summary = driver.latest_daily_summary().unwrap();
    assert_eq!(
        summary.supplies_after - summary.supplies_before,
        summary.station_supplies_produced + summary.gathered_supplies - summary.food_consumed
    );
    assert_eq!(
        summary.materials_after - summary.materials_before,
        summary.gathered_materials
    );
    assert_eq!(
        summary.wild_plants_after - summary.wild_plants_before,
        summary.gathered_wild_plants
    );
}

#[test]
fn save_before_day_boundary_does_not_duplicate_cycle() {
    let mut driver = colony_driver();
    while driver.summary().turn < 23 {
        let player = driver.player().unwrap();
        driver
            .expect_action("pre-boundary", player, "ability.wait", None, None)
            .unwrap();
    }
    let checkpoint = driver.checkpoint().unwrap();
    let mut restored = FoundationDriver::from_checkpoint(&checkpoint).unwrap();
    wait_to_next_day(&mut driver);
    wait_to_next_day(&mut restored);
    assert_eq!(
        driver.latest_daily_summary(),
        restored.latest_daily_summary()
    );
    assert_eq!(
        driver.resource_current(PoolKind::Supplies),
        restored.resource_current(PoolKind::Supplies)
    );
}

#[test]
fn save_after_day_boundary_does_not_duplicate_cycle() {
    let mut driver = colony_driver();
    wait_to_next_day(&mut driver);
    let before = driver.latest_daily_summary().unwrap();
    let checkpoint = driver.checkpoint().unwrap();
    let mut restored = FoundationDriver::from_checkpoint(&checkpoint).unwrap();
    restored.advance_idle();
    assert_eq!(restored.latest_daily_summary(), Some(before.clone()));
    assert_eq!(
        restored.resource_current(PoolKind::Supplies),
        Some(before.supplies_after)
    );
}

#[test]
fn rest_advances_exactly_the_remaining_turns() {
    let mut driver = colony_driver();
    for _ in 0..7 {
        let player = driver.player().unwrap();
        driver
            .expect_action("advance before rest", player, "ability.wait", None, None)
            .unwrap();
    }
    let before = driver.summary();
    assert_eq!(before.turn, 7);

    rest_to_next_day(&mut driver);

    let after = driver.summary();
    assert_eq!(after.day, before.day + 1);
    assert_eq!(after.turn, 0);
    assert_eq!(driver.last_day_advanced_count(), 1);
}

#[test]
fn rest_and_individual_waits_run_the_same_daily_transaction() {
    let mut waits = colony_driver();
    let mut rest = colony_driver();
    build_station(&mut waits, true);
    build_station(&mut rest, true);

    wait_to_next_day(&mut waits);
    rest_to_next_day(&mut rest);

    assert_eq!(waits.latest_daily_summary(), rest.latest_daily_summary());
    for kind in [
        PoolKind::Supplies,
        PoolKind::Materials,
        PoolKind::WildPlants,
    ] {
        assert_eq!(waits.resource_current(kind), rest.resource_current(kind));
    }
    let supplies = rest.resource_current(PoolKind::Supplies);
    rest.advance_idle();
    assert_eq!(rest.resource_current(PoolKind::Supplies), supplies);
}

#[test]
fn rest_boundary_survives_save_load_without_repeating_consumers() {
    let mut driver = colony_driver();
    build_station(&mut driver, true);
    rest_to_next_day(&mut driver);
    let summary = driver.latest_daily_summary().unwrap();
    let supplies = driver.resource_current(PoolKind::Supplies);
    let checkpoint = driver.checkpoint().unwrap();

    let mut restored = FoundationDriver::from_checkpoint(&checkpoint).unwrap();
    restored.advance_idle();

    assert_eq!(restored.latest_daily_summary(), Some(summary));
    assert_eq!(restored.resource_current(PoolKind::Supplies), supplies);
    assert_eq!(restored.last_day_advanced_count(), 0);
}

#[test]
fn rest_replay_is_deterministic() {
    let mut first = colony_driver();
    let mut second = colony_driver();

    rest_to_next_day(&mut first);
    rest_to_next_day(&mut second);

    assert_eq!(first.latest_daily_summary(), second.latest_daily_summary());
    assert_eq!(
        first.summary().replay_intents,
        second.summary().replay_intents
    );
    assert_eq!(
        first
            .summary()
            .replay_intents
            .iter()
            .filter(|record| record.action_id == "ability.rest_until_next_day")
            .count(),
        1
    );
}
