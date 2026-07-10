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

// ── Components ──

/// Marker component for survivor entities.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Survivor;

/// Current task assignment for a survivor.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub enum SurvivorTask {
    Idle,
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

/// Register the assign_task action.
pub fn register_assign_task_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.assign_task".into(),
        label: "Assign Task".into(),
        requirements: vec![
            Requirement::TargetHasComponent("bd_core::colony::survivors::Survivor"),
            Requirement::TargetTaskIsIdle,
            Requirement::PlayerHasEntityInRange(2),
        ],
        cost_effects: vec![],
        effects: vec![
            Effect::SetSurvivorTask("AssignedTo".into()),
            Effect::Log("Task assigned.".into(), crate::gamelog::LogLevel::Info),
        ],
    }
}

/// Register the unassign_task action.
pub fn register_unassign_task_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.unassign_task".into(),
        label: "Unassign Task".into(),
        requirements: vec![
            Requirement::TargetHasComponent("bd_core::colony::survivors::Survivor"),
            Requirement::PlayerHasEntityInRange(2),
        ],
        cost_effects: vec![],
        effects: vec![
            Effect::SetSurvivorTask("Idle".into()),
            Effect::Log("Task unassigned.".into(), crate::gamelog::LogLevel::Info),
        ],
    }
}

// ── Systems ──

/// Consumes shelter resources each day change: food per survivor, mood penalties.
pub fn consume_shelter_resources(
    mut query: Query<(Entity, &mut Pools, Option<&SurvivorTask>)>,
    game_time: Res<GameTime>,
    mut pool_delta_writer: bevy_ecs::message::MessageWriter<crate::signals::PoolDeltaRequested>,
) {
    // Only runs on day change (turn 0)
    if game_time.turn != 0 {
        return;
    }

    for (entity, mut pools, task) in query.iter_mut() {
        // Skip non-survivors
        if pools.get(PoolKind::Mood).is_none() {
            continue;
        }

        // Deduct food (use Supplies from colony or skip if tracking entity-level)
        let _has_supplies = pools.get(PoolKind::Supplies)
            .map_or(false, |p| p.current >= FOOD_PER_SURVIVOR_PER_DAY);

        if _has_supplies {
            pool_delta_writer.write(crate::signals::PoolDeltaRequested {
                source: None,
                target: entity,
                kind: PoolKind::Supplies,
                amount: -FOOD_PER_SURVIVOR_PER_DAY,
                tags: vec![DeltaTag::Action],
                reason: "food consumption".into(),
            });
        } else {
            // Starvation: mood penalty
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::Position,
    };
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
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
    fn assign_task_action_has_correct_id() {
        let def = register_assign_task_action();
        assert_eq!(def.id, "ability.assign_task");
    }

    #[test]
    fn unassign_task_action_has_correct_id() {
        let def = register_unassign_task_action();
        assert_eq!(def.id, "ability.unassign_task");
    }
}
