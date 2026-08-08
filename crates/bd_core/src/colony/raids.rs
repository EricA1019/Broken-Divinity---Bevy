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

pub const RAID_EVENT_ID: &str = "event.raid";

/// Roll for raid at day change. Emits an event instead of spawning directly.
pub fn process_raids(
    mut raid_state: ResMut<RaidState>,
    mut colony_res: ResMut<crate::colony::production::ColonyResources>,
    game_time: Res<crate::time::GameTime>,
    mut last_day: Local<u64>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
    raid_enemies: Query<Entity, With<RaidEnemy>>,
    event_registry: Res<crate::events::EventRegistry>,
    mut trigger_writer: bevy_ecs::message::MessageWriter<crate::signals::EventTrigger>,
    player_query: Query<Entity, With<crate::components::Player>>,
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

    // Push an event instead of spawning directly.
    // Event spawn_on_enter will create enemies via blueprint factory.
    if let Some(actor) = player_query.iter().next() {
        if event_registry.get(RAID_EVENT_ID).is_some() {
            *raid_state = RaidState::Active {
                turn_started: game_time.turn,
                enemy_count,
            };
            trigger_writer.write(crate::signals::EventTrigger {
                actor,
                event_id: RAID_EVENT_ID.into(),
            });
            game_log.push(
                format!(
                    "Raiders attack the shelter! {} enemies sighted.",
                    enemy_count
                ),
                crate::gamelog::LogLevel::Combat,
            );
        } else {
            game_log.push(
                format!(
                    "Raid event '{}' not registered — skipping spawn.",
                    RAID_EVENT_ID
                ),
                crate::gamelog::LogLevel::Warn,
            );
        }
    }

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
    use crate::signals::EventTrigger;
    use bevy_app::App;
    use std::collections::HashMap;

    fn make_rat_bp() -> crate::factory::EntityBlueprint {
        crate::factory::EntityBlueprint {
            id: "blueprint.raid_rat".into(),
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
                id: RAID_EVENT_ID.into(),
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
                    blueprint_id: "blueprint.raid_rat".into(),
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

        // After one update: process_raids queued EventTrigger, but
        // process_event_triggers (IntentCollection) ran earlier this frame.
        // So event is not yet active and no entities spawned.
        let w = app.world_mut();
        let ev = w.resource::<CurrentEvent>();
        assert!(
            !ev.is_active(),
            "event not yet active after raid trigger (processed next frame)"
        );
        let markers = w.query::<&RaidEnemy>().iter(w).count();
        assert_eq!(markers, 0, "no RaidEnemy before event resolution");
        drop(w);

        // Second update: process_event_triggers reads the queued trigger,
        // event becomes active, spawn_on_enter fires.
        app.update();

        let w = app.world_mut();
        let ev = w.resource::<CurrentEvent>();
        assert!(
            ev.is_active(),
            "process_raids must push CurrentEvent (active after 2nd frame)"
        );
        assert_eq!(ev.event_id, RAID_EVENT_ID);
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
                id: RAID_EVENT_ID.into(),
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
                    blueprint_id: "blueprint.raid_rat".into(),
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
                id: RAID_EVENT_ID.into(),
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
                    blueprint_id: "blueprint.raid_rat".into(),
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

    // ── Phase 4: tier selection tests ──

    #[test]
    fn raid_tier_constants_map_correctly() {
        assert_eq!(RAID_EVENT_SMALL, "event.raid.small");
        assert_eq!(RAID_EVENT_MEDIUM, "event.raid.medium");
    }

    #[test]
    fn raid_tier_small_pushed_at_day_five() {
        // Day 5: seed=13272178805, enemy_count=(132721788%3)+1 = 0+1 = 1
        // enemy_count=1 → small tier
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(BlueprintCatalog::new(vec![make_rat_bp()]));
        // Register only the SMALL event
        app.world_mut()
            .resource_mut::<EventRegistry>()
            .register(EventDefinition {
                id: RAID_EVENT_SMALL.into(),
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
                    blueprint_id: "blueprint.raid_rat".into(),
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

        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(ev.is_active(), "raid event must be active at day 5");
        assert_eq!(
            ev.event_id, RAID_EVENT_SMALL,
            "enemy_count=1 must push small tier"
        );
    }

    #[test]
    fn raid_medium_event_spawns_elite_rats() {
        // Register the medium event which has Elite mutators on its spawns
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(BlueprintCatalog::new(vec![make_rat_bp()]));
        app.world_mut()
            .resource_mut::<EventRegistry>()
            .register(EventDefinition {
                id: RAID_EVENT_MEDIUM.into(),
                start_node: "start".into(),
                nodes: HashMap::from([(
                    "start".into(),
                    EventNode {
                        speaker: "Scout".into(),
                        text: "Elite raiders!".into(),
                        choices: vec![crate::dialogue::Choice {
                            label: "Fight!".into(),
                            conditions: vec![crate::dialogue::Condition::Always],
                            effects: vec![],
                            next_node: None,
                        }],
                        on_enter_effects: vec![],
                        on_exit_effects: vec![],
                    },
                )]),
                spawn_on_enter: vec![crate::actions::Effect::SpawnBlueprintAt {
                    blueprint_id: "blueprint.raid_rat".into(),
                    x: 3,
                    y: 2,
                    mutators: vec![crate::factory::Mutator::Elite],
                }],
            });
        // Trigger the medium event directly
        let player = app
            .world_mut()
            .spawn((
                crate::components::Player,
                crate::components::Position { x: 1, y: 1 },
                crate::pools::Pools::new(vec![]),
            ))
            .id();
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventTrigger>>()
            .write(EventTrigger {
                actor: player,
                event_id: RAID_EVENT_MEDIUM.into(),
            });
        app.update();

        // Find the spawned rat — must have Elite mutator: Health 11 → 16
        let (_, pools) = app
            .world_mut()
            .query::<(&RaidEnemy, &crate::pools::Pools)>()
            .iter(app.world())
            .next()
            .expect("medium event must spawn a RaidEnemy");
        let hp = pools.get(PoolKind::Health).unwrap();
        assert_eq!(
            hp.max, 16,
            "Elite mutator must scale Health to 1.5x (11→16)"
        );
    }
}
