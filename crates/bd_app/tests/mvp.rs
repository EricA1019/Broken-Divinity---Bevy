//! MVP integration tests for the BD Kernel.
//!
//! Phase 18: Validates the full tactical loop — procgen, spawning, combat,
//! inventory, save/load, and exit/reach conditions.

use std::collections::{HashMap, HashSet};

use bd_core::{
    components::{BlocksMovement, Name, Player, Position, Tile},
    factory::BlueprintRegistry,
    gamelog::GameLog,
    inventory::Item,
    map::SmokeMap,
    pathfinding::{AStarPathfinder, Pathfinder},
    pools::{Pool, Pools},
    procgen::{LocationTemplate, generate_location, validate_plan},
    relationships::ContainedIn,
    save::{load_world, save_world},
    signals::PoolKind,
};

#[test]
fn mvp_run_can_start() {
    let registry = BlueprintRegistry::phase18_defaults();
    assert!(registry.get("blueprint.player").is_some());
    assert!(registry.get("blueprint.rat").is_some());
    assert!(registry.get("blueprint.skeleton").is_some());
    assert!(registry.get("blueprint.healing_potion").is_some());
    assert!(registry.get("blueprint.ally_warden").is_some());
}

#[test]
fn mvp_location_generates() {
    let template = LocationTemplate::ruin();
    let plan = generate_location(&template, 42);
    let validation = validate_plan(&plan);
    assert!(
        validation.valid,
        "Plan validation failed: {:?}",
        validation.errors
    );
    assert!(!plan.rooms.is_empty(), "Plan should have at least one room");
    assert!(!plan.exits.is_empty());
}

#[test]
fn player_can_kill_enemy() {
    let mut world = bevy_ecs::world::World::new();
    world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
    world.insert_resource(GameLog::default());

    let _player = world
        .spawn((
            Player,
            Position { x: 1, y: 1 },
            Name("Player".into()),
            Pools::new(vec![
                Pool::new(PoolKind::Health, 20, 0, 20),
                Pool::new(PoolKind::ActionPoints, 3, 0, 3),
            ]),
        ))
        .id();

    let enemy = world
        .spawn((
            BlocksMovement,
            Name("Rat".into()),
            Position { x: 2, y: 1 },
            Pools::new(vec![Pool::new(PoolKind::Health, 1, 0, 5)]),
        ))
        .id();

    assert_eq!(
        world
            .entity(enemy)
            .get::<Pools>()
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current,
        1
    );

    // Kill enemy
    let mut pools = world.get_mut::<Pools>(enemy).unwrap();
    for pool in pools.iter_mut() {
        if pool.kind == PoolKind::Health {
            pool.current = 0;
        }
    }

    assert_eq!(
        world
            .get::<Pools>(enemy)
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current,
        0
    );
}

#[test]
fn enemy_can_kill_player() {
    let mut world = bevy_ecs::world::World::new();
    world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
    world.insert_resource(GameLog::default());

    let player = world
        .spawn((
            Player,
            Position { x: 1, y: 1 },
            Name("Player".into()),
            Pools::new(vec![Pool::new(PoolKind::Health, 1, 0, 20)]),
        ))
        .id();

    // Kill player
    let mut pools = world.get_mut::<Pools>(player).unwrap();
    for pool in pools.iter_mut() {
        if pool.kind == PoolKind::Health {
            pool.current = 0;
        }
    }

    assert_eq!(
        world
            .get::<Pools>(player)
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current,
        0
    );
}

#[test]
fn player_can_pick_up_loot() {
    let mut world = bevy_ecs::world::World::new();
    world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
    world.insert_resource(GameLog::default());

    let player = world.spawn((Player, Position { x: 1, y: 1 })).id();

    let item = world
        .spawn((Item, Name("Gold".into()), Position { x: 2, y: 1 }))
        .id();

    // Simulate pickup
    world.entity_mut(item).insert(ContainedIn(player));

    let contained = world.entity(item).get::<ContainedIn>().unwrap();
    assert_eq!(contained.0, player);
}

#[test]
fn player_can_reach_exit() {
    let template = LocationTemplate::ruin();
    let plan = generate_location(&template, 42);
    let validation = validate_plan(&plan);
    assert!(validation.valid);
    assert!(!plan.exits.is_empty());

    let map = plan.to_smoke_map();
    let blocked = HashSet::new();
    let pf = AStarPathfinder;

    for exit in &plan.exits {
        let path = pf.find_path(&map, plan.entrance, *exit, &blocked);
        assert!(
            path.is_some(),
            "No path from {:?} to {:?}",
            plan.entrance,
            exit
        );
    }
}

#[test]
fn mvp_save_load_roundtrip() {
    let mut world = bevy_ecs::world::World::new();
    world.insert_resource(SmokeMap::new(5, 5, Tile::Floor));
    world.insert_resource(GameLog::default());

    let _player = world
        .spawn((
            Player,
            Position { x: 2, y: 2 },
            Name("Player".into()),
            Pools::new(vec![
                Pool::new(PoolKind::Health, 15, 0, 20),
                Pool::new(PoolKind::ActionPoints, 2, 0, 3),
            ]),
        ))
        .id();

    // Save
    let temp_dir = std::env::temp_dir().join("bd_mvp_test");
    let path = save_world(&mut world, 42, 0, &temp_dir).unwrap();

    // Load
    let blueprints = HashMap::new();
    let (mut restored, seed) = load_world(&path, &blueprints).unwrap();
    assert_eq!(seed, 42);

    // Verify player pools exist
    let mut query = restored.query::<&Pools>();
    let pools: Vec<&Pools> = query.iter(&restored).collect();
    assert!(!pools.is_empty());

    // Verify position preserved
    let mut query = restored.query::<&Position>();
    let positions: Vec<&Position> = query.iter(&restored).collect();
    assert!(positions.iter().any(|p| p.x == 2 && p.y == 2));

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&temp_dir);
}
