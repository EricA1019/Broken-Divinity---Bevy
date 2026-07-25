//! Survivors — colony inhabitants with tasks, needs, and mood tracking.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionDefinition, Effect, Requirement},
    pools::{Pool, Pools},
    signals::{DeltaTag, PoolKind},
    time::GameTime,
};

// ── Constants ──

/// Maximum number of survivors allowed.
pub const MAX_SURVIVORS: u32 = 10;

/// Food units consumed per survivor per day.
pub const FOOD_PER_SURVIVOR_PER_DAY: i32 = 1;

/// Maximum mood value.
pub const MOOD_MAX: i32 = 100;

/// Mood penalty when a survivor goes unfed.
pub const MOOD_STARVATION_PENALTY: i32 = 10;

/// Mood bonus when a survivor rests.
pub const MOOD_REST_BONUS: i32 = 5;

// ── Movement constants ──

/// Maximum tiles from center an idle survivor will wander.
pub const SURVIVOR_WANDER_RADIUS: i32 = 8;
/// Manhattan range a gathering survivor will walk toward a resource node.
pub const SURVIVOR_GATHER_RANGE: i32 = 15;
/// Tiles per turn survivors move (currently 1).
pub const SURVIVOR_SPEED: i32 = 1;

// ── Components ──

/// Marker component for survivor entities.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Survivor;

/// Current task assignment for a survivor.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub enum SurvivorTask {
    Idle,
    Gathering,
    Defending,
    AssignedTo(u64),
    Resting,
}

impl Default for SurvivorTask {
    fn default() -> Self {
        Self::Idle
    }
}

// ── Default survivor pools ──

/// Create the default Pools for a survivor (Mood pool + basic stats).
pub fn default_survivor_pools() -> Pools {
    Pools::new(vec![
        Pool::new(PoolKind::Mood, MOOD_MAX, 0, MOOD_MAX),
        Pool::new(PoolKind::ActionPoints, 3, 0, 3),
    ])
}

// ── Action definitions ──

/// Assign the nearest survivor to gathering (produces +1 food/day).
pub fn register_assign_gathering_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.assign_gathering".into(),
        label: "Assign Gathering".into(),
        requirements: vec![Requirement::TargetHasComponent(
            "bd_core::colony::survivors::Survivor",
        )],
        cost_effects: vec![],
        effects: vec![
            Effect::SetSurvivorTask("Gathering".into()),
            Effect::Log(
                "Survivor assigned to gathering.".into(),
                crate::gamelog::LogLevel::Info,
            ),
        ],
    }
}

pub fn register_assign_defending_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.assign_defending".into(),
        label: "Assign Defending".into(),
        requirements: vec![Requirement::TargetHasComponent(
            "bd_core::colony::survivors::Survivor",
        )],
        cost_effects: vec![],
        effects: vec![
            Effect::SetSurvivorTask("Defending".into()),
            Effect::Log(
                "Survivor assigned to defending.".into(),
                crate::gamelog::LogLevel::Info,
            ),
        ],
    }
}

pub fn register_assign_resting_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.assign_resting".into(),
        label: "Assign Resting".into(),
        requirements: vec![Requirement::TargetHasComponent(
            "bd_core::colony::survivors::Survivor",
        )],
        cost_effects: vec![],
        effects: vec![
            Effect::SetSurvivorTask("Resting".into()),
            Effect::Log(
                "Survivor assigned to resting.".into(),
                crate::gamelog::LogLevel::Info,
            ),
        ],
    }
}

pub fn register_assign_idle_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.assign_idle".into(),
        label: "Assign Idle".into(),
        requirements: vec![Requirement::TargetHasComponent(
            "bd_core::colony::survivors::Survivor",
        )],
        cost_effects: vec![],
        effects: vec![
            Effect::SetSurvivorTask("Idle".into()),
            Effect::Log(
                "Survivor assigned to idle.".into(),
                crate::gamelog::LogLevel::Info,
            ),
        ],
    }
}

/// Register the unassign_task action.
pub fn register_unassign_task_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.unassign_task".into(),
        label: "Unassign Task".into(),
        requirements: vec![Requirement::TargetHasComponent(
            "bd_core::colony::survivors::Survivor",
        )],
        cost_effects: vec![],
        effects: vec![
            Effect::SetSurvivorTask("Idle".into()),
            Effect::Log("Task unassigned.".into(), crate::gamelog::LogLevel::Info),
        ],
    }
}

// ── Systems ──

