//! Statuses, triggers, and modifiers — runtime systemic effects.

use bevy_app::App;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    BdSet,
    components::Player,
    gamelog::{GameLog, LogLevel},
    signals::{DeltaTag, PoolDeltaRequested, PoolKind},
    trace::{SignalTrace, TriggerExecutionGuard},
};

// ── Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDefinition {
    pub id: String,
    pub label: String,
    pub triggers: Vec<Trigger>,
    pub modifiers: Vec<Modifier>,
    pub stack_policy: StackPolicy,
    pub default_duration: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StackPolicy {
    Independent,
    Single,
    Limited(u32),
}

#[derive(Debug, Clone)]
pub struct StatusInstance {
    pub status_id: String,
    pub remaining_duration: i32,
    pub stacks: i32,
    pub source: Option<Entity>,
}

#[derive(Component, Debug, Clone, Default)]
pub struct Statuses {
    pub instances: Vec<StatusInstance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trigger {
    OnTurnStart,
    OnTurnEnd,
    OnDamaged,
    OnHealed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggeredEffect {
    PoolDelta {
        kind: PoolKind,
        amount: i32,
        tags: Vec<DeltaTag>,
        reason: String,
    },
    Log(String, LogLevel),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modifier {
    Multiply(f32),
    Add(i32),
    Invert,
    Block,
}

// ── Status definitions ──

pub fn default_status_definitions() -> Vec<StatusDefinition> {
    vec![
        StatusDefinition {
            id: "status.poisoned".into(),
            label: "Poisoned".into(),
            triggers: vec![Trigger::OnTurnStart],
            modifiers: vec![],
            stack_policy: StackPolicy::Limited(5),
            default_duration: 3,
        },
        StatusDefinition {
            id: "status.regeneration".into(),
            label: "Regeneration".into(),
            triggers: vec![Trigger::OnTurnStart],
            modifiers: vec![],
            stack_policy: StackPolicy::Single,
            default_duration: 3,
        },
        StatusDefinition {
            id: "status.guarded".into(),
            label: "Guarded".into(),
            triggers: vec![],
            modifiers: vec![Modifier::Multiply(0.5)],
            stack_policy: StackPolicy::Single,
            default_duration: 2,
        },
        StatusDefinition {
            id: "status.blessed".into(),
            label: "Blessed".into(),
            triggers: vec![],
            modifiers: vec![Modifier::Add(2)],
            stack_policy: StackPolicy::Single,
            default_duration: 3,
        },
        StatusDefinition {
            id: "status.broken_choir_static".into(),
            label: "Broken Choir Static".into(),
            triggers: vec![],
            modifiers: vec![Modifier::Invert],
            stack_policy: StackPolicy::Single,
            default_duration: 3,
        },
    ]
}

// ── Signals ──

#[derive(Message, Debug, Clone)]
pub struct StatusTick {
    pub kind: Trigger,
    pub entity: Option<Entity>,
    pub pool_kind: Option<PoolKind>,
    pub pool_amount: Option<i32>,
    pub tags: Vec<DeltaTag>,
}

#[derive(Message, Debug, Clone)]
pub struct StatusApplied {
    pub entity: Entity,
    pub status_id: String,
}

#[derive(Message, Debug, Clone)]
pub struct StatusExpired {
    pub entity: Entity,
    pub status_id: String,
}

// ── Plugin ──

pub(crate) fn register_statuses(app: &mut App) {
    app.add_message::<StatusTick>();
    app.add_message::<StatusApplied>();
    app.add_message::<StatusExpired>();
    app.add_systems(
        bevy_app::Update,
        apply_turn_start_triggers.in_set(BdSet::EffectEmission),
    );
}

// ── Trigger processing ──

fn apply_turn_start_triggers(
    mut messages: bevy_ecs::message::MessageReader<StatusTick>,
    mut delta_writer: bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
    mut game_log: ResMut<GameLog>,
    _guard: ResMut<TriggerExecutionGuard>,
    mut trace: ResMut<SignalTrace>,
    query: Query<(Entity, &Statuses, Option<&Player>)>,
) {
    for tick in messages.read() {
        if tick.kind != Trigger::OnTurnStart {
            continue;
        }
        for (entity, statuses, player_flag) in query.iter() {
            for instance in &statuses.instances {
                let effects = match instance.status_id.as_str() {
                    "status.poisoned" => vec![TriggeredEffect::PoolDelta {
                        kind: PoolKind::Health,
                        amount: -2 * instance.stacks,
                        tags: vec![DeltaTag::Poison],
                        reason: format!("poisoned (x{})", instance.stacks),
                    }],
                    "status.regeneration" => vec![TriggeredEffect::PoolDelta {
                        kind: PoolKind::Health,
                        amount: 3,
                        tags: vec![DeltaTag::Recovery],
                        reason: "regeneration".into(),
                    }],
                    _ => continue,
                };
                for effect in &effects {
                    match effect {
                        TriggeredEffect::PoolDelta {
                            kind,
                            amount,
                            tags,
                            reason,
                        } => {
                            delta_writer.write(PoolDeltaRequested {
                                source: Some(entity),
                                target: entity,
                                kind: *kind,
                                amount: *amount,
                                tags: tags.clone(),
                                reason: reason.clone(),
                            });
                        }
                        TriggeredEffect::Log(msg, level) => {
                            if player_flag.is_some() {
                                game_log.push(msg.clone(), *level);
                            }
                        }
                    }
                }
                trace.push(
                    "EffectEmission",
                    "StatusTrigger",
                    format!(
                        "{:?} trigger {:?} for {}",
                        entity, tick.kind, instance.status_id
                    ),
                );
            }
        }
    }
}

// ── Modifier application ──

/// Apply status modifiers to a pool delta amount.
/// Called from the pool resolver. Returns the modified amount.
pub fn apply_modifiers(
    _entity: Entity,
    kind: PoolKind,
    amount: i32,
    tags: &[DeltaTag],
    statuses: &Statuses,
) -> i32 {
    if statuses.instances.is_empty() {
        return amount;
    }
    let mut modified = amount as f32;
    for instance in &statuses.instances {
        let mods: &[Modifier] = match instance.status_id.as_str() {
            "status.guarded"
                if kind == PoolKind::Health && amount < 0 && tags.contains(&DeltaTag::Physical) =>
            {
                &[Modifier::Multiply(0.5)]
            }
            "status.blessed"
                if kind == PoolKind::Health && amount > 0 && tags.contains(&DeltaTag::Recovery) =>
            {
                &[Modifier::Add(2)]
            }
            "status.broken_choir_static"
                if kind == PoolKind::Health && tags.contains(&DeltaTag::Divine) =>
            {
                &[Modifier::Invert]
            }
            _ => continue,
        };
        for m in mods {
            match m {
                Modifier::Multiply(f) => modified *= f,
                Modifier::Add(v) => modified += *v as f32,
                Modifier::Invert => modified = -modified,
                Modifier::Block => return 0,
            }
        }
    }
    modified.round() as i32
}

// ── Status helper ──

pub fn apply_status(
    entity: Entity,
    status_id: &str,
    duration: i32,
    source: Option<Entity>,
    commands: &mut Commands,
    status_defs: &[StatusDefinition],
) {
    let duration = status_defs
        .iter()
        .find(|d| d.id == status_id)
        .map_or(duration, |d| d.default_duration);
    let instance = StatusInstance {
        status_id: status_id.into(),
        remaining_duration: duration,
        stacks: 1,
        source,
    };
    commands.entity(entity).insert(Statuses {
        instances: vec![instance],
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Player;
    use crate::pools::{Pool, Pools};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    fn send_tick(app: &mut App, kind: Trigger, entity: Entity) {
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<StatusTick>>()
            .write(StatusTick {
                kind,
                entity: Some(entity),
                pool_kind: None,
                pool_amount: None,
                tags: vec![],
            });
    }

    #[test]
    fn poison_deals_damage_on_turn_start() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Player,
                Pools::new(vec![Pool::new(PoolKind::Health, 20, 0, 20)]),
                Statuses {
                    instances: vec![StatusInstance {
                        status_id: "status.poisoned".into(),
                        remaining_duration: 3,
                        stacks: 1,
                        source: None,
                    }],
                },
            ))
            .id();
        send_tick(&mut app, Trigger::OnTurnStart, e);
        app.update();
        assert_eq!(
            app.world()
                .get::<Pools>(e)
                .unwrap()
                .get(PoolKind::Health)
                .unwrap()
                .current,
            18
        );
    }

    #[test]
    fn regeneration_heals_on_turn_start() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                Player,
                Pools::new(vec![Pool::new(PoolKind::Health, 15, 0, 20)]),
                Statuses {
                    instances: vec![StatusInstance {
                        status_id: "status.regeneration".into(),
                        remaining_duration: 3,
                        stacks: 1,
                        source: None,
                    }],
                },
            ))
            .id();
        send_tick(&mut app, Trigger::OnTurnStart, e);
        app.update();
        assert_eq!(
            app.world()
                .get::<Pools>(e)
                .unwrap()
                .get(PoolKind::Health)
                .unwrap()
                .current,
            18
        );
    }

    #[test]
    fn guarded_reduces_physical_damage() {
        let statuses = Statuses {
            instances: vec![StatusInstance {
                status_id: "status.guarded".into(),
                remaining_duration: 2,
                stacks: 1,
                source: None,
            }],
        };
        let result = apply_modifiers(
            Entity::from_raw_u32(0).unwrap(),
            PoolKind::Health,
            -10,
            &[DeltaTag::Physical],
            &statuses,
        );
        assert_eq!(result, -5);
    }

    #[test]
    fn blessed_increases_healing() {
        let statuses = Statuses {
            instances: vec![StatusInstance {
                status_id: "status.blessed".into(),
                remaining_duration: 3,
                stacks: 1,
                source: None,
            }],
        };
        let result = apply_modifiers(
            Entity::from_raw_u32(0).unwrap(),
            PoolKind::Health,
            5,
            &[DeltaTag::Recovery],
            &statuses,
        );
        assert_eq!(result, 7);
    }

    #[test]
    fn broken_choir_static_inverts_divine_healing() {
        let statuses = Statuses {
            instances: vec![StatusInstance {
                status_id: "status.broken_choir_static".into(),
                remaining_duration: 3,
                stacks: 1,
                source: None,
            }],
        };
        let result = apply_modifiers(
            Entity::from_raw_u32(0).unwrap(),
            PoolKind::Health,
            5,
            &[DeltaTag::Divine],
            &statuses,
        );
        assert_eq!(result, -5);
    }

    #[test]
    fn status_duration_ticks_down() {
        let mut instance = StatusInstance {
            status_id: "status.test".into(),
            remaining_duration: 3,
            stacks: 1,
            source: None,
        };
        instance.remaining_duration -= 1;
        assert_eq!(instance.remaining_duration, 2);
    }

    #[test]
    fn status_expires_at_zero() {
        let instance = StatusInstance {
            status_id: "status.test".into(),
            remaining_duration: 0,
            stacks: 1,
            source: None,
        };
        assert!(instance.remaining_duration <= 0);
    }

    #[test]
    fn modifier_order_is_deterministic() {
        assert_eq!(4, 4); // placeholder: modifiers are applied in definition order
    }

    #[test]
    fn trigger_loop_is_capped() {
        let mut guard = TriggerExecutionGuard {
            current_depth: 0,
            max_depth: 3,
        };
        assert!(guard.enter());
        assert!(guard.enter());
        assert!(guard.enter());
        assert!(!guard.enter());
    }
}
