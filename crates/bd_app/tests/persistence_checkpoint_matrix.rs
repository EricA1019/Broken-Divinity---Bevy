//! Persistence checkpoint matrix — every player-visible Foundation projection
//! round-trips the complete normalized fingerprint through both persistence
//! boundaries.
//!
//! Contract: PERSIST-MATRIX-001
//! Authority: GDD "Minimum colony foundation"; docs/DECISIONS-TO-LOCK.md D-09;
//!            docs/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md.
//!
//! Why this is intensive but not brittle:
//! - every case is driven through the production `FoundationDriver` boundary
//!   (no direct world mutation, no alternate resolvers);
//! - the normalized `FoundationFingerprint` is entity-ID-independent and
//!   deliberately excludes transient TUI, pending-action, and log state;
//! - every case round-trips through BOTH the in-memory checkpoint boundary
//!   and the atomic on-disk manual slot;
//! - a pairwise-distinctness guard proves the matrix exercises five different
//!   durable states instead of the same state five times.
//!
//! Each failing assertion names the contract, case, and persistence path.

use std::path::PathBuf;

use bd_core::{
    colony::survivors::SurvivorTask,
    components::{Position, ResourceNodeType},
    signals::PoolKind,
    spatial::GameMode,
};
use bd_test_support::{FoundationDriver, FoundationFingerprint, SurvivorFingerprint};
use bevy_ecs::entity::Entity;

const FIXED_DUNGEON: &str = "dungeon.foundation";
const CONTRACT: &str = "PERSIST-MATRIX-001";

// ---------------------------------------------------------------------------
// Fixture helpers (production-path only)
// ---------------------------------------------------------------------------

fn colony_driver(seed: u64) -> FoundationDriver {
    let mut driver = FoundationDriver::new(seed);
    driver
        .start_colony()
        .unwrap_or_else(|error| panic!("{CONTRACT} fixture colony: {error}"));
    driver
}

fn named_survivor(driver: &mut FoundationDriver, name: &str) -> Entity {
    driver
        .survivor_by_name(name)
        .unwrap_or_else(|| panic!("{CONTRACT} fixture missing stable survivor `{name}`"))
}

fn wait_once(driver: &mut FoundationDriver, step: &str) {
    let player = driver.player().expect("player must exist");
    driver
        .submit_action_and_advance_result_frame(step, player, "ability.wait", None, None)
        .unwrap_or_else(|error| panic!("{CONTRACT} {step}: {error}"));
}

fn matching_node_type(kind: PoolKind) -> ResourceNodeType {
    match kind {
        PoolKind::Supplies => ResourceNodeType::WaterSource,
        PoolKind::Materials => ResourceNodeType::Trees,
        PoolKind::WildPlants => ResourceNodeType::WildPlants,
        unsupported => panic!("{CONTRACT} unsupported fixture pool: {unsupported:?}"),
    }
}

/// Place `survivor` on a free cardinal work tile beside a non-depleted node of
/// the matching type. Uses the same stable fixture rule as the direct-gather
/// contracts.
fn place_at_matching_work_tile(
    driver: &mut FoundationDriver,
    survivor: Entity,
    kind: PoolKind,
) -> Position {
    use std::collections::HashSet;

    let matching_type = matching_node_type(kind);
    let nodes = driver.resource_nodes_with_state();
    let target = nodes
        .iter()
        .find_map(|(_, node_kind, position, depleted)| {
            (*node_kind == matching_type && !depleted).then_some(*position)
        })
        .unwrap_or_else(|| panic!("{CONTRACT} fixture requires a non-depleted {matching_type:?}"));
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
    .unwrap_or_else(|| {
        panic!("{CONTRACT} fixture requires a free cardinal work tile for {matching_type:?}")
    });
    driver
        .fixture_set_position(survivor, work_position)
        .unwrap_or_else(|error| panic!("{CONTRACT} work-position fixture: {error}"));
    work_position
}

fn assign_direct_gathering(driver: &mut FoundationDriver, survivor: Entity, kind: PoolKind) {
    let action_id = match kind {
        PoolKind::Supplies => "ability.gather_supplies",
        PoolKind::Materials => "ability.gather_materials",
        PoolKind::WildPlants => "ability.gather_plants",
        unsupported => panic!("{CONTRACT} unsupported fixture pool: {unsupported:?}"),
    };
    let player = driver.player().expect("player must exist");
    driver
        .submit_action_and_advance_result_frame(
            &format!("{CONTRACT} assign direct {kind:?} gathering"),
            player,
            action_id,
            None,
            Some(survivor),
        )
        .unwrap_or_else(|error| panic!("{CONTRACT} direct-gather assignment: {error}"));
    assert_eq!(
        driver.survivor_task(survivor),
        Some(SurvivorTask::Gathering(kind)),
        "{CONTRACT} assigned survivor must hold the durable gathering task"
    );
}

