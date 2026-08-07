//! Raids — enemy attacks on the shelter.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{BdSet, signals::PoolKind};

pub const RAID_CHANCE_PER_DAY: f32 = 0.15;
pub const RAID_MIN_DAY: u64 = 3;
pub const RAID_ENEMY_COUNT_MIN: u32 = 1;
pub const RAID_ENEMY_COUNT_MAX: u32 = 3;
pub const RAID_SUPPLIES_LOST_IF_UNDEFENDED: i32 = 5;

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub enum RaidState {
    #[default]
    Inactive,
    Active {
        turn_started: u64,
        enemy_count: u32,
    },
}

/// Marker component for raid-spawned enemies.
#[derive(Component, Debug, Clone)]
pub struct RaidEnemy;

/// Roll for raid at day change. Spawn enemies if raid triggers.
pub fn process_raids(
    mut raid_state: ResMut<RaidState>,
    mut colony_res: ResMut<crate::colony::production::ColonyResources>,
    game_time: Res<crate::time::GameTime>,
    mut last_day: Local<u64>,
    mut commands: Commands,
    mut game_log: ResMut<crate::gamelog::GameLog>,
    raid_enemies: Query<Entity, With<RaidEnemy>>,
) {
    // Only run on day change
    if game_time.day == *last_day || game_time.day == 0 {
        return;
    }
    *last_day = game_time.day;

    // Check if active raid needs resolution: if all enemies dead, end raid
    if matches!(*raid_state, RaidState::Active { .. }) {
        if raid_enemies.iter().next().is_none() {
            *raid_state = RaidState::Inactive;
            game_log.push(
                "The raid has been repelled!".to_string(),
                crate::gamelog::LogLevel::Info,
            );
        }
        return;
    }

    // Don't raid before minimum day
    if game_time.day < RAID_MIN_DAY {
        return;
    }

    // Roll for raid: use day as seed for deterministic roll
    let seed = game_time.day.wrapping_mul(2654435761);
    let roll = (seed % 100) as f32 / 100.0;
    if roll >= RAID_CHANCE_PER_DAY {
        return;
    }

    // Raid triggers! Determine enemy count
    let enemy_count = (seed / 100 % (RAID_ENEMY_COUNT_MAX - RAID_ENEMY_COUNT_MIN + 1) as u64
        + RAID_ENEMY_COUNT_MIN as u64) as u32;

    *raid_state = RaidState::Active {
        turn_started: game_time.turn,
        enemy_count,
    };

    // Spawn raiders at random positions on the shelter map edges
    let positions: [(i32, i32); 3] = [(3, 2), (16, 2), (10, 10)];
    for i in 0..enemy_count.min(3) {
        let (x, y) = positions[i as usize];
        commands.spawn((
            crate::components::Position { x, y },
            crate::components::Name(format!("Raider {}", i + 1)),
            crate::components::BlocksMovement,
            RaidEnemy,
            crate::pools::Pools::new(vec![
                crate::pools::Pool::new(PoolKind::Health, 8, 0, 8),
                crate::pools::Pool::new(PoolKind::ActionPoints, 2, 0, 2),
            ]),
        ));
    }

    game_log.push(
        format!(
            "Raiders attack the shelter! {} enemies sighted.",
            enemy_count
        ),
        crate::gamelog::LogLevel::Combat,
    );

    // If player has no defenders, immediate supply loss
    if colony_res
        .pools
        .get(PoolKind::Supplies)
        .map_or(0, |p| p.current)
        > 0
    {
        if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
            let loss = RAID_SUPPLIES_LOST_IF_UNDEFENDED.min(supplies.current);
            supplies.current -= loss;
            game_log.push(
                format!("Raiders steal {} supplies before you can react!", loss),
                crate::gamelog::LogLevel::Warn,
            );
        }
    }
}

