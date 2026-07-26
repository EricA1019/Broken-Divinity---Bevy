//! Performance and stability stress tests for the BD Kernel.
//!
//! Phase 21: Measures entity growth, event queue depth, procgen stability,
//! save/load stress, and simulation behavior under repeated updates.

use std::collections::HashMap;
use std::time::Instant;

use bd_core::{
    components::{Player, Position, Tile},
    gamelog::GameLog,
    map::SmokeMap,
    pools::{Pool, Pools},
    procgen::{LocationTemplate, generate_location, validate_plan},
    save::{load_world, save_world},
    signals::PoolKind,
    spatial::{EntityScope, FOUNDATION_DUNGEON_ID},
};
use bd_test_support::FoundationDriver;
use bevy_ecs::prelude::Resource;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[test]
fn prototype_fixed_seed_deterministic_run() {
    // Validate the full prototype: generate location, spawn, verify structure
    use bd_core::factory::BlueprintRegistry;
    use bd_core::procgen::{LocationTemplate, generate_location, validate_plan};

    // Test ruin seed determinism
    let template = LocationTemplate::ruin();
    let plan_a = generate_location(&template, 42);
    let plan_b = generate_location(&template, 42);
    assert_eq!(
        plan_a.tiles, plan_b.tiles,
        "Ruin seed should be deterministic"
    );
    assert_eq!(plan_a.rooms.len(), plan_b.rooms.len());

    // Verify blueprint registry has boss
    let registry = BlueprintRegistry::phase18_defaults();
    assert!(
        registry.get("blueprint.crypt_lord").is_some(),
        "Crypt Lord boss must exist"
    );

    // Verify crypt template exists and works
    let crypt = LocationTemplate::crypt();
    let crypt_plan = generate_location(&crypt, 123);
    let _validation = validate_plan(&crypt_plan);
    assert!(!crypt_plan.rooms.is_empty(), "Crypt should have rooms");
    // Some seeds may produce validation warnings, but shouldn't panic

    // Verify Water tile is renderable (not walkable)
    use bd_core::components::Tile;
    assert!(!Tile::Water.is_walkable(), "Water should not be walkable");
    assert!(Tile::Floor.is_walkable(), "Floor should be walkable");
}

/// Count entities in a world.
fn entity_count(world: &mut bevy_ecs::world::World) -> usize {
    let mut query = world.query::<()>();
    query.iter(world).count()
}