/// Process AssignToStation messages: set a survivor's SurvivorTask to
/// AssignedTo referencing the station entity by its raw index.
pub fn process_station_assignments(
    mut commands: Commands,
    mut messages: bevy_ecs::message::MessageReader<crate::signals::AssignToStation>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
    names: Query<&crate::components::Name>,
    stations: Query<Entity, With<crate::colony::stations::Station>>,
    survivors: Query<Entity, With<Survivor>>,
) {
    for msg in messages.read() {
        // Validate both entities still exist
        if stations.get(msg.station).is_err() {
            game_log.push(
                "Cannot assign: station no longer exists.".to_string(),
                crate::gamelog::LogLevel::Warn,
            );
            continue;
        }
        if survivors.get(msg.survivor).is_err() {
            game_log.push(
                "Cannot assign: survivor no longer exists.".to_string(),
                crate::gamelog::LogLevel::Warn,
            );
            continue;
        }
        let station_index = msg.station.to_bits();
        commands
            .entity(msg.survivor)
            .insert(SurvivorTask::AssignedTo(station_index));
        let survivor_name = names
            .get(msg.survivor)
            .map(|n| n.0.as_str())
            .unwrap_or("Survivor");
        game_log.push(
            format!("{} assigned to station.", survivor_name),
            crate::gamelog::LogLevel::Info,
        );
    }
}

/// Consumes shelter resources each day change: food per survivor, mood penalties.
/// P0-A fix: checks colony-level ColonyResources instead of entity-level Supplies
/// (survivors don't have entity-level Supplies — they only have Mood + AP).
pub fn consume_shelter_resources(
    query: Query<(Entity, &mut Pools, Option<&SurvivorTask>)>,
    colony_res: Res<crate::colony::production::ColonyResources>,
    mode: Res<crate::spatial::GameMode>,
    game_time: Res<GameTime>,
    mut pool_delta_writer: bevy_ecs::message::MessageWriter<crate::signals::PoolDeltaRequested>,
) {
    if *mode != crate::spatial::GameMode::Outpost {
        return;
    }
    // Only runs on day change (turn 0)
    if game_time.turn != 0 {
        return;
    }

    // Check colony-level supplies once (not per-survivor entity-level,
    // since survivors don't have a Supplies pool).
    let colony_has_supplies = colony_res
        .pools
        .get(PoolKind::Supplies)
        .map_or(false, |p| p.current >= FOOD_PER_SURVIVOR_PER_DAY);

    for (entity, pools, task) in query.iter() {
        // Skip non-survivors
        if pools.get(PoolKind::Mood).is_none() {
            continue;
        }

        if !colony_has_supplies {
            // Starvation: mood penalty when colony is out of supplies
            pool_delta_writer.write(crate::signals::PoolDeltaRequested {
                source: None,
                target: entity,
                kind: PoolKind::Mood,
                amount: -MOOD_STARVATION_PENALTY,
                tags: vec![DeltaTag::Action],
                reason: "starvation".into(),
            });
        }

        // Resting survivors recover mood
        if matches!(task, Some(SurvivorTask::Resting)) {
            pool_delta_writer.write(crate::signals::PoolDeltaRequested {
                source: None,
                target: entity,
                kind: PoolKind::Mood,
                amount: MOOD_REST_BONUS,
                tags: vec![DeltaTag::Recovery],
                reason: "rest recovery".into(),
            });
        }
    }
}

// ── Survivor Movement ──

