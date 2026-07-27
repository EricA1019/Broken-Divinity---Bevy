use std::collections::{HashMap, HashSet};

use bd_core::{
    colony::resources::{ResourcePlacementError, plan_resource_nodes},
    components::{Position, Tile},
    map::SmokeMap,
};
use bd_test_support::{FoundationDriver, foundation_content};

#[test]
fn configured_source_counts_own_new_colony_node_coverage() {
    let content = foundation_content();
    let expected = content
        .colony_sources
        .iter()
        .map(|source| (source.id.clone(), source.spawn_count as usize))
        .collect::<HashMap<_, _>>();
    let mut driver = FoundationDriver::new(7);
    driver.start_colony().unwrap();
    let actual = driver.resource_node_layout().into_iter().fold(
        HashMap::<String, usize>::new(),
        |mut counts, (source_id, _)| {
            *counts.entry(source_id).or_default() += 1;
            counts
        },
    );

    assert_eq!(actual, expected);
    assert_eq!(driver.summary().survivors, 3);
    assert_eq!(
        driver.summary().stations,
        1,
        "D-20 guarantees one starter Basic Processing station"
    );
    assert_eq!(
        driver.station_types(),
        vec![bd_core::colony::stations::StationType::Custom(1)]
    );
}

#[test]
fn same_seed_repeats_layout_and_seed_matrix_preserves_invariants() {
    let mut first = FoundationDriver::new(81);
    let mut second = FoundationDriver::new(81);
    first.start_colony().unwrap();
    second.start_colony().unwrap();
    assert_eq!(first.resource_node_layout(), second.resource_node_layout());

    for seed in [1, 2, 3, 17, 81, u64::MAX] {
        let mut driver = FoundationDriver::new(seed);
        driver.start_colony().unwrap();
        let layout = driver.resource_node_layout();
        assert_eq!(layout.len(), 3, "seed={seed}: incomplete layout");
        assert_eq!(
            layout
                .iter()
                .map(|(source_id, _)| source_id)
                .collect::<HashSet<_>>()
                .len(),
            3,
            "seed={seed}: source coverage changed"
        );
    }
}

#[test]
fn named_128_seed_profile_preserves_complete_reachable_spaced_layouts() {
    let content = foundation_content();
    let profile = &content.colony_placement_profiles[0];
    let map = SmokeMap::new(40, 30, Tile::Floor);
    let origin = Position { x: 1, y: 1 };
    let forbidden = HashSet::from([origin, Position { x: 3, y: 3 }]);

    for seed in 0..128 {
        let plan = plan_resource_nodes(
            &map,
            origin,
            &forbidden,
            &content.colony_sources,
            profile,
            seed,
        )
        .unwrap_or_else(|error| panic!("seed={seed}: {error:?}"));
        assert_eq!(
            plan.len(),
            content.colony_sources.len(),
            "seed={seed}: incomplete source coverage; plan={plan:?}"
        );
        for (index, left) in plan.iter().enumerate() {
            assert!(
                !forbidden.contains(&left.position),
                "seed={seed}: forbidden placement={left:?}"
            );
            for right in plan.iter().skip(index + 1) {
                let distance = (left.position.x - right.position.x).unsigned_abs()
                    + (left.position.y - right.position.y).unsigned_abs();
                assert!(
                    distance >= profile.minimum_manhattan_spacing,
                    "seed={seed}: spacing={distance}; left={left:?}; right={right:?}"
                );
            }
        }
    }
}

#[test]
fn source_file_order_does_not_change_seeded_node_ownership_or_positions() {
    let content = foundation_content();
    let profile = &content.colony_placement_profiles[0];
    let map = SmokeMap::new(40, 30, Tile::Floor);
    let origin = Position { x: 1, y: 1 };
    let forbidden = HashSet::from([origin, Position { x: 3, y: 3 }]);
    let canonical = plan_resource_nodes(
        &map,
        origin,
        &forbidden,
        &content.colony_sources,
        profile,
        901,
    )
    .unwrap();
    let mut permuted = content.colony_sources.clone();
    permuted.reverse();
    let reordered = plan_resource_nodes(&map, origin, &forbidden, &permuted, profile, 901).unwrap();

    assert_eq!(reordered, canonical);
}

#[test]
fn node_planner_satisfies_spatial_invariants_across_map_fixtures() {
    let content = foundation_content();
    let profile = &content.colony_placement_profiles[0];
    let fixtures = [
        (
            "normal",
            SmokeMap::new(40, 30, Tile::Floor),
            HashSet::from([Position { x: 1, y: 1 }]),
        ),
        (
            "constrained",
            SmokeMap::new(16, 12, Tile::Floor),
            HashSet::from([Position { x: 1, y: 1 }]),
        ),
        (
            "occupied-region",
            SmokeMap::new(40, 30, Tile::Floor),
            (1..20)
                .flat_map(|x| (1..10).map(move |y| Position { x, y }))
                .collect(),
        ),
    ];
    for (fixture_id, map, mut forbidden) in fixtures {
        let origin = Position { x: 1, y: 1 };
        forbidden.remove(&origin);
        let plan = plan_resource_nodes(
            &map,
            origin,
            &forbidden,
            &content.colony_sources,
            profile,
            733,
        )
        .unwrap_or_else(|error| panic!("fixture={fixture_id}: {error:?}"));
        let positions = plan
            .iter()
            .map(|placement| placement.position)
            .collect::<HashSet<_>>();
        assert_eq!(positions.len(), plan.len(), "fixture={fixture_id}: overlap");
        for placement in &plan {
            assert!(
                map.is_walkable(placement.position.x, placement.position.y),
                "fixture={fixture_id}: node is not on a walkable tile"
            );
            assert!(
                !forbidden.contains(&placement.position),
                "fixture={fixture_id}: node occupies a forbidden tile"
            );
        }
        for (index, left) in plan.iter().enumerate() {
            for right in plan.iter().skip(index + 1) {
                let distance = (left.position.x - right.position.x).unsigned_abs()
                    + (left.position.y - right.position.y).unsigned_abs();
                assert!(
                    distance >= profile.minimum_manhattan_spacing,
                    "fixture={fixture_id}: spacing={distance}"
                );
            }
        }
    }
}

#[test]
fn impossible_node_layout_is_typed_and_atomic() {
    let map = SmokeMap::new(3, 3, Tile::Floor);
    let content = foundation_content();
    let error = plan_resource_nodes(
        &map,
        Position { x: 1, y: 1 },
        &HashSet::from([Position { x: 1, y: 1 }]),
        &content.colony_sources,
        &content.colony_placement_profiles[0],
        42,
    )
    .unwrap_err();

    assert_eq!(
        error,
        ResourcePlacementError::NoCompleteLayout { requested: 3 }
    );
}

#[test]
fn persisted_node_layout_is_restored_without_regeneration() {
    let mut driver = FoundationDriver::new(912);
    driver.start_colony().unwrap();
    let before = driver.resource_node_layout();
    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();

    assert_eq!(driver.resource_node_layout(), before);
    assert_eq!(driver.summary().resource_nodes, before.len());
}