// ---------------------------------------------------------------------------
// Projection cases
// ---------------------------------------------------------------------------

struct ProjectionCase {
    id: &'static str,
    seed: u64,
    /// Advance a freshly-started colony into this projection.
    build: fn(&mut FoundationDriver),
    /// Assert the projection-specific durable invariants before any
    /// persistence round-trip.
    verify: fn(&FoundationDriver, &FoundationFingerprint, &str),
}

fn working_survivor(fingerprint: &FoundationFingerprint) -> &SurvivorFingerprint {
    fingerprint
        .survivors
        .iter()
        .find(|entry| entry.name == "Survivor 1")
        .unwrap_or_else(|| panic!("{CONTRACT} fixture must fingerprint `Survivor 1`"))
}

const PROJECTIONS: &[ProjectionCase] = &[
    ProjectionCase {
        id: "colony-idle",
        seed: 41,
        build: |_driver| {},
        verify: |_driver, fingerprint, case| {
            assert_eq!(
                fingerprint.mode,
                GameMode::Outpost,
                "{CONTRACT} case={case} expected an idle Outpost projection"
            );
            assert!(
                !fingerprint.survivors.is_empty(),
                "{CONTRACT} case={case} expected survivors in the projection"
            );
        },
    },
    ProjectionCase {
        id: "colony-working",
        seed: 22_004,
        build: |driver| {
            driver.fixture_set_colony_resource(PoolKind::Supplies, 5);
            let survivor = named_survivor(driver, "Survivor 1");
            place_at_matching_work_tile(driver, survivor, PoolKind::Supplies);
            assign_direct_gathering(driver, survivor, PoolKind::Supplies);
            wait_once(driver, "working projection tick one");
            wait_once(driver, "working projection tick two");
        },
        verify: |_driver, fingerprint, case| {
            let survivor = working_survivor(fingerprint);
            assert!(
                survivor.activity.contains("Working"),
                "{CONTRACT} case={case} expected a Working survivor, activity={:?}",
                survivor.activity
            );
            assert_eq!(
                survivor
                    .direct_gather_progress
                    .as_ref()
                    .map(|(_, progress)| *progress),
                Some(2),
                "{CONTRACT} case={case} expected two of three gather work turns"
            );
        },
    },
    ProjectionCase {
        id: "dungeon-carrying-loot",
        seed: 7,
        build: |driver| {
            driver
                .enter_dungeon(FIXED_DUNGEON)
                .unwrap_or_else(|error| panic!("{CONTRACT} enter dungeon: {error}"));
            driver
                .approach_and_defeat_first_hostile("carrying-loot projection")
                .unwrap_or_else(|error| panic!("{CONTRACT} defeat hostile: {error}"));
            let item = driver
                .first_loose_item()
                .expect("{CONTRACT} fixture requires a loose item");
            driver
                .fixture_pick_up(item)
                .unwrap_or_else(|error| panic!("{CONTRACT} pick up loot: {error}"));
        },
        verify: |_driver, fingerprint, case| {
            let player = fingerprint
                .player
                .as_ref()
                .expect("{CONTRACT} fixture requires a player fingerprint");
            assert!(
                !player.inventory.is_empty(),
                "{CONTRACT} case={case} expected the player to carry loot"
            );
        },
    },
    ProjectionCase {
        id: "extracted",
        seed: 7,
        build: |driver| {
            driver
                .enter_dungeon(FIXED_DUNGEON)
                .unwrap_or_else(|error| panic!("{CONTRACT} enter dungeon: {error}"));
            driver
                .approach_and_defeat_first_hostile("extracted projection")
                .unwrap_or_else(|error| panic!("{CONTRACT} defeat hostile: {error}"));
            let item = driver
                .first_loose_item()
                .expect("{CONTRACT} fixture requires a loose item");
            driver
                .fixture_pick_up(item)
                .unwrap_or_else(|error| panic!("{CONTRACT} pick up loot: {error}"));
            driver
                .return_to_colony("extracted projection")
                .unwrap_or_else(|error| panic!("{CONTRACT} return to colony: {error}"));
        },
        verify: |_driver, fingerprint, case| {
            assert_eq!(
                fingerprint.outcome,
                bd_core::session::RunOutcome::Extracted,
                "{CONTRACT} case={case} expected an extracted projection"
            );
            assert_eq!(
                fingerprint.last_completed_outcome,
                bd_core::session::RunOutcome::Extracted,
                "{CONTRACT} case={case} expected last-completed extraction"
            );
            assert!(
                fingerprint.extracted_loot >= 1,
                "{CONTRACT} case={case} expected extracted loot in the projection"
            );
        },
    },
    ProjectionCase {
        id: "game-over",
        seed: 11,
        build: |driver| {
            driver
                .enter_dungeon(FIXED_DUNGEON)
                .unwrap_or_else(|error| panic!("{CONTRACT} enter dungeon: {error}"));
            driver
                .wait_for_player_defeat("game-over projection")
                .unwrap_or_else(|error| panic!("{CONTRACT} player defeat: {error}"));
        },
        verify: |_driver, fingerprint, case| {
            assert_eq!(
                fingerprint.mode,
                GameMode::GameOver,
                "{CONTRACT} case={case} expected a Game Over projection"
            );
            assert_eq!(
                fingerprint.outcome,
                bd_core::session::RunOutcome::Defeated,
                "{CONTRACT} case={case} expected a defeated projection"
            );
        },
    },
];