/// Move survivors toward their task targets when time advances.
/// Gathering survivors move toward the nearest resource node.
/// AssignedTo survivors move toward their assigned station.
/// Idle/Defending/Resting survivors stay still for now (wander added later).
pub fn process_survivor_movement(
    mut survivors: Query<(&mut crate::components::Position, &SurvivorTask), With<Survivor>>,
    nodes: Query<
        (
            &crate::components::Position,
            &crate::components::ResourceNode,
        ),
        Without<Survivor>,
    >,
    stations: Query<
        (&crate::components::Position, Entity),
        (With<crate::colony::stations::Station>, Without<Survivor>),
    >,
    map: Res<crate::map::SmokeMap>,
    mode: Res<crate::spatial::GameMode>,
    should_advance: Res<crate::time::ShouldAdvanceTime>,
) {
    if *mode != crate::spatial::GameMode::Outpost || !should_advance.0 {
        return;
    }
    for (mut pos, task) in &mut survivors {
        let target = match task {
            SurvivorTask::Gathering => {
                // Move toward nearest non-depleted resource node
                nodes
                    .iter()
                    .filter(|(_, n)| !n.depleted)
                    .min_by_key(|(np, _)| (pos.x - np.x).abs() + (pos.y - np.y).abs())
                    .map(|(np, _)| (np.x, np.y))
            }
            SurvivorTask::AssignedTo(station_bits) => {
                // Move toward the assigned station
                stations
                    .iter()
                    .find(|(_, e)| e.to_bits() == *station_bits)
                    .map(|(sp, _)| (sp.x, sp.y))
            }
            SurvivorTask::Idle | SurvivorTask::Defending | SurvivorTask::Resting => None,
        };
        if let Some((tx, ty)) = target {
            let dx = (tx - pos.x).signum();
            let dy = (ty - pos.y).signum();
            let new_x = pos.x + dx;
            let new_y = pos.y + if dx == 0 { dy } else { 0 };
            if map.is_walkable(new_x, new_y) {
                pos.x = new_x;
                pos.y = new_y;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Position;
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(crate::spatial::GameMode::Outpost);
        app
    }

    fn spawn_survivor(app: &mut App, x: i32, y: i32) -> Entity {
        app.world_mut()
            .spawn((
                Survivor,
                SurvivorTask::Idle,
                Position { x, y },
                default_survivor_pools(),
            ))
            .id()
    }

    #[test]
    fn survivor_has_mood_pool() {
        let mut app = test_app();
        let s = spawn_survivor(&mut app, 5, 5);
        let pools = app.world().get::<Pools>(s).unwrap();
        let mood = pools.get(PoolKind::Mood).unwrap();
        assert_eq!(mood.current, MOOD_MAX);
        assert_eq!(mood.max, MOOD_MAX);
    }

    #[test]
    fn survivor_max_count_is_ten() {
        assert_eq!(MAX_SURVIVORS, 10);
    }

    #[test]
    fn unassign_task_action_has_correct_id() {
        let def = register_unassign_task_action();
        assert_eq!(def.id, "ability.unassign_task");
    }

    // ── P0-A: Starvation bug tests ──

    #[test]
    fn survivors_keep_mood_when_colony_has_food() {
        let mut app = test_app();
        // Spawn 3 survivors
        for i in 0..3 {
            spawn_survivor(&mut app, 5 + i * 5, 5);
        }
        // Set colony supplies to 10
        app.world_mut()
            .resource_mut::<crate::colony::production::ColonyResources>()
            .pools
            .get_mut(PoolKind::Supplies)
            .unwrap()
            .current = 10;
        // Advance to day 1, turn 0
        app.world_mut().resource_mut::<crate::time::GameTime>().day = 1;
        app.world_mut().resource_mut::<crate::time::GameTime>().turn = 0;
        // Set ShouldAdvanceTime so turn advances past 0 after first frame
        app.world_mut()
            .resource_mut::<crate::time::ShouldAdvanceTime>()
            .0 = true;
        // Frame 1: consume_shelter_resources sees supplies>0 → no starvation
        app.update();
        // Frame 2: PoolDeltaRequested messages from frame 1 processed by resolve_pool_deltas
        app.update();
        // All survivors should still have max mood
        let mut query = app.world_mut().query::<&crate::pools::Pools>();
        for pools in query.iter(app.world()) {
            if let Some(mood) = pools.get(PoolKind::Mood) {
                assert_eq!(
                    mood.current, MOOD_MAX,
                    "Survivor mood should stay at {} when colony has food, got {}",
                    MOOD_MAX, mood.current
                );
                break;
            }
        }
    }

    #[test]
    fn survivors_starve_when_colony_has_no_food() {
        let mut app = test_app();
        // Spawn 3 survivors
        for i in 0..3 {
            spawn_survivor(&mut app, 5 + i * 5, 5);
        }
        // Set colony supplies to 0
        app.world_mut()
            .resource_mut::<crate::colony::production::ColonyResources>()
            .pools
            .get_mut(PoolKind::Supplies)
            .unwrap()
            .current = 0;
        // Advance to day 1, turn 0
        app.world_mut().resource_mut::<crate::time::GameTime>().day = 1;
        app.world_mut().resource_mut::<crate::time::GameTime>().turn = 0;
        // Set ShouldAdvanceTime so turn advances past 0 after first frame
        app.world_mut()
            .resource_mut::<crate::time::ShouldAdvanceTime>()
            .0 = true;
        // Frame 1: consume_shelter_resources sees supplies=0 → writes PoolDeltaRequested(Mood, -10)
        app.update();
        // Frame 2: resolve_pool_deltas reads and applies those messages
        app.update();
        // Survivors should have lost mood
        let mut query = app.world_mut().query::<&crate::pools::Pools>();
        for pools in query.iter(app.world()) {
            if let Some(mood) = pools.get(PoolKind::Mood) {
                assert!(
                    mood.current < MOOD_MAX,
                    "Survivor mood should drop when colony has no food, got {}",
                    mood.current
                );
                break;
            }
        }
    }
}