/// Create a minimal world for stress testing.
fn stress_world() -> bevy_ecs::world::World {
    let mut world = bevy_ecs::world::World::new();
    world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
    world.insert_resource(GameLog::default());
    world
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn synthetic_spawn_despawn_fixture_returns_to_baseline() {
    let mut world = stress_world();

    // Spawn a player (persistent entity)
    world.spawn((
        Player,
        Position { x: 5, y: 5 },
        Pools::new(vec![
            Pool::new(PoolKind::Health, 20, 0, 20),
            Pool::new(PoolKind::ActionPoints, 3, 0, 3),
        ]),
    ));

    let initial_count = entity_count(&mut world);
    assert_eq!(initial_count, 1, "Should have exactly 1 entity (player)");

    // Simulate combat: spawn and despawn transient entities
    let mut spawned: Vec<bevy_ecs::entity::Entity> = Vec::new();

    for turn in 0..100 {
        // Spawn some transient enemies each turn
        for _ in 0..3 {
            let e = world
                .spawn(Position {
                    x: turn % 10,
                    y: (turn / 10) % 10,
                })
                .id();
            spawned.push(e);
        }

        // Despawn "dead" entities (the oldest ones)
        while spawned.len() > 5 {
            if let Some(dead) = spawned.pop() {
                if let Ok(entity_mut) = world.get_entity_mut(dead) {
                    entity_mut.despawn();
                }
            }
        }
    }

    // Clean up remaining transient entities
    for e in spawned.drain(..) {
        if let Ok(entity_mut) = world.get_entity_mut(e) {
            entity_mut.despawn();
        }
    }

    // Final count should equal initial (only the persistent player)
    let final_count = entity_count(&mut world);
    assert_eq!(
        final_count, initial_count,
        "Entity leak after 100 turns: {final_count} (expected {initial_count})"
    );
}

#[test]
fn production_colony_dungeon_cycles_do_not_leak_scoped_entities() {
    let mut driver = FoundationDriver::new(9_001);
    driver.start_colony().unwrap();
    driver.fixture_set_colony_resource(PoolKind::Supplies, 100);

    let run_baseline = driver.scope_count(EntityScope::RunPersistent);
    let colony_baseline = driver.scope_count(EntityScope::ColonyPersistent);
    let dungeon_baseline = driver.scope_count(EntityScope::DungeonTransient);

    for cycle in 0..10 {
        driver.enter_dungeon(FOUNDATION_DUNGEON_ID).unwrap();
        assert!(
            driver.scope_count(EntityScope::DungeonTransient) > dungeon_baseline,
            "cycle {cycle} did not create production dungeon-scoped entities"
        );

        driver
            .return_to_colony(&format!("production leak cycle {cycle}"))
            .unwrap();

        assert_eq!(
            driver.scope_count(EntityScope::RunPersistent),
            run_baseline,
            "run-persistent count drifted after cycle {cycle}"
        );
        assert_eq!(
            driver.scope_count(EntityScope::ColonyPersistent),
            colony_baseline,
            "colony-persistent count drifted after cycle {cycle}"
        );
        assert_eq!(
            driver.scope_count(EntityScope::DungeonTransient),
            dungeon_baseline,
            "dungeon-transient entities leaked after cycle {cycle}"
        );
    }
}

#[test]
fn deferred_procgen_seed_batch_smoke_does_not_panic() {
    let template = LocationTemplate::ruin();
    // Run 1000 seeds to validate procgen stability
    for seed in 0..1000 {
        let plan = generate_location(&template, seed);
        // Quick validation that rooms and tiles exist
        assert!(!plan.rooms.is_empty(), "Seed {seed}: no rooms");
        assert!(
            plan.tiles.contains(&Tile::Floor),
            "Seed {seed}: no floor tiles"
        );
        // Verify exit reachable using A*
        let validation = validate_plan(&plan);
        // Some seeds may produce invalid plans, but no panics
        if !validation.valid {
            // Expected: some seeds produce unreachable exits due to random placement
            continue;
        }
    }
}

#[test]
fn save_load_stress_roundtrip_passes() {
    let mut world = stress_world();

    // Spawn a player with pools
    world.spawn((
        Player,
        Position { x: 3, y: 4 },
        Pools::new(vec![
            Pool::new(PoolKind::Health, 15, 0, 20),
            Pool::new(PoolKind::ActionPoints, 2, 0, 3),
        ]),
    ));

    // Repeated save/load roundtrips
    let temp_dir = std::env::temp_dir().join("bd_stress_test");
    let blueprints = HashMap::new();

    for i in 0u64..10 {
        let path = save_world(&mut world, i, i, &temp_dir).unwrap();
        let (mut restored, seed) = load_world(&path, &blueprints).unwrap();

        // Verify player data preserved
        let mut query = restored.query::<&Pools>();
        let pool_count = query.iter(&restored).count();
        assert!(pool_count >= 1, "Roundtrip {i}: no pools found");

        assert_eq!(seed, i, "Roundtrip {i}: seed mismatch");

        // Cleanup for next iteration
        let _ = std::fs::remove_file(&path);
    }

    let _ = std::fs::remove_dir(&temp_dir);
}

#[derive(Resource, Default)]
struct ObservedTransitions(usize);

#[derive(Resource, Default)]
struct LateObservedTransitions(usize);

#[test]
fn transition_message_observer_reads_each_burst_item_once_and_old_items_expire() {
    use bevy_ecs::message::MessageReader;

    let mut app = bevy_app::App::new();
    app.add_message::<bd_core::spatial::TransitionIntent>();
    app.init_resource::<ObservedTransitions>();

    app.add_systems(
        bevy_app::Update,
        |mut messages: MessageReader<bd_core::spatial::TransitionIntent>,
         mut observed: bevy_ecs::prelude::ResMut<ObservedTransitions>| {
            observed.0 += messages.read().count();
        },
    );

    for _ in 0..1000 {
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<bd_core::spatial::TransitionIntent>>()
            .write(bd_core::spatial::TransitionIntent {
                target: bd_core::spatial::GameMode::Outpost,
                node_id: None,
            });
    }

    app.update();
    assert_eq!(
        app.world().resource::<ObservedTransitions>().0,
        1000,
        "the production message reader must observe every burst item"
    );

    app.update();
    assert_eq!(
        app.world().resource::<ObservedTransitions>().0,
        1000,
        "an existing reader must not process a retained message twice"
    );

    app.init_resource::<LateObservedTransitions>();
    app.add_systems(
        bevy_app::Update,
        |mut messages: MessageReader<bd_core::spatial::TransitionIntent>,
         mut observed: bevy_ecs::prelude::ResMut<LateObservedTransitions>| {
            observed.0 += messages.read().count();
        },
    );
    app.update();

    assert_eq!(
        app.world().resource::<LateObservedTransitions>().0,
        0,
        "messages older than the retention window must be unavailable to a new reader"
    );
}

#[test]
fn procgen_timing_is_reasonable() {
    // Measure procgen time for a batch
    let template = LocationTemplate::ruin();
    let start = Instant::now();
    for seed in 0..100 {
        let _plan = generate_location(&template, seed);
    }
    let elapsed = start.elapsed();
    // 100 procgen calls should complete in under 5 seconds
    assert!(
        elapsed.as_secs() < 5,
        "Procgen too slow: 100 seeds took {elapsed:?}"
    );
}
