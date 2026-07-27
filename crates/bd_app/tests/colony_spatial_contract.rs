use bd_core::{
    colony::stations::{StationPlacementDenial, validate_station_placement},
    components::Position,
    direction::Direction,
    pathfinding::{AStarPathfinder, Pathfinder},
    signals::{DenialReason, PoolKind},
};
use bd_test_support::FoundationDriver;
use std::collections::HashSet;

#[test]
fn second_corner_station_is_rejected_before_it_traps_the_player() {
    let mut driver = FoundationDriver::new(0xEC_E55);
    driver.start_colony().expect("colony should start");
    let player = driver.player().expect("shelter player should exist");
    driver
        .expect_action(
            "build Stove east of the shelter return",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .expect("first corner station should be legal");
    let supplies_before = driver.resource_current(PoolKind::Supplies);
    let turn_before = driver.summary().turn;
    let stations_before = driver.summary().stations;

    let reason = driver
        .expect_denied_action(
            "reject Altar south of the shelter return",
            player,
            "ability.build",
            Some(Direction::South),
            None,
        )
        .expect("trapping placement should emit a typed denial");

    assert_eq!(
        reason,
        DenialReason::StationPlacement(StationPlacementDenial::WouldBlockShelterEgress)
    );
    assert_eq!(driver.resource_current(PoolKind::Supplies), supplies_before);
    assert_eq!(driver.summary().turn, turn_before);
    assert_eq!(driver.summary().stations, stations_before);
}

#[test]
fn every_accepted_station_placement_preserves_gate_reachability() {
    let mut driver = FoundationDriver::new(0xEC_E56);
    driver.start_colony().expect("colony should start");
    let map = driver.outpost_map();
    let gate = driver
        .exit_position()
        .expect("fixed shelter must expose its gate");
    let directions = [
        ("north", 0, -1),
        ("south", 0, 1),
        ("east", 1, 0),
        ("west", -1, 0),
    ];

    for player_y in 1..map.height - 1 {
        for player_x in 1..map.width - 1 {
            let player = Position {
                x: player_x,
                y: player_y,
            };
            for (direction, dx, dy) in directions {
                let candidate = Position {
                    x: player.x + dx,
                    y: player.y + dy,
                };
                let result =
                    validate_station_placement(&map, player, gate, &HashSet::new(), candidate);
                let blockers = HashSet::from([candidate]);
                let route = AStarPathfinder.find_path(&map, player, gate, &blockers);

                match result {
                    Ok(()) => assert!(
                        route.is_some(),
                        "accepted placement removed the gate route: player={player:?}, direction={direction}, candidate={candidate:?}"
                    ),
                    Err(StationPlacementDenial::WouldBlockShelterEgress) => assert!(
                        route.is_none(),
                        "egress denial disagrees with pathfinding: player={player:?}, direction={direction}, candidate={candidate:?}, route={route:?}"
                    ),
                    Err(StationPlacementDenial::NotWalkable) => assert!(
                        !map.is_walkable(candidate.x, candidate.y),
                        "walkability denial names a walkable candidate: player={player:?}, direction={direction}, candidate={candidate:?}"
                    ),
                    Err(StationPlacementDenial::Occupied) => panic!(
                        "empty-blocker matrix cannot report Occupied: player={player:?}, direction={direction}, candidate={candidate:?}"
                    ),
                    Err(StationPlacementDenial::NoReachableWorkTile) => assert!(
                        route.is_some(),
                        "work-tile denial must not conceal an egress failure: player={player:?}, direction={direction}, candidate={candidate:?}"
                    ),
                }
            }
        }
    }
}