pub fn register_raids(app: &mut bevy_app::App) {
    app.init_resource::<RaidState>();
    app.add_systems(bevy_app::Update, process_raids.in_set(BdSet::Mutation));
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{CurrentEvent, EventDefinition, EventNode, EventRegistry};
    use crate::factory::BlueprintCatalog;
    use bevy_app::App;
    use std::collections::HashMap;

    fn make_rat_bp() -> crate::factory::EntityBlueprint {
        crate::factory::EntityBlueprint {
            id: "blueprint.rat".into(),
            label: "Rat".into(),
            is_player: false,
            blocks_movement: true,
            pools: vec![
                (PoolKind::Health, 11, 0, 11),
                (PoolKind::ActionPoints, 2, 0, 2),
            ],
            statuses: vec![],
            visual: Some("Enemy".into()),
            markers: vec!["RaidEnemy".into()],
        }
    }

    #[test]
    fn raid_pushes_event_not_direct_spawn() {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(BlueprintCatalog::new(vec![make_rat_bp()]));
        app.world_mut()
            .resource_mut::<EventRegistry>()
            .register(EventDefinition {
                id: "event.raid.test".into(),
                start_node: "start".into(),
                nodes: HashMap::from([(
                    "start".into(),
                    EventNode {
                        speaker: "Scout".into(),
                        text: "Raiders!".into(),
                        choices: vec![crate::dialogue::Choice {
                            label: "Defend".into(),
                            conditions: vec![crate::dialogue::Condition::Always],
                            effects: vec![],
                            next_node: None,
                        }],
                        on_enter_effects: vec![],
                        on_exit_effects: vec![],
                    },
                )]),
                spawn_on_enter: vec![crate::actions::Effect::SpawnBlueprintAt {
                    blueprint_id: "blueprint.rat".into(),
                    x: 3,
                    y: 2,
                    mutators: vec![],
                }],
            });
        app.world_mut()
            .resource_mut::<crate::time::GameTime>()
            .day = 5;
        app.world_mut()
            .resource_mut::<crate::time::GameTime>()
            .turn = 50;
        app.world_mut()
            .resource_mut::<crate::colony::production::ColonyResources>()
            .pools
            .get_mut(PoolKind::Supplies)
            .unwrap()
            .current = 20;
        app.update();

        let w = app.world_mut();
        let ev = w.resource::<CurrentEvent>();
        assert!(
            ev.is_active(),
            "process_raids must push CurrentEvent instead of spawning"
        );
        assert_eq!(ev.event_id, "event.raid.test");
        let markers = w.query::<&RaidEnemy>().iter(w).count();
        assert_eq!(markers, 0, "no RaidEnemy before event resolution");
    }

    #[test]
    fn raid_event_spawn_creates_raiders() {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(BlueprintCatalog::new(vec![make_rat_bp()]));
        app.world_mut()
            .resource_mut::<EventRegistry>()
            .register(EventDefinition {
                id: "event.raid.spawn".into(),
                start_node: "start".into(),
                nodes: HashMap::from([(
                    "start".into(),
                    EventNode {
                        speaker: "Scout".into(),
                        text: "Raiders!".into(),
                        choices: vec![crate::dialogue::Choice {
                            label: "Defend".into(),
                            conditions: vec![crate::dialogue::Condition::Always],
                            effects: vec![],
                            next_node: None,
                        }],
                        on_enter_effects: vec![],
                        on_exit_effects: vec![],
                    },
                )]),
                spawn_on_enter: vec![crate::actions::Effect::SpawnBlueprintAt {
                    blueprint_id: "blueprint.rat".into(),
                    x: 3,
                    y: 2,
                    mutators: vec![],
                }],
            });
        app.world_mut()
            .resource_mut::<crate::time::GameTime>()
            .day = 5;
        app.world_mut()
            .resource_mut::<crate::time::GameTime>()
            .turn = 50;
        app.world_mut()
            .resource_mut::<crate::colony::production::ColonyResources>()
            .pools
            .get_mut(PoolKind::Supplies)
            .unwrap()
            .current = 20;
        app.world_mut().spawn((
            crate::components::Player,
            crate::components::Position { x: 1, y: 1 },
            crate::pools::Pools::new(vec![]),
        ));
        app.update();
        app.update();

        let w = app.world_mut();
        let ev = w.resource::<CurrentEvent>();
        assert!(ev.is_active());
        let raiders: Vec<_> = w
            .query::<(Entity, &RaidEnemy, &crate::components::Position)>()
            .iter(w)
            .collect();
        assert!(!raiders.is_empty());
        let scope = w.get::<crate::spatial::EntityScope>(raiders[0].0);
        assert!(scope.is_some());
    }

    #[test]
    fn raid_spawn_uses_blueprint_pools() {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(BlueprintCatalog::new(vec![make_rat_bp()]));
        app.world_mut()
            .resource_mut::<EventRegistry>()
            .register(EventDefinition {
                id: "event.raid.pools".into(),
                start_node: "start".into(),
                nodes: HashMap::from([(
                    "start".into(),
                    EventNode {
                        speaker: "Scout".into(),
                        text: "Raiders!".into(),
                        choices: vec![crate::dialogue::Choice {
                            label: "Defend".into(),
                            conditions: vec![crate::dialogue::Condition::Always],
                            effects: vec![],
                            next_node: None,
                        }],
                        on_enter_effects: vec![],
                        on_exit_effects: vec![],
                    },
                )]),
                spawn_on_enter: vec![crate::actions::Effect::SpawnBlueprintAt {
                    blueprint_id: "blueprint.rat".into(),
                    x: 3,
                    y: 2,
                    mutators: vec![],
                }],
            });
        app.world_mut()
            .resource_mut::<crate::time::GameTime>()
            .day = 5;
        app.world_mut()
            .resource_mut::<crate::time::GameTime>()
            .turn = 50;
        app.world_mut()
            .resource_mut::<crate::colony::production::ColonyResources>()
            .pools
            .get_mut(PoolKind::Supplies)
            .unwrap()
            .current = 20;
        app.world_mut().spawn((
            crate::components::Player,
            crate::components::Position { x: 1, y: 1 },
            crate::pools::Pools::new(vec![]),
        ));
        app.update();
        app.update();

        let w = app.world_mut();
        let (_, pools) = w
            .query::<(&RaidEnemy, &crate::pools::Pools)>()
            .iter(w)
            .next()
            .expect("must have RaidEnemy with pools");
        let hp = pools.get(PoolKind::Health).unwrap();
        assert_eq!(hp.max, 11);
        let ap = pools.get(PoolKind::ActionPoints).unwrap();
        assert_eq!(ap.max, 2);
    }
}
