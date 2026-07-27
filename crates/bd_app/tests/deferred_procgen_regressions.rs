//! Deferred procedural-dungeon regression coverage.
//!
//! These tests protect reusable post-Foundation infrastructure. They do not
//! provide Foundation acceptance evidence and must not be cited as proof that
//! the fixed Foundation dungeon uses procedural generation.

use std::collections::HashSet;

use bd_core::{
    pathfinding::{AStarPathfinder, Pathfinder},
    procgen::{LocationTemplate, generate_location, validate_plan},
};

#[test]
fn deferred_ruin_plan_is_valid_for_the_named_seed() {
    let plan = generate_location(&LocationTemplate::ruin(), 42);
    let validation = validate_plan(&plan);

    assert!(
        validation.valid,
        "profile=DeferredProcgen seed=42 expected=valid ruin plan actual={:?}",
        validation.errors
    );
    assert!(!plan.rooms.is_empty(), "seed=42 produced no rooms");
    assert!(!plan.exits.is_empty(), "seed=42 produced no exits");
}

#[test]
fn deferred_ruin_exits_are_reachable_for_the_named_seed() {
    let plan = generate_location(&LocationTemplate::ruin(), 42);
    let map = plan.to_smoke_map();
    let blocked = HashSet::new();
    let pathfinder = AStarPathfinder;

    for (case, exit) in plan.exits.iter().enumerate() {
        assert!(
            pathfinder
                .find_path(&map, plan.entrance, *exit, &blocked)
                .is_some(),
            "profile=DeferredProcgen seed=42 case=exit-{case} \
             expected=reachable entrance-to-exit path entrance={:?} exit={exit:?}",
            plan.entrance
        );
    }
}

#[test]
fn deferred_ruin_generation_is_seed_deterministic() {
    let template = LocationTemplate::ruin();
    let first = generate_location(&template, 42);
    let repeated = generate_location(&template, 42);
    let alternate = generate_location(&template, 99);

    assert_eq!(
        first.tiles, repeated.tiles,
        "profile=DeferredProcgen seed=42 expected=identical repeated tile plan"
    );
    assert_ne!(
        first.tiles, alternate.tiles,
        "profile=DeferredProcgen seeds=42,99 expected=different tile plans"
    );
}
