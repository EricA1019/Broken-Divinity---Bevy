//! Performance and stability stress tests for the BD Kernel.
//!
//! Measures production entity-scope stability, message retention, repeated
//! persistence, and explicitly deferred procgen smoke behavior.

use bd_core::{
    components::Tile,
    procgen::{LocationTemplate, generate_location, validate_plan},
    signals::PoolKind,
    spatial::{EntityScope, FOUNDATION_DUNGEON_ID},
};
use bd_test_support::FoundationDriver;
use bevy_ecs::prelude::Resource;

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
    let mut valid_plans = 0;
    let mut invalid_plans = 0;

    for seed in 0..1000 {
        let plan = generate_location(&template, seed);
        assert!(!plan.rooms.is_empty(), "Seed {seed}: no rooms");
        assert!(
            plan.tiles.contains(&Tile::Floor),
            "Seed {seed}: no floor tiles"
        );
        let validation = validate_plan(&plan);
        if validation.valid {
            valid_plans += 1;
        } else {
            invalid_plans += 1;
        }
    }

    assert!(
        valid_plans > 0,
        "profile=DeferredProcgen expected=at least one valid plan"
    );
    assert_eq!(
        valid_plans + invalid_plans,
        1000,
        "profile=DeferredProcgen expected=all seeds classified"
    );
}

#[test]
fn repeated_production_save_load_preserves_fingerprint_and_next_action() {
    let mut current = FoundationDriver::new(9_002);
    current
        .start_colony()
        .expect("Foundation colony must start");

    for cycle in 0..10 {
        let before = current.fingerprint();
        let checkpoint = current
            .checkpoint()
            .unwrap_or_else(|error| panic!("cycle={cycle} checkpoint failed: {error}"));
        let mut restored = FoundationDriver::from_checkpoint(&checkpoint)
            .unwrap_or_else(|error| panic!("cycle={cycle} restore failed: {error}"));
        assert_eq!(
            restored.fingerprint(),
            before,
            "cycle={cycle} restore changed durable Foundation state"
        );

        let current_player = current.player().expect("current player must exist");
        let restored_player = restored.player().expect("restored player must exist");
        current
            .submit_action_and_advance_result_frame(
                &format!("cycle {cycle} current continuation"),
                current_player,
                "ability.wait",
                None,
                None,
            )
            .expect("current continuation must resolve");
        restored
            .submit_action_and_advance_result_frame(
                &format!("cycle {cycle} restored continuation"),
                restored_player,
                "ability.wait",
                None,
                None,
            )
            .expect("restored continuation must resolve");
        assert_eq!(
            restored.fingerprint(),
            current.fingerprint(),
            "cycle={cycle} restored next action diverged"
        );
        current = restored;
    }
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
