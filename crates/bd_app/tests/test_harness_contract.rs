//! Contracts for the entity-ID-independent Foundation fingerprint.

use bd_core::{colony::stations::StationType, components::Position, direction::Direction};
use bd_test_support::FoundationDriver;

fn colony_driver(seed: u64) -> FoundationDriver {
    let mut driver = FoundationDriver::new(seed);
    driver.start_colony().expect("Foundation colony must start");
    driver
}

fn build_and_assign(driver: &mut FoundationDriver) {
    let player = driver.player().expect("player must exist");
    driver.fixture_select_station(StationType::Stove);
    driver
        .expect_action(
            "build fingerprint station",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .expect("station build must resolve");
    let station = driver
        .stations()
        .into_iter()
        .find(|entity| driver.station_type(*entity) == Some(StationType::Stove))
        .expect("built Stove must exist");
    driver.fixture_complete_construction(station);
    let survivor = driver
        .survivor_by_name("Survivor 1")
        .expect("stable survivor must exist");
    driver
        .fixture_assign_station(survivor, station)
        .expect("station assignment fixture must resolve");
    driver.advance_idle();
}

#[test]
fn fingerprint_tracks_durable_colony_state_but_excludes_transient_build_state() {
    let mut driver = colony_driver(801);
    let baseline = driver.fingerprint();

    driver.fixture_set_build_interaction(true);
    driver.fixture_select_station(StationType::Workshop);
    assert_eq!(
        driver.fingerprint(),
        baseline,
        "transient Build selection must not enter the durable fingerprint"
    );

    let survivor = driver
        .survivor_by_name("Survivor 1")
        .expect("stable survivor must exist");
    driver
        .fixture_set_position(survivor, Position { x: 10, y: 10 })
        .expect("position fixture must resolve");
    assert_ne!(
        driver.fingerprint(),
        baseline,
        "a durable survivor position change must alter the fingerprint"
    );
}

#[test]
fn enroute_worker_fingerprint_and_next_step_survive_save_load() {
    let mut original = colony_driver(802);
    build_and_assign(&mut original);
    let before = original.fingerprint();
    let checkpoint = original.checkpoint().expect("checkpoint must serialize");
    let mut restored =
        FoundationDriver::from_checkpoint(&checkpoint).expect("checkpoint must restore");

    assert_eq!(
        restored.fingerprint(),
        before,
        "restore must reconstruct the same derived EnRoute activity"
    );

    let original_player = original.player().expect("original player exists");
    let restored_player = restored.player().expect("restored player exists");
    original
        .expect_action(
            "original next worker step",
            original_player,
            "ability.wait",
            None,
            None,
        )
        .expect("original wait resolves");
    restored
        .expect_action(
            "restored next worker step",
            restored_player,
            "ability.wait",
            None,
            None,
        )
        .expect("restored wait resolves");

    assert_eq!(
        restored.fingerprint(),
        original.fingerprint(),
        "save/load must preserve the next deterministic physical step"
    );
}

#[test]
fn blocked_worker_reason_is_recomputed_without_duplicate_log_on_restore() {
    let mut original = colony_driver(803);
    build_and_assign(&mut original);
    let survivor = original
        .survivor_by_name("Survivor 1")
        .expect("stable survivor must exist");
    original
        .fixture_set_position(survivor, Position { x: 8, y: 8 })
        .expect("position fixture must resolve");
    for wall in [
        Position { x: 7, y: 8 },
        Position { x: 9, y: 8 },
        Position { x: 8, y: 7 },
        Position { x: 8, y: 9 },
    ] {
        original.fixture_set_outpost_tile(wall, bd_core::components::Tile::Wall);
    }
    original.advance_idle();
    let before = original.fingerprint();
    assert!(
        before.survivors[0].activity.starts_with("Blocked:"),
        "fixture must establish a typed Blocked activity"
    );
    let logs_before = original.log_messages();
    let checkpoint = original.checkpoint().expect("checkpoint must serialize");
    let mut restored =
        FoundationDriver::from_checkpoint(&checkpoint).expect("checkpoint must restore");

    assert_eq!(restored.fingerprint(), before);
    assert_eq!(
        restored.log_messages(),
        logs_before,
        "derived-state restoration must not duplicate the Blocked transition log"
    );
}

#[test]
fn working_station_contribution_survives_save_load() {
    let mut original = colony_driver(804);
    build_and_assign(&mut original);
    let survivor = original
        .survivor_by_name("Survivor 1")
        .expect("stable survivor must exist");
    original
        .fixture_set_position(survivor, Position { x: 3, y: 1 })
        .expect("adjacent work position must resolve");
    original.advance_idle();
    assert!(
        original.fingerprint().survivors[0]
            .activity
            .starts_with("Working:"),
        "fixture must establish Working activity"
    );
    let checkpoint = original.checkpoint().expect("checkpoint must serialize");
    let mut restored =
        FoundationDriver::from_checkpoint(&checkpoint).expect("checkpoint must restore");
    assert_eq!(restored.fingerprint(), original.fingerprint());

    while original.summary().day == 0 {
        let original_player = original.player().expect("original player exists");
        let restored_player = restored.player().expect("restored player exists");
        original
            .expect_action(
                "original Working day",
                original_player,
                "ability.wait",
                None,
                None,
            )
            .expect("original wait resolves");
        restored
            .expect_action(
                "restored Working day",
                restored_player,
                "ability.wait",
                None,
                None,
            )
            .expect("restored wait resolves");
    }

    assert_eq!(
        restored.latest_daily_summary(),
        original.latest_daily_summary(),
        "restored Working state must produce the same next daily contribution"
    );
    assert_eq!(restored.fingerprint(), original.fingerprint());
}
