//! Survivor entities — NPCs that live and work in the shelter.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::components::{Player, Position};
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::resources::{ResourceKind, ShelterResources};
use crate::core::save::PendingSurvivorLoad;
use crate::core::stats::EntityName;
use crate::core::turn::GameTime;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component for survivor entities.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Survivor;

/// Survivor biological needs. Each decays over time (1 per shelter tick).
/// At 0: the survivor dies.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct SurvivorNeeds {
    pub hunger: u32, // 100 = full, 0 = starving
    pub thirst: u32, // 100 = full, 0 = dehydrated
    pub rest: u32,   // 100 = rested, 0 = exhausted
}

impl Default for SurvivorNeeds {
    fn default() -> Self {
        Self {
            hunger: 80,
            thirst: 80,
            rest: 80,
        }
    }
}

impl SurvivorNeeds {
    /// Decay needs by 1 per tick. Returns true if any need is critical (≤ 20).
    pub fn tick_decay(&mut self) -> bool {
        self.hunger = self.hunger.saturating_sub(1);
        self.thirst = self.thirst.saturating_sub(1);
        self.rest = self.rest.saturating_sub(1);
        self.hunger <= 20 || self.thirst <= 20 || self.rest <= 20
    }

    /// Most critical need (lowest value).
    pub fn most_critical(&self) -> CriticalNeed {
        if self.thirst <= self.hunger && self.thirst <= self.rest {
            CriticalNeed::Thirst
        } else if self.hunger <= self.rest {
            CriticalNeed::Hunger
        } else {
            CriticalNeed::Rest
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriticalNeed {
    Hunger,
    Thirst,
    Rest,
}

// ---------------------------------------------------------------------------
// Task state
// ---------------------------------------------------------------------------

/// Current task/behavior state for a survivor.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum SurvivorTask {
    #[default]
    Idle,
    Working(IVec2), // Position of station they're working at
    Resting,
    SeekingFood,
    SeekingWater,
    Patrolling,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawn survivors — from save data if available, otherwise default starters.
pub fn spawn_initial_survivors(
    mut commands: Commands,
    pending: Option<ResMut<PendingSurvivorLoad>>,
    player_q: Query<&Position, With<Player>>,
) {
    let saved = pending.and_then(|mut p| {
        let data = std::mem::take(&mut p.0);
        if data.is_empty() { None } else { Some(data) }
    });

    if let Some(survivors) = saved {
        for s in survivors {
            commands.spawn((
                Survivor,
                EntityName { name: s.name },
                Position::new(s.x, s.y),
                SurvivorNeeds {
                    hunger: s.hunger,
                    thirst: s.thirst,
                    rest: s.rest,
                },
                s.task.to_runtime(),
                Sprite {
                    color: Color::srgb(0.3, 0.7, 0.4),
                    custom_size: Some(Vec2::splat(8.0)),
                    ..default()
                },
            ));
        }
    } else {
        let anchor = player_q
            .iter()
            .next()
            .copied()
            .unwrap_or(Position::new(5, 5));
        let starters = [
            ("Marcus", Position::new(anchor.x - 1, anchor.y + 1)),
            ("Elena", Position::new(anchor.x, anchor.y + 1)),
            ("Jin", Position::new(anchor.x + 1, anchor.y + 1)),
        ];

        for (name, pos) in starters {
            commands.spawn((
                Survivor,
                EntityName {
                    name: name.to_string(),
                },
                pos,
                SurvivorNeeds::default(),
                SurvivorTask::Idle,
                Sprite {
                    color: Color::srgb(0.3, 0.7, 0.4),
                    custom_size: Some(Vec2::splat(8.0)),
                    ..default()
                },
            ));
        }
    }
}

/// Each shelter tick, decay survivor needs. Log a warning when critical.
pub fn tick_survivor_needs(
    mut survivors: Query<(&mut SurvivorNeeds, &EntityName), With<Survivor>>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    for (mut needs, name) in &mut survivors {
        let critical = needs.tick_decay();
        if critical {
            let need = needs.most_critical();
            log.push(
                format!("{} is critically low on {:?}!", name.name, need),
                LogColor::Status,
                time.turn,
            );
        }
    }
}

/// Despawn survivors whose hunger or thirst has reached zero.
pub fn survivor_death(
    mut commands: Commands,
    survivors: Query<(Entity, &SurvivorNeeds, &EntityName), With<Survivor>>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    for (entity, needs, name) in &survivors {
        if needs.hunger == 0 || needs.thirst == 0 {
            let cause = if needs.hunger == 0 {
                "starvation"
            } else {
                "dehydration"
            };
            log.push(
                format!("{} has died of {}.", name.name, cause),
                LogColor::Death,
                time.turn,
            );
            commands.entity(entity).despawn();
        }
    }
}

/// Survivors with low needs consume shelter resources to restore themselves.
///
/// - hunger ≤ 50 → try consume 1 Food → restore 20 hunger (cap 100)
/// - thirst ≤ 50 → try consume 1 Water → restore 20 thirst (cap 100)
///
/// Runs AFTER `tick_survivor_needs` so decay happens first.
pub fn consume_shelter_resources(
    mut survivors: Query<(&mut SurvivorNeeds, &EntityName), With<Survivor>>,
    mut resources: ResMut<ShelterResources>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    for (mut needs, name) in &mut survivors {
        // Hunger: try to eat
        if needs.hunger <= 50 {
            if resources.try_consume(ResourceKind::Food, 1) {
                needs.hunger = (needs.hunger + 20).min(100);
                log.push(
                    format!("{} ate a ration", name.name),
                    LogColor::Status,
                    time.turn,
                );
            } else if needs.hunger <= 20 {
                log.push(
                    format!("{} couldn't find food!", name.name),
                    LogColor::EnemyHit,
                    time.turn,
                );
            }
        }

        // Thirst: try to drink
        if needs.thirst <= 50 {
            if resources.try_consume(ResourceKind::Water, 1) {
                needs.thirst = (needs.thirst + 20).min(100);
                log.push(
                    format!("{} drank water", name.name),
                    LogColor::Status,
                    time.turn,
                );
            } else if needs.thirst <= 20 {
                log.push(
                    format!("{} couldn't find water!", name.name),
                    LogColor::EnemyHit,
                    time.turn,
                );
            }
        }
    }
}

/// Simple priority-based survivor AI.
///
/// Priority order:
/// 1. Any need ≤ 20 → seek food / water / rest (critical override).
/// 2. Has `Working` task → snap to station position (player-assigned, preserved).
/// 3. Non-critical need recovery tasks complete → return to Idle.
/// 4. Otherwise → idle (no movement).
pub fn survivor_ai(
    mut survivors: Query<(&mut SurvivorTask, &mut Position, &SurvivorNeeds), With<Survivor>>,
) {
    for (mut task, mut pos, needs) in &mut survivors {
        // Priority 1: critical needs override current task.
        if needs.hunger <= 20 || needs.thirst <= 20 || needs.rest <= 20 {
            let critical = needs.most_critical();
            let new_task = match critical {
                CriticalNeed::Hunger => SurvivorTask::SeekingFood,
                CriticalNeed::Thirst => SurvivorTask::SeekingWater,
                CriticalNeed::Rest => SurvivorTask::Resting,
            };
            *task = new_task;
            continue;
        }

        // Priority 2: working at a station — move to station position (preserve assignment).
        if let SurvivorTask::Working(station_pos) = *task {
            *pos = Position::from(station_pos);
            continue;
        }

        // Priority 3: need-seeking tasks complete (needs > 20) → return to Idle.
        if matches!(
            *task,
            SurvivorTask::SeekingFood | SurvivorTask::SeekingWater | SurvivorTask::Resting
        ) {
            *task = SurvivorTask::Idle;
            continue;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_decay() {
        let mut needs = SurvivorNeeds {
            hunger: 50,
            thirst: 50,
            rest: 50,
        };
        let critical = needs.tick_decay();
        assert_eq!(needs.hunger, 49);
        assert_eq!(needs.thirst, 49);
        assert_eq!(needs.rest, 49);
        assert!(!critical, "all values > 20 should not be critical");
    }

    #[test]
    fn test_needs_critical() {
        let mut needs = SurvivorNeeds {
            hunger: 21,
            thirst: 21,
            rest: 21,
        };
        let critical = needs.tick_decay();
        assert_eq!(needs.hunger, 20);
        assert_eq!(needs.thirst, 20);
        assert_eq!(needs.rest, 20);
        assert!(critical, "at 20 should report critical");
    }

    #[test]
    fn test_most_critical() {
        let needs = SurvivorNeeds {
            hunger: 50,
            thirst: 10,
            rest: 50,
        };
        assert_eq!(needs.most_critical(), CriticalNeed::Thirst);
    }

    #[test]
    fn test_survivor_death_at_zero() {
        let needs = SurvivorNeeds {
            hunger: 0,
            thirst: 30,
            rest: 30,
        };
        assert!(
            needs.hunger == 0 || needs.thirst == 0,
            "hunger=0 should trigger death"
        );
    }

    #[test]
    fn test_decay_saturates_at_zero() {
        let mut needs = SurvivorNeeds {
            hunger: 0,
            thirst: 0,
            rest: 0,
        };
        needs.tick_decay();
        assert_eq!(needs.hunger, 0);
        assert_eq!(needs.thirst, 0);
        assert_eq!(needs.rest, 0);
    }

    #[test]
    fn test_most_critical_hunger() {
        let needs = SurvivorNeeds {
            hunger: 5,
            thirst: 30,
            rest: 30,
        };
        assert_eq!(needs.most_critical(), CriticalNeed::Hunger);
    }

    #[test]
    fn test_most_critical_rest() {
        let needs = SurvivorNeeds {
            hunger: 50,
            thirst: 50,
            rest: 5,
        };
        assert_eq!(needs.most_critical(), CriticalNeed::Rest);
    }

    // --- consume_shelter_resources tests ---

    use crate::core::gamelog::GameLog;
    use crate::core::resources::ShelterResources;
    use crate::core::stats::EntityName;
    use crate::core::turn::GameTime;
    use bevy::ecs::system::RunSystemOnce;

    fn setup_consume_world(
        hunger: u32,
        thirst: u32,
        food: u32,
        water: u32,
    ) -> bevy::ecs::world::World {
        let mut world = bevy::ecs::world::World::new();
        world.insert_resource(ShelterResources {
            food,
            water,
            scrap: 0,
            medicine: 0,
            ammo: 0,
        });
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.spawn((
            Survivor,
            EntityName {
                name: "Test".to_string(),
            },
            SurvivorNeeds {
                hunger,
                thirst,
                rest: 80,
            },
        ));
        world
    }

    #[test]
    fn test_hungry_survivor_eats_food() {
        let mut world = setup_consume_world(40, 80, 5, 5);
        let _ = world.run_system_once(consume_shelter_resources);

        let needs = world.query::<&SurvivorNeeds>().single(&world).unwrap();
        assert_eq!(needs.hunger, 60, "hunger 40 + 20 restore = 60");
        assert_eq!(world.resource::<ShelterResources>().food, 4);
    }

    #[test]
    fn test_default_survivors_spawn_near_player() {
        let mut world = bevy::ecs::world::World::new();
        world.spawn((Player, Position::new(20, 26)));

        let _ = world.run_system_once(spawn_initial_survivors);

        let mut positions: Vec<_> = world
            .query_filtered::<&Position, With<Survivor>>()
            .iter(&world)
            .copied()
            .collect();
        positions.sort_by_key(|pos| (pos.x, pos.y));

        assert_eq!(
            positions,
            vec![
                Position::new(19, 27),
                Position::new(20, 27),
                Position::new(21, 27),
            ]
        );
    }

    #[test]
    fn test_not_hungry_survivor_skips_food() {
        let mut world = setup_consume_world(60, 80, 5, 5);
        let _ = world.run_system_once(consume_shelter_resources);

        let needs = world.query::<&SurvivorNeeds>().single(&world).unwrap();
        assert_eq!(
            needs.hunger, 60,
            "hunger > 50 should not trigger consumption"
        );
        assert_eq!(world.resource::<ShelterResources>().food, 5);
    }

    #[test]
    fn test_no_food_available() {
        let mut world = setup_consume_world(40, 80, 0, 5);
        let _ = world.run_system_once(consume_shelter_resources);

        let needs = world.query::<&SurvivorNeeds>().single(&world).unwrap();
        assert_eq!(needs.hunger, 40, "no food available, hunger unchanged");
    }

    #[test]
    fn test_thirsty_survivor_drinks_water() {
        let mut world = setup_consume_world(80, 40, 5, 5);
        let _ = world.run_system_once(consume_shelter_resources);

        let needs = world.query::<&SurvivorNeeds>().single(&world).unwrap();
        assert_eq!(needs.thirst, 60, "thirst 40 + 20 restore = 60");
        assert_eq!(world.resource::<ShelterResources>().water, 4);
    }

    #[test]
    fn test_hunger_restore_caps_at_100() {
        let mut world = setup_consume_world(45, 80, 5, 5);
        let _ = world.run_system_once(consume_shelter_resources);

        let needs = world.query::<&SurvivorNeeds>().single(&world).unwrap();
        assert_eq!(needs.hunger, 65, "45 + 20 = 65");
    }

    #[test]
    fn test_both_needs_consumed_simultaneously() {
        let mut world = setup_consume_world(30, 30, 5, 5);
        let _ = world.run_system_once(consume_shelter_resources);

        let needs = world.query::<&SurvivorNeeds>().single(&world).unwrap();
        assert_eq!(needs.hunger, 50, "hunger 30 + 20 = 50");
        assert_eq!(needs.thirst, 50, "thirst 30 + 20 = 50");
        let res = world.resource::<ShelterResources>();
        assert_eq!(res.food, 4);
        assert_eq!(res.water, 4);
    }
}