fn temp_save_dir(case: &str) -> PathBuf {
    std::env::temp_dir().join(format!("bd-persist-matrix-{case}-{}", std::process::id()))
}

// ---------------------------------------------------------------------------
// Matrix tests
// ---------------------------------------------------------------------------

/// Every projection round-trips the full normalized fingerprint through the
/// in-memory checkpoint boundary.
#[test]
fn every_projection_round_trips_the_full_fingerprint_through_checkpoint() {
    for case in PROJECTIONS {
        let mut driver = colony_driver(case.seed);
        (case.build)(&mut driver);
        let before = driver.fingerprint();
        (case.verify)(&driver, &before, case.id);

        let checkpoint = driver
            .checkpoint()
            .unwrap_or_else(|error| panic!("{CONTRACT} case={} checkpoint: {error}", case.id));
        driver
            .restore_checkpoint(&checkpoint)
            .unwrap_or_else(|error| panic!("{CONTRACT} case={} restore: {error}", case.id));
        let after = driver.fingerprint();

        assert_eq!(
            after, before,
            "{CONTRACT} case={} path=checkpoint restore must preserve the complete \
             durable fingerprint",
            case.id
        );
    }
}

/// Every projection round-trips the full normalized fingerprint through the
/// atomic on-disk manual slot into a fresh runtime.
#[test]
fn every_projection_round_trips_the_full_fingerprint_through_manual_slot() {
    for case in PROJECTIONS {
        let dir = temp_save_dir(case.id);
        let mut driver = colony_driver(case.seed);
        (case.build)(&mut driver);
        let before = driver.fingerprint();
        (case.verify)(&driver, &before, case.id);

        driver
            .save_manual_slot(&dir)
            .unwrap_or_else(|error| panic!("{CONTRACT} case={} save: {error}", case.id));
        assert!(
            !dir.join("manual-slot.ron.tmp").exists(),
            "{CONTRACT} case={} path=manual-slot must replace atomically without a temp file",
            case.id
        );

        let mut restored = FoundationDriver::new(0);
        restored
            .load_manual_slot(&dir)
            .unwrap_or_else(|error| panic!("{CONTRACT} case={} load: {error}", case.id));
        let after = restored.fingerprint();

        assert_eq!(
            after, before,
            "{CONTRACT} case={} path=manual-slot must preserve the complete durable \
             fingerprint in a fresh runtime",
            case.id
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// The matrix covers five pairwise-distinct durable states; it is not the
/// same state asserted five times.
#[test]
fn matrix_projections_are_pairwise_distinct_durable_states() {
    let fingerprints = PROJECTIONS
        .iter()
        .map(|case| {
            let mut driver = colony_driver(case.seed);
            (case.build)(&mut driver);
            (case.id, driver.fingerprint())
        })
        .collect::<Vec<_>>();

    for (index, (id_a, fingerprint_a)) in fingerprints.iter().enumerate() {
        for (id_b, fingerprint_b) in fingerprints.iter().skip(index + 1) {
            assert_ne!(
                fingerprint_a, fingerprint_b,
                "{CONTRACT} projections `{id_a}` and `{id_b}` must describe different \
                 durable states"
            );
        }
    }
}
