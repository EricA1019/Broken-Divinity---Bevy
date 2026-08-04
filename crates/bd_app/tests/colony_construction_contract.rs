use bd_core::{
    colony::{stations::StationType, survivors::SurvivorTask},
    direction::Direction,
};
use bd_test_support::FoundationDriver;

fn colony_driver() -> FoundationDriver {
    let mut driver = FoundationDriver::new(90210);
    driver.start_colony().expect("colony must start");
    driver
}

#[test]
fn accepted_build_is_a_paid_non_operational_construction_site() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    driver
        .submit_action_and_advance_result_frame(
            "place Stove construction site",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();

    let site = driver.station_by_type(StationType::Stove).unwrap();
    assert_eq!(
        driver.construction_progress(site),
        Some((0, 4)),
        "a placed station must begin as an unfinished four-work site"
    );
    assert!(
        !driver.station_is_operational(site),
        "an unfinished site must not provide station behavior"
    );
}

#[test]
fn idle_survivors_travel_to_and_complete_construction_without_stealing_assigned_workers() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    let protected = driver
        .survivor_by_name("Mara")
        .expect("stable protected survivor must exist");
    driver
        .submit_action_and_advance_result_frame(
            "assign protected survivor to defense",
            player,
            "ability.assign_defending",
            None,
            Some(protected),
        )
        .unwrap();
    driver
        .submit_action_and_advance_result_frame(
            "place Stove construction site",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    let site = driver.station_by_type(StationType::Stove).unwrap();
    let idle_workers = driver
        .survivors()
        .into_iter()
        .filter(|survivor| *survivor != protected)
        .collect::<Vec<_>>();
    let starting_positions = idle_workers
        .iter()
        .map(|worker| (*worker, driver.position(*worker).unwrap()))
        .collect::<Vec<_>>();

    for turn in 0..40 {
        if driver.station_is_operational(site) {
            break;
        }
        driver
            .submit_action_and_advance_result_frame(
                &format!("construction worker turn {turn}"),
                player,
                "ability.wait",
                None,
                None,
            )
            .unwrap();
    }

    assert!(driver.station_is_operational(site));
    assert_eq!(
        driver.survivor_task(protected),
        Some(SurvivorTask::Defending),
        "automatic construction must not steal assigned workers"
    );
    assert!(
        starting_positions
            .iter()
            .any(|(worker, before)| driver.position(*worker) != Some(*before)),
        "at least one idle worker must visibly travel to the site"
    );
}

#[test]
fn render_frames_and_save_load_do_not_grant_construction_work() {
    let mut driver = colony_driver();
    let player = driver.player().unwrap();
    driver
        .submit_action_and_advance_result_frame(
            "place persisted construction site",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .unwrap();
    let site = driver.station_by_type(StationType::Stove).unwrap();
    let before = driver.construction_progress(site).unwrap();
    driver.update_frames(8);
    assert_eq!(driver.construction_progress(site), Some(before));

    let fingerprint_before = driver.fingerprint();
    let checkpoint = driver.checkpoint().unwrap();
    let mut restored = FoundationDriver::from_checkpoint(&checkpoint).unwrap();
    let restored_site = restored.station_by_type(StationType::Stove).unwrap();
    assert_eq!(restored.construction_progress(restored_site), Some(before));
    assert!(!restored.station_is_operational(restored_site));
    assert_eq!(restored.fingerprint(), fingerprint_before);
}
